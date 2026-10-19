#!/usr/bin/env python3
"""Fixture UI sintética por CLI real; no lee datos personales ni altera el corpus."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
from xml.sax.saxutils import escape

sys.dont_write_bytecode = True
REPO = Path(__file__).resolve().parent.parent


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, REPO / path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', required=True, type=Path)
    parser.add_argument('--profile', choices=['debug', 'release'], default='debug')
    args = parser.parse_args()
    allowed = Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True)
    root = args.root.resolve()
    assert root.is_relative_to(allowed) and root != allowed
    assert not root.is_relative_to(REPO) and not root.exists()
    root.mkdir(parents=True)
    ws = root / 'workspace'
    binary = REPO / 'target' / args.profile / 'noteka'
    calls = []

    def call(operation, payload):
        command = [str(binary), '--workspace', str(ws), '--json', *[p.replace('_', '-') for p in operation.split('.')]]
        if sys.platform == 'darwin':
            command = ['/usr/bin/sandbox-exec', '-p', '(version 1) (allow default) (deny network*)', *command]
        result = subprocess.run(command, input=json.dumps(payload).encode(), capture_output=True, timeout=20)
        response = json.loads(result.stdout)
        assert not result.stderr and result.returncode == 0 and response['ok'], response
        calls.append(operation)
        return response['data']

    assert call('workspace.create', dict(timezone='UTC'))['schema_version'] == 13
    control = call('extraction.control', dict(action='get'))
    call('extraction.control', dict(action='set', paused=False, expected_revision=control['revision']))
    pdf = load('ui_pdf', 'tests/fixtures/pdf/generate.py')
    oracle = load('ui_xlsx', 'tests/fixtures/xlsx-search/generate.py')
    xlsx = oracle.reader
    utf8 = 'nkprobe evidencia café 🚀  \n<script>window.__evidenceExecuted=true</script>\n<img src="https://invalid.example/evidence">  fin\n'
    pdf_literal = 'nkprobe evidencia PDF alfa'
    pdf_text = pdf_literal + '\n'  # Tj cierra su objeto de texto con LF en este extractor.
    long_text = 'nkprobe ' + 'é ' * 1600 + 'nkprobe longtail 🚀'
    cells, xml_rows = [], []
    for row in range(1, 28):
        text = long_text if row == 1 else f'nkprobe celda {row}'
        formula = 'nkprobe DO_NOT_EXECUTE()' if row == 1 else None
        cells.append(xlsx.cell(ref=f'A{row}', row=row, cached=xlsx.value('text', text),
                               formula=xlsx.formula(text=formula) if formula else None))
        xml_rows.append(f'<row r="{row}"><c r="A{row}" t="str">' +
                        (f'<f>{escape(formula)}</f>' if formula else '') + f'<v>{escape(text)}</v></c></row>')
    for row, typ, kind, literal, value in [
        (28, 'n', 'number', '145.6900000000000000000001', '145.6900000000000000000001'),
        (29, 'b', 'boolean', '1', True), (30, 'e', 'error', '#DIV/0!', '#DIV/0!'),
        (31, 'd', 'date_iso', '2026-10-15T12:30:00Z', '2026-10-15T12:30:00Z'),
    ]:
        cells.append(xlsx.cell(ref=f'A{row}', row=row, cached=xlsx.value(kind, value)))
        xml_rows.append(f'<row r="{row}"><c r="A{row}" t="{typ}"><v>{escape(literal)}</v></c></row>')
    parts = xlsx.parts(xlsx.worksheet(''.join(xml_rows)))
    parts['xl/workbook.xml'] = parts['xl/workbook.xml'].replace('sheetId="1"', 'sheetId="1" state="hidden"')
    book = xlsx.book([xlsx.sheet(cells, observed='A1:A31', visibility='hidden')])
    sources = [
        ('literal.txt', utf8.encode(), 'utf8_text', True, True),
        ('text.pdf', pdf.serialize(pdf.document([f'BT /F1 12 Tf ({pdf_literal}) Tj ET'.encode()])), 'pdf_text', True, True),
        ('cells.xlsx', xlsx.pack(parts), 'xlsx_cells', True, True),
        ('empty.xlsx', xlsx.pack(xlsx.parts()), 'xlsx_cells', True, True),
        ('unselected.txt', b'nkprobe unselected evidence', 'utf8_text', True, False),
        ('queued.txt', b'nkprobe not extracted', 'utf8_text', False, False),
        ('future.txt', b'nkprobe future configuration', 'utf8_text', True, False),
    ]
    jobs = {}
    for name, content, extractor, run, select in sources:
        path = root / name
        path.write_bytes(content)
        attachment = call('attachment.import', dict(source_path=str(path)))
        job = call('extraction.enqueue', dict(attachment_id=attachment['id'], extractor=extractor))['job']
        if run:
            job = call('extraction.run', dict(id=job['id']))['job']
            assert job['state'] == 'succeeded'
        if select:
            index = call('extraction.index', dict(action='get'))
            call('extraction.index', dict(action='put', job_id=job['id'],
                                         expected_generation_id=job['generation']['id'], expected_revision=index['revision']))
        jobs[name] = job
    previous = jobs['literal.txt']
    previous = call('extraction.retry', dict(id=previous['id'], expected_revision=previous['revision']))
    previous = call('extraction.cancel', dict(id=previous['id'], expected_revision=previous['revision']))
    assert previous['result_from_previous_attempt']
    future = jobs['future.txt']
    config = json.dumps(dict(version=999, chunk_bytes=4096), sort_keys=True, separators=(',', ':'))
    digest = hashlib.sha256()
    for part in [b'noteka.extraction.config.v1', config.encode()]:
        digest.update(len(part).to_bytes(8, 'big'))
        digest.update(part)
    with sqlite3.connect(ws / 'noteka.sqlite') as db:
        db.execute('UPDATE extraction_jobs SET config=?,config_hash=? WHERE id=?', [config, digest.hexdigest(), future['id']])
        db.execute('UPDATE extraction_generations SET config=?,config_hash=? WHERE job_id=?', [config, digest.hexdigest(), future['id']])
    note = call('entity.create', dict(kind='note', title='Nota de evidencia sintética', document=dict(schema_version=2,
                blocks=[dict(id='evidence-paragraph', type='paragraph', content=[dict(type='text', text='nknotefixture cita original')])])))
    other = call('entity.create', dict(kind='note', title='Otra nota sintética'))
    projected = oracle.project(book)
    pages, cursor = [], None
    while True:
        page = call('extraction.search', dict(query='nkprobe', limit=20, **(dict(cursor=cursor) if cursor else {})))
        pages.append(page)
        cursor = page['next_cursor']
        if not cursor:
            break
    hits = [hit for page in pages for hit in page['items']]
    assert len(pages) == 2 and len(hits) > 20
    assert len({(h['source']['job_id'], h['source']['generation_id'], h['ordinal']) for h in hits}) == len(hits)
    for hit in hits:
        kind = hit['locator']['kind']
        if kind == 'utf8_bytes':
            assert hit['text'] == utf8 and hit['result_from_previous_attempt']
        elif kind == 'pdf_page_text':
            assert hit['text'] == pdf_text and hit['locator']['page_number'] == 1
        else:
            expected = projected[hit['ordinal']]
            assert (hit['text'], hit['locator']) == (expected['text'], expected['locator'])
    coverage = pages[0]['coverage']
    assert [coverage[k] for k in ['indexed_generations', 'unindexed_supported_generations', 'unsupported_generations', 'jobs_without_generation']] == [4, 1, 1, 1], coverage
    fields = set()
    for query in ['nkprobe', '145', 'true', 'DIV', '2026']:
        page = call('extraction.search', dict(query=query, extractor='xlsx_cells', limit=64))
        for hit in page['items']:
            expected = projected[hit['ordinal']]
            assert (hit['text'], hit['locator']) == (expected['text'], expected['locator'])
            fields.add(hit['locator']['field'])
    assert fields == {'cached_text', 'cached_number', 'cached_boolean', 'cached_error', 'cached_date_iso', 'formula_text'}, fields
    reader = call('extraction.xlsx_cells', dict(id=jobs['cells.xlsx']['id'], limit=64))
    assert [item['cell'] for item in reader['items']] == cells
    pdf_reader = call('extraction.pdf_fragments', dict(id=jobs['text.pdf']['id'], limit=64))
    literal_reader = call('extraction.fragments', dict(id=jobs['literal.txt']['id'], limit=64))
    assert pdf_reader['items'][0]['text'] == pdf_text
    assert literal_reader['items'][0]['text'] == utf8
    for hit in hits:
        if hit['locator']['kind'] == 'utf8_bytes':
            original = literal_reader['items'][hit['ordinal']]
            assert hit['locator'] == dict(kind='utf8_bytes', **{k: original[k] for k in ['byte_start', 'byte_end', 'line_start', 'line_end']})
        elif hit['locator']['kind'] == 'pdf_page_text':
            original = pdf_reader['items'][hit['ordinal']]
            assert all(hit['locator'][k] == v for k, v in original['locator'].items())
    manifest = dict(workspace=str(ws), root=str(root), jobs=jobs, note=note, other_note=other, pages=pages,
                    utf8_text=utf8, pdf_text=pdf_text, xlsx_projection=projected, xlsx_book=book,
                    pdf_reader=pdf_reader, literal_reader=literal_reader, coverage=coverage,
                    checks=dict(pages=len(pages), hits=len(hits), fields=sorted(fields), real_cli_calls=len(calls)),
                    injections=['future.txt: sólo config/config_hash de job y generación, sin selección ni hits inventados'])
    (root / 'fixture.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(dict(path=str(root / 'fixture.json'), **manifest['checks'])))


if __name__ == '__main__':
    main()
