#!/usr/bin/env python3
"""PDF MIT propio→OCR vigente→proyector independiente; sin modelos/red/UI."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import time

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent.parent


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--profile', choices=['debug', 'release'], required=True)
    ap.add_argument('--output', type=Path, required=True)
    args = ap.parse_args()
    base = Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True)
    out = args.output.resolve()
    assert out != base and out.is_relative_to(base) and not out.exists() and not out.is_relative_to(ROOT)
    out.mkdir()
    spec = importlib.util.spec_from_file_location('pdf', ROOT/'tests/fixtures/pdf/generate.py')
    pdf = importlib.util.module_from_spec(spec); spec.loader.exec_module(pdf)
    golden = [['Item', 'Count'], ['Alpha', '12.50'], ['Beta', '003']]
    (out/'golden-before-ocr.json').write_text(json.dumps(dict(license='MIT; own synthetic corpus', cells=golden, outside=['Footer']), indent=2)+'\n')
    content = b'BT /F1 24 Tf 50 700 Td (Item) Tj 300 0 Td (Count) Tj -300 -60 Td (Alpha) Tj 300 0 Td (12.50) Tj -300 -60 Td (Beta) Tj 300 0 Td (003) Tj -300 -480 Td (Footer) Tj ET'
    source = out/'table.pdf'; original = pdf.serialize(pdf.document([content])); source.write_bytes(original)
    runtime = out/'product-bin'; runtime.mkdir()
    for name in ['noteka', 'noteka-ocr-worker', 'noteka-pdf-raster-worker']:
        shutil.copy2(ROOT/'target'/args.profile/name, runtime/name)
    projector = out/'project'; shutil.copy2(ROOT/'target'/args.profile/'examples/project', projector)
    ws = out/'workspace'
    sandbox = ['/usr/bin/sandbox-exec', '-p', '(version 1)(allow default)(deny network*)'] if sys.platform == 'darwin' else []

    def cli(operation, data):
        command = sandbox+[str(runtime/'noteka'), '--json', '--workspace', str(ws), *[s.replace('_', '-') for s in operation.split('.')]]
        p = subprocess.run(command, input=json.dumps(data).encode(), capture_output=True, timeout=15)
        r = json.loads(p.stdout); assert p.returncode == 0 and not p.stderr and r['ok'], r
        return r['data']

    assert cli('workspace.create', dict(timezone='UTC'))['schema_version'] == 16
    attachment = cli('attachment.import', dict(source_path=str(source)))
    assert attachment['sha256'] == hashlib.sha256(original).hexdigest()
    c = cli('extraction.control', dict(action='get'))
    cli('extraction.control', dict(action='set', paused=False, expected_revision=c['revision']))
    job = cli('extraction.enqueue', dict(attachment_id=attachment['id'], extractor='pdf_ocr'))['job']
    result = cli('extraction.run', dict(id=job['id']))['job']; assert result['state'] == 'succeeded'
    page = cli('extraction.pdf_ocr_regions', dict(id=job['id'], limit=64)); assert page['covers_generation']
    (out/'actual-ocr.json').write_text(json.dumps(page, ensure_ascii=False, indent=2)+'\n')
    descriptor = page['manifest']['pages'][0]; raster = descriptor['raster']
    evidence = dict(version=1, image=dict(format='png', width=raster['width'], height=raster['height'], orientation=1, oriented_width=raster['width'], oriented_height=raster['height']), engine=descriptor['engine'], coordinate_space=descriptor['coordinate_space'], limitations=descriptor['limitations'], regions=[{k:i[k] for k in ['ordinal', 'text', 'confidence', 'bbox']} for i in page['items']])
    request = dict(source_sha256=attachment['sha256'], page_number=1, recognition=evidence, x_edges=['0', '0.5', '1'], y_min='0.5', y_max='1', header_row=1)
    (out/'input.json').write_text(json.dumps(request, ensure_ascii=False, indent=2)+'\n')

    def project(raw, expected=None):
        r = subprocess.run(sandbox+[str(projector)], input=raw, capture_output=True, timeout=5)
        assert not r.stderr and len(r.stdout) <= 2_097_152
        data = json.loads(r.stdout)
        assert data['ok'] == (r.returncode == 0)
        if expected:
            assert not data['ok'] and data['reason'] == expected and set(data) == {'ok', 'reason'}, data
        else:
            assert data['ok'], data
        return data

    response = project(json.dumps(request).encode()); layout = response['layout']
    assert layout['evidence'] == request
    def texts(row):
        return ['\n'.join(evidence['regions'][i]['text'] for i in c['region_ordinals']) for c in row['cells']]
    actual = [texts(row) for row in layout['rows']]
    # Golden declarado ANTES de OCR: no casefold, limpieza de espacios o corrección.
    (out/'actual-cells.json').write_text(json.dumps(dict(cells=actual, outside=[evidence['regions'][i]['text'] for i in layout['outside']]), ensure_ascii=False, indent=2)+'\n')
    assert actual == golden, (actual, golden)
    assert [evidence['regions'][i]['text'] for i in layout['outside']] == ['Footer']
    referenced = layout['outside']+[i for row in layout['rows'] for c in row['cells'] for i in c['region_ordinals']]
    assert sorted(referenced) == list(range(len(evidence['regions'])))
    (out/'layout.json').write_text(json.dumps(layout, ensure_ascii=False, indent=2)+'\n')
    before = hashlib.sha256((ws/'attachments'/attachment['id']).read_bytes()).hexdigest()
    for raw in [b'', b'{}', json.dumps(request).encode()+b'{}', json.dumps({**request, 'workspace': '/private'}).encode(), b' '*2_097_153]:
        project(raw, 'invalid_input')
    bad = json.loads(json.dumps(request)); bad['recognition']['regions'][0]['bbox'].update(x='0.4', width='0.2')
    project(json.dumps(bad).encode(), 'crossing_region')
    assert source.read_bytes() == original and hashlib.sha256((ws/'attachments'/attachment['id']).read_bytes()).hexdigest() == before
    report = dict(profile=args.profile, native_pdf=True, original_sha256=before, ocr_regions=len(evidence['regions']), cells=actual, outside=['Footer'], framing_cases=5, crossing_error=True, evidence_exact=True, all_ordinals_once=True, no_normalization=True)
    if args.profile == 'release':
        large = json.loads(json.dumps(request)); large['x_edges']=[str(c/8).rstrip('0').rstrip('.') if c not in [0,8] else str(c//8) for c in range(9)]
        large['y_min']='0'; large['recognition']['regions']=[]
        for row in range(64):
            for col in range(8):
                large['recognition']['regions'].append(dict(ordinal=row*8+col,text='z'*1700,confidence='0',bbox=dict(x=str((col+0.1)/8),y=str(0.99-row*0.014),width='0.0625',height='0.005')))
        raw=json.dumps(large).encode();samples=[]
        for n in range(25):
            started=time.perf_counter_ns();got=project(raw);elapsed=(time.perf_counter_ns()-started)/1e6
            output=got['layout'];assert output['evidence']==large and len(output['rows'])==64
            assert sum(len(r['cells']) for r in output['rows'])==512
            ids=[i for r in output['rows'] for c in r['cells'] for i in c['region_ordinals']]+output['outside'];assert sorted(ids)==list(range(512))
            if n>=5:samples.append(elapsed)
        p95=sorted(samples)[18];report['benchmark']=dict(synthetic=True,regions=512,cells=512,input_bytes=len(raw),warmups=5,samples_ms=samples,p95_ms=p95,target_ms=300)
        (out/'benchmark-input.json').write_bytes(raw)
        assert p95<=300, p95
    (out/'results.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n');print(json.dumps(report, ensure_ascii=False))


if __name__ == '__main__':
    main()
