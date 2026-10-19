#!/usr/bin/env python3
"""Golden independiente por CLI real: persistencia/páginas/restore sin helper.

Ejecutar con red denegada; sólo genera fixtures sintéticas fuera de Git.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--profile', choices=['debug', 'release'], default='debug')
    args = ap.parse_args()
    root = Path(__file__).resolve().parent.parent
    target = Path(os.environ.get('CARGO_TARGET_DIR', root / 'target')).resolve() / args.profile
    spec = importlib.util.spec_from_file_location('xlsx_corpus', root / 'tests/fixtures/xlsx/generate.py')
    corpus = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(corpus)
    cases = corpus.cases()
    assert len(cases) == 216
    pins = {n: h for h, n in (line.split() for line in (root / 'tests/fixtures/xlsx/golden.sha256').read_text().splitlines())}
    manifest_bytes = (json.dumps(corpus.manifest(), ensure_ascii=False, indent=2) + '\n').encode()
    assert hashlib.sha256(manifest_bytes).hexdigest() == pins['manifest.json']
    assert hashlib.sha256((root / 'tests/fixtures/xlsx/generate.py').read_bytes()).hexdigest() == pins['generate.py']
    records = []
    with tempfile.TemporaryDirectory(prefix='xlsx-cli-', dir=os.environ['NOTEKA_TEST_TMP']) as tmp:
        temp = Path(tmp)
        runtime = temp / 'Noteka.app/Contents/MacOS'
        runtime.mkdir(parents=True)
        cli, worker = runtime / 'noteka', runtime / 'noteka-xlsx-worker'
        shutil.copy2(target / 'noteka', cli)
        shutil.copy2(target / 'noteka-xlsx-worker', worker)
        workspace = temp / 'workspace'

        def call(group, action, payload, error=None, ws=workspace):
            r = subprocess.run([str(cli), '--workspace', str(ws), '--json', group, action],
                               input=json.dumps(payload).encode(), capture_output=True, cwd=temp, timeout=12)
            assert r.stderr == b'', r.stderr[:200]
            assert len(r.stdout) <= 2_097_152, len(r.stdout)
            reply = json.loads(r.stdout)
            assert reply['api_version'] == 1
            if error:
                assert not reply['ok'] and r.returncode == error[0], (group, action, reply)
                assert reply['error']['details']['reason'] == error[1], reply
                return reply['error']
            assert reply['ok'] and r.returncode == 0, (group, action, reply)
            return reply['data']

        assert call('workspace', 'create', {'timezone': 'UTC'})['schema_version'] == 13
        c = call('extraction', 'control', {'action': 'get'})
        assert c['paused']
        call('extraction', 'control', {'action': 'set', 'paused': False, 'expected_revision': c['revision']})
        successes = []
        for case in cases:
            src = temp / (case['name'] + '.XLSX')
            src.write_bytes(case['source'])
            a = call('attachment', 'import', {'source_path': str(src)})
            request = dict(attachment_id=a['id'], extractor='xlsx_cells')
            if len(case['source']) > corpus.CAP:
                call('extraction', 'enqueue', request, (2, 'source_size_limit'))
                records.append(dict(name=case['name'], phase='admission', outcome='source_size_limit'))
                continue
            if a['mime'] not in ['application/octet-stream', 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet']:
                # El fixture not-zip tiene firma PDF: admisión lo rechaza antes del parser.
                assert case['error'] is not None
                call('extraction', 'enqueue', request, (2, 'unsupported_source_format'))
                records.append(dict(name=case['name'], phase='admission', outcome='unsupported_source_format'))
                continue
            j = call('extraction', 'enqueue', request)['job']
            assert call('extraction', 'enqueue', request)['job']['id'] == j['id']
            if case['error']:
                e = call('extraction', 'run', {'id': j['id']}, (2, case['error']))
                assert e['details']['job_id'] == j['id'] and e['details']['attempt'] == 1
                assert call('extraction', 'get', {'id': j['id']})['state'] == 'failed'
            else:
                done = call('extraction', 'run', {'id': j['id']})['job']
                g = done['generation']
                assert done['state'] == 'succeeded' and done['attempts'] == 1 and g['complete']
                assert g['source_sha256'] == hashlib.sha256(case['source']).hexdigest()
                assert g['covered_bytes'] == g['source_bytes'] == len(case['source'])
                cursor, items, manifests = None, [], []
                while True:
                    req = dict(id=j['id'], limit=64, cursor=cursor)
                    page = call('extraction', 'xlsx-cells', req)
                    assert page['generation'] == g and not page['result_from_previous_attempt']
                    assert page['covers_generation'] == (not cursor and not page['next_cursor'])
                    items.extend(page['items'])
                    manifests.append(page['manifest'])
                    cursor = page['next_cursor']
                    if not cursor:
                        break
                assert all(m == manifests[0] for m in manifests)
                m, expected = manifests[0], case['expected']
                assert m['version'] == 1 and m['parser'] == 'zip-9.0.3/roxmltree-0.21.1'
                for key in ['date_1904', 'limitations', 'styles']:
                    assert m[key] == expected[key], (case['name'], key)
                expected_sheets = [{**{k: v for k, v in s.items() if k != 'cells'}, 'cell_count': len(s['cells'])}
                                   for s in expected['sheets']]
                assert m['sheets'] == expected_sheets
                flattened = [(s['id'], c) for s in expected['sheets'] for c in s['cells']]
                assert m['cell_count'] == len(items) == len(flattened)
                for ordinal, (item, (sheet_id, cell)) in enumerate(zip(items, flattened)):
                    assert item == dict(ordinal=ordinal, locator=dict(kind='xlsx_cell', sheet_id=sheet_id,
                                                                     reference=cell['reference']), cell=cell), case['name']
                raw = corpus.compact(expected)
                assert m['representation_bytes'] == len(raw)
                assert m['representation_sha256'] == hashlib.sha256(raw).hexdigest()
                with sqlite3.connect(workspace / 'noteka.sqlite') as db:
                    chunks = list(db.execute('SELECT ordinal,text FROM extraction_xlsx_chunks WHERE generation_id=? ORDER BY ordinal', [g['id']]))
                assert len(chunks) == g['fragment_count'] <= 512
                assert [n for n, _ in chunks] == list(range(len(chunks)))
                assert all(1 <= len(s.encode()) <= 4096 for _, s in chunks)
                assert ''.join(s for _, s in chunks).encode() == raw
                call('extraction', 'fragments', {'id': j['id']}, (2, 'unsupported_generation_config'))
                call('extraction', 'pdf-fragments', {'id': j['id']}, (2, 'unsupported_generation_config'))
                successes.append(done)
            assert (workspace / 'attachments' / a['id']).read_bytes() == case['source']
            records.append(dict(name=case['name'], phase='run', outcome=case['error'] or 'success'))

        worker.unlink()
        job = next(j for j in successes if j['generation']['fragment_count'] > 1)
        with sqlite3.connect(workspace / 'noteka.sqlite') as db:
            before = list(db.iterdump())
            page = call('extraction', 'xlsx-cells', {'id': job['id']})
            assert list(db.iterdump()) == before
        backup, restored = temp / 'backup', temp / 'restored'
        call('workspace', 'backup', {'destination': str(backup)})
        call('workspace', 'restore', {'backup': str(backup), 'destination': str(restored)})
        assert call('extraction', 'xlsx-cells', {'id': job['id']}, ws=restored)['manifest'] == page['manifest']
        retry = call('extraction', 'retry', {'id': job['id'], 'expected_revision': job['revision']})
        call('extraction', 'run', {'id': retry['id']}, (5, 'xlsx_worker_unavailable'))
        previous = call('extraction', 'xlsx-cells', {'id': job['id']})
        assert previous['generation'] == job['generation'] and previous['result_from_previous_attempt']
    print(json.dumps(dict(profile=args.profile, cases=len(records), checks=records, layout='Contents/MacOS',
                          read_restore_without_helper=True, golden='independent Python ZIP/semantic corpus'), indent=2))


if __name__ == '__main__':
    main()
