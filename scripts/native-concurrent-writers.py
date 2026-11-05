#!/usr/bin/env python3
"""Test-only CLI/MCP writers coordinated with the native synthetic app."""
import importlib.util
import json
from pathlib import Path
import sqlite3
import subprocess
import sys
import time

workspace, cli, mcp, root = map(Path, sys.argv[1:5])
cli_id, mcp_id = sys.argv[5:7]
spec = importlib.util.spec_from_file_location('noteka_mcp_wire', Path(__file__).with_name('test-mcp-transport.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
peer = module.Peer(mcp, workspace, '--allow-write', record=False)
db = sqlite3.connect(workspace / 'noteka.sqlite', timeout=5)
cli_process = None
try:
    peer.send('server/discover'); peer.recv()
    db.execute('BEGIN IMMEDIATE')
    cli_process = subprocess.Popen([str(cli), '--json', '--workspace', str(workspace), 'entity', 'update'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    cli_process.stdin.write(json.dumps({'id': cli_id, 'expected_revision': 1, 'title': 'CLI concurrent'}).encode())
    cli_process.stdin.close()
    peer.send('tools/call', {'name': 'noteka_entity_update', 'arguments': {'id': mcp_id, 'expected_revision': 1, 'title': 'MCP concurrent'}}, id=71)
    (root / 'writers-ready').write_text('ready')
    deadline = time.monotonic() + 1.5
    while not (root / 'writers-release').exists():
        if time.monotonic() >= deadline:
            raise TimeoutError('Concurrent test did not release the synthetic writer lock')
        time.sleep(0.01)
    db.rollback()
    cli_stdout = cli_process.stdout.read()
    cli_stderr = cli_process.stderr.read()
    cli_code = cli_process.wait(timeout=5)
    cli_result = json.loads(cli_stdout)
    mcp_result = peer.recv(timeout=5)
    structured = mcp_result.get('result', {}).get('structuredContent', {})
    assert cli_code == 0 and not cli_stderr and cli_result.get('ok'), (cli_code, cli_result, cli_stderr)
    assert structured.get('ok'), mcp_result
    assert db.execute('PRAGMA integrity_check').fetchone()[0] == 'ok'
    result = {'ok': True, 'cli': cli_result['data']['id'], 'mcp': structured['data']['id']}
    (root / 'writers-result.json').write_text(json.dumps(result))
finally:
    db.rollback()
    db.close()
    if cli_process is not None and cli_process.poll() is None:
        cli_process.kill()
        cli_process.wait(timeout=5)
    peer.abort()
