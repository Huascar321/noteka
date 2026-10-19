#!/usr/bin/env python3
"""Sonda física separada de latencia: sólo fixture sintética preparada por el gate mixto."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--workspace', type=Path, required=True)
    ap.add_argument('--probe-library', type=Path, required=True)
    ap.add_argument('--probe-source', type=Path, required=True)
    args = ap.parse_args()
    workspace = args.workspace.resolve()
    directory = Path(tempfile.mkdtemp(prefix='mixed-physical-', dir=os.environ['NOTEKA_TEST_TMP']))
    binary = workspace.parent / 'bin/noteka'
    library = args.probe_library.resolve()
    records = []

    def call(group, action, payload, name=None):
        command = [str(binary), '--workspace', str(workspace), '--json', group, action]
        if name:
            scratch = directory / name
            scratch.mkdir()
            report = directory / (name + '.json')
            command = ['/usr/bin/env', f'DYLD_INSERT_LIBRARIES={library}',
                       f'NOTEKA_SCRATCH_ROOT={scratch}', f'NOTEKA_SCRATCH_WORKSPACE={workspace}',
                       f'NOTEKA_SCRATCH_REPORT={report}', f'TMPDIR={scratch}/', f'SQLITE_TMPDIR={scratch}', *command]
        command = ['/usr/bin/sandbox-exec', '-p', '(version 1)(allow default)(deny network*)', *command]
        result = subprocess.run(command, input=json.dumps(payload).encode(), capture_output=True, timeout=20)
        assert result.returncode == 0 and not result.stderr, result.stderr
        data = json.loads(result.stdout)
        assert data['ok'], data
        if name:
            stats = json.loads(report.read_text())
            assert stats['files'] > 0 and stats['peak_allocated_file_bytes'] > 0
            assert all(f['open_refs'] == 0 and not f['named'] for f in stats['file_peaks'])
            assert all(f['open_refs'] == 0 for f in stats['workspace_files'])
            records.append(dict(case=name, stats=stats))
        return data['data']

    assert not binary.with_name('noteka-xlsx-worker').exists()
    assert not binary.with_name('noteka-pdf-worker').exists()
    state = call('extraction', 'index', dict(action='get'), 'get')
    assert state['coverage']['text_bytes'] == 8388608 and len(state['items']) == 12
    page = call('extraction', 'search', dict(query='evidencia', limit=1), 'query-page')
    call('extraction', 'search', dict(query='evidencia', limit=1, cursor=page['next_cursor']), 'query-next')
    assert not call('extraction', 'search', dict(query='ausente'), 'query-zero')['items']
    call('extraction', 'search', dict(query='term299', extractor='xlsx_cells'), 'query-distinct')
    call('extraction', 'search', dict(query='evidencia', extractor='pdf_text'), 'query-filter-pdf')
    call('extraction', 'search', dict(query='evidencia', extractor='xlsx_cells'), 'query-filter-xlsx')
    xlsx = next(e for e in state['items'] if e['source']['extractor'] == 'xlsx_cells')
    s = xlsx['source']
    noop = call('extraction', 'index', dict(action='put', job_id=s['job_id'], expected_generation_id=s['generation_id'], expected_revision=state['revision']), 'no-op')
    assert not noop['changed']
    rev = call('extraction', 'index', dict(action='reset', expected_revision=state['revision']), 'reset')['revision']
    entries = sorted(state['items'], key=lambda e: (e['source']['extractor'] == 'xlsx_cells', e['source']['job_id']))
    for i, entry in enumerate(entries):
        source = entry['source']
        result = call('extraction', 'index', dict(action='put', job_id=source['job_id'], expected_generation_id=source['generation_id'], expected_revision=rev), 'put-8MiB' if i == 11 else None)
        rev = result['revision']
    assert call('extraction', 'index', dict(action='get'))['coverage']['text_bytes'] == 8388608
    # Recuperación real del índice con los tres extractores, helpers ausentes.
    backup, restored = directory / 'backup', directory / 'restored'
    before = call('extraction', 'index', dict(action='get'))
    call('workspace', 'backup', dict(destination=str(backup)))
    call('workspace', 'restore', dict(backup=str(backup), destination=str(restored)))
    result = subprocess.run([str(binary), '--workspace', str(restored), '--json', 'extraction', 'index'],
                            input=b'{"action":"get"}', capture_output=True, timeout=20)
    assert result.returncode == 0 and not result.stderr
    assert json.loads(result.stdout)['data'] == before
    result = subprocess.run([str(binary), '--workspace', str(restored), '--json', 'extraction', 'search'],
                            input=b'{"query":"term299","extractor":"xlsx_cells"}', capture_output=True, timeout=20)
    assert result.returncode == 0 and not result.stderr and json.loads(result.stdout)['data']['items']
    report = dict(directory=str(directory), workspace=str(workspace), cases=len(records), records=records, mixed_backup_restore_without_helpers=True,
                  binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                  probe_source_sha256=hashlib.sha256(args.probe_source.read_bytes()).hexdigest(),
                  method='DYLD interpose open/openat/write/pwrite/ftruncate/close/unlink/unlinkat/copyfile/fcopyfile/clonefile/clonefileat/fclonefileat; fstat/stat st_size and st_blocks*512. Retains unlinked-open and named-closed scratch; workspace DB/WAL/SHM classified separately.',
                  limitations='Regular-file allocated blocks, not exclusive APFS/dedup, RAM or every syscall. Separate diagnostic, never latency acceptance.')
    report['peak_allocated_file_bytes'] = max(r['stats']['peak_allocated_file_bytes'] for r in records)
    report['peak_scratch_and_shm_bytes'] = max(r['stats']['peak_temp_including_workspace_shm_allocated_bytes'] for r in records)
    (directory / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k != 'records'}, indent=2))


if __name__ == '__main__':
    main()
