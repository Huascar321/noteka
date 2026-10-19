#!/usr/bin/env python3
"""Fencing real CLI/MCP con helper retenido por barrera, sin pantalla/red."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--profile', choices=['debug', 'release'], default='debug')
    args = ap.parse_args()
    root = Path(__file__).resolve().parent.parent
    target = Path(os.environ.get('CARGO_TARGET_DIR', root / 'target')).resolve() / args.profile
    wire = module('wire', root / 'scripts/test-mcp-transport.py')
    corpus = module('corpus', root / 'tests/fixtures/xlsx/generate.py')
    checks = []
    with tempfile.TemporaryDirectory(prefix='xlsx-processes-', dir=os.environ['NOTEKA_TEST_TMP']) as tmp:
        tmp = Path(tmp)
        runtime = tmp / 'runtime'
        runtime.mkdir()
        for name in ['noteka', 'noteka-mcp', 'noteka-xlsx-worker']:
            shutil.copy2(target / name, runtime / name)
        ws = tmp / 'workspace'

        def call(op, data=None, error=None):
            r = subprocess.run([str(runtime / 'noteka'), '--json', '--workspace', str(ws), *op.split('.')],
                               input=json.dumps(data or {}).encode(), capture_output=True, timeout=10)
            assert not r.stderr, r.stderr
            j = json.loads(r.stdout)
            if error:
                assert not j['ok'] and j['error']['details']['reason'] == error, j
                return j['error']
            assert j['ok'] and r.returncode == 0, j
            return j['data']

        call('workspace.create')
        source = tmp / 'synthetic.xlsx'
        source.write_bytes(corpus.pack(corpus.parts(corpus.worksheet('<row r="1"><c r="A1"><v>9007199254740993</v></c><c r="B1"><v>145.6900000001</v></c></row>'))))
        a = call('attachment.import', dict(source_path=str(source)))
        job = call('extraction.enqueue', dict(attachment_id=a['id'], extractor='xlsx_cells'))['job']
        c = call('extraction.control', dict(action='get'))
        call('extraction.control', dict(action='set', paused=False, expected_revision=c['revision']))
        done = call('extraction.run', dict(id=job['id']))['job']
        initial_index = call('extraction.index', dict(action='get'))
        call('extraction.index', dict(action='put', job_id=job['id'], expected_generation_id=done['generation']['id'], expected_revision=initial_index['revision']))
        def search_index():
            return call('extraction.search', dict(query='9007199254740993', extractor='xlsx_cells'))
        assert len(search_index()['items']) == 1
        first = call('extraction.xlsx-cells', dict(id=job['id'], limit=1))
        assert first['next_cursor']
        path = tmp / 'second.xlsx'
        path.write_bytes(corpus.pack(corpus.parts()))
        second = call('attachment.import', dict(source_path=str(path)))
        second = call('extraction.enqueue', dict(attachment_id=second['id'], extractor='xlsx_cells'))['job']
        # Catálogo/lectura/permisos en bytes wire, en las dos eras y tres modos.
        for modern in [True, False]:
            for flags, count in [((), 25), (('--allow-write',), 45), (('--allow-write', '--allow-run'), 46)]:
                p = wire.Peer(runtime / 'noteka-mcp', ws, *flags)
                try:
                    if modern:
                        p.send('server/discover'); p.recv()
                    else:
                        p.send('initialize', dict(protocolVersion='2025-11-25', capabilities={},
                                                  clientInfo=dict(name='xlsx-wire', version='1')), modern=False)
                        p.recv(); p.send('notifications/initialized', id=None, modern=False)
                    tools, cursor = [], None
                    while True:
                        p.send('tools/list', dict(cursor=cursor) if cursor else {}, modern=modern)
                        page = p.recv()['result']; assert len(page['tools']) <= 8
                        tools.extend(page['tools']); cursor = page.get('nextCursor')
                        if not cursor:
                            break
                    assert len(tools) == count
                    assert sum(t['name'] == 'noteka_extraction_xlsx_cells' for t in tools) == 1

                    def tool(name, data):
                        p.send('tools/call', dict(name='noteka_' + name, arguments=data), modern=modern)
                        out = p.recv()['result']; content = out['structuredContent']
                        assert len(out['content']) == 1 and json.loads(out['content'][0]['text']) == content
                        assert out.get('isError', False) == (not content['ok'])
                        return content

                    page = tool('extraction_xlsx_cells', dict(id=job['id'], limit=1))
                    assert page['ok'] and page['data'] == first
                    index = tool('extraction_index', dict(action='get'))
                    assert index['ok'] and index['data'] == call('extraction.index', dict(action='get'))
                    found = tool('extraction_search', dict(query='9007199254740993', extractor='xlsx_cells'))
                    assert found['ok'] and found['data'] == search_index()
                    payload = dict(action='put', job_id=job['id'], expected_generation_id=done['generation']['id'], expected_revision=index['data']['revision'])
                    put = tool('extraction_index', payload)
                    if count == 25:
                        assert not put['ok'] and put['error']['details']['reason'] == 'permission_denied'
                        denied = tool('extraction_index', dict(action='reset', expected_revision=index['data']['revision']))
                        assert not denied['ok'] and denied['error']['details']['reason'] == 'permission_denied'
                    else:
                        assert put['ok'] and not put['data']['changed']
                    for name, data, denied in [('extraction_run', dict(id=second['id']), count != 46),
                                              ('extraction_enqueue', dict(attachment_id=a['id'], extractor='xlsx_cells'), count == 25)]:
                        if denied:
                            error = tool(name, data)
                            assert not error['ok'] and error['error']['details']['reason'] == 'permission_denied'
                    assert call('extraction.get', dict(id=second['id']))['attempts'] == 0
                    checks.append(dict(era='modern' if modern else 'legacy', tools=count,
                                       typed_read_cli_equal=True, denied_before_effects=True))
                    assert p.close() == ''
                finally:
                    p.abort()
        real, helper = runtime / 'real-xlsx-worker', runtime / 'noteka-xlsx-worker'
        helper.rename(real)
        ready, release = tmp / 'ready', tmp / 'release'
        helper.write_text(f'#!{sys.executable}\nimport os,time\nfrom pathlib import Path\np=Path({str(ready)!r});q=p.with_suffix(".tmp");q.write_text(str(os.getpid()));q.replace(p)\nwhile not Path({str(release)!r}).exists(): time.sleep(0.005)\nos.execv({str(real)!r},[{str(real)!r}])\n')
        helper.chmod(0o700)
        peer = wire.Peer(runtime / 'noteka-mcp', ws, '--allow-write', '--allow-run')
        peer.send('server/discover')
        peer.recv()
        try:
            for cancellation in [True, False]:
                current = call('extraction.get', dict(id=job['id']))
                call('extraction.retry', dict(id=job['id'], expected_revision=current['revision']))
                ready.unlink(missing_ok=True)
                release.unlink(missing_ok=True)
                peer.send('tools/call', dict(name='noteka_extraction_run', arguments=dict(id=job['id'])), id=10)
                deadline = time.monotonic() + 4
                while not ready.exists() and time.monotonic() < deadline:
                    time.sleep(0.005)
                assert ready.exists(), 'barrera de helper'
                pid = int(ready.read_text())
                running = call('extraction.get', dict(id=job['id']))
                assert running['state'] == 'running' and running['ownership_active']
                # Mientras el parser espera, no hay lock de escritor y se lee generación previa.
                page = call('extraction.xlsx-cells', dict(id=job['id']))
                assert page['generation'] == done['generation'] and page['result_from_previous_attempt']
                hits = search_index()['items']
                assert len(hits) == 1 and hits[0]['source']['generation_id'] == done['generation']['id']
                assert hits[0]['result_from_previous_attempt']
                call('extraction.run', dict(id=second['id']), error='worker_active')
                if cancellation:
                    call('extraction.cancel', dict(id=job['id'], expected_revision=running['revision']))
                else:
                    release.write_text('release')
                result = peer.recv(timeout=8)['result']['structuredContent']
                assert result['ok'], result
                assert result['data']['outcome'] == ('interrupted' if cancellation else 'succeeded')
                try:
                    os.kill(pid, 0)
                except ProcessLookupError:
                    pass
                else:
                    raise AssertionError('helper no fue recolectado')
                current = call('extraction.get', dict(id=job['id']))
                if cancellation:
                    assert current['generation'] == done['generation'] and current['state'] == 'cancelled'
                    call('extraction.xlsx-cells', dict(id=job['id'], limit=1, cursor=first['next_cursor']))
                    assert len(search_index()['items']) == 1
                else:
                    assert current['generation']['id'] != done['generation']['id']
                    call('extraction.xlsx-cells', dict(id=job['id'], limit=1, cursor=first['next_cursor']), error='cursor_scope_mismatch')
                    assert search_index()['items'] == []  # publicación no autoindexa
                    index = call('extraction.index', dict(action='get'))
                    call('extraction.index', dict(action='put', job_id=job['id'], expected_generation_id=current['generation']['id'], expected_revision=index['revision']))
                    assert search_index()['items'][0]['source']['generation_id'] == current['generation']['id']
                checks.append(dict(mode='cancel' if cancellation else 'publish', read_during_run=True, writer_available=True,
                                   singleton_enforced=True, helper_reaped=True))
            assert peer.close() == ''
        finally:
            release.write_text('cleanup')
            peer.abort()
        # EOF o salida rota avisan al mismo supervisor XLSX, no esperan watchdog.
        for broken in [False, True]:
            current = call('extraction.get', dict(id=job['id']))
            retained = current['generation']
            call('extraction.retry', dict(id=job['id'], expected_revision=current['revision']))
            ready.unlink(missing_ok=True); release.unlink(missing_ok=True)
            p = wire.Peer(runtime / 'noteka-mcp', ws, '--allow-write', '--allow-run')
            try:
                p.send('server/discover'); p.recv()
                p.send('tools/call', dict(name='noteka_extraction_run', arguments=dict(id=job['id'])))
                deadline = time.monotonic() + 4
                while not ready.exists() and time.monotonic() < deadline:
                    time.sleep(0.005)
                assert ready.exists(), 'barrera de apagado XLSX'
                pid = int(ready.read_text()); start = time.monotonic()
                if broken:
                    p.process.stdout.close()
                    p.send('ping', id=2)
                    p.process.wait(timeout=4)
                    error = p.process.stderr.read()
                    # El transporte permite sólo este diagnóstico terminal fijo;
                    # la carrera de cierre puede terminar antes de emitirlo.
                    lines = error.splitlines()
                    assert len(lines) <= 2 and set(lines) <= {b'MCP: lifecycle_closed.', b'MCP: output_closed.'}, repr(error)
                else:
                    assert p.close() == ''
                elapsed = (time.monotonic() - start) * 1000
                assert elapsed < 4000, 'debe interrumpir antes del deadline de 5 s'
                try:
                    os.kill(pid, 0)
                except ProcessLookupError:
                    pass
                else:
                    raise AssertionError('helper residente tras apagado')
                final = call('extraction.get', dict(id=job['id']))
                assert final['generation'] == retained and final['attempts'] == current['attempts'] + 1
                assert final['state'] == 'failed' and not final['ownership_active']
                assert call('extraction.xlsx-cells', dict(id=job['id']))['result_from_previous_attempt']
                assert search_index()['items'][0]['source']['generation_id'] == retained['id']
                assert search_index()['items'][0]['result_from_previous_attempt']
                checks.append(dict(shutdown='broken_pipe' if broken else 'eof', milliseconds=elapsed,
                                   helper_reaped=True, retained_generation=True,
                                   terminal_diagnostics=error.decode().splitlines() if broken else []))
            finally:
                release.write_text('cleanup'); p.abort()
        assert (ws / 'attachments' / a['id']).read_bytes() == source.read_bytes()
        assert call('extraction.get', dict(id=second['id']))['attempts'] == 0
        assert hashlib.sha256(source.read_bytes()).hexdigest() == a['sha256']
    print(json.dumps(dict(profile=args.profile, checks=checks, originals_unchanged=True), indent=2))


if __name__ == '__main__':
    main()
