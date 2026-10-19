#!/usr/bin/env python3
"""Gate adicional de lectura OCR: corpus derivado sintético fijado antes de medir.
No mide reconocimiento nativo, calidad, memoria GPU ni aceptación del host.
"""
import hashlib,json,math,os,shutil,sqlite3,subprocess,tempfile,time,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent

def compact(v):return json.dumps(v,ensure_ascii=False,separators=(',',':')).encode()
def text(size):
    pattern=b'evidencia cafe texto local ';return (pattern*(size//len(pattern)+1))[:size].decode()
def chunks(raw):
    result=[]
    while raw:
        end=min(4096,len(raw))
        while True:
            try:part=raw[:end].decode();break
            except UnicodeDecodeError:end-=1
        result.append(part);raw=raw[end:]
    return result

def main():
    base=Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True);assert not base.is_relative_to(ROOT)
    recipe_path=ROOT/'tests/fixtures/ocr-search/benchmark.json';recipe=json.loads(recipe_path.read_text())
    digest=hashlib.sha256(recipe_path.read_bytes()).hexdigest();assert digest==recipe_path.with_suffix('.sha256').read_text().split()[0]
    out=Path(tempfile.mkdtemp(prefix='ocr-search-bench-',dir=base));runtime=out/'bin';runtime.mkdir();cli=runtime/'noteka';shutil.copy2(ROOT/'target/release/noteka',cli)
    ws=out/'workspace'
    def call(op,payload):
        r=subprocess.run(['/usr/bin/sandbox-exec','-p','(version 1)(allow default)(deny network*)',str(cli),'--json','--workspace',str(ws),*[p.replace('_','-') for p in op.split('.')]],input=compact(payload),capture_output=True,timeout=20)
        v=json.loads(r.stdout);assert not r.stderr and r.returncode==0 and v['ok'],v;return v['data']
    assert call('workspace.create',dict(timezone='UTC'))['schema_version']==13
    blank=(ROOT/'tests/fixtures/ocr/blank.png').read_bytes();total=recipe['indexed_text_bytes'];count=recipe['generations'];jobs=[]
    for index in range(count):
        # PNG original sintético válido con un chunk tEXt propio, sin importar
        # imágenes personales ni usar el motor para fabricar textos del benchmark.
        import struct,zlib
        kind=b'tEXt';body=f'benchmark\0{index}'.encode();extra=struct.pack('>I',len(body))+kind+body+struct.pack('>I',zlib.crc32(kind+body))
        file=out/f'source-{index}.png';file.write_bytes(blank[:-12]+extra+blank[-12:]);a=call('attachment.import',dict(source_path=str(file)))
        j=call('extraction.enqueue',dict(attachment_id=a['id'],extractor='image_ocr'))['job'];size=total//count+int(index<total%count);regions=[];n=recipe['regions_per_generation']
        for i in range(n):regions.append(dict(ordinal=i,text=text(size//n+int(i<size%n)),confidence='0.5',bbox=dict(x='0',y='0',width='1',height='1')))
        r=dict(version=1,image=dict(format='png',width=1200,height=300,orientation=1,oriented_width=1200,oriented_height=300),engine=dict(revision=3,recognition_level='fast',languages=['es-ES','en-US'],language_correction=False,automatic_language_detection=False,os_version='13.0.0',os_build='22A380'),coordinate_space='oriented_normalized_bottom_left',limitations=['inferred_text_may_be_incorrect_or_missing','observation_order_not_guaranteed_reading_order','no_layout_tables_or_visual_description','confidence_not_probability_of_correctness','system_framework_result_may_vary_by_os'],regions=regions)
        raw=compact(r);parts=chunks(raw);assert len(raw)<=1048576 and len(parts)<=512
        generation=str(uuid.uuid4())
        with sqlite3.connect(ws/'noteka.sqlite') as db:
            db.execute('PRAGMA foreign_keys=ON')
            db.execute('INSERT INTO extraction_generations SELECT ?,id,attachment_id,source_sha256,extractor,extractor_version,config,config_hash,source_bytes,source_bytes,?,created_at,1 FROM extraction_jobs WHERE id=?',[generation,len(parts),j['id']])
            db.execute("UPDATE extraction_jobs SET state='succeeded',generation_id=?,attempts=1 WHERE id=?",[generation,j['id']])
            db.execute('INSERT INTO extraction_ocr_generations VALUES (?,?,1,?,?,?,?,?)',[generation,j['id'],'apple_vision_r3_fast_es_en_v1',hashlib.sha256(raw).hexdigest(),len(raw),n,len(parts)])
            db.executemany('INSERT INTO extraction_ocr_chunks VALUES (?,?,?,?)',[(generation,j['id'],i,s) for i,s in enumerate(parts)])
        idx=call('extraction.index',dict(action='get'));call('extraction.index',dict(action='put',job_id=j['id'],expected_generation_id=generation,expected_revision=idx['revision']));jobs.append(dict(job=j['id'],generation=generation,representation_bytes=len(raw)))
    coverage=call('extraction.index',dict(action='get'))['coverage'];assert coverage['text_bytes']==8388608 and coverage['fragments']==4608 and coverage['indexed_generations']==9
    report=dict(directory=str(out),recipe_sha256=digest,cli_sha256=hashlib.sha256(cli.read_bytes()).hexdigest(),mode='release CLI wall time including process start and sandbox, full verification',native_inference_measured=False,helper_available=False,coverage=coverage,jobs=jobs,queries={})
    (out/'prepared.json').write_text(json.dumps(report,indent=2)+'\n')
    for query in recipe['queries']:
        samples=[]
        for i in range(recipe['warmups']+recipe['samples']):
            start=time.perf_counter_ns();r=call('extraction.search',dict(query=query,extractor='image_ocr',limit=recipe['page_limit']));elapsed=(time.perf_counter_ns()-start)/1e6
            assert r['coverage']['text_bytes']==8388608
            assert len(r['items'])==(20 if query=='evidencia' else 0)
            if i>=recipe['warmups']:samples.append(elapsed)
        ordered=sorted(samples);report['queries'][query]=dict(samples_ms=samples,p50=ordered[9],p95=ordered[18],maximum=max(samples))
    report['query_gate_met']=all(v['p95']<=recipe['query_p95_ms_max'] for v in report['queries'].values());(out/'results.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2));assert report['query_gate_met'],out

if __name__=='__main__':main()
