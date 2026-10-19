import test from 'node:test';
import assert from 'node:assert/strict';
import { HomeReader } from '../src/lib/home-reads.ts';
import { ReadQueue } from '../src/lib/embedded-reads.ts';
import { readTheme, saveTheme, THEME_KEY } from '../src/lib/theme.ts';
const deferred = () => { let resolve, reject; const promise = new Promise((r, j) => { resolve = r; reject = j; }); return { promise, resolve, reject }; };
const flush = async () => { for (let i = 0; i < 12; i++) await Promise.resolve(); };
const page = (view, date = '2026-10-12') => ({ view, date, items: [], groups: [], offset: 0, next_cursor: null });
test('Home retira página, descarta generación vieja y conserva IPC en su cupo', async () => {
  const queue = new ReadQueue(2), a = deferred(), b = deferred(), calls = [];
  const r = new HomeReader('tasks_today', queue, q => { calls.push(q); return q.view === 'tasks_today' ? a.promise : b.promise; }, () => {});
  const first = r.load(); await flush(); const second = r.load('tasks_all'); await flush();
  assert.equal(r.state.page, null); a.resolve(page('tasks_today')); await first; assert.equal(r.state.view, 'tasks_all'); assert.equal(r.state.busy, true);
  b.resolve(page('tasks_all')); await second; assert.equal(r.state.page.view, 'tasks_all'); assert.equal(calls.length, 2);
  const third = r.load('tasks_today'); assert.equal(r.state.page, null); await third;
});
test('Home cancela encoladas/desmontadas sin ejecutarlas ni publicar errores', async () => {
  const queue = new ReadQueue(1), blocked = deferred(), calls = [], updates = [];
  const a = new HomeReader('notes_all', queue, () => blocked.promise, () => {});
  const b = new HomeReader('people_new', queue, q => { calls.push(q); return Promise.resolve(page(q.view)); }, s => updates.push(s));
  const first = a.load(); await flush(); const old = b.load(); const latest = b.load('people_contacts'); b.dispose(); await old; await latest;
  blocked.resolve(page('notes_all')); await first; await flush(); assert.equal(calls.length, 0); assert.equal(updates.length, 2);
  await b.load(); assert.equal(updates.length, 2);
});
test('Home errores independientes, retry/página y día sin filas obsoletas', async () => {
  const queue = new ReadQueue(2), queries = []; let fail = false;
  const a = new HomeReader('tags_recent', queue, async q => { queries.push(q); if (fail) throw new Error('storage'); return page(q.view, q.as_of); }, () => {});
  const b = new HomeReader('projects_board', queue, async q => page(q.view), () => {});
  await Promise.all([a.load(), b.load()]); fail = true; await a.load(); assert.equal(a.state.error, 'storage'); assert.equal(a.state.page, null); assert.equal(b.state.page.view, 'projects_board');
  fail = false; await a.load(undefined, 'cursor', '2026-10-11'); assert.equal(a.state.page.date, '2026-10-11'); assert.equal(queries.at(-1).cursor, 'cursor');
  await a.load(); assert.equal(queries.at(-1).as_of, undefined);
  fail = true; await a.load(undefined, 'second-page', '2026-10-10'); fail = false; await a.retry(); assert.equal(queries.at(-1).cursor, 'second-page'); assert.equal(queries.at(-1).as_of, '2026-10-10');
});
test('Home rechaza respuesta incompatible y no publica IPC después de dispose', async () => {
  const d = deferred(); let updates = 0;
  const r = new HomeReader('notes_inbox', new ReadQueue(), () => d.promise, () => updates++);
  const loading = r.load(); await flush(); r.dispose(); d.resolve(page('notes_inbox')); await loading; assert.equal(updates, 1);
  const wrong = new HomeReader('tasks_all', new ReadQueue(), async () => page('tasks_today'), () => {});
  await wrong.load(); assert.equal(wrong.state.page, null); assert.match(wrong.state.error, /incompatible/);
});
test('Tema defaults, valores inválidos, lectura/escritura fallida y clave local aislada', () => {
  let value = null; const store = { getItem: key => { assert.equal(key, THEME_KEY); return value; }, setItem: (key, v) => { assert.equal(key, THEME_KEY); value = v; } };
  assert.equal(readTheme(() => store), 'dark'); value = 'invalid'; assert.equal(readTheme(() => store), 'dark');
  assert.equal(saveTheme('light', () => store), ''); assert.equal(readTheme(() => store), 'light');
  const unavailable = () => { throw new Error('SecurityError'); }; assert.equal(readTheme(unavailable), 'dark'); assert.match(saveTheme('light', unavailable), /no se pudo guardar/);
  assert.match(saveTheme('dark', () => ({ ...store, setItem() { throw new Error('quota'); } })), /no se pudo guardar/);
});
