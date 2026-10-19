#!/usr/bin/env python3
"""Medición explícita release; corpus sintético y logs sólo en temporal externo."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
root = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location('corpus', root / 'tests/fixtures/pdf/generate.py')
corpus = importlib.util.module_from_spec(spec)
spec.loader.exec_module(corpus)
manifest = (json.dumps(corpus.manifest(), indent=2, ensure_ascii=False) + '\n').encode()
pins = {name: digest for digest, name in (line.split() for line in (root / 'tests/fixtures/pdf/golden.sha256').read_text().splitlines())}
assert hashlib.sha256(manifest).hexdigest() == pins['corpus-manifest.json']
assert hashlib.sha256((root / 'tests/fixtures/pdf/generate.py').read_bytes()).hexdigest() == pins['generate.py']
directory = Path(tempfile.mkdtemp(prefix='pdf-bench-', dir=os.environ['NOTEKA_TEST_TMP'])).resolve()
assert not directory.is_relative_to(root)
(directory / 'corpus-manifest.json').write_bytes(manifest)
for case in corpus.cases():
    if not case['error']:
        (directory / (case['name'] + '.pdf')).write_bytes(case['source'])
env = {**os.environ, 'NOTEKA_PDF_BENCH_ROOT': str(directory)}
commands = [
    ['cargo', 'build', '--release', '--offline', '--locked', '-p', 'noteka-pdf-worker', '--bins', '--examples'],
    ['cargo', 'test', '--release', '--offline', '--locked', '-p', 'noteka-core', '--lib',
     'pdf_tests::measurements::release_pdf_resources', '--', '--ignored', '--nocapture', '--test-threads=1'],
]
logs = []
status = 0
for command in commands:
    result = subprocess.run(command, cwd=root, env=env, capture_output=True, timeout=1800)
    text = result.stdout.decode() + result.stderr.decode()
    logs.append('$ ' + ' '.join(command) + '\n' + text)
    if result.returncode:
        status = result.returncode
        break
(directory / 'run.log').write_text('\n'.join(logs))
record = next((line.split('NOTEKA_PDF_BENCH ', 1)[1] for text in logs for line in text.splitlines() if 'NOTEKA_PDF_BENCH {' in line), None)
if record:
    report = json.loads(record)
    report['hardware'] = {'machine': platform.machine()}
    if sys.platform == 'darwin':
        for key in ('machdep.cpu.brand_string', 'hw.memsize'):
            report['hardware'][key] = subprocess.check_output(['/usr/sbin/sysctl', '-n', key], text=True).strip()
    report['method'] = {'parent_heap': 'requested live Rust Layouts; unittest allocator, no production cap',
                        'parent_rss': 'getrusage SELF process high-water',
                        'children_rss': 'getrusage CHILDREN aggregate high-water, waited helpers and runtime probes',
                        'helper_heap_rss': 'separate runtime_metrics process; same operational entrypoint/IPC/IO/allocator, sampled after stdout flush and before metrics JSON',
                        'saturation': '31 MiB structurally valid seeded derivatives, not 31 parser measurements'}
    (directory / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'directory': str(directory), 'cases': len(report['cases']),
                      'maximum_lock_p95_ms': report['maximum_lock_p95_ms'], 'lock_target_met': report['lock_target_met']}))
else:
    print(json.dumps({'directory': str(directory), 'error': 'No measurement record; inspect run.log'}))
sys.exit(status)
