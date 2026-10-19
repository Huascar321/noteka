import { test } from 'node:test';
import assert from 'node:assert/strict';
import { GroupedTextSearch } from '../src/lib/grouped-text-search.ts';
const tick = () => new Promise(resolve => setImmediate(resolve));
const deferred = () => { let resolve, reject; const promise = new Promise((a,b) => { resolve=a; reject=b; }); return {promise,resolve,reject}; };
const notes = cursor => ({ items: [{ entity_id:'entity', block_id:'block', title:'Nota', revision:1, excerpt:'<script>dato</script>', kind:'note', score:-1 }], next_cursor:cursor });
const attachments = cursor => ({ items: [], next_cursor:cursor, version:1, revision:3, profile:'literal_unicode61_v1', coverage:{} });
const error = () => Object.assign(new Error('Error seguro'), { detail: { code:'revision_conflict',message:'Error seguro',details:{reason:'index_revision_changed'} } });
function setup(respond = () => undefined) {
  const calls=[], changes=[]; const c=new GroupedTextSearch(async command => { calls.push(command); const r=await respond(command); return r ?? (command.operation==='search.query'?notes(null):attachments(null)); }, value=>changes.push(value)); return {c,calls,changes};
}
test('Montar/editar no consulta y una acción envía la misma entrada literal a ambos contratos', async()=>{
  const {c,calls}=setup(); c.edit(); await c.search(' '); assert.equal(calls.length,0); await c.search(' café 🚀 ');
  assert.deepEqual(calls.find(c=>c.operation==='search.query').input,{query:' café 🚀 ',archived:false,limit:20}); assert.deepEqual(calls.find(c=>c.operation==='extraction.search').input,{query:' café 🚀 ',limit:20}); c.dispose();
});
test('No trunca 256 bytes ni agrega filtros ajenos a contratos', async()=>{
  const {c,calls}=setup(); const text='🚀'.repeat(100); await c.search(text); assert.ok(calls.every(c=>c.input.query===text)); assert.ok(calls.every(c=>!('kind' in c.input)&&!('related_id' in c.input)&&!('extractor' in c.input))); c.dispose();
});
for(const failing of ['search.query','extraction.search']) test('Error parcial '+failing+' no oculta ni reconsulta otro grupo',async()=>{
  let fail=true; const {c,calls}=setup(cmd=>{if(fail&&cmd.operation===failing)throw error();}); await c.search('literal');
  const bad=failing==='search.query'?'notes':'attachments', good=bad==='notes'?'attachments':'notes'; assert.equal(c.state[bad].error.code,'revision_conflict'); assert.ok(c.state[good].page); const other=c.state[good].page;
  fail=false; await (bad==='notes'?c.startNotes():c.startAttachments()); assert.equal(calls.length,3); assert.equal(c.state[good].page,other); c.dispose();
});
test('Paginar reemplaza sólo su grupo, conservando cursor/contrato propios',async()=>{
  let n=0,a=0; const {c,calls}=setup(cmd=>cmd.operation==='search.query'?notes(++n===1?'notes-cursor':null):attachments(++a===1?'attachment-cursor':null));
  await c.search('literal'); const att=c.state.attachments.page; await c.nextNotes(); assert.equal(c.state.attachments.page,att); assert.equal(c.state.notes.pageNumber,2); assert.equal(c.state.notes.page.items.length,1); assert.equal(calls.at(-1).input.cursor,'notes-cursor');
  const note=c.state.notes.page; await c.nextAttachments(); assert.equal(c.state.notes.page,note); assert.equal(calls.at(-1).input.cursor,'attachment-cursor'); c.dispose();
});
test('Conflicto de índice al paginar no retira notas ni reinicia adjuntos',async()=>{
  let fail=false; const {c,calls}=setup(cmd=>{if(cmd.operation==='extraction.search'){if(fail)throw error();return attachments('cursor');}}); await c.search('literal'); const note=c.state.notes.page; fail=true; await c.nextAttachments();
  assert.equal(c.state.notes.page,note); assert.equal(c.state.attachments.page,null); assert.equal(c.state.attachments.error.reason,'index_revision_changed'); await c.nextAttachments(); assert.equal(calls.length,3); c.dispose();
});
test('Editar retira ambos grupos sin consultar ni permitir cursores/reintentos viejos',async()=>{
  const {c,calls}=setup(cmd=>cmd.operation==='search.query'?notes('n'):attachments('a')); await c.search('literal'); c.edit(); await c.nextNotes(); await c.nextAttachments(); await c.startNotes(); await c.startAttachments(); assert.equal(calls.length,2); assert.equal(c.state.notes.page,null); assert.equal(c.state.attachments.page,null); c.dispose();
});
test('Respuesta vieja descartada en ambos grupos y remount conserva cupos reales',async()=>{
  const n=deferred(),a=deferred(),one=setup(cmd=>cmd.operation==='search.query'?n.promise:a.promise); const running=one.c.search('old'); await tick(); one.c.dispose();
  const two=setup(); const queued=two.c.search('new'); await tick(); assert.equal(two.calls.length,0); two.c.edit(); n.resolve(notes(null)); a.resolve(attachments(null)); await Promise.all([running,queued]); await tick(); assert.equal(two.calls.length,0); assert.equal(two.c.state.notes.page,null); two.c.dispose();
});
test('Una respuesta excesiva de notas es error del grupo, sin truncar ni ocultar adjuntos',async()=>{
  const {c}=setup(cmd=>cmd.operation==='search.query'?{items:Array(21).fill(notes(null).items[0])}:undefined); await c.search('literal'); assert.equal(c.state.notes.page,null); assert.equal(c.state.notes.error.code,'transport_error'); assert.ok(c.state.attachments.page); c.dispose();
});
