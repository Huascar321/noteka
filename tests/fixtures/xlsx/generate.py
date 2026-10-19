#!/usr/bin/env python3
"""Corpus XLSX sintético MIT. ZIP por Python, golden semántico independiente de Rust.

No usa Excel, LibreOffice, calamine, el parser bajo prueba ni archivos personales.
Los casos se generan en memoria; el harness guarda sólo evidencia fuera de Git.
"""
import hashlib
import io
import json
import struct
import warnings
import zipfile
import zlib
from xml.sax.saxutils import escape

MAIN = 'http://schemas.openxmlformats.org/spreadsheetml/2006/main'
REL = 'http://schemas.openxmlformats.org/officeDocument/2006/relationships'
PACKAGE = 'http://schemas.openxmlformats.org/package/2006/relationships'
TYPES = 'http://schemas.openxmlformats.org/package/2006/content-types'
CT = 'application/vnd.openxmlformats-officedocument.spreadsheetml.'
LIMITATIONS = [
    'cached_values_not_recalculated', 'shared_formulas_not_expanded',
    'no_visual_formatting_or_date_conversion', 'no_header_or_table_inference',
    'no_images_charts_comments_or_shapes_transcription', 'bounded_transitional_xlsx_only',
]
CAP = 1_048_576
MAX_TEXT = 131_072


def compact(x):
    return json.dumps(x, ensure_ascii=False, separators=(',', ':')).encode()


def col(n):
    s = ''
    while n:
        n, c = divmod(n - 1, 26)
        s = chr(65 + c) + s
    return s


def value(kind, text):
    return {'kind': kind, 'value': text}


def cell(ref='A1', row=1, column=1, cached=None, formula=None, style=0):
    return dict(reference=ref, row=row, column=column, style=style, cached=cached, formula=formula)


def formula(kind='normal', text='1+1', reference=None, shared_index=None):
    return dict(kind=kind, text=text, reference=reference, shared_index=shared_index)


def sheet(cells=None, declared=None, observed=None, merges=None, name='Sheet1', ident=1, visibility='visible'):
    return dict(id=ident, name=name, visibility=visibility, declared_range=declared,
                observed_range=observed, merges=merges or [], cells=cells or [])


def book(sheets=None, styles=None, date_1904=False):
    return dict(date_1904=date_1904, limitations=LIMITATIONS,
                styles=styles or [dict(num_fmt_id=0, format_code=None)], sheets=sheets or [sheet()])


def worksheet(rows='', extra='', dimension=None):
    dim = '' if dimension is None else f'<dimension ref="{dimension}"/>'
    return f'<worksheet xmlns="{MAIN}">{dim}<sheetData>{rows}</sheetData>{extra}</worksheet>'


def one_cell(xml, dimension=None, extra=''):
    return worksheet(f'<row r="1">{xml}</row>', extra, dimension)


def rel(ident, kind, target, mode=''):
    return f'<Relationship Id="{ident}" Type="{REL}/{kind}" Target="{target}"{mode}/>'


def relationships(rows):
    return f'<Relationships xmlns="{PACKAGE}">{rows}</Relationships>'


def parts(sheet_xml=None, shared=None, styles=None):
    overrides = [('/xl/workbook.xml', CT + 'sheet.main+xml'), ('/xl/worksheets/sheet1.xml', CT + 'worksheet+xml')]
    rels = rel('r1', 'worksheet', 'worksheets/sheet1.xml')
    out = {
        '_rels/.rels': relationships(rel('root', 'officeDocument', 'xl/workbook.xml')),
        'xl/workbook.xml': f'<workbook xmlns="{MAIN}" xmlns:r="{REL}"><sheets><sheet name="Sheet1" sheetId="1" r:id="r1"/></sheets></workbook>',
        'xl/worksheets/sheet1.xml': sheet_xml or worksheet(),
    }
    for source, filename, kind in [(shared, 'sharedStrings', 'sharedStrings'), (styles, 'styles', 'styles')]:
        if source is not None:
            out[f'xl/{filename}.xml'] = source
            overrides.append((f'/xl/{filename}.xml', CT + kind + '+xml'))
            rels += rel(filename, kind, filename + '.xml')
    out['xl/_rels/workbook.xml.rels'] = relationships(rels)
    out['[Content_Types].xml'] = f'<Types xmlns="{TYPES}"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Default Extension="bin" ContentType="application/octet-stream"/>' + ''.join(f'<Override PartName="{p}" ContentType="{t}"/>' for p, t in overrides) + '</Types>'
    return out


def pack(p, method=zipfile.ZIP_STORED, comment=b''):
    out = io.BytesIO()
    entries = p.items() if isinstance(p, dict) else p
    with warnings.catch_warnings():
        warnings.simplefilter('ignore', UserWarning)  # duplicado intencional de un caso negativo
        with zipfile.ZipFile(out, 'w', compression=method) as z:
            for name, data in entries:
                info = zipfile.ZipInfo(name, (2026, 10, 15, 0, 0, 0))
                info.compress_type = method
                info.external_attr = 0o100600 << 16
                z.writestr(info, data)
            z.comment = comment
    return out.getvalue()


def descriptor_pack(p, method=0, signed=True, extra=b''):
    """ZIP propio independiente con descriptors; Python ZipFile lo verifica también."""
    local = bytearray(); central = bytearray()
    for name, content in p.items():
        data = content.encode() if isinstance(content, str) else content
        if method == 8:
            compressor = zlib.compressobj(wbits=-15)
            compressed = compressor.compress(data) + compressor.flush()
        else:
            compressed = data
        n = name.encode(); crc = zlib.crc32(data); offset = len(local)
        local += struct.pack('<IHHHHHIIIHH', 0x04034b50, 20, 8, method, 0, 0, 0, 0, 0, len(n), len(extra)) + n + extra + compressed
        local += (b'PK\x07\x08' if signed else b'') + struct.pack('<III', crc, len(compressed), len(data))
        central += struct.pack('<IHHHHHHIIIHHHHHII', 0x02014b50, 20, 20, 8, method, 0, 0, crc, len(compressed), len(data), len(n), len(extra), 0, 0, 0, 0, offset) + n + extra
    return bytes(local + central + struct.pack('<IHHHHIIH', 0x06054b50, 0, 0, len(p), len(p), len(central), len(local), 0))


def cases():
    out = []

    def add(name, p=None, expected=None, error=None, source=None, method=zipfile.ZIP_STORED):
        out.append(dict(name=name, source=source if source is not None else pack(p or parts(), method),
                        expected=None if error else (expected or book()), error=error))

    add('empty-stored')
    add('empty-deflate', method=zipfile.ZIP_DEFLATED)
    for method in [0, 8]:
        for signed in [False, True]:
            source = descriptor_pack(parts(), method, signed)
            with zipfile.ZipFile(io.BytesIO(source)) as z:
                assert len(z.infolist()) == len(parts()) and z.testzip() is None
            add(f'descriptor-{method}-{signed}', source=source)
    add('zip-comment', source=pack(parts(), comment=b'comment, not a path'))
    p = parts(); p['docProps/'] = b''
    add('directory-entry', p)
    for literal in ['0', '-0', '+01.2300E-9999', '.5', '1.', '9007199254740993', '145.6900000000000000000001', '1e9999999999']:
        add('number-' + str(len(out)), parts(one_cell(f'<c r="A1"><v>{literal}</v></c>')),
            book([sheet([cell(cached=value('number', literal))], observed='A1')]))
    for typ, literal, expected in [
        ('b', '1', value('boolean', True)), ('b', '0', value('boolean', False)),
        ('b', 'true', value('boolean', True)), ('b', 'false', value('boolean', False)),
        ('e', '#DIV/0!', value('error', '#DIV/0!')), ('e', '#SPILL!', value('error', '#SPILL!')),
        ('d', '2026-10-15T12:30:00Z', value('date_iso', '2026-10-15T12:30:00Z')),
        ('str', '', value('text', '')), ('str', '🚀 Ñ <&>', value('text', '🚀 Ñ <&>')),
    ]:
        add('typed-' + str(len(out)), parts(one_cell(f'<c r="A1" t="{typ}"><v>{escape(literal)}</v></c>')),
            book([sheet([cell(cached=expected)], observed='A1')]))
    for typ in ['', ' t="n"', ' t="str"', ' t="b"']:
        add('absent-' + str(len(out)), parts(one_cell(f'<c r="A1"{typ}/>')),
            book([sheet([cell()], observed='A1')]))
    for text, expected in [('', ''), ('  hola\n Ñ 🚀  ', '  hola\n Ñ 🚀  '),
                           ('_x000D_', '\r'), ('_xD83D__xDE80_', '🚀'),
                           ('_x005F_x0041_', '_x0041_'), ('_xZZZZ_', '_xZZZZ_')]:
        add('inline-' + str(len(out)), parts(one_cell(f'<c r="A1" t="inlineStr"><is><t xml:space="preserve">{escape(text)}</t></is></c>')),
            book([sheet([cell(cached=value('text', expected))], observed='A1')]))
    add('inline-rich', parts(one_cell('<c r="A1" t="inlineStr"><is><r><rPr><b/></rPr><t>Hola </t></r><r><t>Ñ🚀</t></r></is></c>')),
        book([sheet([cell(cached=value('text', 'Hola Ñ🚀'))], observed='A1')]))
    shared = f'<sst xmlns="{MAIN}" count="2" uniqueCount="2"><si><t/></si><si><r><t>A&amp;</t></r><r><t>🚀</t></r></si></sst>'
    add('shared-strings', parts(one_cell('<c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c>'), shared=shared),
        book([sheet([cell(cached=value('text', '')), cell('B1', column=2, cached=value('text', 'A&🚀'))], observed='A1:B1')]))
    add('cdata', parts(one_cell('<c r="A1" t="str"><v><![CDATA[<&>]]></v></c>')),
        book([sheet([cell(cached=value('text', '<&>'))], observed='A1')]))
    add('xml-newline-normalization', parts(one_cell('<c r="A1" t="str"><v>a\r\nb</v></c>')),
        book([sheet([cell(cached=value('text', 'a\nb'))], observed='A1')]))
    for cached in [None, value('number', '2')]:
        v = '' if cached is None else '<v>2</v>'
        add('formula-' + ('absent' if cached is None else 'cached'), parts(one_cell(f'<c r="A1"><f>1+1</f>{v}</c>')),
            book([sheet([cell(cached=cached, formula=formula())], observed='A1')]))
    add('formula-external-is-literal', parts(one_cell('<c r="A1"><f>WEBSERVICE("https://example.invalid/")</f></c>')),
        book([sheet([cell(formula=formula(text='WEBSERVICE("https://example.invalid/")'))], observed='A1')]))
    add('formula-array', parts(one_cell('<c r="A1"><f t="array" ref="A1:B2">SUM(B1:B2)</f></c>')),
        book([sheet([cell(formula=formula('array', 'SUM(B1:B2)', 'A1:B2'))], observed='A1')]))
    shared_cells = [cell(formula=formula('shared', 'B1+1', 'A1:A2', 7)),
                    cell('A2', row=2, formula=formula('shared', '', None, 7), cached=value('number', '9'))]
    add('formula-shared', parts(worksheet('<row r="1"><c r="A1"><f t="shared" ref="A1:A2" si="7">B1+1</f></c></row><row r="2"><c r="A2"><f t="shared" si="7"/><v>9</v></c></row>')),
        book([sheet(shared_cells, observed='A1:A2')]))
    add('sparse-maximum-coordinate', parts(worksheet('<row r="1048576"><c r="XFD1048576"><v>1</v></c></row>', dimension='A1')),
        book([sheet([cell('XFD1048576', row=1048576, column=16384, cached=value('number', '1'))], declared='A1', observed='XFD1048576')]))
    add('merge-not-expanded', parts(one_cell('<c r="A1"/>', extra='<mergeCells count="1"><mergeCell ref="A1:XFD1048576"/></mergeCells>')),
        book([sheet([cell()], observed='A1', merges=['A1:XFD1048576'])]))
    styles = f'<styleSheet xmlns="{MAIN}"><numFmts count="1"><numFmt numFmtId="164" formatCode="0.00&quot; BOB&quot;"/></numFmts><cellXfs count="3"><xf numFmtId="0"/><xf numFmtId="14"/><xf numFmtId="164"/></cellXfs></styleSheet>'
    style_golden = [dict(num_fmt_id=0, format_code=None), dict(num_fmt_id=14, format_code=None), dict(num_fmt_id=164, format_code='0.00" BOB"')]
    p = parts(one_cell('<c r="A1" s="1"><v>60</v></c><c r="B1" s="2"><v>145.69</v></c>'), styles=styles)
    p['xl/workbook.xml'] = p['xl/workbook.xml'].replace('<sheets>', '<workbookPr date1904="true"/><sheets>')
    add('date-system-and-number-formats', p, book([sheet([cell(style=1, cached=value('number', '60')), cell('B1', column=2, style=2, cached=value('number', '145.69'))], observed='A1:B1')], style_golden, True))
    for state, golden in [('hidden', 'hidden'), ('veryHidden', 'very_hidden')]:
        p = parts(); p['xl/workbook.xml'] = p['xl/workbook.xml'].replace('name="Sheet1"', f'name="Hoja Ñ" state="{state}"')
        add('visibility-' + state, p, book([sheet(name='Hoja Ñ', visibility=golden)]))
    p = parts(); p['xl/workbook.xml'] = p['xl/workbook.xml'].replace('name="Sheet1"', 'name="_x005F_x0041_"')
    add('sheet-name-escape', p, book([sheet(name='_x0041_')]))
    p = parts(); p['xl/_rels/workbook.xml.rels'] = p['xl/_rels/workbook.xml.rels'].replace('Target="worksheets/', 'Target="/xl/worksheets/')
    add('absolute-internal-target', p)
    p = parts(); p['xl/worksheets/sheet1.xml'] = p['xl/worksheets/sheet1.xml'].replace('<worksheet xmlns=', '<s:worksheet xmlns:s=').replace('<sheetData>', '<s:sheetData>').replace('</sheetData>', '</s:sheetData>').replace('</worksheet>', '</s:worksheet>')
    add('namespace-prefix', p)
    for decl in ['<?xml version="1.0" encoding="UTF-8" standalone="yes"?>', "<?xml version='1.0' encoding='utf-8'?>", '\ufeff<?xml version="1.0"?>']:
        p = parts(); p['xl/worksheets/sheet1.xml'] = decl + p['xl/worksheets/sheet1.xml']
        add('xml-declaration-' + str(len(out)), p)

    # Límites positivos y negativos con expectativas independientes completas.
    for n in [32, 33]:
        p = parts(); sheets = []
        for i in range(1, n + 1):
            p[f'xl/worksheets/sheet{i}.xml'] = worksheet()
            sheets.append(sheet(name=f'Sheet{i}', ident=i))
        p['xl/workbook.xml'] = f'<workbook xmlns="{MAIN}" xmlns:r="{REL}"><sheets>' + ''.join(f'<sheet name="Sheet{i}" sheetId="{i}" r:id="r{i}"/>' for i in range(1, n+1)) + '</sheets></workbook>'
        p['xl/_rels/workbook.xml.rels'] = relationships(''.join(rel(f'r{i}', 'worksheet', f'worksheets/sheet{i}.xml') for i in range(1, n+1)))
        p['[Content_Types].xml'] = p['[Content_Types].xml'].replace('</Types>', ''.join(f'<Override PartName="/xl/worksheets/sheet{i}.xml" ContentType="{CT}worksheet+xml"/>' for i in range(2, n+1)) + '</Types>')
        add(f'sheets-{n}', p, book(sheets), error='xlsx_structure_limit' if n>32 else None)
    for n in [10_000, 10_001]:
        cells = [cell(f'{col(i)}1', column=i) for i in range(1, n+1)]
        p = parts(one_cell(''.join(f'<c r="{col(i)}1"/>' for i in range(1, n+1))))
        add(f'cells-{n}', p, book([sheet(cells, observed=f'A1:{col(n)}1')]), error='xlsx_structure_limit' if n>10_000 else None)
    for n in [MAX_TEXT, MAX_TEXT+1]:
        add(f'text-{n}', parts(one_cell('<c r="A1" t="str"><v>' + 'a'*n + '</v></c>')),
            book([sheet([cell(cached=value('text', 'a'*n))], observed='A1')]), error='xlsx_structure_limit' if n>MAX_TEXT else None)
    for n in [512, 513]:
        st = f'<styleSheet xmlns="{MAIN}"><cellXfs count="{n}">' + '<xf numFmtId="0"/>'*n + '</cellXfs></styleSheet>'
        add(f'styles-{n}', parts(styles=st), book(styles=[dict(num_fmt_id=0, format_code=None)]*n), error='xlsx_structure_limit' if n>512 else None)
    for n in [10_000, 10_001]:
        sst = f'<sst xmlns="{MAIN}" uniqueCount="{n}">' + '<si><t>x</t></si>'*n + '</sst>'
        add(f'strings-{n}', parts(shared=sst), error='xlsx_structure_limit' if n>10_000 else None)
    for n in [1_000, 1_001]:
        merges = [f'A{i}:B{i}' for i in range(1, n+1)]
        extra = f'<mergeCells count="{n}">' + ''.join(f'<mergeCell ref="{r}"/>' for r in merges) + '</mergeCells>'
        add(f'merges-{n}', parts(worksheet(extra=extra)), book([sheet(merges=merges)]), error='xlsx_structure_limit' if n>1_000 else None)
    for n in [256, 257]:
        p = parts()
        for i in range(n-len(p)):
            p[f'docProps/blob{i}.bin'] = b''
        add(f'entries-{n}', p, error='xlsx_structure_limit' if n>256 else None)
    for n in [64, 65]:
        p = parts(); p['docProps/depth.xml'] = '<x>'*n + '</x>'*n
        add(f'depth-{n}', p, error='xlsx_xml_limit' if n>64 else None)
    for n in [100_000, 100_001]:
        p = parts(); p['docProps/nodes.xml'] = '<x>' + '<a/>'*(n-2) + '</x>'
        add(f'nodes-{n}', p, error='xlsx_xml_limit' if n>100_000 else None)
    for n in [4*CAP, 4*CAP+1]:
        p = parts(); p['docProps/blob.bin'] = b'0'*n
        add(f'entry-expansion-{n}', p, error='xlsx_expansion_limit' if n>4*CAP else None, method=zipfile.ZIP_DEFLATED)
    for delta in [0, 1]:
        # 16 MiB incluye todas las partes, no sólo blobs ni tamaños comprimidos.
        p = parts(); overhead = sum(len(v.encode() if isinstance(v, str) else v) for v in p.values())
        for i in range(4):
            p[f'docProps/blob{i}.bin'] = b'0'*(4*CAP - (overhead if i==0 else 0) + (delta if i==0 else 0))
        add(f'total-expansion-{16*CAP+delta}', p, error='xlsx_expansion_limit' if delta else None, method=zipfile.ZIP_DEFLATED)
    for delta in [0, 1]:
        p = parts(); p['docProps/padding.bin'] = b''
        p['docProps/padding.bin'] = b'0'*(CAP-len(pack(p))+delta)
        add(f'source-{CAP+delta}', p, error='xlsx_worker_protocol' if delta else None)
    for delta in [0, 1]:
        cells = [cell(f'{col(i)}1', column=i, cached=value('text', 'a'*MAX_TEXT if i<8 else '')) for i in range(1, 9)]
        expected = book([sheet(cells, observed='A1:H1')])
        missing = CAP - len(compact(expected)) + delta
        assert 0 <= missing <= MAX_TEXT
        cells[-1]['cached']['value'] = 'a'*missing
        rows = ''.join(f'<c r="{c["reference"]}" t="str"><v>{c["cached"]["value"]}</v></c>' for c in cells)
        add(f'result-{CAP+delta}', parts(one_cell(rows)), expected, error='xlsx_result_limit' if delta else None, method=zipfile.ZIP_DEFLATED)

    # Formato, semántica y fronteras negativas. El mensaje esperado nunca incluye contenido.
    for text in ['', 'NaN', 'INF', '1,2', '1e', ' 1', '1 ', '+']:
        add('bad-number-' + str(len(out)), parts(one_cell(f'<c r="A1"><v>{text}</v></c>')), error='xlsx_invalid_cell')
    for ref in ['A0', 'A01', 'a1', '$A$1', 'XFE1', 'A1048577']:
        add('bad-coordinate-' + str(len(out)), parts(one_cell(f'<c r="{ref}"/>')), error='xlsx_invalid_cell')
    for name, xml, error in [
        ('wrong-row', '<c r="A2"/>', 'xlsx_invalid_cell'),
        ('duplicate-cell', '<c r="A1"/><c r="A1"/>', 'xlsx_invalid_cell'),
        ('unordered-cell', '<c r="B1"/><c r="A1"/>', 'xlsx_invalid_cell'),
        ('missing-ref', '<c/>', 'xlsx_invalid_workbook'),
        ('bad-type', '<c r="A1" t="binary"><v>x</v></c>', 'xlsx_unsupported_cell'),
        ('bad-bool', '<c r="A1" t="b"><v>2</v></c>', 'xlsx_invalid_cell'),
        ('missing-shared', '<c r="A1" t="s"><v>0</v></c>', 'xlsx_invalid_cell'),
        ('negative-shared', '<c r="A1" t="s"><v>-1</v></c>', 'xlsx_invalid_workbook'),
        ('bad-style-index', '<c r="A1" s="1"/>', 'xlsx_invalid_cell'),
        ('duplicate-value', '<c r="A1"><v>1</v><v>2</v></c>', 'xlsx_invalid_workbook'),
        ('cell-extension', '<c r="A1"><extLst/></c>', 'xlsx_unsupported_cell'),
        ('cell-metadata', '<c r="A1" cm="1"/>', 'xlsx_unsupported_cell'),
        ('inline-and-value', '<c r="A1" t="inlineStr"><is><t>x</t></is><v>1</v></c>', 'xlsx_invalid_cell'),
        ('empty-inline', '<c r="A1" t="inlineStr"><is/></c>', 'xlsx_invalid_cell'),
        ('phonetic-text', '<c r="A1" t="inlineStr"><is><t>x</t><rPh><t>y</t></rPh></is></c>', 'xlsx_unsupported_cell'),
        ('nul-escape', '<c r="A1" t="str"><v>_x0000_</v></c>', 'xlsx_unsupported_cell'),
        ('unpaired-surrogate', '<c r="A1" t="str"><v>_xD800_</v></c>', 'xlsx_unsupported_cell'),
        ('low-surrogate', '<c r="A1" t="str"><v>_xDC00_</v></c>', 'xlsx_unsupported_cell'),
        ('empty-formula', '<c r="A1"><f/></c>', 'xlsx_invalid_formula'),
        ('data-table-formula', '<c r="A1"><f t="dataTable">B1</f></c>', 'xlsx_unsupported_formula'),
        ('formula-flags', '<c r="A1"><f ca="1">B1</f></c>', 'xlsx_unsupported_formula'),
        ('array-without-ref', '<c r="A1"><f t="array">B1</f></c>', 'xlsx_invalid_formula'),
        ('shared-without-anchor', '<c r="A1"><f t="shared" si="1"/></c>', 'xlsx_invalid_formula'),
        ('shared-missing-index', '<c r="A1"><f t="shared" ref="A1">B1</f></c>', 'xlsx_invalid_formula'),
        ('shared-outside-range', '<c r="A1"><f t="shared" si="1" ref="A1">B1</f></c><c r="B1"><f t="shared" si="1"/></c>', 'xlsx_invalid_formula'),
        ('shared-duplicate-anchor', '<c r="A1"><f t="shared" si="1" ref="A1:B1">B1</f></c><c r="B1"><f t="shared" si="1" ref="B1">C1</f></c>', 'xlsx_invalid_formula'),
        ('formula-ref-normal', '<c r="A1"><f ref="A1">B1</f></c>', 'xlsx_invalid_formula'),
        ('formula-array-wrong-anchor', '<c r="B1"><f t="array" ref="A1:B1">A2</f></c>', 'xlsx_invalid_formula'),
        ('formula-shared-ref-follower', '<c r="A1"><f t="shared" si="1" ref="A1:B1">B1</f></c><c r="B1"><f t="shared" si="1" ref="B1"/></c>', 'xlsx_invalid_formula'),
        ('formula-cache-empty-is-distinct', '<c r="A1"><f>B1</f><v/></c>', 'xlsx_invalid_cell'),
        ('invalid-boolean-empty', '<c r="A1" t="b"><v/></c>', 'xlsx_invalid_cell'),
        ('invalid-error-empty', '<c r="A1" t="e"><v/></c>', 'xlsx_invalid_cell'),
        ('invalid-date-empty', '<c r="A1" t="d"><v/></c>', 'xlsx_invalid_cell'),
        ('mixed-rich-simple', '<c r="A1" t="inlineStr"><is><t>A</t><r><t>B</t></r></is></c>', 'xlsx_unsupported_cell'),
    ]:
        add(name, parts(one_cell(xml)), error=error)
    for name, replacement, error in [
        ('row-order', worksheet('<row r="2"/><row r="1"/>'), 'xlsx_invalid_cell'),
        ('duplicate-sheet-data', f'<worksheet xmlns="{MAIN}"><sheetData/><sheetData/></worksheet>', 'xlsx_invalid_workbook'),
        ('bad-dimension', worksheet(dimension='B2:A1'), 'xlsx_invalid_cell'),
        ('bad-merge', worksheet(extra='<mergeCells><mergeCell ref="XFE1"/></mergeCells>'), 'xlsx_invalid_cell'),
        ('duplicate-merge', worksheet(extra='<mergeCells><mergeCell ref="A1:B1"/><mergeCell ref="A1:B1"/></mergeCells>'), 'xlsx_invalid_workbook'),
        ('strict', worksheet().replace(MAIN, 'http://purl.oclc.org/ooxml/spreadsheetml/main'), 'xlsx_unsupported_format'),
        ('wrong-namespace', worksheet().replace(MAIN, 'urn:fake'), 'xlsx_unsupported_format'),
        ('alternate-content', worksheet(extra='<mc:AlternateContent xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"/>'), 'xlsx_unsupported_format'),
        ('xml-dtd', '<!DOCTYPE worksheet [<!ENTITY x "sensitive">]>' + worksheet(), 'xlsx_invalid_xml'),
        ('xml-external-dtd', '<!DOCTYPE worksheet SYSTEM "file:///sensitive">' + worksheet(), 'xlsx_invalid_xml'),
        ('xml-custom-entity', worksheet().replace('<sheetData>', '<sheetData>&custom;'), 'xlsx_invalid_xml'),
        ('xml-duplicate-attribute', worksheet().replace('<sheetData>', '<sheetData x="a" x="b">'), 'xlsx_invalid_xml'),
        ('xml-truncated', worksheet()[:-3], 'xlsx_invalid_xml'),
        ('xml-latin1-declaration', '<?xml version="1.0" encoding="ISO-8859-1"?>' + worksheet(), 'xlsx_unsupported_format'),
    ]:
        p = parts(); p['xl/worksheets/sheet1.xml'] = replacement
        add(name, p, error=error)
    p = parts(); p['xl/worksheets/sheet1.xml'] = b'\xff'
    add('invalid-utf8', p, error='xlsx_invalid_xml')
    for missing in ['[Content_Types].xml', '_rels/.rels', 'xl/workbook.xml', 'xl/_rels/workbook.xml.rels', 'xl/worksheets/sheet1.xml']:
        p = parts(); del p[missing]
        add('missing-' + str(len(out)), p, error='xlsx_invalid_workbook')
    for replacement in ['name="a/b"', 'name=""', 'name="' + 'n'*32 + '"', 'name="Sheet1" state="invalid"']:
        p = parts(); p['xl/workbook.xml'] = p['xl/workbook.xml'].replace('name="Sheet1"', replacement)
        add('bad-sheet-name-' + str(len(out)), p, error='xlsx_invalid_workbook')
    p = parts(); p['xl/workbook.xml'] = p['xl/workbook.xml'].replace('<sheets>', '<workbookPr date1904="notbool"/><sheets>')
    add('bad-date-system', p, error='xlsx_invalid_workbook')
    for st in [f'<styleSheet xmlns="{MAIN}"><cellXfs><xf numFmtId="164"/></cellXfs></styleSheet>',
               f'<styleSheet xmlns="{MAIN}"><cellXfs count="2"><xf numFmtId="0"/></cellXfs></styleSheet>',
               f'<styleSheet xmlns="{MAIN}"><cellXfs/></styleSheet>']:
        add('bad-styles-' + str(len(out)), parts(styles=st), error='xlsx_invalid_workbook')
    sst = f'<sst xmlns="{MAIN}" uniqueCount="2"><si><t>x</t></si></sst>'
    add('bad-shared-count', parts(shared=sst), error='xlsx_invalid_workbook')
    p = parts(); p['[Content_Types].xml'] = p['[Content_Types].xml'].replace('sheet.main+xml', 'sheet.macroEnabled.main+xml')
    add('macro-content-type', p, error='xlsx_active_content')
    for extra in ['xl/vbaProject.bin', 'xl/connections.xml', 'xl/externalLinks/externalLink1.xml', 'xl/activeX/activeX1.bin']:
        p = parts(); p[extra] = '<x/>' if extra.endswith('.xml') else b'0'
        add('active-' + str(len(out)), p, error='xlsx_active_content')
    p = parts(); p['xl/_rels/workbook.xml.rels'] = p['xl/_rels/workbook.xml.rels'].replace('</Relationships>', rel('remote', 'hyperlink', 'https://example.invalid', ' TargetMode="External"') + '</Relationships>')
    add('external-relationship', p, error='xlsx_external_relationship')
    for target in ['../worksheets/sheet1.xml', '/xl//worksheets/sheet1.xml', 'worksheets/%73heet1.xml', 'file:///secret', 'worksheets\\sheet1.xml']:
        p = parts(); p['xl/_rels/workbook.xml.rels'] = p['xl/_rels/workbook.xml.rels'].replace('worksheets/sheet1.xml', target)
        add('bad-target-' + str(len(out)), p, error='xlsx_invalid_workbook')
    p = parts(); p['xl/_rels/workbook.xml.rels'] = p['xl/_rels/workbook.xml.rels'].replace('</Relationships>', rel('r1', 'worksheet', 'worksheets/sheet1.xml') + '</Relationships>')
    add('duplicate-relationship-id', p, error='xlsx_invalid_workbook')
    p = parts(); p['xl/_rels/workbook.xml.rels'] = p['xl/_rels/workbook.xml.rels'].replace('Target="worksheets/sheet1.xml"', 'Target="worksheets/sheet1.xml" TargetMode="other"')
    add('invalid-relationship-mode', p, error='xlsx_invalid_workbook')
    for name in ['../escape.bin', '/absolute.bin', 'xl/../alias.bin', 'xl\\alias.bin', 'xl/%61lias.bin']:
        p = parts(); p[name] = b'0'
        add('bad-path-' + str(len(out)), p, error='xlsx_invalid_zip')
    p = list(parts().items()); p.append(p[-1])
    add('duplicate-zip-name', p, error='xlsx_invalid_zip')
    base = pack(parts())
    for name, source in [('zip-truncated', base[:-1]), ('zip-trailing', base+b'x'), ('zip-prefix', b'x'+base), ('not-zip', b'%PDF-1.7')]:
        add(name, source=source, error='xlsx_invalid_zip')
    central = base.find(b'PK\x01\x02')
    eocd = base.rfind(b'PK\x05\x06')
    for name, mutation, error in [
        ('zip-multidisk', [(eocd+4, struct.pack('<H', 1))], 'xlsx_unsupported_zip'),
        ('zip-encrypted', [(central+8, struct.pack('<H', 1)), (6, struct.pack('<H', 1))], 'xlsx_unsupported_zip'),
        ('zip-unsupported-method', [(central+10, struct.pack('<H', 12)), (8, struct.pack('<H', 12))], 'xlsx_unsupported_zip'),
        ('zip-local-name', [(30, b'X')], 'xlsx_invalid_zip'),
        ('zip-local-crc', [(14, b'\0\0\0\0')], 'xlsx_invalid_zip'),
        ('zip-overlap', [(central+42, struct.pack('<I', 1))], 'xlsx_invalid_zip'),
        ('zip-crc-real', [(30+len('_rels/.rels'), b'X')], 'xlsx_invalid_zip'),
        ('zip-symlink', [(central+38, struct.pack('<I', 0o120777 << 16))], 'xlsx_unsupported_zip'),
        ('zip-local-version', [(4, struct.pack('<H', 10))], 'xlsx_invalid_zip'),
        ('zip-version', [(central+6, struct.pack('<H', 45)), (4, struct.pack('<H', 45))], 'xlsx_unsupported_zip'),
        ('zip-central-count', [(eocd+10, struct.pack('<H', 4)), (eocd+8, struct.pack('<H', 4))], 'xlsx_invalid_zip'),
        ('zip-stored-deflate-flags', [(central+8, struct.pack('<H', 2)), (6, struct.pack('<H', 2))], 'xlsx_unsupported_zip'),
    ]:
        b = bytearray(base)
        for at, data in mutation:
            b[at:at+len(data)] = data
        add(name, source=bytes(b), error=error)
    for ident in [1, 0x7075]:
        add('zip-extra-' + str(ident), source=descriptor_pack(parts(), extra=struct.pack('<HH', ident, 0)), error='xlsx_unsupported_zip')
    for name, at in [('descriptor-crc', 14), ('descriptor-size', 18)]:
        b = bytearray(descriptor_pack(parts()))
        b[at:at+4] = struct.pack('<I', 1)
        add(name, source=bytes(b), error='xlsx_invalid_zip')
    b = bytearray(pack(parts(), zipfile.ZIP_DEFLATED))
    central = b.find(b'PK\x01\x02'); eocd = b.rfind(b'PK\x05\x06')
    first_size = struct.unpack_from('<I', b, central+20)[0]
    end = 30 + len('_rels/.rels') + first_size
    # Añadir padding dentro del tamaño comprimido (CRC del contenido sigue válido).
    b[end:end] = b'X'; central += 1; eocd += 1
    struct.pack_into('<I', b, 18, first_size+1)
    struct.pack_into('<I', b, central+20, first_size+1)
    pos = central
    for i in range(len(parts())):
        if i:
            offset = struct.unpack_from('<I', b, pos+42)[0]
            struct.pack_into('<I', b, pos+42, offset+1)
        nl, el, cl = struct.unpack_from('<HHH', b, pos+28)
        pos += 46+nl+el+cl
    struct.pack_into('<I', b, eocd+16, central)
    add('zip-deflate-hidden-padding', source=bytes(b), error='xlsx_invalid_zip')
    # Casos añadidos tras revisión final1: no renombrar ni regenerar expectativas
    # anteriores. XML admite todos estos separadores, no solamente un espacio.
    for label, separator in [('tab', '\t'), ('cr', '\r'), ('lf', '\n'), ('mixed', ' \t')]:
        for encoding in ['UTF-8', 'ISO-8859-1']:
            p = parts(); p['xl/worksheets/sheet1.xml'] = f'<?xml{separator}version="1.0" encoding="{encoding}"?>' + worksheet()
            add(f'xml-separator-{label}-{encoding}', p, error='xlsx_unsupported_format' if encoding != 'UTF-8' else None)
    for n in [64, 65]:
        for label, text in [('text', 'text'), ('cdata-comment', '<![CDATA[text]]><!--comment-->')]:
            p = parts(); p['docProps/depth.xml'] = '<x>'*n + text + '</x>'*n
            add(f'depth-{n}-{label}', p, error='xlsx_xml_limit' if n>64 else None)
    # El presupuesto de texto es lógico: ST_Xstring puede usar siete bytes de
    # lexema por cada byte resultante. No subir el cap de texto para admitirlo.
    for mode in ['inline', 'shared', 'str']:
        for n in [MAX_TEXT, MAX_TEXT+1]:
            text = '_x0041_'*n
            if mode == 'inline':
                p = parts(one_cell('<c r="A1" t="inlineStr"><is><t>'+text+'</t></is></c>'))
            elif mode == 'shared':
                p = parts(one_cell('<c r="A1" t="s"><v>0</v></c>'), shared=f'<sst xmlns="{MAIN}"><si><t>{text}</t></si></sst>')
            else:
                p = parts(one_cell('<c r="A1" t="str"><v>'+text+'</v></c>'))
            add(f'logical-escaped-{mode}-{n}', p, book([sheet([cell(cached=value('text','A'*n))], observed='A1')]), error='xlsx_structure_limit' if n>MAX_TEXT else None)
    assert len({c['name'] for c in out}) == len(out)
    return out


def manifest():
    return [dict(name=c['name'], source_bytes=len(c['source']), source_sha256=hashlib.sha256(c['source']).hexdigest(),
                 expected_sha256=hashlib.sha256(compact(c['expected'])).hexdigest() if not c['error'] else None,
                 error=c['error']) for c in cases()]


if __name__ == '__main__':
    print(json.dumps(manifest(), ensure_ascii=False, indent=2))
