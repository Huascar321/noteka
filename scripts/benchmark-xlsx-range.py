#!/usr/bin/env python3
"""Latencia CLI release de rangos: originales/fixtures externos y ningún helper."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import time


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--fixture', type=Path, required=True)
    ap.add_argument('--output', type=Path, required=True)
    args = ap.parse_args()
    base = Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True)
    fixture = args.fixture.resolve(strict=True); output = args.output.resolve()
    assert fixture.is_relative_to(base) and output.is_relative_to(base) and not output.exists()
    cli = fixture/'new/noteka'; workspace = fixture/'workspace'
    assert sorted(p.name for p in cli.parent.iterdir()) == ['noteka']
    requests = []
    for name, region, header, limit in [('result-1048576', 'A1:H1', None, 64), ('result-1048576', 'A1:H1', 1, 1), ('sparse-maximum-coordinate', 'XFD1048576', 1048576, 1), ('empty-stored', 'A1:BL1048576', 1, 64)]:
        sha = hashlib.sha256((fixture/(name+'.xlsx')).read_bytes()).hexdigest()
        with sqlite3.connect(workspace/'noteka.sqlite') as c:
            c.execute('PRAGMA query_only=ON')
            job = c.execute("SELECT id FROM extraction_jobs WHERE source_sha256=? AND extractor='xlsx_cells'", [sha]).fetchone()[0]
        requests.append((name, dict(id=job, sheet_id=1, range=region, header_row=header, limit=limit)))
    results = []
    for name, input in requests:
        command = [str(cli), '--workspace', str(workspace), '--json', 'extraction', 'xlsx-range']
        if sys.platform == 'darwin':
            command = ['/usr/bin/sandbox-exec', '-p', '(version 1)(allow default)(deny network*)', *command]
        samples = []; response_bytes = 0
        for n in range(25):
            start = time.perf_counter_ns()
            r = subprocess.run(command, input=json.dumps(input).encode(), capture_output=True, timeout=15)
            elapsed = (time.perf_counter_ns()-start)/1e6
            assert r.returncode == 0 and not r.stderr
            reply = json.loads(r.stdout); assert reply['ok'] and len(r.stdout) <= 2_097_152
            if n >= 5:
                samples.append(elapsed)
            response_bytes = len(r.stdout)
        p95 = sorted(samples)[18]
        results.append(dict(case=name, input=input, samples_ms=samples, p50_ms=sorted(samples)[9], p95_ms=p95, response_bytes=response_bytes))
    report = dict(profile='release', clock='perf_counter_ns; full CLI wall time', helpers_present=False, warmups=5, samples=20, query_target_ms=300, passed=all(r['p95_ms'] <= 300 for r in results), results=results)
    output.write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(dict(passed=report['passed'], max_query_p95_ms=max(r['p95_ms'] for r in results))))
    assert report['passed'], 'query p95 > 300ms; conservar muestras e investigar'


if __name__ == '__main__':
    main()
