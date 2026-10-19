#!/usr/bin/env python3
"""Gate mixto real: fixture fijada antes de medir; no altera el gate previo."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import threading
from xml.sax.saxutils import escape

sys.dont_write_bytecode = True
from importlib.util import module_from_spec, spec_from_file_location

ROOT = Path(__file__).resolve().parent.parent


def load(name, path):
    spec = spec_from_file_location(name, ROOT / path)
    value = module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def fill(pattern, size):
    raw = (pattern * (size // len(pattern.encode()) + 2)).encode()[:size]
    while True:
        try:
            return raw.decode() + 'z' * (size-len(raw))
        except UnicodeDecodeError:
            raw = raw[:-1]


def split(text):
    raw, result = text.encode(), []
    while raw:
        end = min(4096, len(raw))
        while True:
            try:
                chunk = raw[:end].decode()
                break
            except UnicodeDecodeError:
                end -= 1
        result.append(chunk)
        raw = raw[end:]
    return result


def prepare(directory, profile, recipe):
    support = load('support', 'scripts/test-xlsx-search-migration.py')
    corpus = load('xlsx', 'tests/fixtures/xlsx-search/generate.py')
    pdf = support.module('pdf', 'tests/fixtures/pdf/generate.py')
    target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target')).resolve() / profile
    runtime = directory / 'bin'
    runtime.mkdir()
    for name in ['noteka', 'noteka-pdf-worker', 'noteka-xlsx-worker']:
        shutil.copy2(target / name, runtime / name)
    cli = runtime / 'noteka'
    ws = directory / 'workspace'
    call = lambda group, action, payload: support.call(cli, ws, group, action, payload)
    call('workspace', 'create', dict(timezone='UTC'))
    ctl = call('extraction', 'control', dict(action='get'))
    call('extraction', 'control', dict(action='set', paused=False, expected_revision=ctl['revision']))
    sources = []
    for n in range(recipe['utf8']['generations']):
        text = fill(f"utf{n} " + recipe['common_pattern'], 1048576)
        sources.append((f'utf{n}.txt', text.encode(), 'utf8_text', len(text.encode()), None, split(text)))
    for n in range(recipe['pdf']['generations']):
        text = fill(f"pdf{n} " + recipe['common_pattern'].replace('\n', ' '), 1048575)
        # El extractor agrega un LF por operador textual; evitar ambigüedad de tail whitespace.
        text = text[:-1] + 'z'
        original = pdf.serialize(pdf.document([b'BT /F1 12 Tf (' + text.encode() + b') Tj ET'], compressed=True))
        sources.append((f'pdf{n}.pdf', original, 'pdf_text', 1048576, None, split(text + '\n')))
    for n in range(recipe['xlsx']['generations']):
        fields = []
        vocab = recipe['vocabulary']
        varied = ' '.join(f"{vocab['prefix']}{i:0{vocab['width']}d}" for i in range(vocab['count'])) + ' '
        for i, size in enumerate(recipe['xlsx']['field_bytes']):
            pattern = varied if i == 2 else (recipe['unicode_pattern'] if i == 5 else recipe['common_pattern'])
            fields.append(fill(pattern, size))
        rows = [f'<row r="1"><c r="A1" t="str"><f>{escape(fields[1])}</f><v>{escape(fields[0])}</v></c></row>']
        cells = [corpus.reader.cell(cached=corpus.reader.value('text', fields[0]), formula=corpus.reader.formula(text=fields[1]))]
        for row, text in enumerate(fields[2:], 2):
            rows.append(f'<row r="{row}"><c r="A{row}" t="str"><v>{escape(text)}</v></c></row>')
            cells.append(corpus.reader.cell(ref=f'A{row}', row=row, cached=corpus.reader.value('text', text)))
        parts = corpus.reader.parts(corpus.reader.worksheet(''.join(rows)))
        visibility = 'hidden' if n % 2 else 'visible'
        parts['xl/workbook.xml'] = parts['xl/workbook.xml'].replace('name="Sheet1"', f'name="Bench{n}" state="{visibility}"')
        expected = corpus.reader.book([corpus.reader.sheet(cells, observed=f'A1:A{len(cells)}', name=f'Bench{n}', visibility=visibility)])
        sources.append((f'xlsx{n}.xlsx', corpus.reader.pack(parts), 'xlsx_cells', 524288, expected, [f['text'] for f in corpus.project(expected)]))
    pins = []
    oracle = sqlite3.connect(':memory:')
    oracle.execute("CREATE VIRTUAL TABLE evidence USING fts5(job UNINDEXED,generation UNINDEXED,ordinal UNINDEXED,extractor UNINDEXED,text,tokenize='unicode61')")
    for name, original, extractor, size, expected, expected_texts in sources:
        path = directory / name
        path.write_bytes(original)
        attachment = call('attachment', 'import', dict(source_path=str(path)))
        job = call('extraction', 'enqueue', dict(attachment_id=attachment['id'], extractor=extractor))['job']
        job = call('extraction', 'run', dict(id=job['id']))['job']
        assert job['state'] == 'succeeded', job
        if expected:
            page = call('extraction', 'xlsx-cells', dict(id=job['id']))
            assert page['manifest']['representation_sha256'] == hashlib.sha256(corpus.reader.compact(expected)).hexdigest()
            projected = corpus.project(expected)
            assert sum(len(f['text'].encode()) for f in projected) == size
        idx = call('extraction', 'index', dict(action='get'))
        put = call('extraction', 'index', dict(action='put', job_id=job['id'], expected_generation_id=job['generation']['id'], expected_revision=idx['revision']))
        assert put['entry']['text_bytes'] == size, (name, put['entry']['text_bytes'], size)
        with sqlite3.connect(ws / 'noteka.sqlite') as db:
            stored = db.execute('SELECT text FROM extraction_search_fts WHERE job_id=? ORDER BY ordinal', [job['id']]).fetchall()
        assert [r[0] for r in stored] == expected_texts, name
        oracle.executemany('INSERT INTO evidence(job,generation,ordinal,extractor,text) VALUES(?,?,?,?,?)',
                           [(job['id'], job['generation']['id'], i, extractor, text) for i, text in enumerate(expected_texts)])
        pins.append(dict(name=name, source_sha256=hashlib.sha256(original).hexdigest(), text_bytes=size,
                         projection_sha256=hashlib.sha256(corpus.reader.compact(projected)).hexdigest() if expected else None))
    idx = call('extraction', 'index', dict(action='get'))
    assert idx['coverage']['text_bytes'] == 8388608 and idx['coverage']['indexed_generations'] == 12
    (runtime / 'noteka-pdf-worker').unlink()
    (runtime / 'noteka-xlsx-worker').unlink()
    for query, extractor in [('evidencia', None), ('ausente', None), ('café contrato', 'utf8_text'),
                              ('evidencia', 'pdf_text'), ('evidencia', 'xlsx_cells'), ('term299', None)]:
        match = ' AND '.join('"' + word + '"' for word in query.split())
        rows = oracle.execute('SELECT job,generation,ordinal,text,bm25(evidence) FROM evidence WHERE evidence MATCH ?' +
                              (' AND extractor=?' if extractor else '') + ' ORDER BY bm25(evidence),job COLLATE BINARY,generation COLLATE BINARY,ordinal LIMIT 64',
                              [match, extractor] if extractor else [match]).fetchall()
        page = call('extraction', 'search', dict(query=query, extractor=extractor, limit=64))
        actual = [(h['source']['job_id'], h['source']['generation_id'], h['ordinal'], h['text']) for h in page['items']]
        assert actual == [r[:4] for r in rows], (query, extractor)
        assert all(math.isclose(h['score'], r[4], rel_tol=1e-12, abs_tol=1e-15) for h, r in zip(page['items'], rows))
    oracle.close()
    (directory / 'corpus-pins.json').write_text(json.dumps(pins, indent=2) + '\n')
    return ws


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--profile', choices=['debug', 'release'], default='release')
    ap.add_argument('--prepare-only', action='store_true')
    args = ap.parse_args()
    recipe_path = ROOT / 'tests/fixtures/xlsx-search/benchmark.json'
    pins = {name: digest for digest, name in (line.split() for line in (ROOT / 'tests/fixtures/xlsx-search/benchmark.sha256').read_text().splitlines())}
    assert hashlib.sha256(recipe_path.read_bytes()).hexdigest() == pins['benchmark.json']
    recipe = json.loads(recipe_path.read_text())
    directory = Path(tempfile.mkdtemp(prefix='xlsx-search-bench-', dir=os.environ['NOTEKA_TEST_TMP']))
    ws = prepare(directory, args.profile, recipe)
    assert hashlib.sha256((directory / 'corpus-pins.json').read_bytes()).hexdigest() == pins['corpus-pins.json']
    if args.prepare_only:
        print(json.dumps(dict(directory=str(directory), workspace=str(ws), prepared=True)))
        return
    scratch = directory / 'scratch'
    scratch.mkdir()
    env = {**os.environ, 'TMPDIR': str(scratch), 'NOTEKA_XLSX_SEARCH_BENCH_WORKSPACE': str(ws),
           'NOTEKA_XLSX_SEARCH_BENCH_REPORT': str(directory / 'measurements.json')}
    peak = [0, 0]
    probe_errors = []
    stop = threading.Event()

    def probe():
        try:
            while not stop.is_set():
                sizes = []
                # os.walk tolera un directorio efímero que desaparezca durante
                # el recorrido; rglob puede lanzar fuera del stat protegido.
                for parent, _, names in os.walk(scratch):
                    for name in names:
                        try:
                            path = Path(parent) / name
                            if path.is_file():
                                s = path.stat()
                                sizes.append((s.st_size, s.st_blocks * 512))
                        except FileNotFoundError:
                            pass
                for i in range(2):
                    peak[i] = max(peak[i], sum(x[i] for x in sizes))
                stop.wait(.002)
        except Exception as error:
            probe_errors.append(repr(error))

    thread = threading.Thread(target=probe)
    thread.start()
    try:
        process = subprocess.run(['cargo', 'test', '--release', '--offline', '--locked', '-p', 'noteka-core',
                                  'release_xlsx_search_resources', '--', '--ignored', '--nocapture', '--test-threads=1'],
                                 cwd=ROOT, env=env, capture_output=True, timeout=1800)
    finally:
        stop.set()
        thread.join()
    (directory / 'run.log').write_bytes(process.stdout + process.stderr)
    assert not probe_errors, probe_errors
    assert process.returncode == 0, directory
    report = json.loads((directory / 'measurements.json').read_text())
    summary = {}
    for case in sorted({r['case'] for r in report['results']}):
        rows = [r for r in report['results'] if r['case'] == case and not r['warmup']]
        assert len(rows) == 20 and all(r['ok'] for r in rows), case
        item = {}
        for key in ['wall_ms', 'lock_ms', 'editor_wait_ms']:
            values = sorted(r[key] for r in rows if r.get(key) is not None)
            if values:
                item[key] = dict(p50=values[9], p95=values[18], max=max(values))
        summary[case] = item
    report.update(directory=str(directory), summary=summary, hardware=dict(platform=platform.platform(), machine=platform.machine()),
                  recipe_sha256=hashlib.sha256(recipe_path.read_bytes()).hexdigest(),
                  scratch_observed_logical_bytes=peak[0], scratch_observed_allocated_bytes=peak[1],
                  scratch_method='2 ms sampled regular-file sum under private TMPDIR; observed peak, not exclusive APFS usage or exact transient maximum')
    report['query_gate_met'] = all(v['wall_ms']['p95'] <= 300 for k, v in summary.items() if k.startswith('query-'))
    report['lock_gate_met'] = all(v['lock_ms']['p95'] <= 150 for v in summary.values() if 'lock_ms' in v)
    (directory / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))
    assert report['query_gate_met'] and report['lock_gate_met'], directory


if __name__ == '__main__':
    main()
