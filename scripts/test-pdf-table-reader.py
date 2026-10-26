#!/usr/bin/env python3
"""Cliente56/57, PDF propio con golden previo, lecturas sin helpers y fuentes intactas."""
import argparse,hashlib,importlib.util,json,os,shutil,sqlite3,subprocess,sys,time
from pathlib import Path
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parent.parent

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--profile',choices=['debug','release'],required=True);ap.add_argument('--old-bin',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);args=ap.parse_args()
 base=Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True);out=args.output.resolve();assert out.is_relative_to(base) and not out.exists() and not out.is_relative_to(ROOT);out.mkdir()
 spec=importlib.util.spec_from_file_location('pdf',ROOT/'tests/fixtures/pdf/generate.py');pdf=importlib.util.module_from_spec(spec);spec.loader.exec_module(pdf)
 expected=[['Item','Count']]+[['Alpha' if n%2 else 'Beta','12.50' if n%2 else '003'] for n in range(1,11)]
 (out/'golden-before-ocr.json').write_text(json.dumps({'license':'MIT; own synthetic source','cells':expected,'outside':['Footer'],'pages':3},indent=2)+'\n')
 pieces=[b'BT /F1 24 Tf']
 for n,(left,right) in enumerate(expected):
  y=700-n*44;pieces.append(f'1 0 0 1 50 {y} Tm ({left}) Tj 1 0 0 1 350 {y} Tm ({right}) Tj'.encode())
 pieces.append(b'1 0 0 1 50 60 Tm (Footer) Tj ET');source=out/'table.pdf';original=pdf.serialize(pdf.document([b'',b' '.join(pieces),b'']));source.write_bytes(original)
 writer=out/'old';reader=out/'new';writer.mkdir();reader.mkdir()
 for name in ['noteka','noteka-pdf-worker','noteka-xlsx-worker','noteka-ocr-worker','noteka-pdfkit-worker','noteka-pdf-raster-worker']:shutil.copy2(args.old_bin/name,writer/name)
 target=Path(os.environ.get('CARGO_TARGET_DIR',ROOT/'target'))
 for name in ['noteka','noteka-mcp']:shutil.copy2(target/args.profile/name,reader/name)
 ws=out/'workspace';calls=0
 def call(binary,op,data,error=None,workspace=None):
  nonlocal calls
  p=subprocess.run(['/usr/bin/sandbox-exec','-p','(version 1)(allow default)(deny network*)',str(binary),'--workspace',str(workspace or ws),'--json',*[s.replace('_','-') for s in op.split('.')]],input=json.dumps(data).encode(),capture_output=True,timeout=15);calls+=1
  assert not p.stderr,p.stderr[:200];v=json.loads(p.stdout);assert v['ok']==(p.returncode==0),v
  if error:assert not v['ok'] and v['error']['details']['reason']==error,v;return v['error']
  assert v['ok'],v;return v['data']
 old,new=writer/'noteka',reader/'noteka'
 assert call(old,'workspace.create',{'timezone':'UTC'})['schema_version']==16
 a=call(old,'attachment.import',{'source_path':str(source)});ctl=call(old,'extraction.control',{'action':'get'});call(old,'extraction.control',{'action':'set','paused':False,'expected_revision':ctl['revision']})
 j=call(old,'extraction.enqueue',{'attachment_id':a['id'],'extractor':'pdf_ocr'})['job'];j=call(old,'extraction.run',{'id':j['id']})['job'];assert j['state']=='succeeded',j
 regions=call(old,'extraction.pdf_ocr_regions',{'id':j['id'],'limit':64});assert regions['covers_generation'] and regions['manifest']['page_count']==3
 request={'id':j['id'],'page_number':2,'x_edges':['0','0.5','1'],'y_min':'0.4','y_max':'1','header_row':1,'limit':20}
 def snapshot(path):
  with sqlite3.connect(path/'noteka.sqlite') as c:
   c.execute('PRAGMA query_only=ON');tables={n:repr(sorted(c.execute('SELECT * FROM "'+n.replace('"','""')+'"').fetchall(),key=repr)) for (n,) in c.execute("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name").fetchall()};ddl=c.execute('SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name').fetchall()
  return {'tables':tables,'ddl':ddl,'originals':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in (path/'attachments').iterdir()}}
 before=snapshot(ws);p=call(new,'extraction.pdf_ocr_table',request);items=p['items'][:];cursor=p['next_cursor'];assert cursor and not p['covers_table'] and not p['covers_generation']
 while cursor:
  q=call(new,'extraction.pdf_ocr_table',{**request,'cursor':cursor});assert not q['covers_table'] and not q['covers_generation'];items+=q['items'];cursor=q['next_cursor']
 assert len(items)==22 and [c['ordinal'] for c in items]==list(range(22))
 actual=[['\n'.join(r['text'] for r in items[row*2+col]['regions']) for col in range(2)] for row in range(11)]
 (out/'actual.json').write_text(json.dumps({'cells':actual,'page':p,'regions':regions},indent=2,ensure_ascii=False)+'\n');assert actual==expected,(actual,expected)
 assert p['header_row']==1 and p['profile']=='pdf_ocr_columns_rows_v1'
 refs=[r['ordinal'] for c in items for r in c['regions']]+[r['ordinal'] for r in p['outside']];assert sorted(refs)==[r['ordinal'] for r in regions['items'] if r['page_number']==2]
 assert [regions['items'][r['ordinal']]['text'] for r in p['outside']]==['Footer']
 for c in items:
  for r in c['regions']:assert r==regions['items'][r['ordinal']]
 assert snapshot(ws)==before
 # Every criterion belongs to the cursor, including absence of the header.
 for key,value in [('page_number',1),('x_edges',['0','1']),('y_min','0.2'),('y_max','0.9'),('header_row',None),('limit',19)]:call(new,'extraction.pdf_ocr_table',{**request,'cursor':p['next_cursor'],key:value},'cursor_scope_mismatch')
 box=items[0]['regions'][0]['bbox'];crossing=str(float(box['x'])+float(box['width'])/2)
 call(new,'extraction.pdf_ocr_table',{**request,'x_edges':['0',crossing,'1']},'pdf_table_crossing_region')
 empty=call(new,'extraction.pdf_ocr_table',{**request,'page_number':1,'header_row':None});assert empty['items']==[] and empty['covers_table'] and not empty['covers_generation']
 # FTS remains region based; table reads create no selection or duplicate fragments.
 control=call(new,'extraction.index',{'action':'get'});call(new,'extraction.index',{'action':'put','job_id':j['id'],'expected_generation_id':j['generation']['id'],'expected_revision':control['revision']})
 search={'query':'Alpha','extractor':'pdf_ocr','job_id':j['id'],'limit':64};hits=call(old,'extraction.search',search);assert hits==call(new,'extraction.search',search)
 for hit in hits['items']:
  r=regions['items'][hit['locator']['global_region_ordinal']];assert r in [r for cell in items for r in cell['regions']];assert hit['locator']['bbox']==r['bbox']
 selected=snapshot(ws);call(new,'extraction.pdf_ocr_table',request);assert snapshot(ws)==selected
 backup=out/'backup';call(new,'workspace.backup',{'destination':str(backup)});restored=out/'restored';call(old,'workspace.restore',{'backup':str(backup),'destination':str(restored)});assert snapshot(restored)==selected
 assert call(new,'extraction.pdf_ocr_table',request,workspace=restored)['items']==p['items'];call(new,'extraction.pdf_ocr_table',{**request,'cursor':p['next_cursor']},'cursor_scope_mismatch',restored)
 # Source changes fail, including reading an otherwise blank page; never repaired.
 original_path=ws/'attachments'/a['id'];original_path.write_bytes(b'changed');call(new,'extraction.pdf_ocr_table',{**request,'page_number':1,'header_row':None},'input_stale');original_path.write_bytes(original)
 # Whole canonical check even outside chosen page/first table cell.
 with sqlite3.connect(ws/'noteka.sqlite') as c:chunk=c.execute('SELECT ordinal,text FROM extraction_pdf_ocr_chunks WHERE generation_id=? ORDER BY ordinal DESC LIMIT 1',[j['generation']['id']]).fetchone();c.execute('UPDATE extraction_pdf_ocr_chunks SET text=? WHERE generation_id=? AND ordinal=?',['x'+chunk[1][1:],j['generation']['id'],chunk[0]])
 call(new,'extraction.pdf_ocr_table',request,'invalid_pdf_ocr_integrity')
 with sqlite3.connect(ws/'noteka.sqlite') as c:c.execute('UPDATE extraction_pdf_ocr_chunks SET text=? WHERE generation_id=? AND ordinal=?',[chunk[1],j['generation']['id'],chunk[0]])
 assert snapshot(ws)==selected
 benchmarks={}
 if args.profile=='release':
  for name,criteria in [('first',request),('next',{**request,'cursor':p['next_cursor']}),('whole',{**request,'limit':64})]:
   samples=[]
   for n in range(25):
    start=time.perf_counter_ns();got=call(new,'extraction.pdf_ocr_table',criteria);ms=(time.perf_counter_ns()-start)/1e6
    assert got['manifest']==p['manifest']
    if n>=5:samples.append(ms)
   p95=sorted(samples)[18];assert p95<=300,p95;benchmarks[name]={'warmups':5,'samples_ms':samples,'p95_ms':p95,'target_ms':300}
 report={'profile':args.profile,'schema':16,'native_pdf':True,'golden_exact':True,'rows':11,'cells':22,'physical_pages':3,'outside':1,'calls':calls,'helper_free':True,'pure_reads':True,'old_client_backup_restore_exact':True,'search_region_provenance_exact':True,'benchmarks':benchmarks}
 (out/'results.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
if __name__=='__main__':main()
