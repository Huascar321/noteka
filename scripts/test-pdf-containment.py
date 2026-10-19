#!/usr/bin/env python3
"""Pruebas de procesos reales sin UI. Fixtures/probes propios, no datos del usuario."""
import argparse
import json
import os
from pathlib import Path
import signal
import struct
import subprocess
import tempfile
import time


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", choices=["debug", "release"], default="debug")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    target = Path(os.environ.get("CARGO_TARGET_DIR", root / "target")).resolve()
    probe = target / args.profile / "examples" / "containment_probe"
    worker = target / args.profile / "noteka-pdf-worker"
    temp_root = Path(os.environ["NOTEKA_TEST_TMP"])
    assert temp_root.is_absolute()
    records = []
    with tempfile.TemporaryDirectory(prefix="pdf-containment-", dir=temp_root) as directory:
        clean_env = {"PATH": "/usr/bin:/bin", "TMPDIR": str(directory)}

        def run(mode, expected, limit=8):
            start = time.monotonic()
            result = subprocess.run([probe, mode], cwd=directory, env=clean_env,
                                    capture_output=True, timeout=limit)
            elapsed = time.monotonic() - start
            assert result.returncode == expected, (mode, result.returncode)
            assert result.stderr == b"", (mode, "unexpected stderr")
            records.append({"mode": mode, "exit": expected, "elapsed_ms": round(elapsed * 1000, 2)})
            return result, elapsed

        result, _ = run("limits", 0)
        assert result.stdout == b"4 4 0 0\n"
        result, _ = run("accounting", 0)
        live, peak = map(int, result.stdout.split())
        assert 0 <= live <= peak <= 128 * 1024 * 1024
        records[-1].update({"heap_live_requested_bytes": live, "heap_peak_requested_bytes": peak})
        result, _ = run("concurrent", 0)
        assert 16_384 <= int(result.stdout) <= 128 * 1024 * 1024
        records[-1]["heap_peak_requested_bytes"] = int(result.stdout)
        for mode, code in [("oom", 73), ("realloc-oom", 73), ("panic", 74), ("abort", -signal.SIGABRT)]:
            result, _ = run(mode, code)
            assert result.stdout == b""
        # macOS entrega SIGXCPU al alcanzar el límite CPU blando (también fijado en 4 s).
        _, elapsed = run("cpu", -signal.SIGXCPU)
        assert elapsed < 6.5
        _, elapsed = run("short-budget", 75)
        assert 0.075 <= elapsed < 1.5

        # No enviar ningún byte y mantener stdin abierto: el watchdog debe cerrar el proceso.
        start = time.monotonic()
        child = subprocess.Popen([worker], cwd=directory, env=clean_env,
                                 stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            assert child.wait(timeout=7) == 75
            assert child.stdout.read() == b"" and child.stderr.read() == b""
        finally:
            if child.poll() is None:
                child.kill()
                child.wait(timeout=2)
            for pipe in (child.stdin, child.stdout, child.stderr):
                pipe.close()
        records.append({"mode": "worker-blocked-input", "exit": 75,
                        "elapsed_ms": round((time.monotonic() - start) * 1000, 2)})

        # Presupuesto pequeño válido y EOF; respuesta cerrada, nunca un panic o contenido.
        header = b"NKPDF001" + struct.pack("<HHIII", 1, 0, 1, 100, 0)
        result = subprocess.run([worker], input=header, cwd=directory, env=clean_env,
                                capture_output=True, timeout=2)
        reply = json.loads(result.stdout)
        assert reply["protocol_version"] == 1 and reply["parser_version"] == "0.45.0"
        assert reply["outcome"] == {"status": "error", "reason": "pdf_worker_protocol"}
        assert result.stderr == b""
        records.append({"mode": "worker-closed-error", "exit": result.returncode})

        # El helper conserva su timer cuando SIGKILL elimina al padre ejecutor.
        parent = subprocess.Popen([probe, "parent"], cwd=directory, env=clean_env,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        orphan_pid = None
        terminated = False
        try:
            # read acotado por el propio watchdog del probe padre (5 segundos).
            early = b""
            while orphan_pid is None:
                line = parent.stdout.readline()
                assert line, "parent terminó sin PID del probe"
                if line.startswith(b"pid="):
                    orphan_pid = int(line[4:])
                else:
                    early += line
            parent.kill()
            assert parent.wait(timeout=2) == -signal.SIGKILL
            # stdout pertenece también al child. EOF demuestra que terminó; no necesita
            # ps/setuid (no disponible dentro de sandbox-exec con red denegada).
            stdout, stderr = parent.communicate(timeout=7)
            assert early + stdout == b"ready\n" and stderr == b""
            terminated = True
            records.append({"mode": "parent-sigkill", "helper_terminated": True})
        finally:
            if parent.poll() is None:
                parent.kill()
                parent.wait(timeout=2)
            for pipe in (parent.stdout, parent.stderr):
                pipe.close()
            # Falla del test no deja un probe residente. Este PID sólo es cleanup del harness.
            if orphan_pid and not terminated:
                try:
                    os.kill(orphan_pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
        assert list(Path(directory).iterdir()) == [], "archivo inesperado/core dump"
    print(json.dumps({"profile": args.profile, "checks": records}, indent=2))


if __name__ == "__main__":
    main()
