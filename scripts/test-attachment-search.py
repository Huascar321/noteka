#!/usr/bin/env python3
"""API/CLI, migración v9 y carreras reales, sólo fixtures/temporales externos."""
import argparse
from concurrent.futures import ThreadPoolExecutor
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
    p = argparse.ArgumentParser()
    p.add_argument('--baseline-cli', type=Path, required=True)
    p.add_argument('--profile', choices=['debug', 'release'], default='debug')
    p.add_argument('--rounds', type=int, default=20)
    args = p.parse_args()
    root = Path(__file__).resolve().parent.parent
    target = Path(os.environ.get('CARGO_TARGET_DIR', root / 'target')) / args.profile
    spec = importlib.util.spec_from_file_location('corpus', root / 'tests/fixtures/pdf/generate.py')
    corpus = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(corpus)
    case = next(c for c in corpus.cases() if not c['error'] and c.get('texts') and len(c['source']) < 10000)
    records = []
    with tempfile.TemporaryDirectory(prefix='attachment-cli-', dir=os.environ['NOTEKA_TEST_TMP']) as tmp:
        temp = Path(tmp)
        cli = temp / 'noteka'
        worker = temp / 'noteka-pdf-worker'
        shutil.copy2(target / 'noteka', cli)
        shutil.copy2(target / 'noteka-pdf-worker', worker)

        def run(ws, group, action, data, binary=cli, expect=0):
            r = subprocess.run([str(binary), '--workspace', str(ws), '--json', group, action],
                               input=json.dumps(data).encode(), capture_output=True, cwd=temp, timeout=20)
            assert not r.stderr, r.stderr
            v = json.loads(r.stdout)
            assert v['api_version'] == 1
            if expect is not None:
                assert r.returncode == expect and v['ok'] == (expect == 0), (r.returncode, v)
            return r.returncode, v

        def ok(ws, group, action, data, **kw):
            return run(ws, group, action, data, **kw)[1]['data']

        def rows(ws):
            with sqlite3.connect(ws / 'noteka.sqlite') as db:
                return {n: sorted(db.execute(f'SELECT * FROM "{n}"').fetchall(), key=repr)
                        for n, in db.execute("SELECT name FROM sqlite_schema WHERE type='table'")}

        def files(ws):
            return {str(f.relative_to(ws)): hashlib.sha256(f.read_bytes()).hexdigest()
                    for f in ws.rglob('*') if f.is_file()}

        def add(ws, name, text, ext='utf8_text', binary=cli):
            src = temp / name
            src.write_bytes(text)
            a = ok(ws, 'attachment', 'import', {'source_path': str(src)}, binary=binary)
            j = ok(ws, 'extraction', 'enqueue', {'attachment_id': a['id'], 'extractor': ext}, binary=binary)['job']
            c = ok(ws, 'extraction', 'control', {'action': 'get'}, binary=binary)
            if c['paused']:
                ok(ws, 'extraction', 'control', {'action': 'set', 'paused': False, 'expected_revision': c['revision']}, binary=binary)
            return ok(ws, 'extraction', 'run', {'id': j['id']}, binary=binary)['job']

        # El workspace se origina exclusivamente en el binario v9 externo.
        for running in (False, True):
            ws = temp / f'v9-{running}'
            old = args.baseline_cli.resolve()
            assert ok(ws, 'workspace', 'create', {'timezone': 'UTC'}, binary=old)['schema_version'] == 9
            j = add(ws, f'old-{running}.txt', b'original literal\r\n', binary=old)
            add(ws, f'old-{running}.pdf', case['source'], 'pdf_text', binary=old)
            if running:
                ok(ws, 'extraction', 'retry', {'id': j['id'], 'expected_revision': j['revision']}, binary=old)
            with sqlite3.connect(ws / 'noteka.sqlite') as db:
                config = '{ "version":99, "opaque": [true,null] }'
                h = hashlib.sha256()
                for part in (b'noteka.extraction.config.v1', json.dumps(json.loads(config), sort_keys=True, separators=(',', ':')).encode()):
                    h.update(len(part).to_bytes(8, 'big')); h.update(part)
                db.execute('UPDATE extraction_jobs SET config=?,config_hash=? WHERE id=?', (config, h.hexdigest(), j['id']))
                db.execute('UPDATE extraction_generations SET config=?,config_hash=? WHERE job_id=?', (config, h.hexdigest(), j['id']))
            if running:
                scope = json.loads(ok(ws, 'extraction', 'list', {'limit': 1}, binary=old)['next_cursor'])['scope']
                with sqlite3.connect(ws / 'noteka.sqlite') as db:
                    db.execute("UPDATE extraction_jobs SET state='running',attempts=2 WHERE id=?", (j['id'],))
                    db.execute("UPDATE extraction_runtime SET job_id=?,token=?,scope=?,control_revision=2,lease_until='2099-01-01T00:00:00Z'", (j['id'], str(uuid.uuid4()), scope))
            plan = {'version': 1, 'batch_id': str(uuid.uuid4()), 'steps': [
                {'key': 'note', 'operation': 'entity.create', 'input': {'kind': 'note', 'title': 'v9 receipt'}}]}
            receipt = ok(ws, 'changes', 'preview', plan, binary=old)['receipt']
            before = rows(ws)
            originals = files(ws / 'attachments')
            for group, action, data in [('extraction', 'index', {'action': 'get'}), ('extraction', 'search', {'query': 'literal'})]:
                code, v = run(ws, group, action, data, expect=6)
                assert v['error']['code'] == 'unsupported_workspace_version'
                assert rows(ws) == before
            assert ok(ws, 'workspace', 'open', {})['schema_version'] == 10
            after = rows(ws)
            assert all(after[n] == r for n, r in before.items() if n != 'workspace_meta')
            assert files(ws / 'attachments') == originals
            assert not after['extraction_search_entries'] and not after['extraction_search_fts']
            assert ok(ws, 'extraction', 'index', {'action': 'get'})['coverage']['unsupported_generations'] == 1
            code, v = run(ws, 'changes', 'apply', {'plan': plan, 'expected': receipt}, expect=4)
            assert v['error']['details']['reason'] == 'base_hash_mismatch'
            backup = next(d for d in temp.glob('noteka-pre-migration-*') if (d / 'manifest.json').is_file() and
                          json.loads((d / 'manifest.json').read_text())['workspace_id'] == before['workspace_meta'][0][0])
            restored = temp / f'rollback-{running}'
            ok(ws, 'workspace', 'restore', {'backup': str(backup), 'destination': str(restored)}, binary=old)
            assert rows(restored) == before
            assert files(restored / 'attachments') == originals
            backup_v10 = temp / f'future-v10-backup-{running}'
            restored_v10 = temp / f'future-v10-restored-{running}'
            ok(ws, 'workspace', 'backup', {'destination': str(backup_v10)})
            ok(ws, 'workspace', 'restore', {'backup': str(backup_v10), 'destination': str(restored_v10)})
            assert rows(restored_v10) == after and files(restored_v10 / 'attachments') == originals
            assert ok(restored_v10, 'extraction', 'index', {'action': 'get'})['coverage']['unsupported_generations'] == 1
            before_files = files(ws)
            run(ws, 'workspace', 'open', {}, binary=old, expect=6)
            assert files(ws) == before_files
            records.append({'migration_running': running, 'rows_preserved': True, 'old_client_pure': True, 'backup_restored_v9': True})

        ws = temp / 'current'
        ok(ws, 'workspace', 'create', {'timezone': 'UTC'})
        literal = 'café OR NEAR\r\ntexto literal'
        j = add(ws, 'literal.txt', literal.encode())
        pdf = add(ws, 'golden.pdf', case['source'], 'pdf_text')
        empty_case = next(c for c in corpus.cases() if c['name'] == 'no-contents')
        empty = add(ws, 'no-text.pdf', empty_case['source'], 'pdf_text')
        ranked = [add(ws, f'ranking-{n}.txt', b'ranking literal cafe') for n in range(2)]
        worker.unlink()  # Todas las operaciones siguientes deben funcionar sin parser.
        assert not ok(ws, 'extraction', 'search', {'query': 'café'})['items']

        def put(job, rev):
            return {'action': 'put', 'job_id': job['id'], 'expected_generation_id': job['generation']['id'], 'expected_revision': rev}

        rev = 1
        for job in (j, pdf, empty, *ranked):
            out = ok(ws, 'extraction', 'index', put(job, rev)); assert out['changed']; rev = out['revision']
        hit = ok(ws, 'extraction', 'search', {'query': 'cafe OR NEAR'})['items'][0]
        assert hit['text'] == literal and hit['source']['job_id'] == j['id']
        assert hit['source']['source_sha256'] == hashlib.sha256(literal.encode()).hexdigest()
        assert hit['locator'] == {'kind': 'utf8_bytes', 'byte_start': 0, 'byte_end': len(literal.encode()), 'line_start': 1, 'line_end': 2}
        pdf_text = ''.join(case['texts'])
        assert isinstance(pdf_text, str), type(pdf_text)
        term = next(t for t in pdf_text.split() if t.isalnum())
        hits = ok(ws, 'extraction', 'search', {'query': term, 'job_id': pdf['id']})['items']
        assert hits and all(h['locator']['kind'] == 'pdf_page_text' for h in hits)
        assert ''.join(h['text'] for h in sorted(hits, key=lambda h: h['ordinal'])) in pdf_text
        assert len(hits) == 1 and hits[0]['ordinal'] == 0 and hits[0]['text'] == case['texts'][0]
        assert hits[0]['source']['job_id'] == pdf['id'] and hits[0]['source']['generation_id'] == pdf['generation']['id']
        assert hits[0]['source']['source_sha256'] == hashlib.sha256(case['source']).hexdigest()
        assert hits[0]['locator'] == {'kind': 'pdf_page_text', 'page_number': 1,
            'text_byte_start': 0, 'text_byte_end': len(case['texts'][0].encode()), 'scope': 'page_text_objects',
            'limitations': ['no_ocr', 'no_visual_order', 'no_geometry', 'no_structured_tables']}
        ranking = ok(ws, 'extraction', 'search', {'query': 'ranking'})['items']
        assert [h['source']['job_id'] for h in ranking] == sorted(job['id'] for job in ranked)
        assert len(ranking) == 2 and ranking[0]['score'] == ranking[1]['score']
        assert all(h['text'] == 'ranking literal cafe' and h['ordinal'] == 0 for h in ranking)
        assert ok(ws, 'extraction', 'index', put(j, rev))['changed'] is False
        for payload in [{'action': 'get', 'archived': True}, {'action': 'reset', 'expected_revision': None},
                        {'action': 'put', 'job_id': 'bad'}, {'action': 'get', 'limit': 0}]:
            run(ws, 'extraction', 'index', payload, expect=2)
        code, reply = run(ws, 'extraction', 'index', {'action': 'put', 'job_id': str(uuid.uuid4()),
            'expected_generation_id': str(uuid.uuid4()), 'expected_revision': rev}, expect=3)
        assert reply['error']['details']['reason'] == 'job_not_found'
        dbpath = ws / 'noteka.sqlite'
        mode = dbpath.stat().st_mode & 0o777
        before_read = rows(ws)
        try:
            dbpath.chmod(0o444)
            ok(ws, 'extraction', 'index', {'action': 'get'})
            ok(ws, 'extraction', 'search', {'query': 'cafe'})
        finally:
            dbpath.chmod(mode)
        assert rows(ws) == before_read
        for op in ('extraction.index', 'extraction.search'):
            invalid = {'version': 1, 'batch_id': str(uuid.uuid4()), 'steps': [{'key': 'x', 'operation': op, 'input': {}}]}
            run(ws, 'changes', 'preview', invalid, expect=2)
            invalid = {'version': 1, 'batch_id': str(uuid.uuid4()), 'steps': [
                {'key': 'n', 'operation': 'entity.create', 'input': {'kind': 'note', 'title': 'must not persist'}}],
                'checks': [{'key': 'x', 'input': {'operation': op, 'input': {}}}]}
            run(ws, 'changes', 'preview', invalid, expect=2)

        # Roundtrip mixto, no_text, ranking/estado/cobertura y cursor de otra ruta.
        state = ok(ws, 'extraction', 'index', {'action': 'get'})
        assert state['coverage']['indexed_generations'] == 5
        assert any(e['source']['job_id'] == empty['id'] and e['fragment_count'] == 0 for e in state['items'])
        seen, next_cursor = [], None
        while True:
            payload = {'action': 'get', 'limit': 1}
            if next_cursor:
                payload['cursor'] = next_cursor
            page = ok(ws, 'extraction', 'index', payload)
            assert page['revision'] == state['revision'] and page['coverage'] == state['coverage']
            seen.extend(e['source']['job_id'] for e in page['items'])
            next_cursor = page['next_cursor']
            if next_cursor is None:
                break
        assert seen == sorted(job['id'] for job in (j, pdf, empty, *ranked))
        cursor = ok(ws, 'extraction', 'index', {'action': 'get', 'limit': 1})['next_cursor']
        backup = temp / 'offline-mixed-backup'
        restored = temp / 'offline-mixed-restored'
        ok(ws, 'workspace', 'backup', {'destination': str(backup)})
        ok(ws, 'workspace', 'restore', {'backup': str(backup), 'destination': str(restored)})
        assert state == ok(restored, 'extraction', 'index', {'action': 'get'})
        assert files(restored / 'attachments') == files(ws / 'attachments')
        for query in ('cafe OR NEAR', term):
            assert ok(ws, 'extraction', 'search', {'query': query}) == ok(restored, 'extraction', 'search', {'query': query})
        code, reply = run(restored, 'extraction', 'index', {'action': 'get', 'limit': 1, 'cursor': cursor}, expect=2)
        assert reply['error']['details']['reason'] == 'cursor_scope_mismatch'
        records.append({'mixed_offline_roundtrip': True, 'pdf_no_text': True, 'cursor_path_fence': True})

        # Dos procesos, CAS único; también reset/publicación/backup concurrentes.
        with ThreadPoolExecutor(max_workers=4) as pool:
            for n in range(args.rounds):
                rev = ok(ws, 'extraction', 'index', {'action': 'get'})['revision']
                rev = ok(ws, 'extraction', 'index', {'action': 'reset', 'expected_revision': rev})['revision']
                futures = [pool.submit(run, ws, 'extraction', 'index', put(j, rev), expect=None) for _ in range(2)]
                put_backup = temp / f'put-backup-{n}'
                futures.append(pool.submit(run, ws, 'workspace', 'backup', {'destination': str(put_backup)}, expect=None))
                outcomes = [f.result() for f in futures]
                assert sorted(code for code, _ in outcomes[:2]) == [0, 4] and outcomes[2][0] == 0, outcomes
                put_restored = temp / f'put-restored-{n}'
                ok(ws, 'workspace', 'restore', {'backup': str(put_backup), 'destination': str(put_restored)})
                put_page = ok(put_restored, 'extraction', 'search', {'query': 'literal'})
                assert len(put_page['items']) <= 1
                assert len(ok(ws, 'extraction', 'search', {'query': 'literal'})['items']) == 1
                rev += 1
                current = ok(ws, 'extraction', 'get', {'id': j['id']})
                ok(ws, 'extraction', 'retry', {'id': j['id'], 'expected_revision': current['revision']})
                backup = temp / f'race-backup-{n}'
                futures = [pool.submit(run, ws, 'extraction', 'run', {'id': j['id']}, expect=None),
                           pool.submit(run, ws, 'extraction', 'index', {'action': 'reset', 'expected_revision': rev}, expect=None),
                           pool.submit(run, ws, 'workspace', 'backup', {'destination': str(backup)}, expect=None),
                           pool.submit(run, ws, 'entity', 'create', {'kind': 'note', 'title': f'concurrent editor {n}'}, expect=None)]
                outcomes = [f.result() for f in futures]
                assert outcomes[0][0] == 0 and outcomes[1][0] in (0, 4) and outcomes[2][0] == 0 and outcomes[3][0] == 0, outcomes
                j = ok(ws, 'extraction', 'get', {'id': j['id']})
                assert not ok(ws, 'extraction', 'search', {'query': 'literal'})['items']
                restored = temp / f'race-restored-{n}'
                ok(ws, 'workspace', 'restore', {'backup': str(backup), 'destination': str(restored)})
                page = ok(restored, 'extraction', 'search', {'query': 'literal'})
                assert len(page['items']) <= 1
                records.append({'concurrency_round': n, 'one_index_winner': True, 'put_backup_atomic': True,
                                'reset_publish_backup_atomic': True, 'concurrent_editor': True})
    print(json.dumps({'profile': args.profile, 'checks': records}, indent=2))


if __name__ == '__main__':
    main()
