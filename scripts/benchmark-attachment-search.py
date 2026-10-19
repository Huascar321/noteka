#!/usr/bin/env python3
"""Benchmark explícito offline; evidencia sólo en temporal externo, sin UI."""
import argparse
import json
import math
import os
from pathlib import Path
import platform
import subprocess
import tempfile
import time


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--baseline-root', type=Path, help='Copia independiente v9 para comparar notas/10.000')
    p.add_argument('--baseline-cli', type=Path, help='CLI v9 independiente para comparar ediciones de notas')
    args = p.parse_args()
    root = Path(__file__).resolve().parent.parent
    directory = Path(tempfile.mkdtemp(prefix='attachment-bench-', dir=os.environ['NOTEKA_TEST_TMP']))
    assert not directory.resolve().is_relative_to(root)
    env = {**os.environ, 'CARGO_NET_OFFLINE': 'true', 'NOTEKA_ATTACHMENT_BENCH_REPORT': str(directory / 'index.json')}

    def run(name, command):
        result = subprocess.run(command, cwd=root, env=env, capture_output=True, timeout=1800)
        (directory / f'{name}.log').write_bytes(result.stdout + result.stderr)
        assert result.returncode == 0, f'{name}: inspeccioná {directory}'
        return result.stdout.decode()

    run('index', ['cargo', 'test', '--release', '--offline', '--locked', '-p', 'noteka-core',
                  'release_attachment_index_resources', '--', '--ignored', '--nocapture', '--test-threads=1'])
    report = json.loads((directory / 'index.json').read_text())
    report['hardware'] = {'platform': platform.platform(), 'machine': platform.machine()}
    if platform.system() == 'Darwin':
        for key in ('machdep.cpu.brand_string', 'hw.memsize'):
            report['hardware'][key] = subprocess.check_output(['/usr/sbin/sysctl', '-n', key], text=True).strip()
    summary = {}
    for case in sorted({r['case'] for r in report['results']}):
        rows = [r for r in report['results'] if r['case'] == case and not r['warmup']]
        assert len(rows) == 20
        item = {'samples': len(rows), 'ok': all(r['ok'] for r in rows)}
        for key in ('wall_ms', 'lock_ms', 'editor_wait_ms'):
            values = sorted(r[key] for r in rows if r.get(key) is not None)
            if values:
                item[key] = {'p95': values[math.ceil(len(values) * .95) - 1], 'max': max(values)}
        summary[case] = item
    report['summary'] = summary
    report['lock_gate_met'] = all(s['lock_ms']['p95'] <= 150 for s in summary.values() if 'lock_ms' in s)
    report['query_target_met'] = all(s['wall_ms']['p95'] <= 300 for k, s in summary.items() if k.startswith('query-'))
    report['notes_comparison'] = None
    report['edits_comparison'] = None
    if args.baseline_root:
        baseline = args.baseline_root.resolve()
        assert baseline != root and (baseline / 'Cargo.toml').is_file()
        old = run('notes-v9', ['cargo', 'run', '--release', '--offline', '--locked', '--manifest-path',
                              str(baseline / 'Cargo.toml'), '--target-dir', str(directory / 'baseline-target'),
                              '-p', 'noteka-core', '--example', 'search_benchmark'])
        new = run('notes-v10', ['cargo', 'run', '--release', '--offline', '--locked', '-p', 'noteka-core',
                               '--example', 'search_benchmark'])
        report['notes_comparison'] = {'v9': json.loads(old), 'v10': json.loads(new)}
    if args.baseline_cli:
        target = Path(os.environ.get('CARGO_TARGET_DIR', root / 'target')) / 'release' / 'noteka'
        comparison = {}
        for label, binary in [('v9', args.baseline_cli.resolve()), ('v10', target)]:
            ws = directory / f'edit-{label}'

            def cli(group, action, payload):
                result = subprocess.run([str(binary), '--workspace', str(ws), '--json', group, action],
                                        input=json.dumps(payload).encode(), capture_output=True, env=env, timeout=10)
                assert result.returncode == 0 and not result.stderr
                return json.loads(result.stdout)['data']

            cli('workspace', 'create', {'timezone': 'UTC'})
            note = cli('entity', 'create', {'kind': 'note', 'title': 'edición sintética'})
            samples = []
            for n in range(25):
                start = time.perf_counter()
                note = cli('entity', 'update', {'id': note['id'], 'expected_revision': note['revision'], 'title': f'edit {n}',
                    'document': {'schema_version': 2, 'blocks': [{'id': 'benchmark', 'type': 'paragraph',
                        'content': [{'type': 'text', 'text': ('local ' * 1800)[:10_240]}]}]}})
                if n >= 5:
                    samples.append((time.perf_counter() - start) * 1000)
            comparison[label] = {'warmups': 5, 'samples': samples, 'p95_ms': sorted(samples)[18],
                                 'scope': 'CLI real, startup+preflight+update documento de 10 KiB ASCII; fixture vacía salvo una nota, sin índice seleccionado'}
        report['edits_comparison'] = comparison
    report['method'] = {'rss': 'RUSAGE_SELF high-water del proceso de tests; bytes en macOS, no del compilador',
                        'scratch': 'PRAGMA temp.page_count*page_size del verificador; lógico, no pico RSS ni todos los archivos temporales SQL',
                        'editor': 'conexión SQLite independiente con barrera; espera BEGIN IMMEDIATE/ROLLBACK, no aceptación nativa',
                        'saturation': 'pruebas separadas exact_entry_fragment_and_utf8_byte_caps_are_independent_without_eviction y limits_tests',
                        'notes': 'mismo example search_benchmark histórico, 10.000 notas/80 muestras; no corpus de adjuntos'}
    (directory / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'directory': str(directory), 'lock_gate_met': report['lock_gate_met'],
                      'query_target_met': report['query_target_met'], 'summary': summary}, indent=2))


if __name__ == '__main__':
    main()
