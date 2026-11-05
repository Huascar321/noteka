import { test } from 'node:test';
import assert from 'node:assert/strict';
import { WorkspaceReadPool } from '../src/lib/workspace-reads.ts';

const deferred = () => { let resolve; const promise = new Promise(r => resolve = r); return { promise, resolve }; };
const turn = () => new Promise(resolve => setImmediate(resolve));

test('workspace read pools retain physical slots after cancellation and remove queued work', async () => {
  const pool = new WorkspaceReadPool(), first = deferred(), second = deferred(), third = deferred();
  const controller = new AbortController(), cancelled = new AbortController();
  let started = 0, active = 0, maximum = 0, cancelledStarted = false;
  const work = gate => async () => { started++; active++; maximum = Math.max(maximum, active); try { return await gate.promise; } finally { active--; } };
  const a = pool.run('/workspace', work(first), controller.signal);
  const b = pool.run('/workspace', work(second));
  await turn();
  assert.equal(started, 2);
  controller.abort();
  await assert.rejects(a, { name: 'AbortError' });
  const c = pool.run('/workspace', work(third));
  const d = pool.run('/workspace', async () => { cancelledStarted = true; }, cancelled.signal);
  cancelled.abort();
  await assert.rejects(d, { name: 'AbortError' });
  await turn();
  assert.equal(started, 2);
  assert.equal(pool.size, 1);
  first.resolve('a'); await turn();
  assert.equal(started, 3);
  assert.equal(maximum, 2);
  second.resolve('b'); third.resolve('c');
  assert.equal(await b, 'b'); assert.equal(await c, 'c');
  await turn();
  assert.equal(pool.size, 0);
  assert.equal(cancelledStarted, false);
});

test('different workspaces have separate pools and aborted signals leave no entry', async () => {
  const pool = new WorkspaceReadPool(), left = deferred(), right = deferred();
  const a = pool.run('/left', () => left.promise), b = pool.run('/right', () => right.promise);
  await turn(); assert.equal(pool.size, 2);
  const controller = new AbortController(); controller.abort();
  await assert.rejects(pool.run('/other', async () => 1, controller.signal), { name: 'AbortError' });
  assert.equal(pool.size, 2);
  left.resolve(1); right.resolve(2); await Promise.all([a, b]); await turn();
  assert.equal(pool.size, 0);
});
