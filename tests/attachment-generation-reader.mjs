import { test } from 'node:test';
import assert from 'node:assert/strict';
import { AttachmentGenerationReader } from '../src/lib/attachment-generation-reader.ts';
const tick = () => new Promise(resolve => setImmediate(resolve));
const deferred = () => { let resolve, reject; const promise = new Promise((a, b) => { resolve = a; reject = b; }); return { promise, resolve, reject }; };
const page = (kind = 'utf8_text', items = [], cursor = null, generation = 'generation') => ({ job_id: 'job', generation: { id: generation, extractor: kind, future: { unchanged: true } }, items, next_cursor: cursor, covers_generation: !cursor, result_from_previous_attempt: true });
const criteria = kind => ({ id: ' job ', kind });
const coreError = code => Object.assign(new Error('Lectura fallida'), { detail: { code, message: 'Lectura fallida', details: { reason: 'input_stale' } } });
function setup(read) { const calls = [], changes = []; const c = new AttachmentGenerationReader(cmd => { calls.push(cmd); return read(cmd); }, s => changes.push(s)); return { c, calls, changes }; }

test('Montar/editar/dispose no lee; ID vacío no dispara operación', async () => {
  const { c, calls } = setup(() => page()); c.edit(); await c.read({ id: ' ', kind: 'pdf_text' }); c.dispose(); await c.read(criteria('pdf_text')); assert.deepEqual(calls, []);
});
for (const [kind, operation] of [['utf8_text', 'extraction.fragments'], ['pdf_text', 'extraction.pdf_fragments'], ['xlsx_cells', 'extraction.xlsx_cells'], ['image_ocr', 'extraction.ocr_regions']]) test(`${kind}: una lectura explícita exacta, sin get/import/índice`, async () => {
  const raw = page(kind), { c, calls } = setup(async () => raw); await c.read(criteria(kind));
  assert.deepEqual(calls, [{ operation, input: { id: 'job', limit: 20 } }]); assert.equal(c.state.page.kind, kind); assert.equal(c.state.page.data, raw); assert.equal(c.state.page.data.generation.future.unchanged, true); c.dispose();
});
test('Página siguiente reemplaza los veinte items, inicio no manda cursor', async () => {
  let count = 0; const { c, calls } = setup(async () => ++count === 1 ? page('utf8_text', Array.from({ length: 20 }, (_, ordinal) => ({ ordinal })), 'opaque') : page('utf8_text', [{ ordinal: 20 }]));
  const input = criteria('utf8_text'); await c.read(input); input.id = 'changed'; await c.next(); assert.deepEqual(c.state.page.data.items, [{ ordinal: 20 }]); assert.equal(c.state.pageNumber, 2); assert.deepEqual(calls[1].input, { id: 'job', limit: 20, cursor: 'opaque' });
  await c.next(); assert.equal(calls.length, 2); await c.read(criteria('utf8_text')); assert.equal(calls[2].input.cursor, undefined); c.dispose();
});
test('Vacío es página válida y no se confunde con error', async () => {
  const { c } = setup(async () => page()); await c.read(criteria('utf8_text')); assert.equal(c.state.page.data.items.length, 0); assert.equal(c.state.error, null); assert.equal(c.state.pageNumber, 1); c.dispose();
});
for (const code of ['validation_error', 'storage_error', 'revision_conflict', 'not_found', 'workspace_busy']) test(`${code}: retira página/cursor y conserva details sin reintento`, async () => {
  let count = 0; const { c, calls } = setup(async () => { if (++count > 1) throw coreError(code); return page('utf8_text', [{ text: '<script>dato</script>' }], 'old'); });
  await c.read(criteria('utf8_text')); await c.next(); assert.equal(c.state.page, null); assert.equal(c.state.pageNumber, 0); assert.equal(c.state.error.code, code); assert.equal(c.state.error.details.reason, 'input_stale'); await c.next(); assert.equal(calls.length, 2); c.dispose();
});
test('Fallo de transporte no se convierte en vacío ni reinicia lectura', async () => {
  const { c, calls } = setup(async () => { throw new Error('IPC perdido'); }); await c.read(criteria('pdf_text')); assert.equal(c.state.page, null); assert.equal(c.state.error.code, 'transport_error'); await c.next(); assert.equal(calls.length, 1); c.dispose();
});
for (const mismatch of ['generation', 'job', 'extractor', 'oversize']) test(`Respuesta ${mismatch} incompatible se rechaza entera`, async () => {
  let count = 0; const { c } = setup(async () => {
    if (++count === 1) return page('utf8_text', [], 'cursor');
    const p = page(); if (mismatch === 'generation') p.generation.id = 'other';
    if (mismatch === 'job') p.job_id = 'other'; if (mismatch === 'extractor') p.generation.extractor = 'pdf_text';
    if (mismatch === 'oversize') p.items = Array(21).fill({}); return p;
  }); await c.read(criteria('utf8_text')); await c.next(); assert.equal(c.state.page, null); assert.equal(c.state.error.code, 'transport_error'); c.dispose();
});
for (const rejected of [false, true]) test(`Éxito/error tardío ${rejected} descartado al editar`, async () => {
  const d = deferred(), { c, changes } = setup(() => d.promise); const reading = c.read(criteria('utf8_text')); await tick(); c.edit(); const count = changes.length;
  if (rejected) d.reject(coreError('storage_error')); else d.resolve(page()); await reading; await tick(); assert.equal(changes.length, count); assert.equal(c.state.page, null); c.dispose();
});
test('Remount no libera cupo físico y cancelar cola no inicia otro lector', async () => {
  const d = deferred(), one = setup(() => d.promise); const reading = one.c.read(criteria('utf8_text')); await tick(); one.c.dispose();
  const two = setup(async () => page('pdf_text')); const waiting = two.c.read(criteria('pdf_text')); await tick(); assert.equal(two.calls.length, 0); two.c.edit(); d.resolve(page()); await Promise.all([reading, waiting]); await tick(); assert.equal(two.calls.length, 0); two.c.dispose();
});
test('Cambiar tipo invalida criterios sin leer ni mezclar páginas', async () => {
  const { c, calls } = setup(async cmd => page(cmd.operation.endsWith('xlsx_cells') ? 'xlsx_cells' : 'utf8_text', [], 'cursor'));
  await c.read(criteria('utf8_text')); c.edit(); await c.next(); assert.equal(calls.length, 1); await c.read(criteria('xlsx_cells')); assert.equal(c.state.page.kind, 'xlsx_cells'); assert.equal(c.state.pageNumber, 1); c.dispose();
});
