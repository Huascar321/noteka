import test from 'node:test';
import assert from 'node:assert/strict';
import { openStartupWorkspace, readWorkspacePath, saveWorkspacePath, clearWorkspacePath, WORKSPACE_PATH_KEY } from '../src/lib/workspace-startup.ts';

const info = path => ({ id: '11111111-1111-4111-8111-111111111111', path, schema_version: 15, timezone: 'America/La_Paz' });
const failure = code => Object.assign(new Error('La ruta no contiene un workspace Noteka.'), { detail: { code, message: 'La ruta no contiene un workspace Noteka.', details: {} } });
const memoryStore = () => {
  const values = new Map();
  return { getItem: key => values.get(key) ?? null, setItem: (key, value) => values.set(key, value), removeItem: key => values.delete(key), values };
};

test('la preferencia guarda, lee y borra la ruta', () => {
  const store = memoryStore();
  assert.equal(readWorkspacePath(() => store), '');
  assert.equal(saveWorkspacePath('/tmp/espacio', () => store), '');
  assert.equal(store.values.get(WORKSPACE_PATH_KEY), '/tmp/espacio');
  assert.equal(readWorkspacePath(() => store), '/tmp/espacio');
  clearWorkspacePath(() => store);
  assert.equal(readWorkspacePath(() => store), '');
});

test('la preferencia tolera un storage no disponible', () => {
  const broken = () => { throw new Error('sin storage'); };
  assert.equal(readWorkspacePath(broken), '');
  assert.match(saveWorkspacePath('/tmp/espacio', broken), /could not be saved/);
  assert.doesNotThrow(() => clearWorkspacePath(broken));
});

test('abre el último workspace guardado sin consultar el predeterminado', async () => {
  const calls = [];
  const result = await openStartupWorkspace('/tmp/guardado', async path => { calls.push(path); return info(path); });
  assert.deepEqual(calls, ['/tmp/guardado']);
  assert.equal(result.workspace.path, '/tmp/guardado');
  assert.equal(result.error, '');
});

test('si el guardado no existe, cae al directorio de datos predeterminado', async () => {
  const calls = [];
  const result = await openStartupWorkspace('/tmp/borrado', async path => {
    calls.push(path);
    if (path) throw failure('not_found');
    return info('/data/Noteka/workspace');
  });
  assert.deepEqual(calls, ['/tmp/borrado', undefined]);
  assert.equal(result.workspace.path, '/data/Noteka/workspace');
  assert.equal(result.error, '');
});

test('sin workspace guardado ni predeterminado devuelve primer uso silencioso', async () => {
  const result = await openStartupWorkspace('', async () => { throw failure('not_found'); });
  assert.equal(result.workspace, null);
  assert.equal(result.error, '');
  const fallback = await openStartupWorkspace('/tmp/borrado', async () => { throw failure('not_found'); });
  assert.equal(fallback.workspace, null);
  assert.equal(fallback.error, '');
});

test('un error distinto de not_found se informa y no abre otro workspace', async () => {
  const calls = [];
  const result = await openStartupWorkspace('/tmp/futuro', async path => { calls.push(path); throw failure('unsupported_workspace_version'); });
  assert.deepEqual(calls, ['/tmp/futuro']);
  assert.equal(result.workspace, null);
  assert.match(result.error, /workspace Noteka/);
});

test('el predeterminado con error de almacenamiento se informa', async () => {
  const result = await openStartupWorkspace('', async () => { throw Object.assign(new Error('Base ocupada.'), { detail: { code: 'workspace_busy' } }); });
  assert.equal(result.workspace, null);
  assert.equal(result.error, 'Base ocupada.');
});
