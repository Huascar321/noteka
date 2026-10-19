#!/usr/bin/env python3
"""Migración real desde el CLI v8 capturado, sin modificar su baseline."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import uuid


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--baseline-cli', type=Path, required=True)
    p.add_argument('--profile', choices=['debug', 'release'], default='debug')
    args = p.parse_args()
    root = Path(__file__).resolve().parent.parent
    current = Path(os.environ.get('CARGO_TARGET_DIR', root / 'target')).resolve() / args.profile / 'noteka'
    records = []
    with tempfile.TemporaryDirectory(prefix='pdf-v8-', dir=os.environ['NOTEKA_TEST_TMP']) as tmp:
        temp = Path(tmp)
        for paused in (True, False):
            ws = temp / str(paused)

            def call(binary, group, action, data):
                result = subprocess.run([str(binary), '--workspace', str(ws), '--json', group, action],
                                        input=json.dumps(data).encode(), capture_output=True, timeout=12)
                assert not result.stderr
                return result, json.loads(result.stdout)

            def old(group, action, data):
                result, reply = call(args.baseline_cli.resolve(), group, action, data)
                assert result.returncode == 0 and reply['ok'], reply
                return reply['data']

            assert old('workspace', 'create', {'timezone': 'UTC'})['schema_version'] == 8
            src = temp / 'source.txt'
            src.write_text('literal original\n')
            a = old('attachment', 'import', {'source_path': str(src)})
            job = old('extraction', 'enqueue', {'attachment_id': a['id'], 'extractor': 'utf8_text'})['job']
            c = old('extraction', 'control', {'action': 'get'})
            c = old('extraction', 'control', {'action': 'set', 'paused': False, 'expected_revision': c['revision']})
            old('extraction', 'run', {'id': job['id']})
            if paused:
                old('extraction', 'control', {'action': 'set', 'paused': True, 'expected_revision': c['revision']})
            plan = {'version': 1, 'batch_id': str(uuid.uuid4()), 'steps': [
                {'key': 'n', 'operation': 'entity.create', 'input': {'kind': 'note', 'title': 'v8 receipt'}}]}
            # Una configuración futura conservable conserva su representación raw.
            db = sqlite3.connect(ws / 'noteka.sqlite')
            config = '{ "version":9, "future": {"opaque": true} }'
            canonical = json.dumps(json.loads(config), sort_keys=True, separators=(',', ':'))
            # config_hash usa la serialización canónica JSON del core.
            digest = hashlib.sha256()
            for part in (b'noteka.extraction.config.v1', canonical.encode()):
                digest.update(len(part).to_bytes(8, 'big'))
                digest.update(part)
            ch = digest.hexdigest()
            db.execute('UPDATE extraction_jobs SET config=?,config_hash=?', (config, ch))
            db.execute('UPDATE extraction_generations SET config=?,config_hash=?', (config, ch))
            if not paused:
                old('extraction', 'list', {})  # no recupera ni modifica
                # El scope es un dato de fixture, no autorización obtenida del PID.
                source2 = temp / 'second.txt'
                source2.write_text('second')
                # Cerrar la transacción de configuración antes del segundo CLI.
                db.commit()
                b = old('attachment', 'import', {'source_path': str(source2)})
                second = old('extraction', 'enqueue', {'attachment_id': b['id'], 'extractor': 'utf8_text'})['job']
                cursor = json.loads(old('extraction', 'list', {'limit': 1})['next_cursor'])
                revision = old('extraction', 'control', {'action': 'get'})['revision']
                db.execute("UPDATE extraction_jobs SET state='running',revision=2,attempts=1 WHERE id=?", (second['id'],))
                db.execute("UPDATE extraction_runtime SET job_id=?,token=?,scope=?,control_revision=?,lease_until='2099-01-01T00:00:00Z' WHERE id=1",
                           (second['id'], str(uuid.uuid4()), cursor['scope'], revision))
            db.commit()
            receipt = old('changes', 'preview', plan)['receipt']
            before = {name: db.execute(f'SELECT * FROM "{name}"').fetchall()
                      for name, in db.execute("SELECT name FROM sqlite_schema WHERE type='table'")}
            result, reply = call(current, 'workspace', 'open', {})
            assert result.returncode == 0 and reply['data']['schema_version'] == 10, reply
            after = {name: db.execute(f'SELECT * FROM "{name}"').fetchall()
                     for name, in db.execute("SELECT name FROM sqlite_schema WHERE type='table'")}
            for name, rows in before.items():
                if name != 'workspace_meta':
                    assert after[name] == rows, name
            assert set(after) - set(before) == {'extraction_pdf_generations', 'extraction_pdf_pages', 'extraction_pdf_fragments',
                'extraction_search_control', 'extraction_search_entries', 'extraction_search_fts',
                'extraction_search_fts_data', 'extraction_search_fts_idx', 'extraction_search_fts_content',
                'extraction_search_fts_docsize', 'extraction_search_fts_config'}
            assert all(not after[n] for n in ('extraction_pdf_generations', 'extraction_pdf_pages',
                'extraction_pdf_fragments', 'extraction_search_entries', 'extraction_search_fts'))
            assert after['extraction_search_control'][0][1:4] == (1, 1, 'literal_unicode61_v1')
            result, reply = call(current, 'changes', 'apply', {'plan': plan, 'expected': receipt})
            assert result.returncode == 4 and reply['error']['details']['reason'] == 'base_hash_mismatch', reply
            db.close()
            files_before = {f.name: f.read_bytes() for f in ws.iterdir() if f.is_file()}
            result, reply = call(args.baseline_cli.resolve(), 'workspace', 'open', {})
            assert result.returncode == 6 and reply['error']['code'] == 'unsupported_workspace_version', reply
            assert files_before == {f.name: f.read_bytes() for f in ws.iterdir() if f.is_file()}
            records.append({'paused': paused, 'rows_preserved': True, 'future_config_raw': True,
                            'v8_receipt_rejected': True, 'old_client_no_writes': True})
    print(json.dumps({'profile': args.profile, 'checks': records}, indent=2))


if __name__ == '__main__':
    main()
