#!/usr/bin/env python3
"""Migración desde binarios v10 independientes; no recrea v10 con DDL nuevo."""
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
import uuid

sys.dont_write_bytecode = True


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--profile', choices=['debug', 'release'], default='debug')
    ap.add_argument('--baseline-bin', type=Path, required=True)
    args = ap.parse_args()
    root = Path(__file__).resolve().parent.parent
    target = Path(os.environ.get('CARGO_TARGET_DIR', root / 'target')).resolve() / args.profile
    spec = importlib.util.spec_from_file_location('pdf', root / 'tests/fixtures/pdf/generate.py')
    pdf = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(pdf)
    results = []
    with tempfile.TemporaryDirectory(prefix='xlsx-migrate-', dir=os.environ['NOTEKA_TEST_TMP']) as tmp:
        temp = Path(tmp)
        old, new = temp / 'v10', temp / 'v11'
        old.mkdir()
        new.mkdir()
        for name in ['noteka', 'noteka-pdf-worker']:
            shutil.copy2(args.baseline_bin / name, old / name)
        shutil.copy2(target / 'noteka', new / 'noteka')

        def call(binary, ws, group, action, payload, error=None):
            r = subprocess.run([str(binary / 'noteka'), '--workspace', str(ws), '--json', group, action],
                               input=json.dumps(payload).encode(), capture_output=True, cwd=temp, timeout=12)
            assert not r.stderr, r.stderr[:200]
            j = json.loads(r.stdout)
            if error:
                assert not j['ok'] and j['error']['code'] == error and r.returncode != 0, j
                return j['error']
            assert j['ok'] and r.returncode == 0, j
            return j['data']

        def snapshot(ws):
            with sqlite3.connect(ws / 'noteka.sqlite') as db:
                tables = [r[0] for r in db.execute("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name")]
                return {t: sorted(db.execute('SELECT * FROM "' + t + '"').fetchall(), key=repr) for t in tables}

        def files(ws):
            return {str(p.relative_to(ws)): hashlib.sha256(p.read_bytes()).hexdigest()
                    for p in ws.rglob('*') if p.is_file()}

        for state in ['paused', 'running-future']:
            ws = temp / state
            assert call(old, ws, 'workspace', 'create', {'timezone': 'UTC'})['schema_version'] == 10
            entity = call(old, ws, 'entity', 'create', dict(kind='note', title='Synthetic v10 original'))
            ctl = call(old, ws, 'extraction', 'control', dict(action='get'))
            call(old, ws, 'extraction', 'control', dict(action='set', paused=False, expected_revision=ctl['revision']))
            jobs = []
            for name, content, extractor in [('original.txt', b'Synthetic v10 original', 'utf8_text'),
                                             ('original.pdf', pdf.serialize(pdf.document([b'BT /F1 12 Tf (Synthetic PDF original) Tj ET'])), 'pdf_text')]:
                src = temp / name
                src.write_bytes(content)
                a = call(old, ws, 'attachment', 'import', dict(source_path=str(src)))
                j = call(old, ws, 'extraction', 'enqueue', dict(attachment_id=a['id'], extractor=extractor))['job']
                j = call(old, ws, 'extraction', 'run', dict(id=j['id']))['job']
                idx = call(old, ws, 'extraction', 'index', dict(action='get'))
                call(old, ws, 'extraction', 'index', dict(action='put', job_id=j['id'],
                     expected_generation_id=j['generation']['id'], expected_revision=idx['revision']))
                jobs.append(j)
            unknown_document = ' {"schema_version":999,"future_block":{"keep":"1.00"}} '
            with sqlite3.connect(ws / 'noteka.sqlite') as db:
                db.execute('UPDATE documents SET json=? WHERE entity_id=?', [unknown_document, entity['id']])
            if state == 'paused':
                ctl = call(old, ws, 'extraction', 'control', dict(action='get'))
                call(old, ws, 'extraction', 'control', dict(action='set', paused=True, expected_revision=ctl['revision']))
            else:
                # Resultado previo de job running y scope ajeno se conservan, no se recuperan al migrar.
                j = jobs[0]
                call(old, ws, 'extraction', 'retry', dict(id=j['id'], expected_revision=j['revision']))
                with sqlite3.connect(ws / 'noteka.sqlite') as db:
                    db.execute("UPDATE extraction_jobs SET state='running',revision=revision+1,attempts=attempts+1 WHERE id=?", [j['id']])
                    ctl = db.execute('SELECT revision FROM extraction_control').fetchone()[0]
                    db.execute("UPDATE extraction_runtime SET job_id=?,token=?,scope=?,control_revision=?,lease_until=? WHERE id=1",
                               [j['id'], str(uuid.uuid4()), '0'*64, ctl, '2020-01-01T00:00:00Z'])
                    # Job futuro sin generación, válido y visible pero no ejecutable.
                    future_id = str(uuid.uuid4())
                    db.execute("INSERT INTO extraction_jobs SELECT ?,attachment_id,source_sha256,source_bytes,'future_xlsx',2,config,config_hash,'queued',1,0,created_at,updated_at,NULL,NULL,NULL,NULL FROM extraction_jobs WHERE id=?", [future_id, j['id']])
            before_query = call(old, ws, 'extraction', 'search', dict(query='original'))
            before = snapshot(ws)
            originals = files(ws / 'attachments')
            assert call(new, ws, 'workspace', 'open', {})['schema_version'] == 13
            after = snapshot(ws)
            for name, rows in before.items():
                if name != 'workspace_meta':
                    assert after[name] == rows, (state, name)
            assert after['extraction_xlsx_generations'] == after['extraction_xlsx_chunks'] == []
            assert files(ws / 'attachments') == originals
            query = call(new, ws, 'extraction', 'search', dict(query='original'))
            # Job state is intentionally not a search locator; current rows/hits must remain equal.
            assert query['items'] == before_query['items'] and query['revision'] == before_query['revision']
            backups = [p for p in temp.glob('noteka-pre-migration-*') if json.loads((p / 'manifest.json').read_text())['workspace_id'] == after['workspace_meta'][0][0]]
            assert len(backups) == 1
            backup = backups[0]
            assert json.loads((backup / 'manifest.json').read_text())['schema_version'] == 10
            restored = temp / (state + '-restored-v10')
            assert call(old, ws, 'workspace', 'restore', dict(backup=str(backup), destination=str(restored)))['schema_version'] == 10
            assert snapshot(restored) == before
            physical = files(ws)
            call(old, ws, 'workspace', 'open', {}, error='unsupported_workspace_version')
            assert files(ws) == physical
            assert not (new / 'noteka-xlsx-worker').exists()
            results.append(dict(state=state, old_tables_preserved=len(before)-1, backup_v10_restored=True,
                                originals_preserved=len(originals), old_client_rejected_without_writes=True))
    print(json.dumps(dict(profile=args.profile, baseline='independent v10 executable', checks=results), indent=2))


if __name__ == '__main__':
    main()
