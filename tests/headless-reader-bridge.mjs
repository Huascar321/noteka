import { test } from 'node:test';
import assert from 'node:assert/strict';
import { Readable } from 'node:stream';
import { resolve } from 'node:path';
import { startReaderBridge } from '../scripts/headless-reader-bridge.mjs';

test('UTF-8 fragmentado requiere decoder continuo, conserva exactamente el texto', async () => {
  const literal = '  café 🚀\r\n<script>dato</script>  ', bytes = Buffer.from(literal);
  const pieces = Array.from(bytes, (_, i) => bytes.subarray(i, i + 1));
  let previous = ''; for (const piece of pieces) previous += piece;
  assert.notEqual(previous, literal); assert.ok(previous.includes('\ufffd'));
  const stream = Readable.from(pieces); stream.setEncoding('utf8'); let actual = '';
  for await (const piece of stream) actual += piece;
  assert.equal(actual, literal); assert.deepEqual(Buffer.from(actual), bytes);
});

test('Puente lector confina workspaces y sólo admite operaciones UI de lectura', async t => {
  const bridge = await startReaderBridge(resolve('dist'), true); t.after(() => bridge.close());
  const post = async (path, body, origin = bridge.origin) => fetch(bridge.origin + path, { method: 'POST', headers: { 'content-type': 'application/json', origin }, body: JSON.stringify(body) });
  const seed = await post('/__test__/control', { step: 'reader_seed', input: { theme: 'dark' } }); assert.equal(seed.status, 200);
  const f = await seed.json(), job = f.fixtures['literal.txt'].job;
  const command = { operation: 'extraction.fragments', input: { id: job.id, limit: 20 } };
  assert.equal((await post('/__test__/operate', { workspace: '/tmp/not-registered', command })).status, 500);
  assert.equal((await post('/__test__/operate', { workspace: f.workspace, command }, 'https://invalid.example')).status, 500);
  assert.equal((await post('/__test__/operate', { workspace: f.workspace, command: { operation: 'extraction.run', input: { id: job.id } } })).status, 500);
  const response = await post('/__test__/operate', { workspace: f.workspace, command }); assert.equal(response.status, 200);
  const actual = await response.json(); assert.equal(actual.ok, true); assert.deepEqual(actual.data, f.fixtures['literal.txt'].pages[0]);
  const operations = await post('/__test__/control', { step: 'reader_operations', input: { workspace: f.workspace } });
  const rows = await operations.json(); assert.equal(rows.length, 1); assert.equal(rows[0].readonly_unchanged, true);
});
