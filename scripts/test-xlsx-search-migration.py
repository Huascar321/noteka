#!/usr/bin/env python3
"""Fixture v11 real y comprobación de migración v12; todos los datos son sintéticos."""
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


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT / path)
    out = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(out)
    return out


def call(binary, ws, group, action, payload, error=None):
    result = subprocess.run([str(binary), '--workspace', str(ws), '--json', group, action],
                            input=json.dumps(payload).encode(), capture_output=True, timeout=15)
    assert not result.stderr, result.stderr[:300]
    data = json.loads(result.stdout)
    if error:
        assert not data['ok'] and data['error']['code'] == error and result.returncode, data
        return data['error']
    assert data['ok'] and result.returncode == 0, data
    return data['data']


def snapshot(ws):
    with sqlite3.connect(ws / 'noteka.sqlite') as db:
        names = [r[0] for r in db.execute("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name")]
        return {name: repr(sorted(db.execute('SELECT * FROM "' + name + '"').fetchall(), key=repr))
                for name in names}


def schema(ws):
    with sqlite3.connect(ws / 'noteka.sqlite') as db:
        return list(db.execute('SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name'))


def files(path):
    return {str(p.relative_to(path)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in path.rglob('*') if p.is_file()}


def capture(old, directory):
    ws = directory / 'workspace'
    directory.mkdir(parents=True)
    assert call(old, ws, 'workspace', 'create', {'timezone': 'UTC'})['schema_version'] == 11
    ctl = call(old, ws, 'extraction', 'control', {'action': 'get'})
    call(old, ws, 'extraction', 'control', dict(action='set', paused=False, expected_revision=ctl['revision']))
    pdf = module('pdf', 'tests/fixtures/pdf/generate.py')
    xlsx = module('xlsx', 'tests/fixtures/xlsx/generate.py')
    sources = [('source.txt', ('evidencia café alfa\n' * 500).encode(), 'utf8_text'),
               ('source.pdf', pdf.serialize(pdf.document([b'BT /F1 12 Tf (evidencia PDF alfa) Tj ET'])), 'pdf_text'),
               ('source.xlsx', xlsx.pack(xlsx.parts(xlsx.one_cell('<c r="A1" t="str"><v>evidencia XLSX alfa</v></c>'))), 'xlsx_cells')]
    jobs = []
    for name, content, extractor in sources:
        source = directory / name
        source.write_bytes(content)
        attachment = call(old, ws, 'attachment', 'import', {'source_path': str(source)})
        job = call(old, ws, 'extraction', 'enqueue', dict(attachment_id=attachment['id'], extractor=extractor))['job']
        job = call(old, ws, 'extraction', 'run', {'id': job['id']})['job']
        assert job['state'] == 'succeeded', job
        jobs.append(job)
        if extractor != 'xlsx_cells':
            idx = call(old, ws, 'extraction', 'index', {'action': 'get'})
            call(old, ws, 'extraction', 'index', dict(action='put', job_id=job['id'],
                 expected_generation_id=job['generation']['id'], expected_revision=idx['revision']))
    entity = call(old, ws, 'entity', 'create', dict(kind='note', title='Synthetic unknown document'))
    call(old, ws, 'extraction', 'retry', dict(id=jobs[0]['id'], expected_revision=jobs[0]['revision']))
    future_config = ' {"version":2,"chunk_bytes":4096,"future":{"keep":"1.00"}} '
    encoded = json.dumps(json.loads(future_config), sort_keys=True, separators=(',', ':')).encode()
    h = hashlib.sha256()
    for part in [b'noteka.extraction.config.v1', encoded]:
        h.update(len(part).to_bytes(8, 'big'))
        h.update(part)
    with sqlite3.connect(ws / 'noteka.sqlite') as db:
        db.execute('UPDATE documents SET json=? WHERE entity_id=?', [' {"schema_version":999,"future":{"keep":"1.00"}} ', entity['id']])
        db.execute("UPDATE extraction_jobs SET state='running',revision=revision+1,attempts=attempts+1 WHERE id=?", [jobs[0]['id']])
        db.execute("UPDATE extraction_runtime SET job_id=?,token=?,scope=?,control_revision=(SELECT revision FROM extraction_control),lease_until='2020-01-01T00:00:00Z' WHERE id=1", [jobs[0]['id'], str(uuid.uuid4()), '0'*64])
        db.execute("INSERT INTO extraction_jobs SELECT ?,attachment_id,source_sha256,source_bytes,'future_xlsx',2,?,?,'queued',1,0,created_at,updated_at,NULL,NULL,NULL,NULL FROM extraction_jobs WHERE id=?", [str(uuid.uuid4()), future_config, h.hexdigest(), jobs[0]['id']])
    plan = dict(version=1, batch_id=str(uuid.uuid4()), steps=[dict(key='n', operation='entity.create', input=dict(kind='note', title='Versioned batch'))])
    receipt = call(old, ws, 'changes', 'preview', plan)['receipt']
    data = dict(jobs=jobs, plan=plan, receipt=receipt, index=call(old, ws, 'extraction', 'index', dict(action='get', limit=1)),
                query=call(old, ws, 'extraction', 'search', dict(query='evidencia', limit=1)),
                tables=snapshot(ws), schema=schema(ws), originals=files(ws / 'attachments'))
    assert data['index']['next_cursor'] and data['query']['next_cursor']
    (directory / 'baseline.json').write_text(json.dumps(data, indent=2, ensure_ascii=False) + '\n')
    return ws, data


def verify(new, old, directory):
    clients = directory / 'clients'
    for label, binary in [('v12', new), ('v11', old)]:
        runtime = clients / label
        runtime.mkdir(parents=True, exist_ok=True)
        for name in ['noteka', 'noteka-mcp']:
            shutil.copy2(binary.with_name(name), runtime / name)
        assert not list(runtime.glob('*worker*'))
    new, old = clients / 'v12/noteka', clients / 'v11/noteka'
    ws = directory / 'workspace'
    data = json.loads((directory / 'baseline.json').read_text())
    before = files(ws)
    call(new, ws, 'extraction', 'index', dict(action='get'), 'unsupported_workspace_version')
    call(new, ws, 'extraction', 'search', dict(query='evidencia'), 'unsupported_workspace_version')
    admission = subprocess.run([str(new.with_name('noteka-mcp')), '--workspace', str(ws)], input=b'', capture_output=True, timeout=8)
    assert admission.returncode != 0 and b'UnsupportedWorkspaceVersion' in admission.stderr and not admission.stdout, admission.stderr
    assert files(ws) == before
    assert call(new, ws, 'workspace', 'open', {})['schema_version'] == 12
    after = snapshot(ws)
    assert set(after) == set(data['tables'])
    for table, rows in data['tables'].items():
        if table != 'workspace_meta':
            assert after[table] == rows, table
    normalized_meta = [tuple(11 if i == 1 else value for i, value in enumerate(row)) for row in ast.literal_eval(after['workspace_meta'])]
    assert repr(normalized_meta) == data['tables']['workspace_meta']
    assert [list(row) for row in schema(ws)] == data['schema']
    assert files(ws / 'attachments') == data['originals']
    unchanged = snapshot(ws)
    receipt_error = call(new, ws, 'changes', 'apply', dict(plan=data['plan'], expected=data['receipt']), 'revision_conflict')
    assert receipt_error['details']['reason'] == 'base_hash_mismatch', receipt_error
    assert snapshot(ws) == unchanged
    for key, action, payload in [('query', 'search', dict(query='evidencia', limit=1)),
                                 ('index', 'index', dict(action='get', limit=1))]:
        expected = json.loads(json.dumps(data[key]))
        # Único cambio observable antes de seleccionar: XLSX ahora es compatible.
        expected['coverage']['unsupported_generations'] -= 1
        expected['coverage']['unindexed_supported_generations'] += 1
        assert call(new, ws, 'extraction', action, payload) == expected
    call(new, ws, 'extraction', 'search', dict(query='evidencia', limit=1, cursor=data['query']['next_cursor']))
    backups = list(directory.glob('noteka-pre-migration-*'))
    assert len(backups) == 1
    restored = directory / 'restored-v11'
    assert call(old, ws, 'workspace', 'restore', dict(backup=str(backups[0]), destination=str(restored)))['schema_version'] == 11
    assert snapshot(restored) == data['tables']
    before = files(ws)
    call(old, ws, 'workspace', 'open', {}, 'unsupported_workspace_version')
    admission = subprocess.run([str(old.with_name('noteka-mcp')), '--workspace', str(ws)], input=b'', capture_output=True, timeout=8)
    assert admission.returncode != 0 and b'UnsupportedWorkspaceVersion' in admission.stderr and not admission.stdout, admission.stderr
    call(old, ws, 'extraction', 'index', dict(action='reset', expected_revision=data['index']['revision']), 'unsupported_workspace_version')
    assert files(ws) == before
    job = data['jobs'][2]
    put = call(new, ws, 'extraction', 'index', dict(action='put', job_id=job['id'], expected_generation_id=job['generation']['id'], expected_revision=data['index']['revision']))
    assert put['changed'] and put['entry']['source']['extractor'] == 'xlsx_cells'
    expired = call(new, ws, 'extraction', 'search', dict(query='evidencia', limit=1, cursor=data['query']['next_cursor']), 'revision_conflict')
    assert expired['details']['reason'] == 'index_revision_changed'
    return dict(tables_preserved=len(after)-1, old_fingerprints_preserved=True, cursor_survives_migration=True,
                cursor_invalidated_by_put=True, v11_receipt_rejected=True, v11_backup_restored=True, old_client_rejected=True,
                helpers_absent_during_migration_index_and_restore=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--profile', choices=['debug', 'release'], default='debug')
    ap.add_argument('--baseline-bin', type=Path, required=True)
    ap.add_argument('--directory', type=Path, required=True)
    ap.add_argument('--capture-only', action='store_true')
    args = ap.parse_args()
    old = args.baseline_bin.resolve() / 'noteka'
    directory = args.directory.resolve()
    if not directory.exists():
        capture(old, directory)
    if args.capture_only:
        print(json.dumps(dict(profile=args.profile, baseline='independent v11', directory=str(directory))))
    else:
        new = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target')).resolve() / args.profile / 'noteka'
        print(json.dumps(dict(profile=args.profile, checks=verify(new, old, directory)), indent=2))


if __name__ == '__main__':
    main()
