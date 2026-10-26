#!/usr/bin/env python3
"""Raster real, corpus y contención. Sólo evidencia externa; sin ventanas ni red."""
import argparse, base64, hashlib, importlib.util, json, os, struct, sys
from pathlib import Path
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parent.parent
spec=importlib.util.spec_from_file_location('bounded',ROOT/'scripts/test-ocr-reader.py');support=importlib.util.module_from_spec(spec);spec.loader.exec_module(support)

def packet(source,page=1,ms=5000):return b'NKPDFR01'+struct.pack('<HHIIII',1,0,len(source),ms,page,0)+source

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--profile',choices=['debug','release'],required=True);ap.add_argument('--corpus',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);ap.add_argument('--probes',action='store_true');a=ap.parse_args()
    base=Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True);corpus=a.corpus.resolve(strict=True);out=a.output.resolve();assert corpus.is_relative_to(base) and out.is_relative_to(base) and not out.exists();out.mkdir()
    worker=ROOT/'target'/a.profile/'noteka-pdf-raster-worker';probes=worker.with_name('noteka-pdf-raster-worker-probes')
    manifest=json.loads((corpus/'manifest.json').read_bytes());assert manifest['expected_before_ocr'];report=dict(profile=a.profile,headless=True,network_denied=True,workspace_used=False,binary_sha256=hashlib.sha256(worker.read_bytes()).hexdigest(),cases=[],protocol_checks=0,probes=[])
    for case in manifest['cases']:
        source=(corpus/case['name']).read_bytes();assert hashlib.sha256(source).hexdigest()==case['sha256']
        pages=[1] if 'error' in case else list(range(1,case['pages']+1))
        records=[]
        for page in pages:
            result=support.bounded([worker],packet(source,page));assert result['code']==0 and not result['stderr'],(case['name'],result['code'],result['stderr'])
            value=json.loads(result['stdout']);assert value['version']==1 and value['profile']=='cgpdf_media_gray_scale2_png_v1';v=value['outcome']
            if 'error' in case:assert v==dict(status='error',reason=case['error']),(case['name'],v.get('status'),v.get('reason'),len(v.get('raster',{}).get('png','')));continue
            assert v['status']=='success' and v['source_sha256']==case['sha256'];r=v['raster'];png=base64.b64decode(r.pop('png'),validate=True)
            assert png[:8]==b'\x89PNG\r\n\x1a\n' and 0<len(png)<=1048576
            assert struct.unpack('>II',png[16:24])==(case['width'],case['height']) and png[24:29]==bytes([8,0,0,0,0])
            assert r['page_count']==case['pages'] and r['page_number']==page and r['rotation']==case['rotation'] and r['width']==case['width'] and r['height']==case['height']
            assert r['total_pixels']==case['pages']*case['width']*case['height']<=16000000
            if case['name']=='offset-box.pdf':assert list(map(float,r['box']))==[10,-20,600,160]
            r.update(png_sha256=hashlib.sha256(png).hexdigest(),png_bytes=len(png));records.append(r)
        report['cases'].append(dict(name=case['name'],error=case.get('error'),pages=records))
    source=(corpus/'scanned.pdf').read_bytes();valid=packet(source)
    bad=[b'',valid[:27],valid[:-1],valid+b'X',packet(source,page=0),packet(source,page=21),packet(source,page=2),packet(source,ms=0),packet(source,ms=5001),packet(source,ms=5000)[:12]+struct.pack('<I',1048577)+valid[16:]]
    # Page2 is framed correctly but outside this one-page document.
    for i in [0,8,9,10,11,24,25,26,27]:
        changed=bytearray(valid);changed[i]^=1;bad.append(bytes(changed))
    for wire in bad:
        p=support.bounded([worker],wire);assert p['code']==0 and not p['stderr'];v=json.loads(p['stdout']);assert v['outcome']['status']=='error',v;report['protocol_checks']+=1
    # Binary with production flags cannot expose probes.
    p=support.bounded([worker,'--probe-limits']);assert json.loads(p['stdout'])['outcome']==dict(status='error',reason='pdf_ocr_raster_protocol');report['protocol_checks']+=1
    if a.probes:
        p=support.bounded([probes,'--probe-limits']);assert p['code']==0 and json.loads(p['stdout'])==dict(core=0,cpu=4)
        for flag,expected in [('--probe-short',75),('--probe-sleep',75),('--probe-memory',73),('--probe-cpu',None)]:
            p=support.bounded([probes,flag],timeout=7);assert not p['stdout'] and not p['stderr'];assert p['code']==expected if expected is not None else p['code']<0,(flag,p)
            assert p['wall_ms']<6500
            report['probes'].append(dict(flag=flag,code=p['code'],wall_ms=p['wall_ms'],cpu_s=p['cpu_s']))
        try:support.bounded([probes,'--probe-stdout'])
        except support.Rejected as e:assert str(e)=='stdout_limit'
        else:raise AssertionError('flood accepted')
        report['probes'].append(dict(flag='--probe-stdout',rejected='stdout_limit'))
    (out/'results.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(cases=len(report['cases']),protocol_checks=report['protocol_checks'],probes=len(report['probes']))))
if __name__=='__main__':main()
