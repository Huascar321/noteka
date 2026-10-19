// Transporte de pruebas aislado: UI de producción -> CLI Rust real -> SQLite sintético.
// No es servidor de producto ni sustituye pruebas de IPC/cierre nativos.
import { createServer } from 'node:http';
import { spawn } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync, mkdirSync, lstatSync, realpathSync } from 'node:fs';
import { join, resolve, extname, dirname } from 'node:path';
import { DatabaseSync } from 'node:sqlite';
import { randomUUID } from 'node:crypto';
import { tmpdir } from 'node:os';
import { createEvidenceControls } from './headless-evidence.mjs';
import { createPreparationControls } from './headless-preparation.mjs';
import { createOcrControls } from './headless-ocr.mjs';
export async function startHeadlessBridge(port = 0) {
// macOS puede exponer TMPDIR mediante /var -> /private/var; comparar rutas canónicas.
const root=realpathSync(mkdtempSync(join(process.env.NOTEKA_TEST_TMP || tmpdir(),'noteka-headless-')));
const binary=resolve('target/debug/noteka');
let phase='main',theme='dark',record=null,homeRead={};const results=[];
const workspace=()=>join(root,theme,phase==='restored'?'restored':'workspace');
// Ninguna ruta entregada por el navegador puede leer/escribir fuera del corpus.
function confined(path) {
  if(typeof path!=='string')throw new Error('Ruta de prueba requerida');
  const absolute=resolve(path);
  if(!absolute.startsWith(root+'/'))throw new Error('Ruta fuera del corpus de prueba');
  let parent=absolute;
  // lstat no ignora un symlink colgante: realpath debe rechazarlo, no aprobar al padre.
  while(!lstatSync(parent,{throwIfNoEntry:false}))parent=dirname(parent);
  const real=realpathSync(parent);
  if(real!==root&&!real.startsWith(root+'/'))throw new Error('Symlink fuera del corpus de prueba');
  return absolute;
}
const cli=(operation,input,path=workspace())=>new Promise((done,reject)=>{
  confined(path);
  if(!/^[a-z_]+\.[a-z_]+$/.test(operation))throw new Error('Operación inválida');
  for(const key of ['source_path','destination','backup'])if(input?.[key]!==undefined)confined(input[key]);
  const args=['--json','--workspace',path,...operation.split('.').map(part=>part.replaceAll('_','-'))];
  const sandbox=process.platform==='darwin';
  const child=spawn(sandbox?'/usr/bin/sandbox-exec':binary,sandbox?['-p','(version 1) (allow default) (deny network*)',binary,...args]:args,{stdio:['pipe','pipe','pipe']});child.stdout.setEncoding('utf8');child.stderr.setEncoding('utf8');let stdout='',stderr='';
  child.stdout.on('data',c=>stdout+=c);child.stderr.on('data',c=>stderr+=c);child.on('error',reject);
  child.on('close',(code)=>{try {const r=JSON.parse(stdout);if(stderr||typeof r.ok!=='boolean'||r.ok!==(code===0))throw new Error(stderr||'Contrato/exit code CLI inválido');done(r);}catch(e){reject(e);}});child.stdin.end(JSON.stringify(input));
});
const data=async(operation,input)=>{const r=await cli(operation,input);if(!r.ok)throw new Error(JSON.stringify(r));return r.data;};
const evidence=createEvidenceControls(root,confined,cli);
const preparation=createPreparationControls(root,confined,cli);
const ocr=createOcrControls(root,confined,cli);
const sql=(fn)=>{const db=new DatabaseSync(join(workspace(),'noteka.sqlite'));try{return fn(db);}finally{db.close();}};
const mapped={get:'entity.get',list:'entity.list',entity_create:'entity.create',cli_update:'entity.update',entity_archive:'entity.archive',entity_restore:'entity.restore',relation_add:'relation.add',relation_remove:'relation.remove',home:'home.query',workflow:'workflow.query',search:'search.query',collections:'collection.list',collection_get:'collection.get',property_define:'property.define',formula_set:'formula.set',record_get:'record.get',record_create:'record.create',record_update:'record.update',database_query:'database.query',view_save:'view.save',templates:'template.list',template_get:'template.get',template_save:'template.save',template_instantiate:'template.instantiate',import_attachment:'attachment.import',read_attachment:'attachment.read'};
async function control(step,input) {
  if(step.startsWith('ocr_'))return ocr.control(step,input);
  if(step.startsWith('prep_'))return preparation.control(step,input);
  if(step.startsWith('evidence_'))return evidence.control(step,input);
  if(mapped[step])return data(mapped[step],input);
  const base=join(root,theme);mkdirSync(base,{recursive:true});
  switch(step){
    case 'configure':
      if(!['main','reopen','restored'].includes(input.phase)||!['dark','light'].includes(input.theme))throw new Error('Sesión inválida');
      phase=input.phase;theme=input.theme;record=null;homeRead={};return true;
    case 'init': {const file=join(base,'synthetic.txt'),image=join(base,'synthetic.png');writeFileSync(file,'original offline');writeFileSync(image,readFileSync('src-tauri/icons/256x256.png'));return {path:workspace(),restored:join(base,'restored'),file,image,export:join(base,'ui-export'),phase,theme,root:base};}
    case 'home_read':homeRead=input;return true;
    case 'entity_count':return sql(db=>db.prepare('SELECT count(*) AS n FROM entities').get().n);
    case 'failure_on': case 'failure_off':return sql(db=>{db.exec(step==='failure_on'?"CREATE TRIGGER smoke_fail BEFORE INSERT ON changes BEGIN SELECT RAISE(ABORT,'injected'); END;":"DROP TRIGGER smoke_fail;");return true;});
    case 'template_failure_on':case 'template_failure_off':return sql(db=>{db.exec(step==='template_failure_on'?"CREATE TRIGGER smoke_template_fail BEFORE UPDATE ON templates BEGIN SELECT RAISE(ABORT,'test'); END;":"DROP TRIGGER smoke_template_fail;");return true;});
    case 'template_raw':return sql(db=>{db.prepare('UPDATE templates SET definition=? WHERE id=?').run(JSON.stringify(input.definition),input.id);return true;});
    case 'view_raw':return sql(db=>{db.prepare('UPDATE views SET json=? WHERE id=?').run(JSON.stringify(input.view),input.id);return true;});
    case 'entity_raw':return sql(db=>{db.prepare('UPDATE entities SET payload=? WHERE id=?').run(JSON.stringify(input.payload),input.id);return true;});
    case 'query_value':return sql(db=>{db.prepare('UPDATE property_values SET json=? WHERE entity_id=? AND property_id=?').run(JSON.stringify(input.value),input.entity_id,input.property_id);return true;});
    case 'unknown': {const note=await data('entity.create',{kind:'note',title:'Formato futuro'});sql(db=>db.prepare('UPDATE documents SET json=? WHERE entity_id=?').run(JSON.stringify({schema_version:7,blocks:[{id:'future',type:'widget',keep:{value:42}}]}),note.id));return note;}
    case 'rich':return data('entity.create',{kind:'note',title:'Bloques enriquecidos',document:JSON.parse(readFileSync('tests/fixtures/rich-document.json','utf8'))});
    case 'archived_project': {const project=await data('entity.create',{kind:'project',title:'Proyecto relacionado'});const relation=await data('relation.add',{source_id:input.id,target_id:project.id,kind:'project',expected_revision:input.revision});await data('entity.archive',{id:project.id,expected_revision:1});return relation;}
    case 'check_export':return JSON.parse(readFileSync(join(base,'ui-export/manifest.json'),'utf8'));
    case 'home_seed':return sql(db=>{db.exec('BEGIN');try{for(const [k,kind] of ['task','note','project','tag','person'].entries()){for(let i=0;i<(['task','project'].includes(kind)?102:51);i++){const payload=kind==='task'?(i<51?{state:'todo',inbox:true,due_date:input.date}:{state:'done'}):kind==='project'?{state:i<51?'doing':'planned'}:kind==='tag'?{classification:'topic'}:{};db.prepare("INSERT INTO entities VALUES (?,?,?,1,NULL,'2026-10-01T12:00:00.000000Z','2026-10-01T12:00:00.000000Z',?)").run(`00000000-0000-4000-8000-${String(k*1000+i).padStart(12,'0')}`,kind,`Home synthetic ${kind} ${String(i).padStart(2,'0')} título largo para verificar ajuste y controles visibles`,JSON.stringify(payload));}}db.exec('COMMIT');return true;}catch(e){db.exec('ROLLBACK');throw e;}});
    case 'roundtrip':await data('workspace.backup',{destination:join(base,'backup')});return data('workspace.restore',{backup:join(base,'backup'),destination:join(base,'restored')});
    case 'record':record=input;return true;
    case 'close_confirmed':case 'finish': {const result=step==='finish'?input:record;if(!result)throw new Error('Cierre sin resultado');results.push({transport:'headless-cli-not-native-ipc',theme,phase,...result});writeFileSync(join(root,'results.json'),JSON.stringify(results,null,2));return result;}
    case 'status':return {root,results};
    case 'keyboard_report':writeFileSync(join(base,'keyboard-results.json'),JSON.stringify(input,null,2));return true;
    case 'network_report':writeFileSync(join(root,'network-results.json'),JSON.stringify({...input,cli_network:process.platform==='darwin'?'sandbox-exec deny network*':'sin sandbox OS'},null,2));return true;
    default:throw new Error(`Control de prueba no implementado: ${step}`);
  }
}
const server=createServer(async(req,res)=>{
  try{
    const origin='http://127.0.0.1:'+server.address().port;
    if(req.headers.host!==new URL(origin).host||(req.headers.origin&&req.headers.origin!==origin))throw new Error('Origen no autorizado');
    if(req.method==='POST'){
      if(req.headers['content-type']!=='application/json')throw new Error('POST requiere JSON');
      req.setEncoding('utf8');
      let raw='';for await(const chunk of req){raw+=chunk;if(raw.length>3*1024*1024)throw new Error('Input excesivo');}const input=JSON.parse(raw);
      let result;
      if(req.url==='/__test__/operate'){
        const path=confined(input.workspace);
        const instrument={...homeRead};
        if(input.command.operation==='home.query' && input.command.input.view===instrument.view){if(instrument.delay)await new Promise(r=>setTimeout(r,instrument.delay));if(instrument.failure){result={api_version:1,request_id:randomUUID(),ok:false,error:{code:'storage_error',message:'Lectura Home fallida inyectada sólo en transporte headless.',details:{}}};}}
        result??=await ocr.run(path,input.command,()=>preparation.run(path,input.command,()=>evidence.run(path,input.command,()=>cli(input.command.operation,input.command.input,path))));
      }else if(req.url==='/__test__/control'){result=await control(input.step,input.input||{});}else throw new Error('Ruta POST inválida');
      res.writeHead(200,{'content-type':'application/json'});res.end(JSON.stringify(result));return;
    }
    if(req.url==='/__test__/status'){res.setHeader('content-type','application/json');res.end(JSON.stringify({root,results}));return;}
    if(req.url==='/__test__/evidence.cjs'){res.setHeader('content-type','text/javascript');res.end(readFileSync('tests/headless-evidence.cjs'));return;}
    if(req.url==='/__test__/home.js'||req.url==='/__test__/smoke.js'){res.setHeader('content-type','text/javascript');res.end(readFileSync('tests/native/'+req.url.split('/').at(-1)));return;}
    const path=resolve('dist','.'+(req.url==='/'?'/index.html':req.url.split('?')[0]));if(!path.startsWith(resolve('dist')+'/'))throw new Error('Ruta GET inválida');
    res.setHeader('content-type',({'.html':'text/html','.js':'text/javascript','.css':'text/css'})[extname(path)]||'application/octet-stream');res.end(readFileSync(path));
  }catch(e){res.writeHead(500,{'content-type':'application/json'});res.end(JSON.stringify({error:String(e)}));}
});
await new Promise((done,reject)=>{server.once('error',reject);server.listen(port,'127.0.0.1',done);});
return {root,origin:'http://127.0.0.1:'+server.address().port,close:()=>new Promise((done,reject)=>server.close(error=>error?reject(error):done()))};
}
