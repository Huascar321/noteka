#!/usr/bin/env python3
"""Oracle independiente de proyección/hash/hits por CLI, con helper retirado al indexar."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import sys
import tempfile

sys.dont_write_bytecode = True
from importlib.util import module_from_spec, spec_from_file_location

ROOT = Path(__file__).resolve().parent.parent


def load(name, path):
    spec = spec_from_file_location(name, ROOT / path)
    result = module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--profile', choices=['debug', 'release'], default='debug')
    args = ap.parse_args()
    support = load('support', 'scripts/test-xlsx-search-migration.py')
    corpus = load('corpus', 'tests/fixtures/xlsx-search/generate.py')
    pins = {name: digest for digest, name in (line.split() for line in (ROOT / 'tests/fixtures/xlsx-search/golden.sha256').read_text().splitlines())}
    assert hashlib.sha256((ROOT / 'tests/fixtures/xlsx-search/generate.py').read_bytes()).hexdigest() == pins['generate.py']
    manifest = (json.dumps(corpus.manifest(), indent=2, ensure_ascii=False) + '\n').encode()
    assert hashlib.sha256(manifest).hexdigest() == pins['manifest.json']
    target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target')).resolve() / args.profile
    records = []
    with tempfile.TemporaryDirectory(prefix='xlsx-search-', dir=os.environ['NOTEKA_TEST_TMP']) as tmp:
        root = Path(tmp)
        cli = root / 'bin/noteka'
        cli.parent.mkdir()
        shutil.copy2(target / 'noteka', cli)
        helper = cli.with_name('noteka-xlsx-worker')
        shutil.copy2(target / helper.name, helper)
        ws = root / 'workspace'
        call = lambda group, action, payload, error=None: support.call(cli, ws, group, action, payload, error)
        assert call('workspace', 'create', dict(timezone='UTC'))['schema_version'] == 13
        ctl = call('extraction', 'control', dict(action='get'))
        call('extraction', 'control', dict(action='set', paused=False, expected_revision=ctl['revision']))
        done = []
        for case in corpus.cases():
            src = root / (case['name'] + '.xlsx')
            src.write_bytes(case['source'])
            attachment = call('attachment', 'import', dict(source_path=str(src)))
            job = call('extraction', 'enqueue', dict(attachment_id=attachment['id'], extractor='xlsx_cells'))['job']
            job = call('extraction', 'run', dict(id=job['id']))['job']
            assert job['state'] == 'succeeded', case['name']
            done.append((case, job))
        helper.unlink()
        for case, job in done:
            reader = call('extraction', 'xlsx-cells', dict(id=job['id'], limit=1))
            cells, page = list(reader['items']), reader
            while page['next_cursor']:
                page = call('extraction', 'xlsx-cells', dict(id=job['id'], limit=1, cursor=page['next_cursor']))
                cells.extend(page['items'])
            expected_cells = [(sheet['id'], cell) for sheet in case['expected']['sheets'] for cell in sheet['cells']]
            assert [(item['locator']['sheet_id'], item['cell']) for item in cells] == expected_cells, case['name']
            assert [item['ordinal'] for item in cells] == list(range(len(expected_cells)))
            raw = corpus.reader.compact(case['expected'])
            assert reader['manifest']['representation_sha256'] == hashlib.sha256(raw).hexdigest()
            before = support.snapshot(ws)
            index = call('extraction', 'index', dict(action='get'))
            payload = dict(action='put', job_id=job['id'], expected_generation_id=job['generation']['id'], expected_revision=index['revision'])
            try:
                expected = corpus.project(case['expected'])
            except corpus.OverQuota:
                e = call('extraction', 'index', payload, 'validation_error')
                assert e['details']['reason'] == 'index_budget_exceeded', e
                assert support.snapshot(ws) == before
                records.append(dict(name=case['name'], outcome='complete_rejection'))
                continue
            result = call('extraction', 'index', payload)
            entry = result['entry']
            assert entry['fragment_count'] == len(expected)
            assert entry['text_bytes'] == sum(len(f['text'].encode()) for f in expected)
            material = dict(source=entry['source'], generation=job['generation'], manifest=reader['manifest'], fragments=expected)
            encoded = json.dumps(material, sort_keys=True, ensure_ascii=False, separators=(',', ':')).encode()
            h = hashlib.sha256()
            for part in [b'noteka.extraction.index.content.v1', encoded]:
                h.update(len(part).to_bytes(8, 'big'))
                h.update(part)
            assert entry['content_hash'] == h.hexdigest(), (case['name'], entry['content_hash'], h.hexdigest())
            with sqlite3.connect(ws / 'noteka.sqlite') as db:
                rows = db.execute('SELECT ordinal,text FROM extraction_search_fts ORDER BY ordinal').fetchall()
            assert rows == [(f['ordinal'], f['text']) for f in expected], case['name']
            with sqlite3.connect(':memory:') as oracle:
                oracle.execute("CREATE VIRTUAL TABLE expected USING fts5(text,tokenize='unicode61')")
                oracle.executemany('INSERT INTO expected(rowid,text) VALUES (?,?)', [(f['ordinal']+1, f['text']) for f in expected])
                for query in ['nkprobe', '9007199254740993', 'cacheonly', 'formulaonly', '1', 'cafe']:
                    expected_ids = [r[0]-1 for r in oracle.execute('SELECT rowid FROM expected WHERE expected MATCH ? ORDER BY bm25(expected),rowid', ['"'+query+'"'])]
                    hits, cursor = [], None
                    while True:
                        page = call('extraction', 'search', dict(query=query, job_id=job['id'], extractor='xlsx_cells', limit=64, cursor=cursor))
                        hits.extend(page['items'])
                        cursor = page['next_cursor']
                        if not cursor:
                            break
                    assert [f['ordinal'] for f in hits] == expected_ids, (case['name'], query)
                    for hit in hits:
                        assert {k: hit[k] for k in ['ordinal', 'text', 'locator']} == expected[hit['ordinal']], case['name']
                        assert hit['source'] == entry['source']
            assert not helper.exists()
            unchanged = support.snapshot(ws)
            payload['expected_revision'] = result['revision']
            noop = call('extraction', 'index', payload)
            assert not noop['changed'] and noop['entry'] == entry
            assert support.snapshot(ws) == unchanged
            call('extraction', 'index', dict(action='reset', expected_revision=result['revision']))
            records.append(dict(name=case['name'], outcome='verified', fragments=len(expected), bytes=entry['text_bytes']))
    print(json.dumps(dict(profile=args.profile, cases=len(records), checks=records, independent_hash_and_projection=True,
                          helpers_absent_during_index_and_search=True), indent=2))


if __name__ == '__main__':
    main()
