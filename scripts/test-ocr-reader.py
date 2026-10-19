#!/usr/bin/env python3
"""OCR real independiente, protocolo Rust y procesos; fixtures/evidencia externas."""
import argparse
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
MAX_SOURCE = 1048576
MAX_STDOUT = 8 * 1048576
MAX_STDERR = 8192
sys.dont_write_bytecode = True


class Rejected(Exception):
    pass


def bounded(argv, wire=b'', timeout=8, hold_stdin=False, block_stdout=False):
    """No acepta pipes ilimitados. Mata/recolecta sólo su grupo hijo al rechazar."""
    command = ['/usr/bin/sandbox-exec', '-p', '(version 1)(allow default)(deny network*)', *map(str, argv)]
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    started = time.monotonic()
    child = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                             start_new_session=True, cwd=ROOT)
    selector = selectors.DefaultSelector()
    output = {'stdout': bytearray(), 'stderr': bytearray()}
    sent = 0
    try:
        for name in ['stdout', 'stderr']:
            stream = getattr(child, name)
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


WIRE_VERSION = 1


def packet(source, ms=5000):
    return (b'NKOCR001' if WIRE_VERSION == 1 else b'NKOCR002') + struct.pack('<HHIII', WIRE_VERSION, 0, len(source), ms, 0) + source


def distance(a, b):
    row = list(range(len(b)+1))
    for i, left in enumerate(a, 1):
        previous, row = row, [i]
        for j, right in enumerate(b, 1):
            row.append(min(row[-1]+1, previous[j]+1, previous[j-1]+(left != right)))
    return row[-1]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--wire-version', type=int, choices=[1, 2], default=1)
    ap.add_argument('--profile', choices=['debug', 'release'], default='debug')
    ap.add_argument('--corpus', type=Path, required=True)
    ap.add_argument('--output', type=Path, required=True)
    args = ap.parse_args()
    global WIRE_VERSION
    WIRE_VERSION = args.wire_version
    assert sys.platform == 'darwin', 'Real Vision acceptance requires macOS'
    base = Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True)
    corpus = args.corpus.resolve(strict=True)
    out = args.output.resolve()
    assert corpus.is_relative_to(base) and out.is_relative_to(base) and not base.is_relative_to(ROOT)
    out.mkdir()
    manifest_raw = (corpus/'manifest.json').read_bytes()
    manifest = json.loads(manifest_raw)
    assert manifest['license'] == 'MIT' and manifest['expected_before_ocr']
    assert manifest['quality_max_aggregate_cer'] == .05
    target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT/'target')).resolve()/args.profile
    worker = target/'noteka-ocr-worker'
    probes = target/'noteka-ocr-worker-probes'
    validator = target/('examples/validate_ocr_reply' if WIRE_VERSION == 1 else 'examples/validate_ocr_reply_v2')
    assert all(p.is_file() for p in [worker, probes, validator])
    report = {'wire_version': WIRE_VERSION, 'profile': args.profile, 'headless': True, 'network_denied': True, 'workspace_used': False,
              'manifest_sha256': hashlib.sha256(manifest_raw).hexdigest(),
              'binaries': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in [worker, probes, validator]},
              'cases': [], 'processes': [], 'malformed': [], 'failures': []}
    def checkpoint():
        (out/'results.partial.json').write_text(json.dumps(report, indent=2, ensure_ascii=False)+'\n')
    checkpoint()
    characters = character_errors = words = word_errors = 0
    for case in manifest['cases']:
        source = (corpus/case['name']).read_bytes()
        digest = hashlib.sha256(source).hexdigest(); assert digest == case['sha256']
        timing = out/(case['name']+'.time')
        result = bounded(['/usr/bin/time', '-l', '-o', timing, worker], packet(source))
        (out/(case['name']+'.reply.json')).write_bytes(result['stdout'])
        valid = bounded([validator, digest], result['stdout'])
        row = {'name': case['name'], 'code': result['code'], 'wall_ms': result['wall_ms'], 'cpu_s': result['cpu_s'],
               'stderr_bytes': len(result['stderr']), 'rust_validated': valid['code'] == 0}
        if result['code'] != 0 or result['stderr'] or valid['code'] != 0:
            report['failures'].append(case['name']+': process/protocol'); report['cases'].append(row); checkpoint(); continue
        data = json.loads(valid['stdout'])['outcome']
        if 'error' in case:
            row['error'] = data.get('reason')
            if data != {'status': 'error', 'reason': case['error']}:
                report['failures'].append(case['name']+': expected error')
        else:
            if data['status'] != 'success':
                report['failures'].append(case['name']+': recognition failed'); report['cases'].append(row); checkpoint(); continue
            recognition = data['recognition']; image = recognition['image']
            assert data['source_sha256'] == digest
            assert image == {k: case[k] for k in ['format', 'width', 'height', 'orientation', 'oriented_width', 'oriented_height']}
            actual = '\n'.join(r['text'] for r in recognition['regions'])
            expected = '\n'.join(case['expected_lines'])
            row.update(expected=expected, recognized=actual, regions=len(recognition['regions']),
                       confidence=[r['confidence'] for r in recognition['regions']], engine=recognition['engine'])
            if case['quality']:
                cer = distance(list(expected), list(actual)); wer = distance(expected.split(), actual.split())
                row.update(character_errors=cer, character_count=len(expected), cer=cer/len(expected),
                           word_errors=wer, word_count=len(expected.split()), wer=wer/len(expected.split()))
                characters += len(expected); character_errors += cer; words += len(expected.split()); word_errors += wer
            elif actual:
                report['failures'].append(case['name']+': nonempty blank')
        report['cases'].append(row)
        checkpoint()
    report['quality'] = {'scope': manifest['quality_profile'], 'normalization': manifest['quality_normalization'],
                         'characters': characters, 'character_errors': character_errors, 'cer': character_errors/characters,
                         'words': words, 'word_errors': word_errors, 'wer': word_errors/words, 'target_cer': .05}
    if report['quality']['cer'] > .05:
        report['failures'].append('quality_cer')
    checkpoint()

    first = (corpus/manifest['cases'][0]['name']).read_bytes()
    good = packet(first)
    reserved = bytearray(good); reserved[10] = 1
    malformed = [('empty', b'', 'ocr_worker_protocol'),
                 ('magic', b'X'+good[1:], 'ocr_worker_protocol'),
                 ('reserved', bytes(reserved), 'ocr_worker_protocol'),
                 ('truncated', good[:-1], 'ocr_worker_protocol'),
                 ('extra', good+b'NEVER_ECHO_PRIVATE_MARKER', 'ocr_worker_protocol'),
                 ('zero-size', packet(b''), 'ocr_worker_protocol'),
                 ('over-source', packet(b'X'*(MAX_SOURCE+1)), 'ocr_worker_protocol'),
                 ('zero-ms', packet(first, 0), 'ocr_worker_protocol'),
                 ('over-ms', packet(first, 5001), 'ocr_worker_protocol'),
                 ('invalid-image', packet(b'NEVER_ECHO_PRIVATE_MARKER'), 'ocr_invalid_image')]
    for name, wire, expected_error in malformed:
        result = bounded([worker], wire)
        # Sin encabezado completo no hay versión negociada: el helper conserva
        # el error v1. Un consumidor v2 MUST rechazarlo como protocolo, sin
        # aceptar respuestas v1 exitosas ni reinterpretar el payload.
        if WIRE_VERSION == 2 and name == 'empty':
            assert bounded([validator, 'a'*64], result['stdout'])['code'] != 0
            valid = bounded([target/'examples/validate_ocr_reply', 'a'*64], result['stdout'])
            assert json.loads(result['stdout'])['version'] == 1
        else:
            valid = bounded([validator, 'a'*64], result['stdout'])
        assert result['code'] == 0 and not result['stderr'] and valid['code'] == 0, name
        data = json.loads(valid['stdout'])['outcome']
        assert data == {'status': 'error', 'reason': expected_error}, (name, data)
        assert b'NEVER_ECHO_PRIVATE_MARKER' not in result['stdout']
        report['malformed'].append({'case': name, 'error': data['reason'], 'wall_ms': result['wall_ms']})
        checkpoint()

    for stream, size in [('stdout', MAX_STDOUT+1), ('stderr', MAX_STDERR+1)]:
        try:
            bounded([sys.executable, '-c', f'import os;os.write({1 if stream=="stdout" else 2},b"A"*{size})'])
        except Rejected as error:
            assert str(error) == stream+'_limit'
            report['processes'].append({'case': stream+'_excess', 'rejected': str(error), 'reaped': True})
            checkpoint()
        else:
            raise AssertionError(stream+'_excess accepted')
    try:
        bounded([sys.executable, '-c', 'import time;time.sleep(10)'], timeout=.15)
    except Rejected as error:
        assert str(error) == 'execution_deadline'
        report['processes'].append({'case': 'consumer_deadline', 'rejected': str(error), 'reaped': True})
        checkpoint()
    else:
        raise AssertionError('consumer deadline failed')

    checks = [('--probe-short', {75}, False, False, 1.5), ('--probe-sleep', {75}, False, False, 6.5),
              ('--probe-memory', {73}, False, False, 3), ('--probe-cpu', {-signal.SIGKILL, -signal.SIGXCPU, 75}, False, False, 6.5),
              ('--probe-stdout', {75}, False, True, 6.5), ('--probe-internal', {74}, False, False, 1.5),
              ('--probe-limits', {0}, False, False, 1.5), ('--probe-profile', {0}, False, False, 1.5)]
    for flag, codes, hold, block, seconds in checks:
        result = bounded([probes, flag], timeout=seconds+1, hold_stdin=hold, block_stdout=block)
        assert result['code'] in codes and not result['stderr'] and result['wall_ms'] <= seconds*1000, (flag, result['code'])
        if flag == '--probe-limits': assert json.loads(result['stdout']) == {'core': 0, 'cpu': 4}
        if flag == '--probe-profile': assert json.loads(result['stdout']) == {'predicate_checks': True}
        if flag == '--probe-cpu': assert result['cpu_s'] <= 4.5, result['cpu_s']
        report['processes'].append({'case': flag, 'code': result['code'], 'wall_ms': result['wall_ms'], 'cpu_s': result['cpu_s'], 'reaped': True})
        checkpoint()
    blocked = bounded([worker], hold_stdin=True)
    assert blocked['code'] == 75 and not blocked['stdout'] and not blocked['stderr'] and blocked['wall_ms'] <= 6500
    report['processes'].append({'case': 'blocked_input', 'code': blocked['code'], 'wall_ms': blocked['wall_ms'], 'reaped': True})
    shorter = bounded([worker], packet(first, 100)[:24], hold_stdin=True)
    assert shorter['code'] == 75 and not shorter['stdout'] and not shorter['stderr'] and shorter['wall_ms'] <= 1500
    report['processes'].append({'case': 'real_header_shortens_blocked_body', 'code': shorter['code'], 'wall_ms': shorter['wall_ms'], 'reaped': True})
    # La optimización puede construir strings cortos sin guardar un literal UTF-8
    # contiguo. Se comprueba cada comportamiento, no el layout del binario.
    for flag, *_ in checks:
        hidden = bounded([worker, flag])
        assert hidden['code'] == 0 and not hidden['stderr'] and json.loads(hidden['stdout'])['outcome'] == {'status': 'error', 'reason': 'ocr_worker_protocol'}
        report['processes'].append({'case': 'production_rejects_'+flag, 'code': hidden['code'], 'reaped': True})
    (out/'results.json').write_text(json.dumps(report, indent=2, ensure_ascii=False)+'\n')
    print(json.dumps({'directory': str(out), 'corpus_cases': len(report['cases']), 'malformed': len(report['malformed']),
                      'process_cases': len(report['processes']), 'quality': report['quality'], 'failures': report['failures']}, indent=2))
    assert not report['failures'], report['failures']


if __name__ == '__main__':
    main()
