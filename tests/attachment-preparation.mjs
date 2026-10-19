import { test } from 'node:test';
import assert from 'node:assert/strict';
import { AttachmentPreparation, attachmentReferences } from '../src/lib/attachment-preparation.ts';
const tick = () => new Promise(resolve => setImmediate(resolve));
const deferred = () => { let resolve, reject; const promise = new Promise((a, b) => { resolve = a; reject = b; }); return { promise, resolve, reject }; };
const job = () => ({ id: 'job', attachment_id: 'attachment', revision: 7, attempts: 1, state: 'succeeded', supported: true, result_from_previous_attempt: false, generation: { id: 'generation', fragment_count: 0 }, config: { future: '<script>' } });
const index = () => ({ action: 'get', revision: 11, items: [], coverage: { indexed_generations: 0, unindexed_supported_generations: 1, unsupported_generations: 0 } });
const coreError = code => Object.assign(new Error('Error del núcleo'), { detail: { code, message: 'Error del núcleo', details: { reason: 'expected_revision_mismatch', current_revision: 99 } } });
function setup(respond = () => undefined) {
  const calls = [], changes = [];
  const c = new AttachmentPreparation(async cmd => {
    calls.push(cmd); const r = await respond(cmd); if (r !== undefined) return r;
    if (cmd.operation === 'extraction.get') return job();
    if (cmd.operation === 'extraction.control') return { paused: true, revision: 3 };
    if (cmd.operation === 'extraction.index') return index();
    if (cmd.operation === 'extraction.list') return { items: [job()], next_cursor: null };
    if (cmd.operation === 'extraction.enqueue') return { job: job(), deduplicated: true };
    if (cmd.operation === 'extraction.run') return { outcome: 'idle', reason: 'paused', recovered_count: 0, incompatible_count: 0 };
    return job();
  }, s => changes.push(s));
  return { c, calls, changes };
}
async function inspected(respond) { const s = setup(respond); s.c.select(' attachment '); s.c.chooseJob('job'); await s.c.inspect(); return s; }

test('Referencias conocidas únicas y rótulos literales sin I/O ni metadata inventada', () => {
  const block = (id, type, attrs, children = []) => ({ id, type, attrs, children, content: [] });
  const doc = { schema_version: 3, blocks: [block('a', 'file', { attachment_id: 'x', name: '<img src=x>' }), block('b', 'paragraph', {}, [block('c', 'image', { attachment_id: 'x', alt: 'otro' }), block('d', 'file', { attachment_id: 'y' })]), block('e', 'future', { attachment_id: 'z', name: 'no' }), block('f', 'file', 'invalid')] };
  assert.deepEqual(attachmentReferences(doc), [{ id: 'x', label: '<img src=x>' }, { id: 'y', label: 'Sin rótulo' }]); assert.deepEqual(attachmentReferences(null), []);
});
test('Montar y seleccionar no consulta ni escribe; IDs vacíos no operan', async () => {
  const { c, calls } = setup(); await c.list(); await c.inspect(); await c.enqueue('pdf_text'); c.select('attachment'); c.chooseJob('job'); assert.deepEqual(calls, []); c.dispose();
});
test('Listar filtra por ID, reemplaza páginas de 20 y reinicia cursor explícitamente', async () => {
  let n = 0; const { c, calls } = setup(cmd => cmd.operation === 'extraction.list' ? { items: [job()], next_cursor: ++n === 1 ? 'opaque' : null } : undefined);
  c.select(' attachment '); await c.list(); await c.list(true); assert.equal(c.state.pageNumber, 2); assert.equal(c.state.jobs.items.length, 1);
  assert.deepEqual(calls[1].input, { attachment_id: 'attachment', limit: 20, cursor: 'opaque' }); await c.list(true); assert.equal(calls.length, 2);
  await c.list(); assert.equal(calls[2].input.cursor, undefined); c.dispose();
});
test('Inspección usa tres lecturas sin writes y conserva soporte/metadata cruda', async () => {
  const { c, calls } = await inspected(cmd => cmd.operation === 'extraction.get' ? { ...job(), supported: false, unsupported_reason: 'future' } : undefined);
  assert.deepEqual(calls.map(c => c.operation), ['extraction.get', 'extraction.control', 'extraction.index']);
  assert.deepEqual(calls[2].input, { action: 'get', job_id: 'job', limit: 1 }); assert.equal(c.state.job.config.future, '<script>'); assert.equal(c.state.job.supported, false); c.dispose();
});
test('Una lectura fallida retira inspección anterior y no reintenta', async () => {
  let fail = false; const { c, calls } = await inspected(() => { if (fail) throw coreError('storage_error'); });
  fail = true; await c.inspect(); assert.equal(c.state.job, null); assert.equal(c.state.index, null); assert.equal(c.state.error.code, 'storage_error'); assert.equal(calls.length, 4); c.dispose();
});
for (const action of ['run', 'cancel', 'retry', 'pause', 'put']) test(`${action}: una sola mutación con identidad/CAS observados, sin encadenar`, async () => {
  const { c, calls } = await inspected();
  if (action === 'pause') await c.setPaused(); else if (action === 'put') await c.selectGeneration(); else await c.jobAction(action);
  const expected = action === 'pause' ? { operation: 'extraction.control', input: { action: 'set', paused: false, expected_revision: 3 } }
    : action === 'put' ? { operation: 'extraction.index', input: { action: 'put', job_id: 'job', expected_generation_id: 'generation', expected_revision: 11 } }
    : { operation: `extraction.${action}`, input: action === 'run' ? { id: 'job' } : { id: 'job', expected_revision: 7 } };
  assert.deepEqual(calls[3], expected); assert.equal(calls.length, 4); assert.equal(c.state.needsConsult, true); assert.equal(c.state.job, null); assert.equal(c.state.receipt.operation, expected.operation);
  await c.setPaused(); await c.jobAction('run'); await c.enqueue('utf8_text'); await c.selectGeneration(); assert.equal(calls.length, 4); c.dispose();
});
test('Enqueue manda extractor elegido/defaults core; dedupe no ejecuta ni selecciona', async () => {
  const { c, calls } = setup(); c.select('attachment'); await c.enqueue('xlsx_cells');
  assert.deepEqual(calls, [{ operation: 'extraction.enqueue', input: { attachment_id: 'attachment', extractor: 'xlsx_cells' } }]);
  assert.equal(c.state.receipt.data.deduplicated, true); assert.equal(c.state.jobId, 'job'); assert.equal(c.state.needsConsult, true); c.dispose();
});
for (const problem of ['revision_conflict', 'transport']) test(`${problem}: invalida CAS y muestra incertidumbre sin repetir`, async () => {
  const { c, calls } = await inspected(cmd => { if (cmd.operation === 'extraction.index' && cmd.input.action === 'put') throw problem === 'transport' ? new Error('IPC perdido') : coreError(problem); });
  await c.selectGeneration(); assert.equal(c.state.error.uncertain, true); assert.equal(c.state.error.code, problem === 'transport' ? 'transport_error' : problem); assert.equal(c.state.index, null);
  await c.selectGeneration(); assert.equal(calls.length, 4); c.dispose();
});
test('Generación sin texto es seleccionable; incompatible o ausente no se ofrece', async () => {
  const { c } = await inspected(); assert.equal(c.canSelectGeneration, true); c.dispose();
  const bad = await inspected(cmd => cmd.operation === 'extraction.index' ? { ...index(), coverage: { indexed_generations: 0, unindexed_supported_generations: 0, unsupported_generations: 1 } } : undefined);
  assert.equal(bad.c.canSelectGeneration, false); await bad.c.selectGeneration(); assert.equal(bad.calls.length, 3); bad.c.dispose();
});
for (const rejection of [false, true]) test(`Respuesta tardía ${rejection ? 'rechazada' : 'correcta'} no publica al cambiar ID`, async () => {
  const wait = deferred(), { c, changes } = setup(() => wait.promise); c.select('attachment'); const read = c.list(); await tick(); c.select('other'); const count = changes.length;
  if (rejection) wait.reject(new Error('tardío')); else wait.resolve({ items: [job()] }); await read; await tick(); assert.equal(changes.length, count); assert.equal(c.state.attachmentId, 'other'); c.dispose();
});
test('Remount conserva cupo físico; mutación pendiente cancelada no inicia', async () => {
  const wait = deferred(), first = setup(() => wait.promise); first.c.select('attachment'); const running = first.c.enqueue('utf8_text'); await tick(); first.c.dispose();
  const second = setup(); second.c.select('attachment'); const queued = second.c.enqueue('utf8_text'); await tick(); assert.equal(second.calls.length, 0); second.c.select('other');
  wait.resolve({ job: job() }); await Promise.all([running, queued]); await tick(); assert.equal(first.calls.length, 1); assert.equal(second.calls.length, 0); second.c.dispose();
});
test('Abortar inspección no inicia segunda lectura y retiene cupo hasta resolver primera', async () => {
  const wait = deferred(), { c, calls } = setup(() => wait.promise); c.select('attachment'); c.chooseJob('job'); const p = c.inspect(); await tick(); c.chooseJob('other'); wait.resolve(job()); await p; await tick(); assert.equal(calls.length, 1); assert.equal(c.state.jobId, 'other'); c.dispose();
});
test('Página ajena/excesiva se rechaza sin mostrar datos viejos', async () => {
  const { c } = setup(cmd => cmd.operation === 'extraction.list' ? { items: [{ ...job(), attachment_id: 'other' }] } : undefined);
  c.select('attachment'); await c.list(); assert.equal(c.state.jobs, null); assert.equal(c.state.error.code, 'transport_error'); c.dispose();
});
