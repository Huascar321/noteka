import test from 'node:test';
import assert from 'node:assert/strict';
import { HomeReader } from '../src/lib/home-reads.ts';
import { ReadQueue } from '../src/lib/embedded-reads.ts';
import { readTheme, resolveTheme, saveTheme, THEME_KEY } from '../src/lib/theme.ts';
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
  await Promise.all([a.load(), b.load()]); fail = true; await a.load(); assert.equal(a.state.error, 'storage'); assert.equal(a.state.page.view, 'tags_recent'); assert.equal(a.state.stale, true); assert.equal(b.state.page.view, 'projects_board');
  fail = false; await a.retry(); assert.equal(a.state.stale, false);
  await a.load(undefined, 'cursor', '2026-10-11'); assert.equal(a.state.page.date, '2026-10-11'); assert.equal(queries.at(-1).cursor, 'cursor');
  await a.load(); assert.equal(queries.at(-1).as_of, '2026-10-11');
  await a.load('tags_recent'); assert.equal(queries.at(-1).as_of, undefined);
  fail = true; await a.load(undefined, 'second-page', '2026-10-10'); fail = false; await a.retry(); assert.equal(queries.at(-1).cursor, 'second-page'); assert.equal(queries.at(-1).as_of, '2026-10-10');
});
test('Home rechaza respuesta incompatible y no publica IPC después de dispose', async () => {
  const d = deferred(); let updates = 0;
  const r = new HomeReader('notes_inbox', new ReadQueue(), () => d.promise, () => updates++);
  const loading = r.load(); await flush(); r.dispose(); d.resolve(page('notes_inbox')); await loading; assert.equal(updates, 1);
  const wrong = new HomeReader('tasks_all', new ReadQueue(), async () => page('tasks_today'), () => {});
  await wrong.load(); assert.equal(wrong.state.page, null); assert.match(wrong.state.error, /does not match the query/);
});
test('20 refresh events coalesce before the read and retain at most one trailing read', async () => {
  const first = deferred(), second = deferred(), calls = [];
  const reader = new HomeReader('notes_inbox', new ReadQueue(), query => {
    calls.push(query); return calls.length === 1 ? first.promise : second.promise;
  }, () => {});
  const started = reader.load();
  const early = Array.from({ length: 20 }, () => reader.load());
  await flush(); assert.equal(calls.length, 1);
  const during = Array.from({ length: 20 }, () => reader.load());
  first.resolve(page('notes_inbox')); await flush();
  assert.equal(calls.length, 2); assert.equal(reader.state.busy, true); assert.equal(reader.state.stale, true);
  second.resolve(page('notes_inbox')); await Promise.all([started, ...early, ...during]);
  assert.equal(calls.length, 2); assert.equal(reader.state.busy, false); assert.equal(reader.state.stale, false);
});
test('refresh preserves rows, rejects old cursors, reports resets and keeps stale rows on failure', async () => {
  const gate = deferred(), queries = []; let fail = false;
  const result = (ids, cursor) => ({ ...page('notes_all'), items: ids.map(id => ({ entity: { id } })), next_cursor: cursor });
  const reader = new HomeReader('notes_all', new ReadQueue(), async query => {
    queries.push(query);
    if (fail) throw new Error('offline');
    if (query.cursor) return result(['second'], null);
    if (queries.length === 3) return gate.promise;
    return result(['first'], 'next');
  }, () => {}, 1, true);
  await reader.load(); await reader.load('notes_all', 'next', '2026-10-12');
  assert.deepEqual(reader.state.page.items.map(item => item.entity.id), ['first', 'second']);
  const refresh = reader.load(); await flush();
  assert.equal(reader.state.busy, true); assert.equal(reader.state.stale, true); assert.equal(reader.state.page.items.length, 2);
  await reader.load(undefined, 'next'); assert.equal(queries.length, 3);
  gate.resolve(result(['fresh'], 'new-cursor')); await refresh;
  assert.deepEqual(reader.state.page.items.map(item => item.entity.id), ['fresh']);
  assert.equal(queries.at(-1).as_of, undefined, 'an implicit today stays implicit after pagination');
  assert.match(reader.state.notice, /Additional pages were reset/);
  fail = true; await reader.load();
  assert.equal(reader.state.error, 'offline'); assert.equal(reader.state.stale, true); assert.equal(reader.state.page.items[0].entity.id, 'fresh');
  await reader.load(undefined, 'new-cursor'); assert.equal(queries.length, 4);
  fail = false; await reader.retry(); assert.equal(reader.state.stale, false);
});
test('Tema defaults, valores inválidos, lectura/escritura fallida y clave local aislada', () => {
  let value = null; const store = { getItem: key => { assert.equal(key, THEME_KEY); return value; }, setItem: (key, v) => { assert.equal(key, THEME_KEY); value = v; } };
  assert.equal(readTheme(() => store), 'dark'); value = 'invalid'; assert.equal(readTheme(() => store), 'dark');
  assert.equal(saveTheme('light', () => store), ''); assert.equal(readTheme(() => store), 'light');
  assert.equal(saveTheme('system', () => store), ''); assert.equal(readTheme(() => store), 'system');
  assert.equal(resolveTheme('system', true), 'dark'); assert.equal(resolveTheme('system', false), 'light');
  assert.equal(resolveTheme('light', true), 'light'); assert.equal(resolveTheme('dark', false), 'dark');
  const unavailable = () => { throw new Error('SecurityError'); }; assert.equal(readTheme(unavailable), 'dark'); assert.match(saveTheme('system', unavailable), /could not be saved/);
  assert.match(saveTheme('dark', () => ({ ...store, setItem() { throw new Error('quota'); } })), /could not be saved/);
});
