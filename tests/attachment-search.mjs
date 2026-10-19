import { test } from 'node:test';
import assert from 'node:assert/strict';
import { AttachmentReader, ATTACHMENT_PAGE_SIZE } from '../src/lib/attachment-search.ts';

const criteria = (query = ' café 🚀 ') => ({ query, extractor: '', job_id: '', attachment_id: '' });
const tick = () => new Promise(resolve => setImmediate(resolve));
const deferred = () => { let resolve, reject; const promise = new Promise((a, b) => { resolve = a; reject = b; }); return { promise, resolve, reject }; };
const hit = ordinal => ({ ordinal, text: `texto ${ordinal}`, score: -1, result_from_previous_attempt: false,
  source: { job_id: 'job', attachment_id: 'attachment', generation_id: 'generation', source_sha256: 'hash', extractor: 'utf8_text', extractor_version: 1, config_hash: 'config' },
  locator: { kind: 'utf8_bytes', byte_start: 0, byte_end: 1, line_start: 1, line_end: 1 } });
const page = (items = [], cursor = null) => ({ version: 1, profile: 'literal_unicode61_v1', revision: 5, items, next_cursor: cursor,
  coverage: { scope: 'selected_generations', index_validation: 'canonical_text_and_postings', source_validation: 'returned_candidates', indexed_generations: 4, unindexed_supported_generations: 1, unsupported_generations: 1, jobs_without_generation: 1, fragments: 31, text_bytes: 9000 } });
const operationError = (code, reason) => Object.assign(new Error('Error seguro del núcleo'), { detail: { code, message: 'Error seguro del núcleo', details: { reason } } });

test('No consulta al montar; distingue estado inicial de cero hits y conserva consulta exacta', async () => {
  const calls = [], changes = [];
  const reader = new AttachmentReader(async input => { calls.push(input); return page(); }, s => changes.push(s));
  assert.equal(calls.length, 0); assert.equal(reader.state.page, null);
  await reader.search(criteria('   ')); assert.equal(calls.length, 0);
  await reader.search(criteria());
  assert.deepEqual(calls, [{ query: ' café 🚀 ', limit: 20 }]);
  assert.equal(reader.state.page.items.length, 0); assert.equal(reader.state.pageNumber, 1);
  assert.equal(changes[0].busy, true); reader.dispose();
});

test('Filtros opcionales omitidos y AND enviados sin validación de dominio duplicada', async () => {
  const calls = [];
  const reader = new AttachmentReader(async input => { calls.push(input); return page(); }, () => {});
  const query = '🚀'.repeat(100);
  await reader.search({ query, extractor: 'xlsx_cells', job_id: ' invalid-id ', attachment_id: ' attachment ' });
  assert.deepEqual(calls[0], { query, limit: ATTACHMENT_PAGE_SIZE, extractor: 'xlsx_cells', job_id: 'invalid-id', attachment_id: 'attachment' });
  reader.edit(); await reader.search({ ...criteria(), job_id: '  ', attachment_id: '\t' });
  assert.deepEqual(calls[1], { query: ' café 🚀 ', limit: 20 }); reader.dispose();
});

test('Página siguiente reemplaza, conserva cursor/criterios y reinicio explícito no lo envía', async () => {
  const calls = [], first = page(Array.from({ length: 20 }, (_, i) => hit(i)), 'opaque'), second = page([hit(20)]);
  const reader = new AttachmentReader(async input => { calls.push(input); return calls.length === 2 ? second : first; }, () => {});
  const draft = criteria('nkprobe'); await reader.search(draft); draft.query = 'otro';
  await reader.next();
  assert.deepEqual(reader.state.page.items.map(h => h.ordinal), [20]); assert.equal(reader.state.pageNumber, 2);
  assert.deepEqual(calls[1], { query: 'nkprobe', limit: 20, cursor: 'opaque' });
  await reader.next(); assert.equal(calls.length, 2);
  await reader.search(criteria('nkprobe')); assert.equal(calls[2].cursor, undefined); assert.equal(reader.state.pageNumber, 1); reader.dispose();
});

test('Editar invalida página/cursor/cobertura y no ejecuta otra lectura', async () => {
  let calls = 0;
  const reader = new AttachmentReader(async () => { calls++; return page([hit(0)], 'cursor'); }, () => {});
  await reader.search(criteria()); reader.edit(); await reader.next();
  assert.deepEqual(reader.state, { page: null, pageNumber: 0, busy: false, error: null }); assert.equal(calls, 1); reader.dispose();
});

for (const code of ['validation_error', 'revision_conflict', 'workspace_busy', 'storage_error', 'unsupported_workspace_version']) {
  test(`${code} retira página y razón conservada; nunca reintenta automáticamente`, async () => {
    let calls = 0;
    const reader = new AttachmentReader(async () => { if (++calls === 1) return page([hit(0)], 'cursor'); throw operationError(code, 'razon_segura'); }, () => {});
    await reader.search(criteria()); await reader.next();
    assert.equal(reader.state.page, null); assert.equal(reader.state.busy, false);
    assert.deepEqual(reader.state.error, { code, message: 'Error seguro del núcleo', reason: 'razon_segura' });
    await reader.next(); await tick(); assert.equal(calls, 2); reader.dispose();
  });
}

test('Fallo de transporte y valor no Error no aparecen como cero hits', async () => {
  for (const error of [new Error('IPC falló'), null]) {
    const reader = new AttachmentReader(async () => { throw error; }, () => {});
    await reader.search(criteria()); assert.equal(reader.state.error.code, 'transport_error'); assert.equal(reader.state.page, null); reader.dispose();
  }
});

test('No publica páginas excesivas o de revisión/perfil/versión distinta', async () => {
  for (const patch of [{ revision: 6 }, { profile: 'future' }, { version: 2 }, { items: Array.from({ length: 21 }, (_, i) => hit(i)) }]) {
    let n = 0;
    const reader = new AttachmentReader(async () => ++n === 1 ? page([hit(0)], 'cursor') : { ...page(), ...patch }, () => {});
    await reader.search(criteria()); await reader.next();
    assert.equal(reader.state.page, null); assert.equal(reader.state.error.code, 'transport_error'); reader.dispose();
  }
});

test('Ignora éxito/rechazo tardíos y no libera el cupo físico al editar', async () => {
  for (const reject of [false, true]) {
    const old = deferred(), calls = [], changes = [];
    const reader = new AttachmentReader(input => { calls.push(input.query); return calls.length === 1 ? old.promise : Promise.resolve(page([hit(1)])); }, s => changes.push(s));
    const initial = reader.search(criteria('antes')); await tick();
    reader.edit(); const later = reader.search(criteria('después')); await tick();
    assert.deepEqual(calls, ['antes']); assert.equal(reader.state.busy, true);
    const count = changes.length;
    if (reject) old.reject(operationError('storage_error', 'tardio')); else old.resolve(page([hit(0)]));
    await Promise.all([initial, later]);
    assert.deepEqual(calls, ['antes', 'después']);
    assert.deepEqual(reader.state.page.items.map(h => h.ordinal), [1]); assert.equal(reader.state.error, null);
    assert.equal(changes.length, count + 1); reader.dispose();
  }
});

test('Dispose/remontaje comparte cupo y elimina espera cancelada sin iniciar IPC', async () => {
  const old = deferred(), calls = [], oldChanges = [];
  const first = new AttachmentReader(() => { calls.push('old'); return old.promise; }, s => oldChanges.push(s));
  const pending = first.search(criteria()); await tick(); first.dispose(); const count = oldChanges.length;
  const second = new AttachmentReader(async input => { calls.push(input.query); return page([hit(2)]); }, () => {});
  const canceled = second.search(criteria('cancelada')); await tick(); second.edit();
  const latest = second.search(criteria('nueva')); await tick(); assert.deepEqual(calls, ['old']);
  old.reject(new Error('error del workspace viejo')); await Promise.all([pending, canceled, latest]);
  assert.deepEqual(calls, ['old', 'nueva']); assert.equal(oldChanges.length, count);
  assert.equal(second.state.page.items[0].ordinal, 2);
  second.dispose(); await second.search(criteria('no')); second.edit(); await second.next(); assert.deepEqual(calls, ['old', 'nueva']);
});

test('Doble envío en curso sólo ejecuta un IPC; dispose ignora éxito tardío', async () => {
  const waiting = deferred(), changes = []; let calls = 0;
  const reader = new AttachmentReader(() => { calls++; return waiting.promise; }, s => changes.push(s));
  const initial = reader.search(criteria()); await reader.search(criteria('otra')); await reader.next(); await tick();
  assert.equal(calls, 1); reader.dispose(); const count = changes.length;
  waiting.resolve(page([hit(0)])); await initial; assert.equal(changes.length, count);
});
