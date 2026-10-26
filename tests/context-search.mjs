import test from 'node:test';
import assert from 'node:assert/strict';
import { ContextSearch, documentPreview, resultTarget } from '../src/lib/context-search.ts';
import { PRIMARY_SECTIONS, matchingSections } from '../src/lib/navigation.ts';
import { readCover, saveCover, COVER_KEY } from '../src/lib/home-cover.ts';
import { createSharkMesh, projectShark, sharkPoses } from '../src/lib/shark-scene.ts';
const criteria = query => ({ query, kind:'', archived:false, related:'' });
const flush = async () => { for(let i=0;i<16;i++) await Promise.resolve(); };
const deferred = () => { let resolve;const promise = new Promise(r=>resolve=r);return {promise,resolve}; };
const entity = (id,title=id) => ({id,title,kind:'note',revision:1,payload:{},document:{schema_version:2,blocks:[]}});
const hit = (id,block_id) => ({entity_id:id,title:id,kind:'note',block_id,revision:1});
test('seven ordered primary routes; secondary commands remain discoverable', () => {
  assert.deepEqual(PRIMARY_SECTIONS.map(s=>s.label),['Home','Tasks','Notes','Projects','Tags','Goals','People']);
  assert.equal(matchingSections('archivo')[0].id,'archive');
  assert.equal(matchingSections('mi dia')[0].id,'my_day');
  assert.equal(matchingSections('colecciones')[0].id,'collections');
});
test('search deduplicates sources, preserves cursor and uses actual filters', async () => {
  const calls=[];
  const reader=new ContextSearch(async command=>{calls.push(command);if(command.operation==='entity.get')return entity(command.input.id);return {items:command.input.cursor?[hit('b'),hit('c')]:[hit('a','one'),hit('a','two'),hit('b')],next_cursor:command.input.cursor?null:'next'};},()=>{});
  await reader.search({...criteria('hello'),kind:'note',archived:true,related:'ctx'});
  assert.deepEqual(reader.state.results.map(r=>r.id),['a','b']);
  assert.equal(calls[0].input.related_id,'ctx');assert.equal(calls[0].input.archived,true);
  assert.equal(resultTarget(reader.state.results[0]).hit.block_id,'one');
  await reader.more();assert.deepEqual(reader.state.results.map(r=>r.id),['a','b','c']);assert.equal(reader.state.cursor,null);
  reader.dispose();
});
test('new criteria immediately invalidate stale reads and previews; dispose cancels publication', async () => {
  const old=deferred(), preview=deferred(), updates=[];
  const reader=new ContextSearch(async command=>{if(command.operation==='entity.get')return preview.promise;if(command.input.query==='old')return old.promise;return {items:[hit('new')],next_cursor:null};},s=>updates.push(s));
  const pending=reader.search(criteria('old'));await flush();reader.invalidate();
  const fresh=reader.search(criteria('new'));await fresh;assert.equal(reader.state.results[0].id,'new');
  old.resolve({items:[hit('old')],next_cursor:null});await pending;assert.equal(reader.state.results[0].id,'new');
  await flush();reader.dispose();const count=updates.length;preview.resolve(entity('new'));await flush();assert.equal(updates.length,count);
});
test('preview selection generations and query failures keep explicit errors', async () => {
  const a=deferred();let fail=false;
  const reader=new ContextSearch(async c=>{if(fail)throw new Error('storage');if(c.operation==='entity.get')return c.input.id==='a'?a.promise:entity('b');return {items:[entity('a'),entity('b')],next_cursor:null};},()=>{});
  await reader.search(criteria(''));await flush();const invalid=reader.state.results.length+1;await reader.select(invalid);await reader.select(invalid); // no reads for invalid index
  // Initial command results are followed by actual pages.
  const b=reader.state.results.findIndex(r=>r.id==='b');await reader.select(b);assert.equal(reader.state.preview.id,'b');
  const ai=reader.state.results.findIndex(r=>r.id==='a');const pa=reader.select(ai);await flush();await reader.select(b);a.resolve(entity('a'));await pa;assert.equal(reader.state.preview.id,'b');
  fail=true;await reader.search(criteria('oops'));assert.equal(reader.state.error,'storage');assert.equal(reader.state.results.length,0);reader.dispose();
});
test('only known text is previewed, without evaluating content', () => {
  assert.equal(documentPreview({schema_version:99,blocks:[{content:[{text:'secret'}]}]}),'');
  assert.equal(documentPreview({schema_version:2,blocks:[{content:[{text:'<img onerror=alert(1)>'}],children:[{content:[{text:'child'}]}]}]}),'<img onerror=alert(1)>\n\nchild');
});
test('cover preference defaults to ASCII and recovers from unavailable storage', () => {
  const values=new Map(),store=()=>({getItem:k=>values.get(k),setItem:(k,v)=>values.set(k,v)});
  assert.equal(readCover(store),'ascii');assert.equal(saveCover('photo',store),'');assert.equal(readCover(store),'photo');
  values.set(COVER_KEY,'bad');assert.equal(readCover(store),'ascii');assert.equal(readCover(()=>{throw Error();}),'ascii');assert.match(saveCover('photo',()=>{throw Error();}),/no se pudo/);
});
test('procedural shark has finite volume, tail deformation and theme-independent geometry', () => {
  const mesh=createSharkMesh();assert(mesh.length>10000);assert(mesh.some(p=>p.y>1));assert(mesh.some(p=>p.z>1));
  const poses=sharkPoses(1000,288,0),later=sharkPoses(1000,288,4);assert.equal(poses.length,3);assert.notDeepEqual(poses,later);
  for(const p of mesh){const xy=projectShark(p,poses[0]);assert(Number.isFinite(xy.x)&&Number.isFinite(xy.y)&&Number.isFinite(xy.depth));}
});
