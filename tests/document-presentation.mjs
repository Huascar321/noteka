import test from 'node:test';
import assert from 'node:assert/strict';
import { importedPresentation, composeImportedBody } from '../src/lib/imported-properties.ts';
import { markdownTable } from '../src/lib/markdown-table.ts';
const p=(id,text)=>({id,type:'paragraph',content:[{type:'text',text}]});
const properties=[['Favorite','Yes'],['Type','Reference'],['Created','June 1, 2026'],['Archived','No']];
const prefix=[{id:'import-heading',type:'heading',attrs:{level:3},content:[{type:'text',text:'Propiedades de Notion'}]}, {id:'import-list',type:'bullet_list',content:[],children:properties.map(([key,value],i)=>({id:'item-'+i,type:'list_item',content:[],children:[p('value-'+i,`${key}: ${value}`)]}))},p('duplicate','Favorite: Yes\nType: Reference\nCreated: June 1, 2026\nArchived: No')];
const doc={schema_version:2,blocks:[...prefix,p('body','Document body')]};
test('import prefix and duplicate property paragraph become presentation metadata, retaining every original block',()=>{
 const original=JSON.stringify(doc),view=importedPresentation(doc);assert.equal(view.properties.length,4);assert.equal(view.prefix.length,3);assert.equal(view.body.blocks[0].id,'body');
 assert.deepEqual(composeImportedBody(view.prefix,view.body),doc);assert.equal(JSON.stringify(doc),original);
});
test('body edit recomposes original import prefix and IDs without changing source properties',()=>{
 const view=importedPresentation(doc),edited=composeImportedBody(view.prefix,{schema_version:2,blocks:[p('body','Edited content')]});
 assert.deepEqual(edited.blocks.slice(0,3),prefix);assert.equal(edited.blocks[3].content[0].text,'Edited content');assert.notEqual(edited.blocks[0],prefix[0]);
});
test('partial and ambiguous property structures stay in the original document',()=>{
 for(const value of [{schema_version:7,blocks:doc.blocks},{schema_version:2,blocks:[p('a','Propiedades de Notion'),...doc.blocks.slice(1)]},{schema_version:2,blocks:[doc.blocks[0],{...doc.blocks[1],children:[p('x','Type: Reference')]},p('body','Original')]}]){const view=importedPresentation(value);assert.equal(view.prefix.length,0);assert.equal(view.body,value);}
 const extra={...doc,blocks:[...prefix.slice(0,2),p('extra','Type: Reference\nActual content')]};assert.equal(importedPresentation(extra).prefix.length,2);assert.equal(importedPresentation(extra).body.blocks[0].id,'extra');
});
test('Markdown table view handles literal data and escaped pipes without evaluating HTML',()=>{
 const source='| Field | Value |\n| --- | --- |\n| Label | <img onerror=alert(1)> |\n| Pipe | one\\|two |';
 assert.deepEqual(markdownTable(source,'markdown'),[['Field','Value'],['Label','<img onerror=alert(1)>'],['Pipe','one|two']]);
 assert.equal(markdownTable(source,'rust'),null);assert.equal(markdownTable('| a | b |\n| c | d |','markdown'),null);assert.equal(markdownTable('| a |\n| --- |\n| x | y |','markdown'),null);
});
