#!/usr/bin/env python3
"""Release MCP vs same CLI/core fixture; no model, UI or provider."""
import importlib.util
import json
import math
import os
from pathlib import Path
import statistics
import subprocess
import sys
import tempfile
import time


def stats(a):
    a = sorted(a)
    return {'samples': len(a), 'min_ms': a[0], 'median_ms': statistics.median(a),
            'p95_ms': a[math.ceil(.95 * len(a))-1], 'max_ms': a[-1]}


def main():
    assert sys.platform == 'darwin', 'This measurement uses macOS sandbox-exec and ps RSS.'
    # The sampler itself needs ps (denied inside sandbox-exec). Every product
    # process is sandboxed independently; ps samples that child PID after exec.
    denied = ['sandbox-exec', '-p', '(version 1)(allow default)(deny network*)']
    root = Path(tempfile.mkdtemp(prefix='noteka-mcp-bench-', dir=os.environ['NOTEKA_TEST_TMP']))
    target = Path(os.environ.get('CARGO_TARGET_DIR', 'target')).resolve() / 'release'
    cli, binary = target / 'noteka', target / 'noteka-mcp'
    workspace = root / 'workspace'
    spec = importlib.util.spec_from_file_location('mcp_wire', 'scripts/test-mcp-transport.py')
    wire = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(wire)

    def call(op, data=None):
        r = subprocess.run([*denied, str(cli), '--json', '--workspace', str(workspace), *op.split('.')],
                           input=wire.compact(data or {}), capture_output=True, timeout=10)
        assert r.returncode == 0 and not r.stderr, r.stderr
        envelope = json.loads(r.stdout)
        assert envelope['ok']
        return envelope['data']

    call('workspace.create', {'timezone': 'UTC'})
    e = call('entity.create', {'kind': 'note', 'title': 'Synthetic benchmark', 'document': {
        'schema_version': 2, 'blocks': [{'id': 'text', 'type': 'paragraph', 'content': [{'type': 'text', 'text': 'x' * 10240}]}]}})
    startup = []
    rss = []

    def sample_rss(p):
        rss.append(int(subprocess.check_output(['ps', '-o', 'rss=', '-p', str(p.process.pid)]).strip()) * 1024)

    for i in range(25):
        start = time.perf_counter()
        p = wire.Peer(binary, workspace, record=False, prefix=denied)
        p.send('server/discover')
        assert 'result' in p.recv()
        elapsed = (time.perf_counter() - start) * 1000
        if i >= 5:
            startup.append(elapsed)
            sample_rss(p)
        assert p.close() == ''
    p = wire.Peer(binary, workspace, record=False, prefix=denied)
    p.send('server/discover'); p.recv()
    cases = [('entity.get', {'id': e['id']}), ('entity.list', {}), ('home.query', {'view': 'notes_all', 'as_of': '2026-10-15'})]
    result = {'startup_discover': stats(startup)}
    for op, data in cases:
        for kind in ['mcp', 'cli_process']:
            samples = []
            for i in range(25):
                start = time.perf_counter()
                if kind == 'mcp':
                    r = p.call(op, data)['result']['structuredContent']
                    assert r['ok']
                else:
                    call(op, data)
                elapsed = (time.perf_counter() - start) * 1000
                if i >= 5:
                    samples.append(elapsed)
            result[op + '.' + kind] = stats(samples)
            sample_rss(p)
    assert p.close() == ''
    core = json.loads(subprocess.check_output([*denied, str(target / 'examples/mcp_benchmark'), str(workspace), e['id']]))
    for key, samples in core.items():
        result[key] = stats(samples)
    report = {'fixture': {'notes': 1, 'document_text_bytes': 10240, 'warmups': 5, 'samples': 20},
              'observed_mcp_rss_bytes': {'min': min(rss), 'max': max(rss), 'method': 'ps rss after discovery/case; sampled resident set, not exclusive peak'},
              'results': result, 'notes': 'CLI includes spawn; core excludes JSON wire. Difference is not a causal pure-transport measurement. No universal MCP SLA.'}
    (root / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'evidence': str(root), **report}, indent=2))


if __name__ == '__main__':
    main()
