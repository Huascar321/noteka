import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { toEditor, fromEditor, pendingJson } from '../src/lib/editor-adapter.ts';
const rich = JSON.parse(readFileSync(new URL('./fixtures/rich-document.json', import.meta.url)));
test('v2 semantic round-trip with nesting, marks and stable IDs', () => assert.deepEqual(fromEditor(toEditor(rich)), rich));
test('v1 upgrades only by explicit edit and keeps IDs', () => {
  const old = { schema_version: 1, blocks: [{ id: 'old', type: 'paragraph', content: [{ type: 'text', text: 'Hola' }] }] };
  assert.deepEqual(fromEditor(toEditor(old)), { ...old, schema_version: 2 }); assert.equal(old.schema_version, 1);
});
test('attachments have references, not remote URLs or editor serialization', () => {
  const d = { schema_version: 2, blocks: ['image', 'file'].map(type => ({ id: type, type, content: [], attrs: { attachment_id: 'a', ...(type === 'image' ? { alt: 'imagen' } : { name: 'archivo' }) } })) };
  assert.deepEqual(fromEditor(toEditor(d)), d);
});
test('duplicate and absent IDs are rejected, unsupported nodes and marks fail', () => {
  for (const content of [[{ type: 'paragraph' }], [{ type: 'paragraph', attrs: { id: 'a' } }, { type: 'paragraph', attrs: { id: 'a' } }], [{ type: 'widget', attrs: { id: 'a' } }], [{ type: 'paragraph', attrs: { id: 'a' }, content: [{ type: 'text', text: 'x', marks: [{ type: 'alien' }] }] }]]) assert.throws(() => fromEditor({ type: 'doc', content }));
});
test('pending recovery retains nesting, marks and references', () => assert.deepEqual(JSON.parse(pendingJson('Pendiente', rich)), { title: 'Pendiente', document: rich }));
test('v3 reference is semantic, nested, stable and retains version after deletion', () => {
  const d = structuredClone(rich); d.schema_version = 3;
  d.blocks.push({ id: 'embedded', type: 'collection_view', content: [], attrs: { collection_id: 'c', view_id: 'v' } });
  assert.deepEqual(fromEditor(toEditor(d), 3), d);
  assert.equal(fromEditor(toEditor(d)).schema_version, 3);
  assert.equal(fromEditor(toEditor({ schema_version: 3, blocks: [] }), 3).schema_version, 3);
  assert.deepEqual(JSON.parse(pendingJson('Page', d)).document, d);
  const nested = structuredClone(d); nested.blocks = [{ id: 'container', type: 'blockquote', content: [], children: nested.blocks }];
  assert.deepEqual(fromEditor(toEditor(nested), 3), nested);
  const bad = structuredClone(d); bad.blocks.at(-1).attrs.rows = [{ content: 'not source' }]; assert.throws(() => toEditor(bad));
  const json = toEditor(d); json.content.at(-1).content = [{ type: 'text', text: 'snapshot' }]; assert.throws(() => fromEditor(json));
  d.schema_version = 2; assert.throws(() => toEditor(d));
});
