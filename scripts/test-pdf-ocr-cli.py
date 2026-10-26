#!/usr/bin/env python3
"""Cola y lector PDF OCR reales; canónico/SQLite/originales, sin UI nativa."""
import argparse,hashlib,importlib.util,json,os,shutil,sqlite3,subprocess,sys
from pathlib import Path
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parent.parent
PROFILE='apple_cgpdf_vision_r3_fast_es_en_v1';RASTER='cgpdf_media_gray_scale2_png_v1'
def compact(v):return json.dumps(v,ensure_ascii=False,separators=(',',':'),allow_nan=False).encode()
def main():
    ap=argparse.ArgumentParser();ap.add_argument('--profile',choices=['debug','release'],required=True);ap.add_argument('--corpus',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);a=ap.parse_args()
    base=Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True);corpus=a.corpus.resolve(strict=True);out=a.output.resolve();assert out.is_relative_to(base) and corpus.is_relative_to(base) and not out.exists();out.mkdir();runtime=out/'bin';runtime.mkdir()
    for name in ['noteka','noteka-pdf-raster-worker','noteka-ocr-worker']:shutil.copy2(ROOT/'target'/a.profile/name,runtime/name)
    spec=importlib.util.spec_from_file_location('ocr',ROOT/'scripts/test-ocr-cli.py');ocr=importlib.util.module_from_spec(spec);spec.loader.exec_module(ocr)
    ws=out/'workspace';calls=[];report=dict(profile=a.profile,headless=True,network_denied=True,cases=[],helper_free_reads=0)
    def call(op,payload,error=None):
        p=subprocess.run(['/usr/bin/sandbox-exec','-p','(version 1)(allow default)(deny network*)',str(runtime/'noteka'),'--workspace',str(ws),'--json',*[s.replace('_','-') for s in op.split('.')]],input=compact(payload),capture_output=True,timeout=12)
        assert not p.stderr,p.stderr[:300];r=json.loads(p.stdout);calls.append(op)
        if error:assert not r['ok'] and p.returncode and r['error']['details']['reason']==error,r;return r['error']
        assert r['ok'] and not p.returncode,r;return r['data']
    assert call('workspace.create',dict(timezone='UTC'))['schema_version']==16
    ctl=call('extraction.control',dict(action='get'));assert ctl['paused']
    first_source=corpus/'scanned.pdf';attachment=call('attachment.import',dict(source_path=str(first_source)));paused=call('extraction.enqueue',dict(attachment_id=attachment['id'],extractor='pdf_ocr'))['job']
    assert call('extraction.run',dict(id=paused['id']))['reason']=='queue_paused'
    call('extraction.control',dict(action='set',paused=False,expected_revision=ctl['revision']))
    fixtures={};manifest=json.loads((corpus/'manifest.json').read_text())
    for case in manifest['cases']:
        content=(corpus/case['name']).read_bytes();assert hashlib.sha256(content).hexdigest()==case['sha256']
        attachment=call('attachment.import',dict(source_path=str(corpus/case['name'])))
        job=call('extraction.enqueue',dict(attachment_id=attachment['id'],extractor='pdf_ocr'))['job'];same=call('extraction.enqueue',dict(attachment_id=attachment['id'],extractor='pdf_ocr'));assert same['deduplicated'] and same['job']['id']==job['id']
        if 'error' in case:
            call('extraction.run',dict(id=job['id']),case['error']);after=call('extraction.get',dict(id=job['id']));assert after['state']=='failed' and after['generation'] is None;report['cases'].append(dict(name=case['name'],error=case['error']));continue
        job=call('extraction.run',dict(id=job['id']))['job'];assert job['state']=='succeeded'
        with sqlite3.connect(ws/'noteka.sqlite') as c:chunks=list(c.execute('SELECT ordinal,text FROM extraction_pdf_ocr_chunks WHERE generation_id=? ORDER BY ordinal',[job['generation']['id']]))
        assert [i for i,_ in chunks]==list(range(len(chunks))) and all(0<len(s.encode())<=4096 for _,s in chunks)
        raw=''.join(s for _,s in chunks).encode();doc=json.loads(raw);assert doc['version']==1 and doc['profile']==PROFILE and doc['raster_profile']==RASTER and doc['source_sha256']==case['sha256']
        pages=[];items=[];expected_order=[]
        keys=['page_count','page_number','total_pixels','box_rect','rotation','width','height','png_sha256','png_bytes','os_version','os_build']
        for number,page in enumerate(doc['pages'],1):
            r=page['raster'];assert r['page_number']==number and r['page_count']==case['pages'] and r['width']==case['width'] and r['height']==case['height'] and r['rotation']==case['rotation']
            recognition=ocr.canonical(page['recognition']);assert recognition==page['recognition'];assert recognition['engine']['os_build']==r['os_build'] and recognition['engine']['os_version']==r['os_version']
            assert recognition['image']['format']=='png' and recognition['image']['orientation']==1 and recognition['image']['width']==r['width'] and recognition['image']['height']==r['height']
            expected_order.append(dict(raster={k:r[k] for k in keys},recognition=recognition))
            for region in recognition['regions']:items.append(dict(ordinal=len(items),page_number=number,region_ordinal=region['ordinal'],text=region['text'],confidence=region['confidence'],bbox=region['bbox']))
        canonical=dict(version=1,profile=PROFILE,raster_profile=RASTER,source_sha256=case['sha256'],pages=expected_order);assert raw==compact(canonical)
        assert hashlib.sha256((ws/'attachments'/attachment['id']).read_bytes()).hexdigest()==case['sha256']
        hidden=[]
        for name in ['noteka-pdf-raster-worker','noteka-ocr-worker']:
            p=runtime/name;absent=runtime/(name+'.absent');p.rename(absent);hidden.append((p,absent))
        try:
            actual=[];cursor=None
            while True:
                page=call('extraction.pdf_ocr_regions',dict(id=job['id'],limit=1,cursor=cursor));actual+=page['items'];pages.append(page);cursor=page['next_cursor'];report['helper_free_reads']+=1
                m=page['manifest'];assert m['representation_sha256']==hashlib.sha256(raw).hexdigest() and m['representation_bytes']==len(raw) and m['source_sha256']==case['sha256'] and m['page_count']==case['pages'] and m['region_count']==len(items)
                assert len(m['pages'])==case['pages'] and sum(p['region_count'] for p in m['pages'])==len(items)
                if not cursor:break
            assert actual==items
            call('extraction.ocr_regions',dict(id=job['id']),'unsupported_generation_config');call('extraction.pdfkit_pages',dict(id=job['id']),'unsupported_generation_config')
            idx=call('extraction.index',dict(action='get'));call('extraction.index',dict(action='put',job_id=job['id'],expected_generation_id=job['generation']['id'],expected_revision=idx['revision']))
            if items:
                token=next((word for item in items for word in item['text'].split() if word.isascii() and word.isalpha()),None)
                if token:
                    hits=call('extraction.search',dict(query=token,extractor='pdf_ocr',job_id=job['id'],limit=64))
                    assert hits['items']
                    for hit in hits['items']:
                        loc=hit['locator'];assert loc['kind']=='pdf_ocr_region_text';region=items[loc['global_region_ordinal']];descriptor=pages[0]['manifest']['pages'][loc['page_number']-1]
                        assert loc['raster']==descriptor['raster'] and loc['engine']==descriptor['engine'] and loc['bbox']==region['bbox'] and loc['confidence']==region['confidence']
                        assert hit['text'].encode()==region['text'].encode()[loc['text_byte_start']:loc['text_byte_end']]
        finally:
            for p,absent in hidden:absent.rename(p)
        fixtures[case['name']]=dict(job=job,attachment=attachment,manifest=pages[0]['manifest'],items=items)
        report['cases'].append(dict(name=case['name'],pages=len(doc['pages']),regions=len(items),canonical_bytes=len(raw)))
    first=fixtures['scanned.pdf'];old=first['job'];call('extraction.retry',dict(id=old['id'],expected_revision=old['revision']))
    for name in ['noteka-pdf-raster-worker','noteka-ocr-worker']:(runtime/name).rename(runtime/(name+'.absent'))
    call('extraction.run',dict(id=old['id']),'pdf_ocr_raster_worker_unavailable');prior=call('extraction.pdf_ocr_regions',dict(id=old['id']));assert prior['result_from_previous_attempt'] and prior['generation']==old['generation'] and prior['items']==first['items']
    backup=out/'backup16';restored=out/'restored16';call('workspace.backup',dict(destination=str(backup)));assert call('workspace.restore',dict(backup=str(backup),destination=str(restored)))['schema_version']==16
    original=ws;ws=restored;assert call('extraction.pdf_ocr_regions',dict(id=old['id']))['manifest']==first['manifest'];ws=original
    damaged=out/'damaged';shutil.copytree(backup,damaged)
    with sqlite3.connect(damaged/'noteka.sqlite') as c:c.execute("UPDATE extraction_pdf_ocr_chunks SET text='x'||substr(text,2) WHERE generation_id=? AND ordinal=(SELECT max(ordinal) FROM extraction_pdf_ocr_chunks WHERE generation_id=?)",[old['generation']['id']]*2);c.commit();c.execute('PRAGMA wal_checkpoint(TRUNCATE)')
    meta=json.loads((damaged/'manifest.json').read_text());meta['sha256']=hashlib.sha256((damaged/'noteka.sqlite').read_bytes()).hexdigest();(damaged/'manifest.json').write_text(json.dumps(meta))
    p=subprocess.run([str(runtime/'noteka'),'--workspace',str(ws),'--json','workspace','restore'],input=compact(dict(backup=str(damaged),destination=str(out/'rejected'))),capture_output=True,timeout=12);reply=json.loads(p.stdout);assert p.returncode and reply['error']['code']=='invalid_backup' and not (out/'rejected').exists()
    assert len(call('operations.describe',{})['inputs'])==55
    report.update(calls=len(calls),paused_without_spawn=True,deduplicated=True,previous_generation_preserved=True,restored_without_helpers=True,repaired_outer_hash_corruption_rejected=True,fts_verified=True)
    (out/'fixtures.json').write_text(json.dumps(fixtures,ensure_ascii=False,indent=2)+'\n');(out/'results.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(profile=a.profile,cases=len(report['cases']),calls=len(calls),helper_free_reads=report['helper_free_reads'])))
if __name__=='__main__':main()
