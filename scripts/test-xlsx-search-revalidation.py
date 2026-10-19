#!/usr/bin/env python3
"""Fixtures SQL tras pérdida de auditoría; nunca son clientes viejos independientes."""
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
import tempfile
import uuid

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent.parent


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT / path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def construct_fixture(seed, destination, version):
    """DDL 001..version + filas sintéticas actuales, sin rebajar una DB de usuario."""
    destination.mkdir()
    shutil.copytree(seed / 'attachments', destination / 'attachments')
    with sqlite3.connect(destination / 'noteka.sqlite') as db:
        for number in range(1, version + 1):
            db.executescript((ROOT / f'crates/noteka-core/migrations/{number:03}.sql').read_text())
        db.execute('ATTACH DATABASE ? AS seed', [str(seed / 'noteka.sqlite')])
        triggers = db.execute("SELECT name,sql FROM sqlite_schema WHERE type='trigger'").fetchall()
        for name, _ in triggers:
            db.execute(f'DROP TRIGGER "{name}"')
        tables = db.execute("SELECT name,sql FROM sqlite_schema WHERE type='table' ORDER BY name").fetchall()
        shadows = {row[1] for row in db.execute('PRAGMA main.table_list') if row[2] == 'shadow'}
        for name, ddl in tables:
            if name in shadows:
                continue  # SQLite protege shadows: cargar la tabla virtual por su API SQL.
            db.execute(f'DELETE FROM "{name}"')
            columns = [row[1] for row in db.execute(f'PRAGMA table_info("{name}")')]
            if 'CREATE VIRTUAL TABLE' in ddl.upper():
                columns.insert(0, 'rowid')
            values = ','.join(str(version) if name == 'workspace_meta' and col == 'schema_version'
                              else f'"{col}"' for col in columns)
            names = ','.join(f'"{col}"' for col in columns)
            db.execute(f'INSERT INTO "{name}" ({names}) SELECT {values} FROM seed."{name}"')
        for _, ddl in triggers:
            db.execute(ddl)
        assert db.execute('PRAGMA user_version').fetchone()[0] == version
        assert db.execute('PRAGMA integrity_check').fetchone()[0] == 'ok'
        assert not db.execute('PRAGMA foreign_key_check').fetchall()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--profile', choices=['debug', 'release'], default='debug')
    args = parser.parse_args()
    target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target')).resolve() / args.profile
    directory = Path(tempfile.mkdtemp(prefix='revalidation-migrate-', dir=os.environ['NOTEKA_TEST_TMP']))
    support = load('support', 'scripts/test-xlsx-search-migration.py')
    pdf = load('pdf', 'tests/fixtures/pdf/generate.py')
    xlsx = load('xlsx', 'tests/fixtures/xlsx/generate.py')
    seed = directory / 'seed-v13'
    cli = target / 'noteka'
    call = lambda ws, group, action, payload, error=None: support.call(cli, ws, group, action, payload, error)
    call(seed, 'workspace', 'create', dict(timezone='UTC'))
    ctl = call(seed, 'extraction', 'control', dict(action='get'))
    call(seed, 'extraction', 'control', dict(action='set', paused=False, expected_revision=ctl['revision']))
    jobs = []
    for name, raw, extractor in [
        ('source.txt', ('evidencia café alfa\n' * 500).encode(), 'utf8_text'),
        ('source.pdf', pdf.serialize(pdf.document([b'BT /F1 12 Tf (evidencia PDF alfa) Tj ET'])), 'pdf_text'),
        ('source.xlsx', xlsx.pack(xlsx.parts(xlsx.one_cell('<c r="A1" t="str"><v>evidencia XLSX alfa</v></c>'))), 'xlsx_cells'),
    ]:
        path = directory / name
        path.write_bytes(raw)
        attachment = call(seed, 'attachment', 'import', dict(source_path=str(path)))
        job = call(seed, 'extraction', 'enqueue', dict(attachment_id=attachment['id'], extractor=extractor))['job']
        job = call(seed, 'extraction', 'run', dict(id=job['id']))['job']
        assert job['state'] == 'succeeded'
        jobs.append(job)
        if extractor != 'xlsx_cells':
            state = call(seed, 'extraction', 'index', dict(action='get'))
            call(seed, 'extraction', 'index', dict(action='put', job_id=job['id'],
                 expected_generation_id=job['generation']['id'], expected_revision=state['revision']))
    query = call(seed, 'extraction', 'search', dict(query='evidencia', limit=1))
    assert query['next_cursor']
    note = call(seed, 'entity', 'create', dict(kind='note', title='Synthetic unknown document'))
    call(seed, 'extraction', 'retry', dict(id=jobs[0]['id'], expected_revision=jobs[0]['revision']))
    with sqlite3.connect(seed / 'noteka.sqlite') as db:
        db.execute('UPDATE documents SET json=? WHERE entity_id=?',
                   [' {"schema_version":999,"future":{"keep":"1.00"}} ', note['id']])
        db.execute("UPDATE extraction_jobs SET state='running',attempts=attempts+1 WHERE id=?", [jobs[0]['id']])
        db.execute("UPDATE extraction_runtime SET job_id=?,token=?,scope=?,control_revision=(SELECT revision FROM extraction_control),lease_until='2020-01-01T00:00:00Z' WHERE id=1",
                   [jobs[0]['id'], str(uuid.uuid4()), '0' * 64])
        db.execute("INSERT INTO extraction_jobs SELECT ?,attachment_id,source_sha256,source_bytes,'future_xlsx',2,config,config_hash,'queued',1,0,created_at,updated_at,NULL,NULL,NULL,NULL FROM extraction_jobs WHERE id=?",
                   [str(uuid.uuid4()), jobs[0]['id']])
    # Migración/index/search deben funcionar con ambos helpers físicamente ausentes.
    runtime = directory / 'bin'
    runtime.mkdir()
    for name in ['noteka', 'noteka-mcp']:
        shutil.copy2(target / name, runtime / name)
    cli = runtime / 'noteka'
    records = []
    for version in [10, 11]:
        ws = directory / f'fixture-sql-v{version}'
        source = seed
        if version == 10:
            # v10 no representa XLSX: fixture separada sin esos jobs/derivados.
            seed10 = directory / 'seed10-v13'
            shutil.copytree(seed, seed10)
            with sqlite3.connect(seed10 / 'noteka.sqlite') as db:
                db.execute('PRAGMA foreign_keys=ON')
                db.execute('UPDATE extraction_jobs SET generation_id=NULL WHERE id=?', [jobs[2]['id']])
                db.execute('DELETE FROM extraction_generations WHERE job_id=?', [jobs[2]['id']])
                db.execute('DELETE FROM extraction_jobs WHERE id=?', [jobs[2]['id']])
            source = seed10
        # Cursor del cliente actual sobre la misma identidad/ruta y filas.
        # Luego se sustituye SÓLO esta fixture desechable por la DB de SQL viejo.
        # Ejercita el contrato de scope, no pretende bytes producidos por v11.
        shutil.copytree(source, ws)
        expected = call(ws, 'extraction', 'search', dict(query='evidencia', limit=1))
        staged = directory / f'constructed-v{version}'
        construct_fixture(source, staged, version)
        shutil.rmtree(ws)
        staged.rename(ws)
        before, ddl = support.snapshot(ws), support.schema(ws)
        originals = support.files(ws / 'attachments')
        physical = support.files(ws)
        for action, payload in [('index', dict(action='get')), ('search', dict(query='evidencia'))]:
            call(ws, 'extraction', action, payload, 'unsupported_workspace_version')
        admission = subprocess.run([str(runtime / 'noteka-mcp'), '--workspace', str(ws)],
                                   input=b'', capture_output=True, timeout=8)
        assert admission.returncode and b'UnsupportedWorkspaceVersion' in admission.stderr and not admission.stdout
        assert support.files(ws) == physical
        assert call(ws, 'workspace', 'open', {})['schema_version'] == 13
        after = support.snapshot(ws)
        for table, rows in before.items():
            if table != 'workspace_meta':
                assert after[table] == rows, (version, table)
        meta = [tuple(version if i == 1 else value for i, value in enumerate(row))
                for row in ast.literal_eval(after['workspace_meta'])]
        assert repr(meta) == before['workspace_meta']
        assert all(row in support.schema(ws) for row in ddl)
        assert support.files(ws / 'attachments') == originals
        assert call(ws, 'extraction', 'search', dict(query='evidencia', limit=1)) == expected
        call(ws, 'extraction', 'search', dict(query='evidencia', limit=1, cursor=expected['next_cursor']))
        backups = [p for p in directory.glob('noteka-pre-migration-*')
                   if json.loads((p / 'manifest.json').read_text())['workspace_id'] == ast.literal_eval(before['workspace_meta'])[0][0]
                   and json.loads((p / 'manifest.json').read_text())['schema_version'] == version]
        assert len(backups) == 1
        assert support.snapshot(backups[0]) == before
        restored = directory / f'restored-v{version}'
        result = call(ws, 'workspace', 'restore', dict(backup=str(backups[0]), destination=str(restored)))
        assert result['schema_version'] == 13  # Restore explícito abre/migra con el cliente actual.
        assert support.snapshot(restored) == after
        assert support.snapshot(backups[0]) == before  # La fuente histórica no se modifica.
        assert support.files(restored / 'attachments') == originals
        page = call(ws, 'extraction', 'search', dict(query='evidencia', limit=1))
        assert page['next_cursor']
        call(ws, 'extraction', 'search', dict(query='evidencia', limit=1, cursor=page['next_cursor']))
        index = call(ws, 'extraction', 'index', dict(action='get'))
        call(ws, 'extraction', 'index', dict(action='reset', expected_revision=index['revision']))
        expired = call(ws, 'extraction', 'search', dict(query='evidencia', limit=1, cursor=page['next_cursor']), 'revision_conflict')
        assert expired['details']['reason'] == 'index_revision_changed'
        records.append(dict(fixture_version=version, tables_preserved=len(before)-1,
                            backup_rows_exact=True, restore_with_current_client=True, helpers_absent=True,
                            synthetic_current_cursor_survives_migration=True))
    print(json.dumps(dict(profile=args.profile, directory=str(directory), checks=records,
                          fixture_provenance='SQL versionado + datos sintéticos creados por v13; NO baseline independiente',
                          old_client_tested=False, old_catalog_compared=False), indent=2))


if __name__ == '__main__':
    main()
