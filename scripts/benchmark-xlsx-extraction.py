#!/usr/bin/env python3
"""Medición release explícita de integración, no del parser aislado."""
import importlib.util
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
root = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location('corpus', root / 'tests/fixtures/xlsx/generate.py')
corpus = importlib.util.module_from_spec(spec)
spec.loader.exec_module(corpus)
pins = {n: h for h, n in (line.split() for line in (root / 'tests/fixtures/xlsx/golden.sha256').read_text().splitlines())}
manifest = (json.dumps(corpus.manifest(), ensure_ascii=False, indent=2) + '\n').encode()
assert hashlib.sha256(manifest).hexdigest() == pins['manifest.json']
assert hashlib.sha256((root / 'tests/fixtures/xlsx/generate.py').read_bytes()).hexdigest() == pins['generate.py']
cases = [c for c in corpus.cases() if c['error'] is None]
assert len(cases) == 77
directory = Path(tempfile.mkdtemp(prefix='xlsx-integration-bench-', dir=os.environ['NOTEKA_TEST_TMP'])).resolve()
assert not directory.is_relative_to(root)
(directory / 'corpus-manifest.json').write_bytes(manifest)
for case in cases:
    (directory / (case['name'] + '.xlsx')).write_bytes(case['source'])
(directory / 'cases.json').write_text(json.dumps([c['name'] for c in cases]))
command = ['cargo', 'test', '--release', '--offline', '--locked', '-p', 'noteka-core', '--lib',
           'xlsx_tests::release_xlsx_integration_measurements', '--', '--ignored', '--nocapture', '--test-threads=1']
r = subprocess.run(command, cwd=root, env={**os.environ, 'NOTEKA_XLSX_BENCH_ROOT': str(directory)}, capture_output=True, timeout=1800)
log = r.stdout.decode() + r.stderr.decode()
(directory / 'run.log').write_text(log)
record = next((line.split('NOTEKA_XLSX_INTEGRATION_BENCH ', 1)[1] for line in log.splitlines() if 'NOTEKA_XLSX_INTEGRATION_BENCH {' in line), None)
if record:
    result = json.loads(record)
    result['machine'] = platform.machine()
    if sys.platform == 'darwin':
        result['hardware'] = {key: subprocess.check_output(['/usr/sbin/sysctl', '-n', key], text=True).strip()
                              for key in ['machdep.cpu.brand_string', 'hw.memsize']}
    (directory / 'results.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(dict(directory=str(directory), cases=len(result['cases']), maximum_lock_p95_ms=result['maximum_lock_p95_ms'], memory=result['memory'])))
else:
    print(json.dumps(dict(directory=str(directory), error='Sin registro; consultar run.log')))
sys.exit(r.returncode)
