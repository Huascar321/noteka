#!/usr/bin/env python3
"""La entrega del lector NO debe anunciar XLSX en la cola/catálogo de dominio."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--profile', choices=['debug', 'release'], default='debug')
    ap.add_argument('--baseline-catalog', required=True)
    args = ap.parse_args()
    root = Path(__file__).resolve().parent.parent
    cli = Path(os.environ.get('CARGO_TARGET_DIR', root / 'target')).resolve() / args.profile / 'noteka'
    spec = importlib.util.spec_from_file_location('corpus', root / 'tests/fixtures/xlsx/generate.py')
    corpus = importlib.util.module_from_spec(spec); spec.loader.exec_module(corpus)
    with tempfile.TemporaryDirectory(prefix='xlsx-nonactivation-', dir=os.environ['NOTEKA_TEST_TMP']) as tmp:
        p = Path(tmp); workspace = p / 'workspace'

        def call(group, action, payload=None, reason=None):
            r = subprocess.run([str(cli), '--workspace', str(workspace), '--json', group, action],
                               input=json.dumps(payload or {}).encode(), capture_output=True, timeout=12, cwd=p)
            assert r.stderr == b'', r.stderr[:200]
            j = json.loads(r.stdout)
            assert j['api_version'] == 1
            if reason:
                assert r.returncode == 2 and not j['ok'] and j['error']['details']['reason'] == reason, j
                return j['error']
            assert r.returncode == 0 and j['ok'], j
            return j['data']

        catalog = call('operations', 'describe')
        assert catalog == json.loads(Path(args.baseline_catalog).read_text())['data']
        call('workspace', 'create', {'timezone': 'UTC'})
        control = call('extraction', 'control', {'action': 'get'})
        assert control['paused']
        source = p / 'synthetic.xlsx'; source.write_bytes(corpus.pack(corpus.parts()))
        attachment = call('attachment', 'import', {'source_path': str(source)})
        with sqlite3.connect(workspace / 'noteka.sqlite') as c:
            assert c.execute('SELECT schema_version FROM workspace_meta').fetchone() == (10,)
            before = list(c.iterdump())
            for extractor, reason in [('utf8_text', 'unsupported_source_format'), ('pdf_text', 'unsupported_source_format'), ('xlsx_cells', 'unsupported_extractor_config')]:
                call('extraction', 'enqueue', {'attachment_id': attachment['id'], 'extractor': extractor}, reason)
            call('workspace', 'open')
            call('extraction', 'list')
            assert call('extraction', 'control', {'action': 'get'}) == control
            assert list(c.iterdump()) == before
            assert c.execute('SELECT count(*) FROM extraction_jobs').fetchone() == (0,)
            assert not c.execute("SELECT 1 FROM sqlite_schema WHERE name LIKE '%xlsx%'").fetchall()
    print(json.dumps(dict(profile=args.profile, schema_version=10, catalog_baseline_equal=True,
                          xlsx_not_enqueuable=True, reads_unchanged=True, queue_paused=True)))


if __name__ == '__main__':
    main()
