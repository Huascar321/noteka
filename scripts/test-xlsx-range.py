#!/usr/bin/env python3
"""Rangos contrastados con golden XLSX independiente y cliente16 conservado.

Sólo fixtures sintéticas en NOTEKA_TEST_TMP; todos los procesos CLI sin red.
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

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent.parent


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--profile', choices=['debug', 'release'], required=True)
    ap.add_argument('--old-bin', type=Path, required=True)
    ap.add_argument('--output', type=Path, required=True)
    args = ap.parse_args()
    allowed = Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True)
    out = args.output.resolve()
    assert out != allowed and out.is_relative_to(allowed) and not out.exists()
    assert not out.is_relative_to(ROOT)
    out.mkdir()
    new_folder = out/'new'; new_folder.mkdir()
    new = new_folder/'noteka'; shutil.copy2(ROOT/'target'/args.profile/'noteka', new)
    # No helper sibling junto al cliente nuevo: lecturas/restore no pueden parsear.
    old_folder = out/'old'; old_folder.mkdir()
    for name in ['noteka', 'noteka-xlsx-worker']:
        shutil.copy2(args.old_bin.resolve(strict=True)/name, old_folder/name)
    old = old_folder/'noteka'; ws = out/'workspace'
    calls = 0

    def envelope(cli, operation, data, workspace=ws):
        nonlocal calls
        command = [str(cli), '--workspace', str(workspace), '--json', *[s.replace('_', '-') for s in operation.split('.')]]
        if sys.platform == 'darwin':
            command = ['/usr/bin/sandbox-exec', '-p', '(version 1)(allow default)(deny network*)', *command]
        result = subprocess.run(command, input=json.dumps(data).encode(), capture_output=True, timeout=15, cwd=out)
        assert not result.stderr, result.stderr[:200]
        reply = json.loads(result.stdout)
        assert reply['api_version'] == 1 and reply['ok'] == (result.returncode == 0)
        assert len(result.stdout) <= 2_097_152
        calls += 1
        return reply

    def call(cli, operation, data, workspace=ws):
        reply = envelope(cli, operation, data, workspace)
        assert reply['ok'], (operation, reply)
        return reply['data']

    def snapshot():
        with sqlite3.connect(ws/'noteka.sqlite') as c:
            c.execute('PRAGMA query_only=ON')
            ddl = c.execute('SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name').fetchall()
            rows = {name: sorted(map(repr, c.execute('SELECT * FROM "'+name.replace('"','""')+'"').fetchall()))
                    for name, in c.execute("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name")}
        files = {f.name: hashlib.sha256(f.read_bytes()).hexdigest() for f in (ws/'attachments').iterdir()}
        return dict(ddl=ddl, rows=rows, files=files)

    assert call(old, 'workspace.create', dict(timezone='UTC'))['schema_version'] == 16
    ctl = call(old, 'extraction.control', dict(action='get'))
    call(old, 'extraction.control', dict(action='set', paused=False, expected_revision=ctl['revision']))
    spec = importlib.util.spec_from_file_location('golden', ROOT/'tests/fixtures/xlsx/generate.py')
    g = importlib.util.module_from_spec(spec); spec.loader.exec_module(g)
    records, saved, bounded_errors = [], [], []
    for case in g.cases():
        if case['error']:
            continue
        source = out/(case['name']+'.xlsx'); source.write_bytes(case['source'])
        attachment = call(old, 'attachment.import', dict(source_path=str(source)))
        job = call(old, 'extraction.enqueue', dict(attachment_id=attachment['id'], extractor='xlsx_cells'))['job']
        done = call(old, 'extraction.run', dict(id=job['id']))['job']
        assert done['state'] == 'succeeded'
        before = snapshot()
        expected = case['expected']
        global_cells = [dict(ordinal=n, locator=dict(kind='xlsx_cell', sheet_id=sid, reference=c['reference']), cell=c)
                        for n, (sid, c) in enumerate((s['id'], c) for s in expected['sheets'] for c in s['cells'])]
        for sheet in expected['sheets']:
            if sheet['cells']:
                first = min(c['row'] for c in sheet['cells']); last = max(c['row'] for c in sheet['cells'])
                left = min(c['column'] for c in sheet['cells']); right = min(left+63, max(c['column'] for c in sheet['cells']))
            else:
                first = last = left = right = 1
            def a1(row, col):
                letters = ''
                while col:
                    col, digit = divmod(col-1, 26); letters = chr(65+digit)+letters
                return letters+str(row)
            region = a1(first, left)+':'+a1(last, right)
            chosen = [i for i in global_cells if i['locator']['sheet_id'] == sheet['id'] and first <= i['cell']['row'] <= last and left <= i['cell']['column'] <= right]
            headers = [dict(column=c, reference=a1(first, c), item=next((i for i in chosen if i['cell']['row'] == first and i['cell']['column'] == c), None)) for c in range(left, right+1)]
            input = dict(id=job['id'], sheet_id=sheet['id'], range=region, header_row=first, limit=64)
            probe = envelope(new, 'extraction.xlsx_range', input)
            if not probe['ok']:
                assert probe['error']['details']['reason'] == 'response_size_limit', probe
                bounded_errors.append(dict(case=case['name'], sheet=sheet['id'], reason='response_size_limit', limit=64))
                input['limit'] = 1
            pages, cursor, items = 0, None, []
            while True:
                page = call(new, 'extraction.xlsx_range', {**input, 'cursor': cursor})
                assert page['items'] == chosen[len(items):len(items)+input['limit']]
                assert page['headers'] == headers
                assert page['matching_cell_count'] == len(chosen)
                assert page['generation'] == done['generation']
                assert page['sheet'] == {k: v for k, v in sheet.items() if k != 'cells'} | dict(cell_count=len(sheet['cells']))
                assert page['manifest']['cell_count'] == len(global_cells)
                assert page['manifest']['styles'] == expected['styles']
                assert page['manifest']['limitations'] == expected['limitations']
                assert page['covers_range'] == (not cursor and len(chosen) <= input['limit'])
                assert page['covers_generation'] == (not cursor and len(chosen) <= input['limit'] and len(chosen) == len(global_cells))
                if page['next_cursor']:
                    wrong = envelope(new, 'extraction.xlsx_range', {**input, 'header_row': None, 'cursor': page['next_cursor']})
                    assert not wrong['ok'] and wrong['error']['details']['reason'] == 'cursor_scope_mismatch'
                pages += 1; items += page['items']; cursor = page['next_cursor']
                if not cursor:
                    break
            assert items == chosen
            no_headers = call(new, 'extraction.xlsx_range', {**input, 'header_row': None})
            assert no_headers['headers'] == []
            records.append(dict(case=case['name'], sheet=sheet['id'], range=region, pages=pages, cells=len(items)))
            saved.append(input)
        assert snapshot() == before
        assert call(old, 'extraction.xlsx_cells', dict(id=job['id'], limit=1)) == call(new, 'extraction.xlsx_cells', dict(id=job['id'], limit=1))
        assert source.read_bytes() == case['source']
    # Unknown document bytes remain opaque while both clients read current data.
    entity = call(old, 'entity.create', dict(kind='note', title='Future preserved'))
    with sqlite3.connect(ws/'noteka.sqlite') as c:
        c.execute('UPDATE documents SET schema_version=999,json=? WHERE entity_id=?', (' {"schema_version":999,"blocks":[{"type":"future","raw":{"keep":1.00}}]} ', entity['id']))
    before = snapshot()
    for input in saved[:3]:
        call(new, 'extraction.xlsx_range', input)
    assert snapshot() == before
    backup = out/'backup'; call(new, 'workspace.backup', dict(destination=str(backup)))
    restored = out/'restored'; call(old, 'workspace.restore', dict(backup=str(backup), destination=str(restored)))
    for input in saved[:3]:
        got = call(new, 'extraction.xlsx_range', input, restored)
        assert got['items'] == call(new, 'extraction.xlsx_range', input)['items']
    assert snapshot() == before
    result = dict(profile=args.profile, cases=len({r['case'] for r in records}), sheets=len(records), pages=sum(r['pages'] for r in records), calls=calls, helper_free_reads=True, prior_client_compatible=True, backup_restored_by_prior_client=True, pure_read=True, bounded_errors=bounded_errors, records=records)
    (out/'results.json').write_text(json.dumps(result, ensure_ascii=False, indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k != 'records'}))


if __name__ == '__main__':
    main()
