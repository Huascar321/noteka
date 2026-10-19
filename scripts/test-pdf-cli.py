#!/usr/bin/env python3
"""Corpus por CLI real + runtime sibling aislado; ningún host/ventana/red."""
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

sys.dont_write_bytecode = True


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--profile', choices=['debug', 'release'], default='debug')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    target = Path(os.environ.get('CARGO_TARGET_DIR', root / 'target')).resolve() / args.profile
    spec = importlib.util.spec_from_file_location('corpus', root / 'tests/fixtures/pdf/generate.py')
    corpus = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(corpus)
    cases = corpus.cases()
    assert len(cases) == 235
    pins = {name: digest for digest, name in
            (line.split() for line in (root / 'tests/fixtures/pdf/golden.sha256').read_text().splitlines())}
    manifest_bytes = (json.dumps(corpus.manifest(), indent=2, ensure_ascii=False) + '\n').encode()
    assert hashlib.sha256(manifest_bytes).hexdigest() == pins['corpus-manifest.json']
    assert hashlib.sha256((root / 'tests/fixtures/pdf/generate.py').read_bytes()).hexdigest() == pins['generate.py']
    checks = []
    with tempfile.TemporaryDirectory(prefix='pdf-cli-', dir=os.environ['NOTEKA_TEST_TMP']) as tmp:
        temp = Path(tmp)
        # Layout Contents/MacOS sin lanzar Tauri; copiar, no tocar el runtime común.
        runtime = temp / 'Noteka.app/Contents/MacOS'
        runtime.mkdir(parents=True)
        cli, worker = runtime / 'noteka', runtime / 'noteka-pdf-worker'
        shutil.copy2(target / 'noteka', cli)
        shutil.copy2(target / 'noteka-pdf-worker', worker)
        workspace = temp / 'workspace'

        def call(group, action, payload, error=None, ws=workspace):
            result = subprocess.run([str(cli), '--workspace', str(ws), '--json', group, action],
                                    input=json.dumps(payload).encode(), capture_output=True,
                                    cwd=temp, timeout=12)
            assert result.stderr == b'', result.stderr[:200]
            reply = json.loads(result.stdout)
            assert reply['api_version'] == 1
            if error:
                assert not reply['ok'] and result.returncode == error[0], (group, action, reply)
                assert reply['error']['details']['reason'] == error[1], reply
                return reply['error']
            assert reply['ok'] and result.returncode == 0, reply
            return reply['data']

        call('workspace', 'create', {'timezone': 'UTC'})
        control = call('extraction', 'control', {'action': 'get'})
        assert control['paused']
        control = call('extraction', 'control', {'action': 'set', 'paused': False, 'expected_revision': control['revision']})
        successes = []
        for case in cases:
            src = temp / (case['name'] + '.PDF')
            src.write_bytes(case['source'])
            attachment = call('attachment', 'import', {'source_path': str(src)})
            request = {'attachment_id': attachment['id'], 'extractor': 'pdf_text'}
            if len(case['source']) > corpus.CAP:
                call('extraction', 'enqueue', request, (2, 'source_size_limit'))
                checks.append({'name': case['name'], 'phase': 'admission', 'outcome': 'source_size_limit'})
                continue
            job = call('extraction', 'enqueue', request)['job']
            assert call('extraction', 'enqueue', request)['job']['id'] == job['id']
            if case['error']:
                e = call('extraction', 'run', {'id': job['id']}, (2, case['error']))
                assert e['details']['job_id'] == job['id'] and e['details']['attempt'] == 1
                assert call('extraction', 'get', {'id': job['id']})['state'] == 'failed'
            else:
                done = call('extraction', 'run', {'id': job['id']})['job']
                assert done['state'] == 'succeeded' and done['attempts'] == 1
                g = done['generation']
                assert g['source_sha256'] == hashlib.sha256(case['source']).hexdigest()
                assert g['source_bytes'] == g['covered_bytes'] == len(case['source']) and g['complete']
                cursor, fragments, manifests = None, [], []
                while True:
                    request = {'id': job['id'], 'limit': 64}
                    if cursor:
                        request['cursor'] = cursor
                    page = call('extraction', 'pdf-fragments', request)
                    fragments.extend(page['items'])
                    manifests.append(page['manifest'])
                    assert page['covers_generation'] == (not cursor and not page['next_cursor'])
                    cursor = page['next_cursor']
                    if not cursor:
                        break
                assert all(m == manifests[0] for m in manifests)
                manifest = manifests[0]
                assert manifest['version'] == 1 and manifest['parser_name'] == 'lopdf' and manifest['parser_version'] == '0.45.0'
                assert manifest['scope'] == 'page_text_objects'
                assert manifest['limitations'] == ['no_ocr', 'no_visual_order', 'no_geometry', 'no_structured_tables']
                assert manifest['page_count'] == len(case['texts'])
                assert manifest['extracted_bytes'] == sum(len(t.encode()) for t in case['texts'])
                ordinal = 0
                for number, text in enumerate(case['texts'], 1):
                    p = manifest['pages'][number-1]
                    own = [f for f in fragments if f['locator']['page_number'] == number]
                    assert p == {'page_number': number, 'state': 'text' if text else 'no_text',
                                 'text_bytes': len(text.encode()), 'text_sha256': hashlib.sha256(text.encode()).hexdigest(),
                                 'fragment_count': len(own)}
                    assert ''.join(f['text'] for f in own) == text
                    offset = 0
                    for f in own:
                        assert f['ordinal'] == ordinal and 1 <= len(f['text'].encode()) <= 4096
                        end = offset + len(f['text'].encode())
                        assert f['locator'] == {'kind': 'pdf_page_text', 'page_number': number,
                                                'text_byte_start': offset, 'text_byte_end': end}
                        ordinal, offset = ordinal+1, end
                assert ordinal == g['fragment_count']
                call('extraction', 'fragments', {'id': job['id']}, (2, 'unsupported_generation_config'))
                successes.append(job)
            checks.append({'name': case['name'], 'phase': 'run', 'outcome': case['error'] or 'success'})
        # Abrir y leer no dependen del helper. El original y SQLite quedan iguales.
        worker.unlink()
        job = successes[0]
        conn = sqlite3.connect(workspace / 'noteka.sqlite')
        before = list(conn.iterdump())
        call('extraction', 'pdf-fragments', {'id': job['id']})
        assert list(conn.iterdump()) == before
        backup = temp / 'backup'
        call('workspace', 'backup', {'destination': str(backup)})
        restored = temp / 'restored'
        call('workspace', 'restore', {'backup': str(backup), 'destination': str(restored)})
        call('extraction', 'pdf-fragments', {'id': job['id']}, ws=restored)
        job = call('extraction', 'get', {'id': job['id']})
        call('extraction', 'retry', {'id': job['id'], 'expected_revision': job['revision']})
        call('extraction', 'run', {'id': job['id']}, (5, 'pdf_worker_unavailable'))
        assert call('extraction', 'pdf-fragments', {'id': job['id']})['generation']['id'] == job['generation']['id']
        conn.close()
    print(json.dumps({'profile': args.profile, 'cases': len(checks), 'checks': checks,
                      'layout': 'Contents/MacOS', 'read_restore_without_helper': True}, indent=2))


if __name__ == '__main__':
    main()
