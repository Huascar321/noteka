#!/usr/bin/env python3
"""Mide procesos reales release; /usr/bin/time macOS informa RSS bytes del hijo.

El corpus/golden es el mismo del harness. No infiere memoria desde el allocator.
Ejecutar fuera de compilaciones, con red denegada y NOTEKA_TEST_TMP externo.
"""
import argparse
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--samples', type=int, default=20)
    ap.add_argument('--warmups', type=int, default=3)
    args = ap.parse_args()
    assert args.samples >= 20 and args.warmups >= 3
    root = Path(__file__).resolve().parent.parent
    target = Path(os.environ.get('CARGO_TARGET_DIR', root / 'target')).resolve() / 'release'
    spec = importlib.util.spec_from_file_location('corpus', root / 'tests/fixtures/xlsx/generate.py')
    corpus = importlib.util.module_from_spec(spec); spec.loader.exec_module(corpus)
    cases = corpus.cases()
    selected = [c for c in cases if c['error'] is None or c['name'] in [
        'result-1048577', 'entry-expansion-4194305', 'total-expansion-16777217', 'cells-10001']]
    rows = []
    with tempfile.TemporaryDirectory(prefix='xlsx-bench-', dir=os.environ['NOTEKA_TEST_TMP']) as tmp:
        for case in selected:
            elapsed = []; rss = []
            source = case['source']
            import struct
            wire = b'NKXLS001' + struct.pack('<HHIII', 1, 0, len(source), 5000, 0) + source
            for index in range(args.samples + args.warmups):
                started = time.monotonic_ns()
                result = subprocess.run(['/usr/bin/time', '-lp', str(target / 'noteka-xlsx-worker')],
                                        input=wire, cwd=tmp, env={'PATH': '/usr/bin:/bin', 'TMPDIR': tmp},
                                        capture_output=True, timeout=8)
                duration = (time.monotonic_ns()-started)/1_000_000
                assert result.returncode == 0, (case['name'], result.returncode)
                outcome = json.loads(result.stdout)['outcome']
                if case['error']:
                    assert outcome == dict(status='error', reason=case['error']), case['name']
                else:
                    assert outcome == dict(status='success', source_sha256=hashlib.sha256(source).hexdigest(), workbook=case['expected']), case['name']
                match = re.search(rb'^\s*(\d+)\s+maximum resident set size\s*$', result.stderr, re.M)
                assert match, result.stderr
                if index >= args.warmups:
                    elapsed.append(round(duration, 3)); rss.append(int(match.group(1)))
            p95 = sorted(elapsed)[math.ceil(.95*len(elapsed))-1]
            rows.append(dict(name=case['name'], outcome=case['error'] or 'success', source_bytes=len(source),
                             samples_ms=elapsed, p95_ms=p95, maximum_ms=max(elapsed),
                             rss_samples_bytes=rss, rss_maximum_bytes=max(rss)))
        assert list(Path(tmp).iterdir()) == []
    print(json.dumps(dict(platform=sys.platform, profile='release', warmups=args.warmups, samples=args.samples,
                          measurement='process wall incl time launcher; macOS time -l child maximum RSS bytes',
                          corpus_cases=len(cases), measured_cases=len(rows), checks=rows), indent=2))


if __name__ == '__main__':
    main()
