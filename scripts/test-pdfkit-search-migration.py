#!/usr/bin/env python3
"""Barrera15 sobre una fixture y cliente14 reales conservados antes del DDL."""
import argparse
import ast
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import uuid

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent.parent


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--profile', choices=['debug', 'release'], required=True)
    ap.add_argument('--fixture', type=Path, required=True)
    ap.add_argument('--old-bin', type=Path, required=True)
    ap.add_argument('--output', type=Path, required=True)
    args = ap.parse_args()
    out, base = args.output.resolve(), Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True)
    assert out.is_relative_to(base) and not out.exists()
    out.mkdir()
    spec = importlib.util.spec_from_file_location('helpers', ROOT/'scripts/test-xlsx-search-migration.py')
    m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
    for label, source in [('old', args.old_bin.resolve()), ('new', ROOT/'target'/args.profile)]:
        folder = out/label; folder.mkdir()
        for name in ['noteka', 'noteka-mcp']: shutil.copy2(source/name, folder/name)
    old, new, ws = out/'old/noteka', out/'new/noteka', out/'workspace'
    shutil.copytree(args.fixture.resolve(strict=True), ws)
    assert m.call(old, ws, 'workspace', 'open', {})['schema_version'] == 14
    entity = m.call(old, ws, 'entity', 'create', dict(kind='note', title='Unknown future block'))
    with sqlite3.connect(ws/'noteka.sqlite') as c:
        c.execute('UPDATE documents SET schema_version=999,json=? WHERE entity_id=?', [' {"schema_version":999,"blocks":[{"id":"unknown","type":"future","raw":{"keep":1.00}}]} ', entity['id']])
        row = c.execute('SELECT id FROM extraction_jobs ORDER BY id LIMIT 1').fetchone()
        raw = ' {"version":2,"chunk_bytes":4096,"future":{"keep":1.00}} '
        h = hashlib.sha256()
        for part in [b'noteka.extraction.config.v1', json.dumps(json.loads(raw), sort_keys=True, separators=(',', ':')).encode()]: h.update(len(part).to_bytes(8, 'big')); h.update(part)
        c.execute("INSERT INTO extraction_jobs SELECT ?,attachment_id,source_sha256,source_bytes,'future_native',2,?,?,'queued',1,0,created_at,updated_at,NULL,NULL,NULL,NULL FROM extraction_jobs WHERE id=?", [str(uuid.uuid4()), raw, h.hexdigest(), row[0]])
        native = c.execute("SELECT id FROM extraction_jobs WHERE extractor='pdfkit_text'").fetchone()[0]
    before, ddl = m.snapshot(ws), m.schema(ws)
    index = m.call(old, ws, 'extraction', 'index', dict(action='get', limit=1))
    query = m.call(old, ws, 'extraction', 'search', dict(query='evidencia', limit=1))
    pages = m.call(old, ws, 'extraction', 'pdfkit-pages', dict(id=native, limit=1))
    originals = m.files(ws/'attachments'); physical = m.files(ws)
    m.call(new, ws, 'extraction', 'index', dict(action='get'), 'unsupported_workspace_version')
    assert m.files(ws) == physical
    assert m.call(new, ws, 'workspace', 'open', {})['schema_version'] == 15
    after = m.snapshot(ws)
    assert set(after) == set(before) and m.schema(ws) == ddl
    for table, rows in before.items():
        if table != 'workspace_meta': assert rows == after[table], table
    meta = [tuple(14 if i == 1 else v for i, v in enumerate(r)) for r in ast.literal_eval(after['workspace_meta'])]
    assert repr(meta) == before['workspace_meta'] and m.files(ws/'attachments') == originals
    for expected, action, payload in [(index, 'index', dict(action='get', limit=1)), (query, 'search', dict(query='evidencia', limit=1))]:
        updated = json.loads(json.dumps(expected)); updated['coverage']['unsupported_generations'] -= 1; updated['coverage']['unindexed_supported_generations'] += 1
        assert m.call(new, ws, 'extraction', action, payload) == updated
        if expected['next_cursor']:
            m.call(new, ws, 'extraction', action, {**payload, 'cursor': expected['next_cursor']})
    assert m.call(new, ws, 'extraction', 'pdfkit-pages', dict(id=native, limit=1)) == pages
    backups = list(out.glob('noteka-pre-migration-*')); assert len(backups) == 1
    restored = out/'restored14'
    assert m.call(old, ws, 'workspace', 'restore', dict(backup=str(backups[0]), destination=str(restored)))['schema_version'] == 14
    assert m.snapshot(restored) == before and m.files(restored/'attachments') == originals
    physical = m.files(ws); m.call(old, ws, 'workspace', 'open', {}, 'unsupported_workspace_version')
    admission = subprocess.run([str(old.with_name('noteka-mcp')), '--workspace', str(ws)], input=b'', capture_output=True, timeout=8)
    assert admission.returncode and not admission.stdout and b'UnsupportedWorkspaceVersion' in admission.stderr and m.files(ws) == physical
    for timing in ['BEFORE', 'AFTER']:
        target = out/timing; shutil.copytree(restored, target)
        with sqlite3.connect(target/'noteka.sqlite') as c: c.execute(f"CREATE TRIGGER fail15 {timing} UPDATE OF schema_version ON workspace_meta WHEN NEW.schema_version=15 BEGIN SELECT RAISE(ABORT,'injected'); END")
        prior = m.snapshot(target); m.call(new, target, 'workspace', 'open', {}, 'storage_error'); assert m.snapshot(target) == prior
    future = out/'future16'; shutil.copytree(ws, future)
    with sqlite3.connect(future/'noteka.sqlite') as c: c.executescript('UPDATE workspace_meta SET schema_version=16;PRAGMA user_version=16;PRAGMA wal_checkpoint(TRUNCATE);')
    physical = m.files(future); m.call(new, future, 'workspace', 'open', {}, 'unsupported_workspace_version'); assert m.files(future) == physical
    a, b = [m.call(binary, ws, 'operations', 'describe', {}) for binary in [old, new]]
    for key in ['inputs', 'outputs']:
        assert set(a[key]) == set(b[key]) and len(b[key]) == 54
        assert [n for n in a[key] if a[key][n] != b[key][n]] == (['extraction.index', 'extraction.search'] if key == 'inputs' else ['extraction.search'])
    assert a['error_schema'] == b['error_schema'] and a['batch'] == b['batch']
    report = dict(profile=args.profile, old_sha256=hashlib.sha256(old.read_bytes()).hexdigest(), tables=len(before), rows_schema_originals_exact=True, old_selections_cursors_hashes_exact=True, coverage_only_recognizes_native=True, native_reader_exact=True, unknown_data_preserved=True, backup14_old_client_exact=True, old_future_admission_before_writes=True, rollback=['before', 'after'], contracts_52_exact=True, batch9_exact=True, helpers_absent=True)
    (out/'results.json').write_text(json.dumps(report, indent=2)+'\n'); print(json.dumps(report))


if __name__ == '__main__': main()
