import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdirSync, symlinkSync, mkdtempSync, realpathSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { startHeadlessBridge } from '../scripts/headless-bridge.mjs';
import { spawnSync } from 'node:child_process';

test('Runner nativo no lanza ventana sin consentimiento explícito', () => {
  const env = { ...process.env }; delete env.NOTEKA_NATIVE_SCREEN;
  const result = spawnSync(process.execPath, ['scripts/native-smoke.mjs'], { env, encoding: 'utf8', timeout: 10_000 });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /requiere pantalla\/foco/);
  assert.equal(result.stdout, '');
});

test('Puente canoniza el temporal enlazado sin ampliar su confinamiento', async () => {
  const parent = realpathSync(mkdtempSync(join(process.env.NOTEKA_TEST_TMP || tmpdir(), 'noteka-bridge-parent-')));
  const previous = process.env.NOTEKA_TEST_TMP;
  let bridge;
  try {
    mkdirSync(join(parent, 'target'));
    symlinkSync(join(parent, 'target'), join(parent, 'alias'));
    process.env.NOTEKA_TEST_TMP = join(parent, 'alias');
    bridge = await startHeadlessBridge();
    assert.equal(bridge.root, realpathSync(bridge.root));
    assert.equal(bridge.root.startsWith(join(parent, 'target') + '/'), true);
    const result = await fetch(bridge.origin + '/__test__/operate', {
      method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ workspace: join(bridge.root, 'workspace'), command: { operation: 'workspace.create', input: {} } }),
    });
    assert.equal(result.status, 200);
    assert.equal((await result.json()).ok, true);
  } finally {
    if (previous === undefined) delete process.env.NOTEKA_TEST_TMP; else process.env.NOTEKA_TEST_TMP = previous;
    if (bridge) await bridge.close();
    rmSync(parent, { recursive: true, force: true });
  }
});

test('Puente aislado: Rust/SQLite real, origen/rutas acotadas y fixtures propias', async () => {
  const bridge = await startHeadlessBridge();
  const post = async (route, data, headers = {}) => fetch(bridge.origin + route, {
    method: 'POST', headers: { 'content-type': 'application/json', ...headers }, body: JSON.stringify(data),
  });
  try {
    const workspace = join(bridge.root, 'dark/workspace');
    const operate = (operation, input = {}, path = workspace) => post('/__test__/operate', { workspace: path, command: { operation, input } });
    const created = await (await operate('workspace.create')).json();
    assert.equal(created.ok, true);
    assert.equal(created.data.path, workspace);
    const task = await (await operate('entity.create', { kind: 'task', title: 'Sólo sintético' })).json();
    assert.equal(task.ok, true);
    const page = await (await operate('home.query', { view: 'tasks_all' })).json();
    assert.equal(page.data.items[0].entity.id, task.data.id);
    assert.equal((await (await operate('home.query', { view: 'future' })).json()).error.code, 'validation_error');
    assert.equal((await operate('entity.list', {}, '/')).status, 500);
    assert.equal((await operate('document.export', { id: task.data.id, destination: '/' })).status, 500);
    assert.equal((await operate('attachment.import', { source_path: '/' })).status, 500);
    mkdirSync(join(bridge.root, 'links'));
    symlinkSync('/', join(bridge.root, 'links/escape'));
    assert.equal((await operate('workspace.create', {}, join(bridge.root, 'links/escape/nonexistent'))).status, 500);
    symlinkSync(join(bridge.root, 'absent-target'), join(bridge.root, 'links/dangling'));
    assert.equal((await operate('workspace.create', {}, join(bridge.root, 'links/dangling/workspace'))).status, 500);
    assert.equal((await post('/__test__/control', { step: 'configure', input: { theme: '../escape', phase: 'main' } })).status, 500);
    assert.equal((await post('/__test__/control', { step: 'status' }, { origin: 'https://example.org' })).status, 500);
    assert.equal((await post('/__test__/control', { step: 'status' }, { 'content-type': 'text/plain' })).status, 500);
  } finally { await bridge.close(); }
});

test('Evidencia: fixture CLI real, snapshots puros, controles confinados y errores sin reparar', async () => {
  const bridge = await startHeadlessBridge();
  const post = async (route, data) => fetch(bridge.origin + route, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(data) });
  const control = async (step, input) => {
    const response = await post('/__test__/control', { step, input });
    assert.equal(response.status, 200); return response.json();
  };
  try {
    for (const input of [{ theme: '../escape' }, { theme: 'dark/../../' }]) assert.equal((await post('/__test__/control', { step: 'evidence_seed', input })).status, 500);
    assert.equal((await post('/__test__/control', { step: 'evidence_snapshot', input: { workspace: '/' } })).status, 500);
    assert.equal((await post('/__test__/control', { step: 'evidence_snapshot', input: { workspace: join(bridge.root, 'unregistered') } })).status, 500);
    const fixture = await control('evidence_seed', { theme: 'dark' }), workspace = fixture.workspace;
    assert.equal(fixture.checks.hits, 31); assert.equal(fixture.checks.fields.length, 6);
    const operate = async query => (await post('/__test__/operate', { workspace, command: { operation: 'extraction.search', input: query } })).json();
    const before = await control('evidence_snapshot', { workspace });
    const result = await operate({ query: 'nkprobe', limit: 20 });
    assert.deepEqual(result.data, fixture.pages[0]);
    assert.deepEqual(await control('evidence_snapshot', { workspace }), before);
    assert.deepEqual((await control('evidence_operations', { workspace })).map(op => op.operation), ['extraction.search']);
    assert.equal((await control('evidence_operations', { workspace }))[0].readonly_unchanged, true);
    assert.equal((await post('/__test__/control', { step: 'evidence_tamper', input: { workspace, altered: 'true' } })).status, 500);
    await control('evidence_tamper', { workspace, altered: true });
    const altered = await control('evidence_snapshot', { workspace });
    const invalid = await operate({ query: 'nkprobe', attachment_id: fixture.jobs['literal.txt'].attachment_id });
    assert.equal(invalid.ok, false); assert.equal(invalid.error.code, 'storage_error');
    assert.deepEqual(await control('evidence_snapshot', { workspace }), altered);
    await control('evidence_tamper', { workspace, altered: false });
    assert.deepEqual(await control('evidence_snapshot', { workspace }), before);
    await control('evidence_reset', { workspace });
    const reset = await control('evidence_snapshot', { workspace });
    const conflict = await operate({ query: 'nkprobe', limit: 20, cursor: result.data.next_cursor });
    assert.equal(conflict.error.code, 'revision_conflict'); assert.equal(conflict.error.details.reason, 'index_revision_changed');
    assert.deepEqual(await control('evidence_snapshot', { workspace }), reset);
  } finally { await bridge.close(); }
});

test('Evidencia restaurada conserva hits, pero rechaza cursor de otra ruta sin mutar', async () => {
  const bridge = await startHeadlessBridge();
  const control = async (step, input) => {
    const response = await fetch(bridge.origin + '/__test__/control', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ step, input }) });
    assert.equal(response.status, 200); return response.json();
  };
  try {
    const fixture = await control('evidence_seed', { theme: 'light' });
    const restored = await control('evidence_roundtrip', { workspace: fixture.workspace });
    assert.notEqual(restored.path, fixture.workspace);
    const before = await control('evidence_snapshot', { workspace: restored.path });
    const response = await control('evidence_query', { workspace: restored.path, query: { query: 'nkprobe', limit: 20 } });
    assert.deepEqual(response.data.items, fixture.pages[0].items); assert.deepEqual(response.data.coverage, fixture.coverage);
    const error = await control('evidence_query', { workspace: restored.path, query: { query: 'nkprobe', limit: 20, cursor: fixture.pages[0].next_cursor } });
    assert.equal(error.error.code, 'validation_error');
    assert.deepEqual(await control('evidence_snapshot', { workspace: restored.path }), before);
  } finally { await bridge.close(); }
});
