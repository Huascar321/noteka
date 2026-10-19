#!/usr/bin/env python3
"""OCR administrado real + oráculo decimal/representación independiente, sin ventanas/red.
El wrapper de evidencia captura la salida del motor real; no genera reconocimiento.
"""
import argparse
from decimal import Decimal
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
PROFILE = 'apple_vision_r3_fast_es_en_v1'

def compact(v):
    return json.dumps(v, ensure_ascii=False, separators=(',', ':'), allow_nan=False).encode()

def canonical_scalar(s):
    assert isinstance(s, str) and len(s) <= 512
    value = float(s)
    assert math.isfinite(value) and 0 <= value <= 1
    result = format(Decimal(repr(value)), 'f')
    if '.' in result:
        result = result.rstrip('0').rstrip('.')
    return result

def canonical(r):
    image = {k: r['image'][k] for k in ['format', 'width', 'height', 'orientation', 'oriented_width', 'oriented_height']}
    engine = {k: r['engine'][k] for k in ['revision', 'recognition_level', 'languages', 'language_correction', 'automatic_language_detection', 'os_version', 'os_build']}
    regions = [dict(ordinal=i, text=x['text'], confidence=canonical_scalar(x['confidence']),
                    bbox={k: canonical_scalar(x['bbox'][k]) for k in ['x', 'y', 'width', 'height']}) for i, x in enumerate(r['regions'])]
    return dict(version=1, image=image, engine=engine, coordinate_space=r['coordinate_space'], limitations=r['limitations'], regions=regions)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--profile', choices=['debug', 'release'], required=True)
    ap.add_argument('--corpus', type=Path, required=True)
    ap.add_argument('--output', type=Path, required=True)
    args = ap.parse_args()
    base = Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True)
    out = args.output.resolve(); corpus = args.corpus.resolve(strict=True)
    assert out.is_relative_to(base) and corpus.is_relative_to(base) and not out.exists() and not base.is_relative_to(ROOT)
    out.mkdir(); runtime = out/'bin'; runtime.mkdir()
    target = ROOT/'target'/args.profile
    for name in ['noteka', 'noteka-mcp', 'noteka-pdf-worker', 'noteka-xlsx-worker']:
        shutil.copy2(target/name, runtime/name)
    native = target/'noteka-ocr-worker'
    capture = out/'native-reply.json'; log = out/'native-calls.txt'
    wrapper = runtime/'noteka-ocr-worker'
    # Test instrumentation only. Native helper has its own deadline/caps; wrapper
    # records the exact actual response used for this publication before returning it.
    wrapper.write_text('#!/usr/bin/python3\nimport subprocess,sys,pathlib\n'
                       f'p=subprocess.run([{str(native)!r}],input=sys.stdin.buffer.read(1048601),capture_output=True,timeout=6)\n'
                       f'pathlib.Path({str(capture)!r}).write_bytes(p.stdout)\n'
                       f'with open({str(log)!r},"a") as f:f.write("native\\n")\n'
                       'sys.stdout.buffer.write(p.stdout);sys.stderr.buffer.write(p.stderr);sys.exit(p.returncode)\n')
    wrapper.chmod(0o700)
    ws = out/'workspace'; calls=[]
    def call(op, payload, error=None):
        command = [str(runtime/'noteka'), '--workspace', str(ws), '--json', *[part.replace('_','-') for part in op.split('.')]]
        process = subprocess.run(['/usr/bin/sandbox-exec','-p','(version 1)(allow default)(deny network*)',*command],input=compact(payload),capture_output=True,timeout=15)
        assert not process.stderr, process.stderr[:400]
        reply = json.loads(process.stdout); calls.append(op)
        if error:
            assert not reply['ok'] and process.returncode, reply
            if isinstance(error,str): assert reply['error']['details']['reason']==error, reply
            return reply['error']
        assert reply['ok'] and process.returncode==0, reply
        return reply['data']
    assert call('workspace.create',dict(timezone='UTC'))['schema_version']==13
    ctl=call('extraction.control',dict(action='get'));call('extraction.control',dict(action='set',paused=False,expected_revision=ctl['revision']))
    report=dict(profile=args.profile,network_denied=True,headless=True,native_sha256=hashlib.sha256(native.read_bytes()).hexdigest(),cases=[],quality_source='unchanged human-declared standalone corpus; this run verifies persistence from actual native output')
    fixtures={}; manifest=json.loads((corpus/'manifest.json').read_text())
    for case in manifest['cases']:
        path=corpus/case['name']; content=path.read_bytes();assert hashlib.sha256(content).hexdigest()==case['sha256']
        attachment=call('attachment.import',dict(source_path=str(path)))
        if path.suffix.lower() not in ['.png','.jpg','.jpeg']:
            call('extraction.enqueue',dict(attachment_id=attachment['id'],extractor='image_ocr'),'unsupported_source_format')
            report['cases'].append(dict(name=case['name'],rejected='unsupported_source_format'));continue
        job=call('extraction.enqueue',dict(attachment_id=attachment['id'],extractor='image_ocr'))['job']
        assert job['state']=='queued' and job['generation'] is None
        same=call('extraction.enqueue',dict(attachment_id=attachment['id'],extractor='image_ocr'));assert same['deduplicated'] and same['job']['id']==job['id']
        if 'error' in case:
            err=call('extraction.run',dict(id=job['id']),True)
            assert err['details']['reason']==case['error'],(case['name'],err)
            assert call('extraction.get',dict(id=job['id']))['state']=='failed'
            report['cases'].append(dict(name=case['name'],rejected=case['error']));continue
        job=call('extraction.run',dict(id=job['id']))['job'];assert job['state']=='succeeded'
        actual=json.loads(capture.read_bytes());assert actual['version']==2 and actual['profile']==PROFILE and actual['outcome']['source_sha256']==attachment['sha256']
        expected=canonical(actual['outcome']['recognition']);raw=compact(expected)
        before=log.read_text();pages=[];cursor=None
        while True:
            page=call('extraction.ocr_regions',dict(id=job['id'],limit=20,**(dict(cursor=cursor) if cursor else {})))
            assert page['generation']==job['generation'];pages.append(page)
            cursor=page['next_cursor']
            if not cursor:break
        assert log.read_text()==before
        items=[x for page in pages for x in page['items']];assert items==expected['regions']
        m=pages[0]['manifest'];assert m['profile']==PROFILE and m['region_count']==len(items)
        assert m['representation_sha256']==hashlib.sha256(raw).hexdigest() and m['representation_bytes']==len(raw)
        for key in ['image','engine','coordinate_space','limitations']:assert m[key]==expected[key]
        with sqlite3.connect(ws/'noteka.sqlite') as db:
            chunks=list(db.execute('SELECT ordinal,text FROM extraction_ocr_chunks WHERE generation_id=? ORDER BY ordinal',[job['generation']['id']]))
        assert [i for i,_ in chunks]==list(range(len(chunks))) and len(chunks)==job['generation']['fragment_count']
        assert all(0<len(s.encode())<=4096 for _,s in chunks) and ''.join(s for _,s in chunks).encode()==raw
        fixtures[case['name']]=dict(job=job,attachment=attachment,pages=pages,expected=expected)
        report['cases'].append(dict(name=case['name'],regions=len(items),representation_bytes=len(raw),canonical_sha256=m['representation_sha256']))
    assert len(report['cases'])==22
    before=log.read_text();index=call('extraction.index',dict(action='get'));assert index['coverage']['indexed_generations']==0
    empty=call('extraction.search',dict(query='noteka',extractor='image_ocr'));assert not empty['items']
    first=fixtures['helvetica-es.png']['job']
    chosen=call('extraction.index',dict(action='put',job_id=first['id'],expected_generation_id=first['generation']['id'],expected_revision=index['revision']))
    assert chosen['changed'];idx=call('extraction.index',dict(action='get'));assert idx['coverage']['indexed_generations']==1
    result=call('extraction.search',dict(query='noteka',extractor='image_ocr'));assert result['items'] and len(result['items'])==1
    hit=result['items'][0];r=fixtures['helvetica-es.png']['expected']['regions'][0]
    assert hit['text']==r['text'] and hit['locator']==dict(kind='image_ocr_region_text',projection='ocr_region_text_v1',region_ordinal=0,bbox=r['bbox'],confidence=r['confidence'],coordinate_space='oriented_normalized_bottom_left',region_bytes=len(r['text'].encode()),text_byte_start=0,text_byte_end=len(r['text'].encode()))
    assert not call('extraction.search',dict(query='bottom',extractor='image_ocr'))['items']
    assert log.read_text()==before
    retry=call('extraction.retry',dict(id=first['id'],expected_revision=first['revision']))
    previous=call('extraction.ocr_regions',dict(id=first['id']));assert previous['result_from_previous_attempt'] and previous['generation']==first['generation']
    # Explicit run with missing helper fails while preserving previous generation.
    wrapper.rename(runtime/'evidence-wrapper-disabled')
    err=call('extraction.run',dict(id=first['id']),'ocr_worker_unavailable')
    assert call('extraction.ocr_regions',dict(id=first['id']))['generation']==first['generation']
    assert call('extraction.search',dict(query='noteka'))['items'][0]['result_from_previous_attempt']
    backup=out/'backup-v13';restored=out/'restored-v13'
    call('workspace.backup',dict(destination=str(backup)))
    assert call('workspace.restore',dict(backup=str(backup),destination=str(restored)))['schema_version']==13
    original_ws=ws;ws=restored
    assert call('extraction.ocr_regions',dict(id=first['id']))['generation']==first['generation']
    assert call('extraction.search',dict(query='noteka'))['items'][0]['source']['generation_id']==first['generation']['id']
    ws=original_ws
    bad=out/'corrupt-backup';shutil.copytree(backup,bad)
    db=sqlite3.connect(bad/'noteka.sqlite')
    db.execute("UPDATE extraction_ocr_chunks SET text='X'||substr(text,2) WHERE generation_id=? AND ordinal=(SELECT max(ordinal) FROM extraction_ocr_chunks WHERE generation_id=?)",[first['generation']['id']]*2)
    db.commit();db.execute('PRAGMA wal_checkpoint(TRUNCATE)');db.close()
    meta=json.loads((bad/'manifest.json').read_text());meta['sha256']=hashlib.sha256((bad/'noteka.sqlite').read_bytes()).hexdigest();(bad/'manifest.json').write_text(json.dumps(meta))
    rejected=out/'rejected-restore';assert call('workspace.restore',dict(backup=str(bad),destination=str(rejected)),True)['code']=='invalid_backup'
    assert not rejected.exists() and call('extraction.ocr_regions',dict(id=first['id']))['generation']==first['generation']
    operation_count = len(call('operations.describe', {})['inputs'])
    report.update(calls=len(calls),native_runs=len(log.read_text().splitlines()),explicit_noop_reads=True,previous_generation_preserved=True,restore_without_helper=True,repaired_outer_hash_corruption_rejected=True,api_operations=operation_count)
    (out/'fixtures.json').write_text(json.dumps(fixtures,ensure_ascii=False,indent=2)+'\n')
    (out/'results.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(dict(profile=args.profile,cases=len(report['cases']),calls=len(calls),native_runs=report['native_runs'])))

if __name__=='__main__':main()
