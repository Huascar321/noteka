#!/usr/bin/env python3
"""Gate parcial del adapter experimental, en procesos contenidos sin UI."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True


def main():
    args = argparse.ArgumentParser()
    args.add_argument("--profile", choices=["debug", "release"], default="debug")
    profile = args.parse_args().profile
    root = Path(__file__).resolve().parent.parent
    spec = importlib.util.spec_from_file_location("pdf_corpus", root / "tests/fixtures/pdf/generate.py")
    corpus = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(corpus)
    cases = corpus.cases()
    assert len({case["name"] for case in cases}) == len(cases), "nombres de casos duplicados"
    fixture_root = root / "tests/fixtures/pdf"
    expected = {name: digest for digest, name in
                (line.split() for line in (fixture_root / "golden.sha256").read_text().splitlines())}
    manifest_bytes = (json.dumps(corpus.manifest(), indent=2, ensure_ascii=False) + "\n").encode()
    assert hashlib.sha256(manifest_bytes).hexdigest() == expected["corpus-manifest.json"], "cambió el corpus/golden/zlib: revisar, no actualizar automáticamente"
    assert hashlib.sha256((fixture_root / "generate.py").read_bytes()).hexdigest() == expected["generate.py"], "cambió el generador: revisar procedencia/golden"
    binary = Path(os.environ.get("CARGO_TARGET_DIR", root / "target")).resolve() / profile / "examples/parser_probe"
    records = []
    failures = []
    with tempfile.TemporaryDirectory(prefix="pdf-parser-", dir=os.environ["NOTEKA_TEST_TMP"]) as temp:
        for case in cases:
            result = subprocess.run([binary], input=case["source"], capture_output=True, timeout=7,
                                    cwd=temp, env={"PATH": "/usr/bin:/bin", "TMPDIR": temp})
            try:
                assert result.returncode == 0, ("exit", result.returncode)
                assert result.stderr == b"", "stderr no vacío"
                reply = json.loads(result.stdout)
                assert set(reply) == {"protocol_version", "extractor_version", "parser_name", "parser_version", "outcome"}
                assert reply["protocol_version"] == 1 and reply["extractor_version"] == 1
                assert reply["parser_name"] == "lopdf" and reply["parser_version"] == "0.45.0"
                if case["error"]:
                    assert reply["outcome"] == {"status": "error", "reason": case["error"]}, reply["outcome"]
                else:
                    outcome = reply["outcome"]
                    assert outcome["status"] == "success", outcome
                    assert outcome["source_sha256"] == hashlib.sha256(case["source"]).hexdigest()
                    assert outcome["pages"] == [{"page_number": n, "text": text} for n, text in enumerate(case["texts"], 1)], "golden difiere"
                records.append({"name": case["name"], "outcome": case["error"] or "success"})
            except (AssertionError, ValueError) as error:
                failures.append({"name": case["name"], "error": str(error)})
        assert list(Path(temp).iterdir()) == [], "archivo inesperado/core dump"
    print(json.dumps({"profile": profile, "cases": len(records), "checks": records, "failures": failures}, indent=2))
    assert not failures, f"{len(failures)} casos fallaron"


if __name__ == "__main__":
    main()
