#!/usr/bin/env python3
"""Probes raw stdio/SQLite on synthetic fixtures; run with network denied."""
import argparse
import atexit
import hashlib
import json
import os
from pathlib import Path
import select
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time

CAP = 4 * 1024 * 1024
META = {'io.modelcontextprotocol/protocolVersion': '2026-07-28',
        'io.modelcontextprotocol/clientInfo': {'name': 'noteka-wire-verifier', 'version': '1'},
        'io.modelcontextprotocol/clientCapabilities': {}}


def compact(v):
    return json.dumps(v, separators=(',', ':'), ensure_ascii=False).encode()


class Peer:
    def __init__(self, binary, workspace, *flags, record=True, prefix=()):
        self.process = subprocess.Popen([*prefix, str(binary), '--workspace', str(workspace), *flags],
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.buffer = bytearray()
        self.wire = []
        root = Path(workspace).parent
        self.raw_in = (root / f'raw-{self.process.pid}-in.jsonl').open('wb') if record else None
        self.raw_out = (root / f'raw-{self.process.pid}-out.jsonl').open('wb') if record else None
        atexit.register(self.abort)

    def abort(self):
        if self.process.poll() is None:
            if not self.process.stdin.closed:
                self.process.stdin.close()
            try:
                self.process.wait(timeout=6)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=5)
        for log in [self.raw_in, self.raw_out]:
            if log and not log.closed:
                log.close()

    def send(self, method, params=None, id=1, modern=True):
        params = dict(params or {})
        if modern:
            params['_meta'] = META
        msg = {'jsonrpc': '2.0', 'method': method, 'params': params}
        if id is not None:
            msg['id'] = id
        self.bytes(compact(msg) + b'\n')

    def bytes(self, raw):
        if self.raw_in:
            self.raw_in.write(raw)
        self.process.stdin.write(raw)
        self.process.stdin.flush()

    def recv(self, timeout=5):
        until = time.monotonic() + timeout
        while b'\n' not in self.buffer:
            left = until - time.monotonic()
            if left <= 0 or not select.select([self.process.stdout], [], [], left)[0]:
                raise AssertionError('MCP response timeout')
            chunk = os.read(self.process.stdout.fileno(), 65536)
            assert chunk, ('MCP EOF', self.process.poll())
            self.buffer.extend(chunk)
            assert len(self.buffer) <= 16 * 1024 * 1024 + 65536
        line, _, rest = self.buffer.partition(b'\n')
        self.buffer = bytearray(rest)
        value = json.loads(line)
        if self.raw_out:
            self.raw_out.write(line + b'\n')
        self.wire.append(value)
        return value

    def call(self, operation, args=None, id=1):
        self.send('tools/call', {'name': 'noteka_' + operation.replace('.', '_'), 'arguments': args or {}}, id)
        reply = self.recv()
        assert reply['id'] == id
        return reply

    def close(self):
        if not self.process.stdin.closed:
            self.process.stdin.close()
        self.process.wait(timeout=31)
        self.process.stdout.close()
        error = self.process.stderr.read().decode()
        self.process.stderr.close()
        for log in [self.raw_in, self.raw_out]:
            if log and not log.closed:
                log.close()
        return error


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--profile', choices=['debug', 'release'], default='debug')
    args = parser.parse_args()
    target = Path(os.environ.get('CARGO_TARGET_DIR', 'target')).resolve() / args.profile
    cli, binary = target / 'noteka', target / 'noteka-mcp'
    root = Path(tempfile.mkdtemp(prefix='noteka-mcp-wire-', dir=os.environ['NOTEKA_TEST_TMP']))
    workspace = root / 'workspace'

    def cli_call(op, data=None, ws=None):
        r = subprocess.run([str(cli), '--json', '--workspace', str(ws or workspace), *op.split('.')],
                           input=compact(data or {}), capture_output=True, timeout=10)
        assert r.stderr == b'', r.stderr
        return json.loads(r.stdout)

    def manifest():
        return {str(p.relative_to(workspace)): hashlib.sha256(p.read_bytes()).hexdigest()
                for p in workspace.rglob('*') if p.is_file()}

    checks = []
    shutdown_ms = {}
    probe_ms = {}
    created = cli_call('workspace.create')
    assert created['ok']
    current = created['data']['schema_version']
    for flags in [[], ['--workspace'], ['--workspace', str(workspace), '--allow-run'],
                  ['--workspace', str(root / 'absent')], ['--workspace', str(workspace), '--unknown']]:
        before = manifest()
        r = subprocess.run([str(binary), *flags], capture_output=True, timeout=5)
        assert r.returncode != 0 and r.stdout == b'' and r.stderr
        assert manifest() == before
    checks.append('invalid startup is pure, stderr only')
    db = sqlite3.connect(workspace / 'noteka.sqlite')
    for metadata, user in [(current-1, current-1), (current+1, current+1), (current, current-1)]:
        db.execute('UPDATE workspace_meta SET schema_version=?', (metadata,))
        db.execute(f'PRAGMA user_version={user}')
        db.commit()
        before = manifest()
        r = subprocess.run([str(binary), '--workspace', str(workspace)], capture_output=True, timeout=5)
        assert r.returncode != 0 and not r.stdout and r.stderr
        assert before == manifest()
    db.execute('UPDATE workspace_meta SET schema_version=?', (current,))
    db.execute(f'PRAGMA user_version={current}')
    db.commit()
    db.close()
    checks.append('old, future and inconsistent workspace reject without migration/backup')

    p = Peer(binary, workspace)
    before = manifest()
    p.send('server/discover')
    discovered = p.recv()['result']
    assert discovered['supportedVersions'] == ['2026-07-28', '2025-11-25']
    assert set(discovered['capabilities']) == {'tools', 'resources'}
    assert p.close() == '' and manifest() == before
    checks.append('modern discovery raw bytes, no initialize, physical purity')

    p = Peer(binary, workspace)
    p.send('tools/call', {'name': 'noteka_entity_get', 'arguments': {'id': 'absent'}}, modern=False)
    assert 'error' in p.recv()
    p.send('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                         'clientInfo': {'name': 'noteka-wire-legacy', 'version': '1'}}, modern=False)
    legacy = p.recv()['result']
    assert legacy['protocolVersion'] == '2025-11-25' and 'resultType' not in legacy
    assert set(legacy['capabilities']) == {'resources', 'tools'}
    p.send('notifications/initialized', id=None, modern=False)
    p.send('tools/list', modern=False)
    tools = p.recv()['result']
    assert len(tools['tools']) == 8 and tools['nextCursor'] and 'resultType' not in tools
    p.send('tools/call', {'name': 'noteka_entity_get', 'arguments': {'id': 'absent'}}, modern=False)
    r = p.recv()['result']
    assert 'resultType' not in r and r['structuredContent']['error']['code'] == 'not_found'
    assert json.loads(r['content'][0]['text']) == r['structuredContent']
    p.send('tools/call', {'name': 'noteka_entity_list', 'arguments': None}, modern=False)
    assert p.recv()['error']['code'] == -32602
    assert p.close() == ''
    checks.append('legacy raw initialize/initialized, pre-handshake denial, pagination and exact tool envelope')

    p = Peer(binary, workspace, '--allow-write')
    p.send('tools/list', modern=False)
    assert p.recv()['error']['code'] == -32602
    p.send('tools/list', {'_meta': {**META, 'io.modelcontextprotocol/protocolVersion': '2099-01-01'}}, modern=False)
    assert 'error' in p.recv()
    p.send('server/discover'); reply = p.recv(); assert 'result' in reply, reply
    p.send('tools/call', {'name': 'noteka_entity_create', 'arguments': {'kind': 'note', 'title': 'must not execute'}}, modern=False)
    assert 'error' in p.recv()
    assert cli_call('entity.list')['data']['items'] == []
    p.send('unknown/method'); assert p.recv()['error']['code'] == -32601
    p.bytes(b'{bad JSON}\n'); assert p.recv()['error']['code'] == -32700
    for arguments in [[], None, False, 1, 'not an object']:
        p.send('tools/call', {'name': 'noteka_entity_list', 'arguments': arguments})
        reply = p.recv()
        assert 'error' in reply, reply  # null is not an omitted arguments object.
    p.send('tools/call', {'name': 'absent', 'arguments': {}}); assert p.recv()['error']['code'] == -32602
    p.send('resources/subscribe', {'uri': 'noteka://catalog'}); assert p.recv()['error']['code'] == -32601
    p.send('tools/list', {'_meta': {**META, 'io.modelcontextprotocol/protocolVersion': '2025-11-25'}}, modern=False)
    assert 'error' in p.recv()
    request_ids = set()
    for _ in range(50):
        r = p.call('entity.get', {'id': 'absent'})['result']['structuredContent']
        assert r['request_id'] not in request_ids
        request_ids.add(r['request_id'])
    assert p.close() == ''
    checks.append('missing metadata/state, unsupported version, malformed JSON/params, unknown method/tool')

    # Exact cap, fragmented frames, and EOF without delimiter.
    p = Peer(binary, workspace)
    request = compact({'jsonrpc': '2.0', 'id': 7, 'method': 'server/discover', 'params': {'_meta': META}})
    exact = request + b' ' * (CAP - len(request))
    started = time.monotonic()
    p.bytes(exact[:123]); p.bytes(exact[123:] + b'\n')
    assert p.recv()['id'] == 7
    probe_ms['fragmented_4mib_roundtrip'] = (time.monotonic() - started) * 1000
    assert p.close() == ''
    p = Peer(binary, workspace)
    started = time.monotonic()
    p.bytes(b' ' * (CAP + 1))
    assert 'input_limit' in p.close()
    probe_ms['overlimit_input_to_exit'] = (time.monotonic() - started) * 1000
    p = Peer(binary, workspace)
    p.bytes(request)
    p.process.stdin.close()
    p.process.wait(timeout=5)
    assert p.process.stdout.read() == b''
    p.close()
    checks.append('fragmented exact 4 MiB, +1 without newline, unterminated EOF never dispatched')
    p = Peer(binary, workspace)
    start = time.monotonic()
    p.send('resources/read', {'uri': 'noteka://catalog'})
    p.process.wait(timeout=7)  # No stdout reads and stdin remains open.
    shutdown_ms['unread_stdout'] = (time.monotonic() - start) * 1000
    assert 'output_closed' in p.close()
    checks.append('real unread stdout times out after bounded write without waiting for stdin EOF')

    e = cli_call('entity.create', {'kind': 'note', 'title': 'delivery fixture'})['data']
    # Unknown future document is deliberately retained; transport cannot
    # retroactively reject/roll back the ordinary title mutation's commit.
    db = sqlite3.connect(workspace / 'noteka.sqlite')
    future = compact({'schema_version': 999, 'unknown': '\\"\n' * 800000}).decode()
    db.execute('UPDATE documents SET json=? WHERE entity_id=?', (future, e['id']))
    db.commit()
    p = Peer(binary, workspace, '--allow-write')
    r = p.call('entity.get', {'id': e['id']})
    assert r['error']['code'] == -32603
    assert r['error']['data']['reason'] == 'response_too_large'
    assert r['error']['data']['core_completed'] and not r['error']['data']['effects_may_be_committed']
    r = p.call('entity.update', {'id': e['id'], 'expected_revision': e['revision'], 'title': 'committed before delivery'})
    assert r['error']['data']['effects_may_be_committed']
    assert db.execute('SELECT title,revision FROM entities WHERE id=?', (e['id'],)).fetchone() == ('committed before delivery', 2)
    assert db.execute('SELECT json FROM documents WHERE entity_id=?', (e['id'],)).fetchone()[0] == future
    assert p.close() == ''
    checks.append('oversized read/mutation delivery errors preserve real commit and unknown document')

    # A real writer holds SQLite while cancellation is consumed by the reader.
    p = Peer(binary, workspace, '--allow-write')
    p.send('server/discover'); p.recv()
    db.execute('BEGIN IMMEDIATE')
    p.send('tools/call', {'name': 'noteka_entity_create', 'arguments': {'kind': 'note', 'title': 'held'}}, 31)
    time.sleep(.15)  # Dispatch observation: writer lock holds the admitted call.
    p.send('notifications/cancelled', {'requestId': 31}, id=None)
    started = time.monotonic()
    p.send('tools/call', {'name': 'noteka_entity_list', 'arguments': {}}, 32)
    r = p.recv(); assert r['id'] == 32 and r['error']['data']['reason'] == 'server_busy', r
    probe_ms['cancel_then_busy_roundtrip_while_core_held'] = (time.monotonic() - started) * 1000
    # Keep the lock through the core's 2s busy timeout: no confirmed mutation,
    # no late response to id31, and cancellation never released its permit.
    time.sleep(2.1)
    db.rollback()
    p.send('tools/call', {'name': 'noteka_entity_get', 'arguments': {'id': 'absent'}}, 33)
    r = p.recv(); assert r['id'] == 33 and r['result']['structuredContent']['error']['code'] == 'not_found', r
    assert db.execute("SELECT count(*) FROM entities WHERE title='held'").fetchone()[0] == 0
    assert p.close() == ''
    checks.append('cancel during real writer contention retains permit, suppresses late response, no retry')
    p = Peer(binary, workspace, '--allow-write')
    p.send('server/discover'); p.recv()
    db.execute('BEGIN IMMEDIATE')
    p.send('tools/call', {'name': 'noteka_entity_create', 'arguments': {'kind': 'note', 'title': 'EOF held'}}, 41)
    time.sleep(.15)
    p.send('tools/call', {'name': 'noteka_entity_list', 'arguments': {}}, 42)
    assert p.recv()['error']['data']['reason'] == 'server_busy'
    start = time.monotonic(); assert p.close() == ''
    shutdown_ms['eof_mutation_contention'] = (time.monotonic() - start) * 1000
    db.rollback()
    assert db.execute("SELECT count(*) FROM entities WHERE title='EOF held'").fetchone()[0] == 0
    db.close()
    p = Peer(binary, workspace, '--allow-write')
    p.send('server/discover'); p.recv()
    original = root / 'original-pinned'
    workspace.rename(original)
    assert cli_call('workspace.create')['ok']
    foreign = sqlite3.connect(workspace / 'noteka.sqlite')
    before = list(foreign.iterdump())
    r = p.call('entity.create', {'kind': 'note', 'title': 'foreign forbidden'})['result']['structuredContent']
    assert r['error']['details']['reason'] == 'workspace_scope_mismatch'
    assert before == list(foreign.iterdump())
    foreign.close(); assert p.close() == ''
    workspace.rename(root / 'foreign-preserved'); original.rename(workspace)
    checks.append('EOF with active mutation and live identity replacement preserve both databases')

    # Retained/failing helper belongs to an isolated runtime, never replaces
    # the project's shared worker. Its readiness file is an explicit barrier.
    runtime = root / 'isolated-runtime'
    runtime.mkdir()
    isolated = runtime / 'noteka-mcp'
    shutil.copy2(binary, isolated)
    worker = runtime / 'noteka-pdf-worker'
    ready = runtime / 'ready'
    worker.write_text(f'#!{sys.executable}\nimport os,sys,time\nsys.stdin.buffer.read()\nopen({str(ready)+".tmp"!r},"w").write(str(os.getpid()))\nos.replace({str(ready)+".tmp"!r},{str(ready)!r})\ntime.sleep(60)\n')
    worker.chmod(0o700)
    import runpy
    gen = runpy.run_path('tests/fixtures/pdf/generate.py')
    source = root / 'synthetic.pdf'
    source.write_bytes(gen['serialize'](gen['document']([b'BT /F1 12 Tf 10 10 Td (MCP shutdown synthetic) Tj ET'])))
    a = cli_call('attachment.import', {'source_path': str(source)})['data']
    control = cli_call('extraction.control', {'action': 'get'})['data']
    cli_call('extraction.control', {'action': 'set', 'paused': False, 'expected_revision': control['revision']})
    job = cli_call('extraction.enqueue', {'attachment_id': a['id'], 'extractor': 'pdf_text'})['data']['job']

    def running():
        ready.unlink(missing_ok=True)
        peer = Peer(isolated, workspace, '--allow-write', '--allow-run')
        peer.send('tools/call', {'name': 'noteka_extraction_run', 'arguments': {'id': job['id']}}, 71)
        until = time.monotonic() + 4
        while not ready.exists() and time.monotonic() < until:
            time.sleep(.005)
        assert ready.exists(), 'helper readiness barrier'
        return peer, int(ready.read_text())

    def dead(pid):
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            return True
        return False

    p, pid = running()
    active = cli_call('extraction.get', {'id': job['id']})['data']
    assert active['state'] == 'running' and active['ownership_active']
    assert cli_call('extraction.cancel', {'id': job['id'], 'expected_revision': active['revision']})['ok']
    interrupted = p.recv()['result']['structuredContent']['data']
    assert interrupted['outcome'] == 'interrupted' and interrupted['job']['state'] == 'cancelled'
    assert p.close() == '' and dead(pid)
    checks.append('other CLI operational cancellation fences active MCP run and reaps retained helper')
    for event in ['eof', 'broken_pipe']:
        current = cli_call('extraction.get', {'id': job['id']})['data']
        assert cli_call('extraction.retry', {'id': job['id'], 'expected_revision': current['revision']})['ok']
        p, pid = running()
        started = time.monotonic()
        if event == 'broken_pipe':
            p.process.stdout.close()
            p.send('server/discover', id=72)  # Force a bounded write to the lost output.
            p.process.wait(timeout=6)  # stdin stays open: this is not the EOF path.
        error = p.close()
        shutdown_ms[event] = (time.monotonic() - started) * 1000
        assert time.monotonic() - started < 30 and dead(pid), (event, error)
        current = cli_call('extraction.get', {'id': job['id']})['data']
        assert current['attempts'] >= 2 and current['generation'] is None
        assert current['state'] == 'failed' and not current['ownership_active'], current
    checks.append('EOF/broken output during run stop admission, reap own helper and retain confirmed attempts')
    worker.unlink()
    current = cli_call('extraction.get', {'id': job['id']})['data']
    cli_call('extraction.retry', {'id': job['id'], 'expected_revision': current['revision']})
    p = Peer(isolated, workspace, '--allow-write', '--allow-run')
    r = p.call('extraction.run', {'id': job['id']})['result']['structuredContent']
    assert r['error']['details']['reason'] == 'pdf_worker_unavailable'
    assert p.close() == '' and source.exists()
    checks.append('missing helper has core error, no remote fallback or original loss')
    (root / 'result.json').write_text(json.dumps({'profile': args.profile, 'checks': checks, 'shutdown_ms': shutdown_ms, 'probe_ms': probe_ms}, indent=2) + '\n')
    print(json.dumps({'evidence': str(root), 'profile': args.profile, 'checks': checks, 'shutdown_ms': shutdown_ms, 'probe_ms': probe_ms}, indent=2))


if __name__ == '__main__':
    main()
