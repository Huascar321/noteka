#!/usr/bin/env python3
"""Fuentes nuevas/golden independientes para lectura UI; sólo raíz externa explícita."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
from xml.sax.saxutils import escape

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent.parent


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT / path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--profile', choices=['debug', 'release'], default='debug')
    args = parser.parse_args()
    allowed = Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True)
    directory = args.root.resolve()
    assert directory != allowed and directory.is_relative_to(allowed)
    assert not directory.is_relative_to(ROOT) and not directory.exists()
    directory.mkdir(parents=True)
    workspace = directory / 'workspace'
    binary = ROOT / 'target' / args.profile / 'noteka'
    calls = []

    def call(operation, input):
        command = [str(binary), '--workspace', str(workspace), '--json', *[part.replace('_', '-') for part in operation.split('.')]]
        if sys.platform == 'darwin':
            command = ['/usr/bin/sandbox-exec', '-p', '(version 1)(allow default)(deny network*)', *command]
        result = subprocess.run(command, input=json.dumps(input).encode(), capture_output=True, timeout=20)
        response = json.loads(result.stdout)
        assert not result.stderr and not result.returncode and response['ok'], response
        calls.append(operation)
        return response['data']

    assert call('workspace.create', dict(timezone='UTC'))['schema_version'] == 13
    control = call('extraction.control', dict(action='get'))
    call('extraction.control', dict(action='set', paused=False, expected_revision=control['revision']))
    pdf = load('reader_ui_pdf', 'tests/fixtures/pdf/generate.py')
    xlsx = load('reader_ui_xlsx', 'tests/fixtures/xlsx/generate.py')
    literal = '\ufeff' + ('  nkreader café 🚀\r\n<script>window.__readerExecuted=true</script>\n' * 1600) + ' fin  '
    pdf_texts = [f'nkreader PDF página {i}\n' for i in range(1, 22)] + ['', '']
    contents = [(b'BT /F1 12 Tf <' + text[:-1].encode('cp1252').hex().encode() + b'> Tj ET') if text else b'' for text in pdf_texts]
    pdf_bytes = pdf.serialize(pdf.document(contents, font=pdf.FONT.replace(b'StandardEncoding', b'WinAnsiEncoding')))
    cells, rows = [], []
    definitions = [
        ('str', 'text', '  nkreader Ñ 🚀\n<script>window.__readerExecuted=true</script>  ', None),
        ('n', 'number', '145.6900000000000000000001', None),
        ('b', 'boolean', True, None), ('e', 'error', '#DIV/0!', None),
        ('d', 'date_iso', '2026-10-17T12:30:00Z', None),
        ('str', 'text', '', None), ('n', None, None, None),
        ('n', None, None, 'WEBSERVICE("https://invalid.example/no")'),
        ('n', 'number', '9007199254740993', '1+1'),
    ] + [('str', 'text', f'nkreader celda {i}', None) for i in range(10, 26)]
    definitions[9] = ('str', 'text', 'nkreader largo ' + 'é 🚀 ' * 4000 + ' final', None)
    for ordinal, (typ, kind, value, formula) in enumerate(definitions, 1):
        cached = xlsx.value(kind, value) if kind else None
        cells.append(xlsx.cell(ref=f'A{ordinal}', row=ordinal, cached=cached, formula=xlsx.formula(text=formula) if formula else None))
        v = ('1' if value else '0') if kind == 'boolean' else (str(value) if kind else None)
        rows.append(f'<row r="{ordinal}"><c r="A{ordinal}" t="{typ}">' + (f'<f>{escape(formula)}</f>' if formula else '') + (f'<v>{escape(v)}</v>' if v is not None else '') + '</c></row>')
    parts = xlsx.parts(xlsx.worksheet(''.join(rows), dimension='A1:A25', extra='<mergeCells count="1"><mergeCell ref="A1:B1"/></mergeCells>'))
    parts['xl/workbook.xml'] = parts['xl/workbook.xml'].replace('name="Sheet1"', 'name="Hoja Ñ" state="veryHidden"')
    book = xlsx.book([xlsx.sheet(cells, name='Hoja Ñ', visibility='very_hidden', declared='A1:A25', observed='A1:A25', merges=['A1:B1'])])
    sources = [('literal.txt', literal.encode(), 'utf8_text', None), ('pages.pdf', pdf_bytes, 'pdf_text', None),
               ('cells.xlsx', xlsx.pack(parts), 'xlsx_cells', book), ('empty.txt', b'', 'utf8_text', None)]
    selected = {case['name']: case for case in xlsx.cases() if case['name'] in ['empty-stored', 'formula-shared', 'formula-array', 'date-system-and-number-formats', 'sparse-maximum-coordinate']}
    for name, case in selected.items():
        sources.append((name + '.xlsx', case['source'], 'xlsx_cells', case['expected']))
    fixtures = {}
    for name, content, kind, expected in sources:
        source = directory / name
        source.write_bytes(content)
        attachment = call('attachment.import', dict(source_path=str(source)))
        job = call('extraction.enqueue', dict(attachment_id=attachment['id'], extractor=kind))['job']
        job = call('extraction.run', dict(id=job['id']))['job']
        assert job['state'] == 'succeeded'
        operation = dict(utf8_text='extraction.fragments', pdf_text='extraction.pdf_fragments', xlsx_cells='extraction.xlsx_cells')[kind]
        pages, cursor = [], None
        while True:
            page = call(operation, dict(id=job['id'], limit=20, **(dict(cursor=cursor) if cursor else {})))
            pages.append(page)
            cursor = page['next_cursor']
            if not cursor:
                break
        items = [item for page in pages for item in page['items']]
        if kind == 'utf8_text':
            assert ''.join(item['text'] for item in items).encode() == content
            for item in items:
                start, end = item['byte_start'], item['byte_end']
                assert item['text'].encode() == content[start:end]
                assert item['line_start'] == content[:start].count(b'\n') + 1
                assert item['line_end'] == content[:end-1].count(b'\n') + 1
        elif kind == 'pdf_text':
            actual = ['' for _ in pdf_texts]
            for item in items:
                loc = item['locator']; actual[loc['page_number']-1] += item['text']
                assert item['text'].encode() == pdf_texts[loc['page_number']-1].encode()[loc['text_byte_start']:loc['text_byte_end']]
            assert actual == pdf_texts
            assert [p['state'] for p in pages[0]['manifest']['pages']] == ['text'] * 21 + ['no_text'] * 2
        else:
            expected_cells = [(sheet['id'], cell) for sheet in expected['sheets'] for cell in sheet['cells']]
            assert [(item['locator']['sheet_id'], item['cell']) for item in items] == expected_cells
            assert pages[0]['manifest']['representation_sha256'] == hashlib.sha256(xlsx.compact(expected)).hexdigest()
            assert pages[0]['manifest']['styles'] == expected['styles']
        assert [item['ordinal'] for item in items] == list(range(len(items)))
        fixtures[name] = dict(attachment=attachment, job=job, kind=kind, pages=pages, expected=expected)
    # Una generación anterior sigue siendo legible: retry no procesa ni indexa.
    prior = fixtures['literal.txt']['job']
    call('extraction.retry', dict(id=prior['id'], expected_revision=prior['revision']))
    previous_pages = []
    cursor = None
    while True:
        page = call('extraction.fragments', dict(id=prior['id'], limit=20, **(dict(cursor=cursor) if cursor else {})))
        assert page['result_from_previous_attempt']
        previous_pages.append(page)
        cursor = page['next_cursor']
        if not cursor:
            break
    fixtures['literal.txt']['pages'] = previous_pages
    index = call('extraction.index', dict(action='get'))
    assert not index['items'] and not index['coverage']['indexed_generations']
    note = call('entity.create', dict(kind='note', title='Lectura sintética', document=dict(
        schema_version=2, blocks=[dict(id='reader-note', type='paragraph', content=[dict(type='text', text='Buffer original de lectura')])])))
    result = dict(root=str(directory), workspace=str(workspace), fixtures=fixtures, note=note, pdf_texts=pdf_texts, literal=literal, calls=len(calls))
    (directory / 'fixture.json').write_text(json.dumps(result, ensure_ascii=False, indent=2)+'\n')
    print(json.dumps(dict(root=str(directory), sources=len(fixtures), pages=sum(len(f['pages']) for f in fixtures.values()), calls=len(calls))))


if __name__ == '__main__':
    main()
