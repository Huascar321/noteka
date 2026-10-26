#!/usr/bin/env python3
"""Recuperación v13 real e independiente; helpers ausentes en ambos clientes."""
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
    base = Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True)
    out = args.output.resolve()
    assert out.is_relative_to(base) and not out.exists()
    out.mkdir()
    spec = importlib.util.spec_from_file_location('migration_helpers', ROOT/'scripts/test-xlsx-search-migration.py')
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    for label, source in [('old', args.old_bin.resolve()), ('new', ROOT/'target'/args.profile)]:
        runtime = out/label
        runtime.mkdir()
        for name in ['noteka', 'noteka-mcp']:
            shutil.copy2(source/name, runtime/name)
    old, new = out/'old/noteka', out/'new/noteka'
    ws = out/'workspace'
    shutil.copytree(args.fixture.resolve(strict=True), ws)
    # Confirmar el baseline real antes de añadir una extensión sintética.
    assert m.call(old, ws, 'workspace', 'open', {})['schema_version'] == 13
    entity = m.call(old, ws, 'entity', 'create', dict(kind='note', title='Unknown fields migration'))
    with sqlite3.connect(ws/'noteka.sqlite') as c:
        c.execute('UPDATE documents SET schema_version=999,json=? WHERE entity_id=?',
                  [' {"schema_version":999,"blocks":[{"id":"future","type":"extension","raw":{"x":1.00}}]} ', entity['id']])
        job = c.execute('SELECT id FROM extraction_jobs ORDER BY id LIMIT 1').fetchone()
        if job:
            raw = ' {"version":2,"chunk_bytes":4096,"future":{"keep":1.00}} '
            h = hashlib.sha256()
            encoded = json.dumps(json.loads(raw), sort_keys=True, separators=(',', ':')).encode()
            for part in [b'noteka.extraction.config.v1', encoded]:
                h.update(len(part).to_bytes(8, 'big')); h.update(part)
            # Job futuro separado: no invalidar una selección FTS existente.
            c.execute("INSERT INTO extraction_jobs SELECT ?,attachment_id,source_sha256,source_bytes,'future_pdf',2,?,?,'queued',1,0,created_at,updated_at,NULL,NULL,NULL,NULL FROM extraction_jobs WHERE id=?", [str(uuid.uuid4()),raw,h.hexdigest(),job[0]])

    before = m.snapshot(ws)
    prior_schema = m.schema(ws)
    originals = m.files(ws/'attachments')
    saved = m.files(ws)
    m.call(new, ws, 'extraction', 'index', dict(action='get'), 'unsupported_workspace_version')
    assert m.files(ws) == saved
    assert m.call(new, ws, 'workspace', 'open', {})['schema_version'] == 14
    after = m.snapshot(ws)
    assert set(after)-set(before) == {'extraction_pdfkit_generations', 'extraction_pdfkit_chunks'}
    for table, rows in before.items():
        if table != 'workspace_meta':
            assert rows == after[table], table
    schema = m.schema(ws)
    assert all(row in schema for row in prior_schema)
    assert m.files(ws/'attachments') == originals
    backups = list(out.glob('noteka-pre-migration-*'))
    assert len(backups) == 1
    restored = out/'restored-v13'
    assert m.call(old, ws, 'workspace', 'restore', dict(backup=str(backups[0]), destination=str(restored)))['schema_version'] == 13
    assert m.snapshot(restored) == before
    assert m.files(restored/'attachments') == originals
    saved = m.files(ws)
    m.call(old, ws, 'workspace', 'open', {}, 'unsupported_workspace_version')
    admission = subprocess.run([str(old.with_name('noteka-mcp')), '--workspace', str(ws)], input=b'', capture_output=True, timeout=8)
    assert admission.returncode and not admission.stdout and b'UnsupportedWorkspaceVersion' in admission.stderr
    assert m.files(ws) == saved
    for timing in ['BEFORE', 'AFTER']:
        target = out/timing.lower()
        shutil.copytree(restored, target)
        with sqlite3.connect(target/'noteka.sqlite') as c:
            c.execute(f"CREATE TRIGGER fail14 {timing} UPDATE OF schema_version ON workspace_meta WHEN NEW.schema_version=14 BEGIN SELECT RAISE(ABORT,'injected'); END")
        saved = m.snapshot(target)
        m.call(new, target, 'workspace', 'open', {}, 'storage_error')
        assert m.snapshot(target) == saved
        with sqlite3.connect(target/'noteka.sqlite') as c:
            assert c.execute('PRAGMA user_version').fetchone()[0] == 13
    future = out/'future15'
    shutil.copytree(ws, future)
    with sqlite3.connect(future/'noteka.sqlite') as c:
        c.executescript('UPDATE workspace_meta SET schema_version=15;PRAGMA user_version=15;PRAGMA wal_checkpoint(TRUNCATE);')
    saved = m.files(future)
    m.call(new, future, 'workspace', 'open', {}, 'unsupported_workspace_version')
    assert m.files(future) == saved
    old_catalog = m.call(old, ws, 'operations', 'describe', {})
    new_catalog = m.call(new, ws, 'operations', 'describe', {})
    for key in ['inputs', 'outputs']:
        assert len(new_catalog[key]) == 54
        assert set(new_catalog[key])-set(old_catalog[key]) == {'extraction.pdfkit_pages'}
        assert all(new_catalog[key][k] == v for k, v in old_catalog[key].items())
    assert new_catalog['error_schema'] == old_catalog['error_schema']
    report = dict(profile=args.profile, old_sha256=hashlib.sha256(old.read_bytes()).hexdigest(),
                  old_tables=len(before), new_tables=len(after), original_rows_exact=True,
                  unknown_document_config_preserved=True, prior_sql_exact=True, originals_exact=True,
                  backup_old_client_exact=True, rollback=['before-version', 'after-version'],
                  old_and_future_admission_no_writes=True, helpers_absent=True, prior_53_contracts_exact=True)
    (out/'results.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
