#!/usr/bin/env python3
"""Aceptación PDFKit real, sin ventanas, fuentes personales ni red."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import resource
import selectors
import signal
import struct
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent.parent
RUNTIME = ROOT
MAX_STDOUT = 8 * 1048576
MAX_STDERR = 8192
sys.dont_write_bytecode = True

class Rejected(Exception):
    pass


def bounded(argv, wire=b'', timeout=8, hold_stdin=False, block_stdout=False, close_stdout=False):
    """No acepta pipes ilimitados. Mata/recolecta sólo su grupo hijo al rechazar."""
    command = ['/usr/bin/sandbox-exec', '-p', '(version 1)(allow default)(deny network*)', *map(str, argv)]
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    started = time.monotonic()
    child = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                             start_new_session=True, cwd=RUNTIME, env={'PATH':'/usr/bin:/bin','LANG':'C','TMPDIR':str(RUNTIME)})
    selector = selectors.DefaultSelector()
    output = {'stdout': bytearray(), 'stderr': bytearray()}
    sent = 0
    try:
        for name in ['stdout', 'stderr']:
            stream = getattr(child, name)
            if name == 'stdout' and close_stdout:
                stream.close(); continue
            os.set_blocking(stream.fileno(), False)
            if not (name == 'stdout' and block_stdout):
                selector.register(stream, selectors.EVENT_READ, name)
        if wire:
            os.set_blocking(child.stdin.fileno(), False)
            selector.register(child.stdin, selectors.EVENT_WRITE, 'stdin')
        elif not hold_stdin:
            child.stdin.close()
        while True:
            if time.monotonic() - started > timeout:
                raise Rejected('execution_deadline')
            if child.poll() is not None and not selector.get_map():
                break
            for key, _ in selector.select(.02):
                if key.data == 'stdin':
                    try:
                        sent += os.write(key.fileobj.fileno(), wire[sent:sent+65536])
                    except BrokenPipeError:
                        sent = len(wire)
                    except BlockingIOError:
                        continue
                    if sent == len(wire):
                        selector.unregister(key.fileobj)
                        if not hold_stdin:
                            key.fileobj.close()
                else:
                    try:
                        chunk = os.read(key.fileobj.fileno(), 8192)
                    except BlockingIOError:
                        continue
                    if not chunk:
                        selector.unregister(key.fileobj); key.fileobj.close(); continue
                    limit = MAX_STDOUT if key.data == 'stdout' else MAX_STDERR
                    if len(output[key.data]) + len(chunk) > limit:
                        raise Rejected(key.data + '_limit')
                    output[key.data].extend(chunk)
        code = child.wait(timeout=1)
        after = resource.getrusage(resource.RUSAGE_CHILDREN)
        return dict(code=code, stdout=bytes(output['stdout']), stderr=bytes(output['stderr']),
                    wall_ms=(time.monotonic()-started)*1000,
                    cpu_s=after.ru_utime+after.ru_stime-before.ru_utime-before.ru_stime, pid=child.pid)
    finally:
        if child.poll() is None:
            os.killpg(child.pid, signal.SIGKILL)
        child.wait(timeout=2)
        selector.close()
        for name in ['stdin', 'stdout', 'stderr']:
            stream = getattr(child, name)
            if not stream.closed:
                stream.close()
        try:
            os.kill(child.pid, 0)
        except ProcessLookupError:
            pass
        else:
            raise AssertionError('worker_not_reaped')


def packet(source, ms=5000):
    return b'NKPDK001' + struct.pack('<HHIII', 1, 0, len(source), ms, 0) + source


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--profile', choices=['debug', 'release'], required=True)
    parser.add_argument('--corpus', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--fixtures-only', action='store_true', help='Sólo archivos; contención se verifica en la ejecución completa.')
    args = parser.parse_args()
    assert sys.platform == 'darwin'
    base = Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True)
    corpus = args.corpus.resolve(strict=True)
    out = args.output.resolve()
    assert corpus.is_relative_to(base) and out.is_relative_to(base) and not base.is_relative_to(ROOT)
    out.mkdir()
    global RUNTIME
    RUNTIME = out / 'working'; RUNTIME.mkdir()
    manifest_raw = (corpus / 'manifest.json').read_bytes()
    manifest = json.loads(manifest_raw)
    assert manifest['license'] == 'MIT' and manifest['expected_before_reader'] and not manifest['generator_uses_pdfkit']
    target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target')) / args.profile
    worker, probes = target / 'noteka-pdfkit-worker', target / 'noteka-pdfkit-worker-probes'
    validator = target / 'examples/validate_pdfkit_reply'
    assert all(p.is_file() for p in [worker, probes, validator])
    report = dict(profile=args.profile, headless=True, network_denied=True, workspace_used=False,
                  manifest_sha256=hashlib.sha256(manifest_raw).hexdigest(),
                  binaries={p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in [worker, probes, validator]},
                  cases=[], malformed=[], processes=[], failures=[])

    def checkpoint():
        (out / 'results.partial.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')

    def check(name, function, group):
        try:
            details = function()
            row = dict(name=name, passed=True, **(details or {}))
        except Exception as error:
            row = dict(name=name, passed=False, error=str(error))
            report['failures'].append(name)
        report[group].append(row); checkpoint()

    for case in manifest['cases']:
        def fixture(case=case):
            source = (corpus / case['name']).read_bytes()
            digest = hashlib.sha256(source).hexdigest(); assert digest == case['sha256']
            result = bounded(['/usr/bin/time', '-l', '-o', out / (case['name'] + '.time'), worker], packet(source))
            (out / (case['name'] + '.reply.json')).write_bytes(result['stdout'])
            assert result['code'] == 0 and not result['stderr'], f"process {result['code']} stderr {len(result['stderr'])}"
            validated = bounded([validator, digest], result['stdout'])
            assert validated['code'] == 0 and not validated['stderr'], 'Rust validation failed'
            outcome = json.loads(validated['stdout'])['outcome']
            if 'error' in case:
                assert outcome == dict(status='error', reason=case['error']), str(outcome)
            else:
                assert outcome['status'] == 'success', str(outcome)
                doc = outcome['document']
                actual = [p['text'] for p in doc['pages']]
                assert actual == case['pages'], repr(actual)
                assert doc['page_count'] == len(case['pages'])
                assert doc['text_bytes'] == sum(len(s.encode()) for s in case['pages'])
                assert [p['status'] for p in doc['pages']] == ['text' if p else 'no_text' for p in case['pages']]
            return dict(code=result['code'], wall_ms=result['wall_ms'], cpu_s=result['cpu_s'])
        check(case['name'], fixture, 'cases')

    if args.fixtures_only:
        for case in manifest['cases']:
            assert hashlib.sha256((corpus/case['name']).read_bytes()).hexdigest() == case['sha256']
        assert (corpus/'manifest.json').read_bytes() == manifest_raw
        report['passed'] = not report['failures']
        report['mode'] = 'fixtures_only'
        (out/'results.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n')
        print(json.dumps(dict(cases=len(report['cases']), failures=len(report['failures']), mode=report['mode'])))
        return 0 if report['passed'] else 1

    source = (corpus / 'unicode.pdf').read_bytes()
    wire = packet(source)
    malformed = [b'', b'bad', wire[:23], wire[:-1], wire+b'x',
                 b'NKOCR001'+wire[8:], wire[:8]+b'\x02'+wire[9:],
                 wire[:10]+b'\x01'+wire[11:], wire[:20]+b'\x01'+wire[21:]]
    for size, ms in [(0, 1), (1048577, 1), (1, 0), (1, 5001)]:
        malformed.append(b'NKPDK001'+struct.pack('<HHIII', 1, 0, size, ms, 0))
    for i, raw in enumerate(malformed):
        def bad(raw=raw):
            result = bounded([worker], raw)
            assert result['code'] == 0 and not result['stderr']
            assert json.loads(result['stdout'])['outcome'] == dict(status='error', reason='pdf_worker_protocol')
        check('wire-'+str(i), bad, 'malformed')

    flags = ['sleep','short','cpu','memory','stdout','stderr','internal','signal','limits','text']
    for flag in flags:
        def normal(flag=flag):
            result = bounded([worker, '--probe-'+flag])
            assert result['code'] == 0 and not result['stderr']
            assert json.loads(result['stdout'])['outcome'] == dict(status='error', reason='pdf_worker_protocol')
        check('normal-rejects-'+flag, normal, 'processes')

    for flag, expected in [('sleep',75),('short',75),('cpu',-signal.SIGXCPU),('memory',73),('internal',74),('signal',-signal.SIGTERM)]:
        def control(flag=flag, expected=expected):
            result = bounded([probes, '--probe-'+flag], timeout=7)
            assert result['code'] == expected, str(result)
            assert not result['stdout'] and not result['stderr']
            assert result['wall_ms'] < 6500
            if flag == 'short': assert result['wall_ms'] < 1500
            if flag == 'cpu': assert 3 < result['cpu_s'] < 6
            return {k:result[k] for k in ['code','wall_ms','cpu_s']}
        check('probe-'+flag, control, 'processes')

    for flag, reason in [('stdout','stdout_limit'),('stderr','stderr_limit')]:
        def overflow(flag=flag, reason=reason):
            try: bounded([probes, '--probe-'+flag])
            except Rejected as error: assert str(error) == reason
            else: raise AssertionError('unbounded pipe accepted')
        check('bounded-'+flag, overflow, 'processes')

    def blocked_input():
        result = bounded([worker], packet(source,100)[:24], hold_stdin=True)
        assert result['code'] == 75 and not result['stdout'] and result['wall_ms'] < 1500
    check('blocked-stdin-shortens', blocked_input, 'processes')

    def blocked_output():
        result = bounded([probes,'--probe-stdout'], block_stdout=True)
        assert result['code'] == 75 and result['wall_ms'] < 6500
    check('blocked-stdout-watchdog', blocked_output, 'processes')

    def lost_consumer():
        result = bounded([probes,'--probe-stdout'], close_stdout=True)
        assert result['code'] in [74, -signal.SIGPIPE] and result['wall_ms'] < 6500
    check('lost-stdout-consumer', lost_consumer, 'processes')

    def parent_exit():
        # Un padre efímero deja el helper con /dev/null; sólo se vigila ese PID.
        pid_file = RUNTIME / 'orphan-pid.txt'
        program = ('import subprocess,os,pathlib;'
                   f'p=subprocess.Popen([{str(probes)!r},"--probe-sleep"],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL);'
                   f'pathlib.Path({str(pid_file)!r}).write_text(str(p.pid));os._exit(0)')
        result = bounded(['/usr/bin/python3','-c',program])
        assert result['code'] == 0
        pid = int(pid_file.read_text()); started = time.monotonic()
        disappeared = False
        try:
            while time.monotonic()-started < 6.5:
                try: os.kill(pid,0)
                except ProcessLookupError:
                    disappeared = True
                    return dict(reaped_by_system=True, wall_ms=(time.monotonic()-started)*1000)
                time.sleep(.02)
            raise AssertionError('orphan remained after deadline')
        finally:
            if not disappeared:
                try: os.kill(pid,signal.SIGKILL)
                except ProcessLookupError: pass
    check('parent-exits-watchdog', parent_exit, 'processes')

    def limits():
        result = bounded([probes,'--probe-limits'])
        assert result['code'] == 0 and json.loads(result['stdout']) == dict(core=0,cpu=4)
    check('rlimits-real', limits, 'processes')

    def text():
        result = bounded([probes,'--probe-text'])
        assert result['code'] == 0 and json.loads(result['stdout']) == dict(invalid_utf16=True,text_boundary=True,text_excess=True,nul=True,multibyte_limit=True,combining_unchanged=True)
    check('strict-utf16-and-text-limits', text, 'processes')

    assert not list(RUNTIME.glob('core*'))
    for case in manifest['cases']:
        assert hashlib.sha256((corpus/case['name']).read_bytes()).hexdigest() == case['sha256']
    assert (corpus/'manifest.json').read_bytes() == manifest_raw
    report['passed'] = not report['failures']
    (out / 'results.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n')
    print(json.dumps({k:len(report[k]) for k in ['cases','malformed','processes','failures']}))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())
