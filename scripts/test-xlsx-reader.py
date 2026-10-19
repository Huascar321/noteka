#!/usr/bin/env python3
"""Corpus independiente por parser/helper reales, protocolo, rechazo de salidas y contención.

Ejecutar con sandbox-exec deny network; sólo fixtures sintéticas y cwd temporal.
"""
import argparse
import concurrent.futures
import copy
import hashlib
import importlib.util
import json
import os
import random
from pathlib import Path
import signal
import struct
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True


def header(n, ms=5000):
    return b'NKXLS001' + struct.pack('<HHIII', 1, 0, n, ms, 0)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--profile', choices=['debug', 'release'], default='debug')
    args = ap.parse_args()
    root = Path(__file__).resolve().parent.parent
    target = Path(os.environ.get('CARGO_TARGET_DIR', root / 'target')).resolve() / args.profile
    worker = target / 'noteka-xlsx-worker'
    probe = target / 'examples/extract_probe'
    validator = target / 'examples/validate_reply'
    spec = importlib.util.spec_from_file_location('corpus', root / 'tests/fixtures/xlsx/generate.py')
    corpus = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(corpus)
    cases = corpus.cases()
    manifest_bytes = (json.dumps(corpus.manifest(), ensure_ascii=False, indent=2) + '\n').encode()
    pins_path = root / 'tests/fixtures/xlsx/golden.sha256'
    pins = {n: h for h, n in (line.split() for line in pins_path.read_text().splitlines())}
    assert hashlib.sha256(manifest_bytes).hexdigest() == pins['manifest.json']
    assert hashlib.sha256((root / 'tests/fixtures/xlsx/generate.py').read_bytes()).hexdigest() == pins['generate.py']
    records = []
    with tempfile.TemporaryDirectory(prefix='xlsx-', dir=os.environ['NOTEKA_TEST_TMP']) as tmp:
        directory = Path(tmp)
        env = {'PATH': '/usr/bin:/bin', 'TMPDIR': tmp}

        def raw(binary, data, arguments=(), timeout=8):
            return subprocess.run([str(binary), *arguments], input=data, capture_output=True,
                                  timeout=timeout, cwd=directory, env=env)

        def run_case(case, binary, framed):
            source = case['source']
            before = hashlib.sha256(source).hexdigest()
            started = time.monotonic()
            result = raw(binary, (header(len(source)) if framed else b'') + source)
            assert result.returncode == 0 and result.stderr == b'', (case['name'], result.returncode, result.stderr[:200])
            assert len(result.stdout) <= 8*corpus.CAP, case['name']
            reply = json.loads(result.stdout)
            assert set(reply) == {'version', 'parser', 'outcome'}
            assert reply['version'] == 1 and reply['parser'] == 'zip-9.0.3/roxmltree-0.21.1'
            error = case['error']
            if not framed and len(source) > corpus.CAP:
                error = 'xlsx_structure_limit'
            if error:
                assert reply['outcome'] == dict(status='error', reason=error), (case['name'], reply['outcome'])
            else:
                assert reply['outcome'] == dict(status='success', source_sha256=before, workbook=case['expected']), (case['name'], reply['outcome'])
            assert hashlib.sha256(source).hexdigest() == before
            return dict(name=case['name'], mode='helper' if framed else 'parser', outcome=error or 'success',
                        elapsed_ms=round((time.monotonic()-started)*1000, 3)), reply

        success = None
        for case in cases:
            for binary, framed in [(probe, False), (worker, True)]:
                record, reply = run_case(case, binary, framed)
                records.append(record)
                if case['name'] == 'formula-shared' and framed:
                    success = reply
        # Lecturas realmente concurrentes; no se comparte proceso/allocator ni workspace.
        selected = [c for c in cases if c['name'] in ['cells-10000', 'text-131072', 'formula-shared', 'empty-deflate']]
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            results = list(pool.map(lambda c: run_case(c, worker, True), selected * 3))
        assert len(results) == 12
        records.append(dict(name='independent-concurrency', count=12))

        # Smoke adversarial determinista, adicional al golden (no lo sustituye).
        rng = random.Random(20261015)
        seeds = [c['source'] for c in cases if c['name'] in ['empty-stored', 'empty-deflate', 'formula-shared', 'descriptor-8-True']]
        for index in range(512):
            payload = bytearray(seeds[index % len(seeds)])
            at = rng.randrange(len(payload))
            if index % 3 == 0:
                payload[at] ^= 1 << rng.randrange(8)
            elif index % 3 == 1:
                del payload[at:]
            else:
                payload[at:at+1] = rng.randbytes(rng.randrange(1, 16))
            result = raw(worker, header(len(payload)) + payload)
            assert result.returncode == 0 and result.stderr == b'', ('mutation', index, result.returncode)
            reply = json.loads(result.stdout)
            assert reply['outcome']['status'] in ['success', 'error']
            check = raw(validator, result.stdout, [hashlib.sha256(payload).hexdigest()])
            assert check.returncode == 0 and check.stderr == check.stdout == b'', ('mutation-validator', index)
        records.append(dict(name='deterministic-mutations', seed=20261015, count=512, crashes=0))

        # El futuro supervisor debe rechazar estructura/tipos/metadata adulterados, no sólo JSON inválido.
        expected_hash = success['outcome']['source_sha256']
        assert raw(validator, json.dumps(success).encode(), [expected_hash]).returncode == 0
        corrupted = []
        for key, val in [('version', 2), ('parser', 'other'), ('unknown', True)]:
            bad = copy.deepcopy(success); bad[key] = val; corrupted.append((key, bad))
        bad = copy.deepcopy(success); bad['outcome']['source_sha256'] = '0'*64; corrupted.append(('hash', bad))
        for key, val in [('row', 2), ('column', 2), ('style', 9), ('reference', 'A0'), ('extra', 'private')]:
            bad = copy.deepcopy(success); bad['outcome']['workbook']['sheets'][0]['cells'][0][key] = val
            corrupted.append(('cell-' + key, bad))
        for key, val in [('text', ''), ('shared_index', None), ('reference', 'B1:A1'), ('kind', 'data_table')]:
            bad = copy.deepcopy(success); bad['outcome']['workbook']['sheets'][0]['cells'][0]['formula'][key] = val
            corrupted.append(('formula-' + key, bad))
        for key, val in [('observed_range', 'B2'), ('name', 'a/b'), ('id', 0)]:
            bad = copy.deepcopy(success); bad['outcome']['workbook']['sheets'][0][key] = val
            corrupted.append(('sheet-' + key, bad))
        bad = copy.deepcopy(success); bad['outcome']['workbook']['limitations'] = []; corrupted.append(('limitations', bad))
        for name, bad in corrupted:
            r = raw(validator, json.dumps(bad).encode(), [expected_hash])
            assert r.returncode == 2 and r.stdout == r.stderr == b'', name
        for name, data in [('truncated', b'{'), ('stdout-limit', b' '* (8*corpus.CAP+1))]:
            r = raw(validator, data, [expected_hash])
            assert r.returncode == 2 and r.stdout == r.stderr == b'', name
        records.append(dict(name='reply-validation', rejected=len(corrupted)+2, accepted=1))

        base = next(c['source'] for c in cases if c['name'] == 'empty-stored')
        wire = header(len(base)) + base
        bad_inputs = [b'', wire[:23], header(len(base)+1)+base, wire+b'x', header(0), header(corpus.CAP+1)]
        for index in [0, 8, 10, 11, 20, 21, 22, 23]:
            b = bytearray(wire); b[index] ^= 1; bad_inputs.append(bytes(b))
        for ms in [0, 5001]:
            bad_inputs.append(header(len(base), ms)+base)
        for data in bad_inputs:
            r = raw(worker, data)
            assert r.returncode == 0 and r.stderr == b''
            assert json.loads(r.stdout)['outcome'] == dict(status='error', reason='xlsx_worker_protocol')
        records.append(dict(name='raw-framing', rejected=len(bad_inputs)))

        def close(child):
            if child.poll() is None:
                child.kill(); child.wait(timeout=2)
            for pipe in [child.stdin, child.stdout, child.stderr]:
                if pipe:
                    pipe.close()

        # Timer iniciado antes de stdin; EOF no es necesario para que termine.
        for name, payload, seconds in [('blocked-header', b'', 5), ('blocked-source', header(len(base), 100), .1),
                                       ('blocked-eof', header(len(base), 100)+base, .1)]:
            started = time.monotonic()
            child = subprocess.Popen([str(worker)], cwd=directory, env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            try:
                if payload:
                    child.stdin.write(payload); child.stdin.flush()
                assert child.wait(timeout=7) == 75, name
                assert child.stdout.read() == child.stderr.read() == b''
                elapsed = time.monotonic()-started
                assert seconds*.7 <= elapsed < seconds+1.5, (name, elapsed)
                records.append(dict(name=name, exit=75, elapsed_ms=round(elapsed*1000, 2)))
            finally:
                close(child)
        # Resultado suficientemente grande para bloquear stdout; productor no espera al consumidor para morir.
        source = next(c['source'] for c in cases if c['name'] == 'result-1048576')
        child = subprocess.Popen([str(worker)], cwd=directory, env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            child.stdin.write(header(len(source), 500)+source); child.stdin.close(); child.stdin = None
            assert child.wait(timeout=3) == 75
            assert child.stderr.read() == b''
            records.append(dict(name='blocked-stdout', exit=75))
        finally:
            close(child)
        # Consumidor cerrado: no stderr sensible, core dumps ni proceso residente.
        child = subprocess.Popen([str(worker)], cwd=directory, env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            child.stdout.close(); child.stdout = None
            child.stdin.write(header(len(base))+base); child.stdin.close(); child.stdin = None
            assert child.wait(timeout=3) in [70, -signal.SIGPIPE]
            assert child.stderr.read() == b''
            records.append(dict(name='closed-consumer', exit=child.returncode))
        finally:
            close(child)
        # El padre muerto no cierra el pipe stdin que el harness mantiene abierto.
        parent_code = 'import subprocess,sys; c=subprocess.Popen([sys.argv[1]],stdin=sys.stdin,stdout=sys.stdout,stderr=sys.stderr); print(c.pid,flush=True); c.wait()'
        parent = subprocess.Popen([sys.executable, '-c', parent_code, str(worker)], cwd=directory, env=env,
                                  stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        orphan = None
        finished = False
        try:
            orphan = int(parent.stdout.readline())
            parent.kill(); assert parent.wait(timeout=2) == -signal.SIGKILL
            # read EOF sin communicate: communicate cerraría stdin del harness.
            import select
            ready, _, _ = select.select([parent.stdout], [], [], 7)
            assert ready and parent.stdout.read() == b'' and parent.stderr.read() == b''
            finished = True
            records.append(dict(name='parent-killed', helper_terminated=True))
        finally:
            close(parent)
            if orphan and not finished:
                try: os.kill(orphan, signal.SIGKILL)
                except ProcessLookupError: pass
        assert list(directory.iterdir()) == [], 'el lector creó archivos/core dumps'
    print(json.dumps(dict(profile=args.profile, corpus_cases=len(cases), checks=records), indent=2))


if __name__ == '__main__':
    main()
