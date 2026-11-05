import { test } from 'node:test';
import assert from 'node:assert/strict';
import { RefreshCoordinator } from '../src/lib/refresh-coordinator.ts';

const deferred = () => { let resolve; const promise = new Promise(r => resolve = r); return { promise, resolve }; };
const turn = () => new Promise(resolve => setImmediate(resolve));

test('panel refresh collapses a burst and performs at most one trailing pass', async () => {
  const first = deferred(), second = deferred(); let calls = 0, failures = 0;
  const refresh = new RefreshCoordinator(() => true, () => (++calls === 1 ? first.promise : second.promise), () => failures++);
  const initial = refresh.request();
  for (let i = 0; i < 20; i++) void refresh.request();
  await turn(); assert.equal(calls, 1);
  for (let i = 0; i < 20; i++) void refresh.request();
  first.resolve(); await turn(); assert.equal(calls, 2);
  second.resolve(); await initial; await turn(); assert.equal(calls, 2); assert.equal(failures, 0);
});

test('blocked refresh is deferred and disposal prevents the trailing pass', async () => {
  const gate = deferred(); let ready = false, calls = 0;
  const refresh = new RefreshCoordinator(() => ready, () => { calls++; return gate.promise; }, () => {});
  for (let i = 0; i < 20; i++) void refresh.request();
  await turn(); assert.equal(calls, 0);
  ready = true; refresh.notifyReady(); await turn(); assert.equal(calls, 1);
  void refresh.request(); refresh.dispose(); gate.resolve(); await turn(); assert.equal(calls, 1);
});

test('changing search criteria cancels a pending refresh', async () => {
  let ready = false, calls = 0;
  const refresh = new RefreshCoordinator(() => ready, async () => { calls++; }, () => {});
  for (let i = 0; i < 20; i++) void refresh.request();
  refresh.cancelPending();
  ready = true; refresh.notifyReady(); await turn();
  assert.equal(calls, 0);
  await refresh.request();
  assert.equal(calls, 1);
});
