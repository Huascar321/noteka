#!/usr/bin/env python3
"""Gate adicional real PDFKit/UTF-8, 8 MiB; no cambia los gates anteriores."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--native-generations', type=int, default=10)
    args = ap.parse_args(); assert 1 <= args.native_generations <= 16
    base = Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True)
    source = Path(os.environ.get('NOTEKA_PDFKIT_BENCH_SOURCE') or os.environ['NOTEKA_PDFKIT_UI_SOURCE']).resolve(strict=True)
    assert source.is_relative_to(base)
    manifest = json.loads(source.with_name('manifest.json').read_text())
    assert manifest['expected_before_reader'] and not manifest['generator_uses_pdfkit']
    assert hashlib.sha256(source.read_bytes()).hexdigest() == manifest['sha256']
    spec = importlib.util.spec_from_file_location('support', ROOT/'scripts/test-xlsx-search-migration.py')
    support = importlib.util.module_from_spec(spec); spec.loader.exec_module(support)
    root = Path(tempfile.mkdtemp(prefix='pdfkit-search-bench-', dir=base))
    runtime = root/'bin'; runtime.mkdir()
    for name in ['noteka', 'noteka-pdfkit-worker']:
        shutil.copy2(ROOT/'target/release'/name, runtime/name)
    ws = root/'workspace'; cli = runtime/'noteka'
    call = lambda op, data: support.call(cli, ws, *op.split('.'), data)
    call('workspace.create', dict(timezone='UTC'))
    c = call('extraction.control', dict(action='get'))
    call('extraction.control', dict(action='set', paused=False, expected_revision=c['revision']))
    pins = []

    def select(path, extractor, expected):
        a = call('attachment.import', dict(source_path=str(path)))
        j = call('extraction.enqueue', dict(attachment_id=a['id'], extractor=extractor))['job']
        j = call('extraction.run', dict(id=j['id']))['job']; assert j['state'] == 'succeeded'
        if extractor == 'pdfkit_text':
            p = call('extraction.pdfkit-pages', dict(id=j['id'], limit=64))
            assert [p['text'] for p in p['items']] == manifest['pages'] and not p['next_cursor']
        c = call('extraction.index', dict(action='get'))
        call('extraction.index', dict(action='put', job_id=j['id'], expected_generation_id=j['generation']['id'], expected_revision=c['revision']))
        with sqlite3.connect(ws/'noteka.sqlite') as db:
            rows = db.execute('SELECT text FROM extraction_search_fts WHERE job_id=? ORDER BY ordinal', [j['id']]).fetchall()
        assert [r[0] for r in rows] == expected
        pins.append(dict(name=path.name, source_sha256=a['sha256'], extractor=extractor, text_bytes=sum(len(t.encode()) for t in expected)))

    def split(text):
        raw = text.encode(); result = []
        while raw:
            end = min(len(raw),4096)
            while True:
                try: part = raw[:end].decode(); break
                except UnicodeDecodeError: end -= 1
            result.append(part); raw = raw[end:]
        return result

    for n in range(args.native_generations):
        path = root/f'native-{n}.pdf'; shutil.copy2(source, path)
        select(path, 'pdfkit_text', [part for text in manifest['pages'] for part in split(text)])
    left = 8_388_608-sum(p['text_bytes'] for p in pins)
    n = 0
    while left:
        size = min(left, 1_048_576); pattern = 'evidencia literal contrato '; text = (pattern*(size//len(pattern)+1))[:size]
        path = root/f'literal-{n}.txt'; path.write_text(text)
        select(path, 'utf8_text', [text[i:i+4096] for i in range(0,len(text),4096)])
        n += 1; left -= size
    assert call('extraction.index', dict(action='get'))['coverage']['text_bytes'] == 8_388_608
    (runtime/'noteka-pdfkit-worker').rename(runtime/'helper.absent')
    env = {**os.environ, 'NOTEKA_PDFKIT_SEARCH_BENCH_WORKSPACE': str(ws), 'NOTEKA_PDFKIT_SEARCH_BENCH_REPORT': str(root/'measurements.json')}
    p = subprocess.run(['cargo','test','--release','--offline','--locked','-p','noteka-core','release_pdfkit_search_resources','--','--ignored','--nocapture','--test-threads=1'], env=env, capture_output=True, timeout=1800)
    (root/'run.log').write_bytes(p.stdout+p.stderr); assert p.returncode == 0, root
    report = json.loads((root/'measurements.json').read_text()); summary = {}
    for case in sorted({r['case'] for r in report['results']}):
        rows = [r for r in report['results'] if r['case']==case and not r['warmup']]
        assert len(rows)==20 and all(r['ok'] for r in rows), (root,case)
        summary[case] = {}
        for key in ['wall_ms','lock_ms']:
            values = sorted(r[key] for r in rows if r.get(key) is not None)
            if values: summary[case][key] = dict(p50=values[9],p95=values[18],max=values[-1])
    report.update(summary=summary, pins=pins, native_physical_pages=len(manifest['pages'])*args.native_generations,
                  native_text_bytes=sum(p['text_bytes'] for p in pins if p['extractor']=='pdfkit_text'),
                  network_isolation='external sandbox-exec required; see recorded stage command',
                  helper_absent_during_measurement=True)
    report['query_gate_met'] = all(v['wall_ms']['p95']<=300 for k,v in summary.items() if k.startswith('query-'))
    report['lock_gate_met'] = all(v['lock_ms']['p95']<=150 for v in summary.values() if 'lock_ms' in v)
    (root/'results.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(dict(directory=str(root),summary=summary,query_gate_met=report['query_gate_met'],lock_gate_met=report['lock_gate_met'])))
    assert report['query_gate_met'] and report['lock_gate_met'], root


if __name__ == '__main__': main()
