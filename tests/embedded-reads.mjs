import { test } from 'node:test';
import assert from 'node:assert/strict';
import { ReadQueue, ReadLifetime } from '../src/lib/embedded-reads.ts';
const deferred = () => { let resolve; const promise = new Promise(r => resolve = r); return { promise, resolve }; };
const turn = () => new Promise(r => setImmediate(r));
test('two real calls at most, cancelled queued work never runs and failures release capacity', async () => {
  const queue = new ReadQueue(), work = [deferred(), deferred(), deferred()];
  let running = 0, max = 0, started = 0;
  const run = i => queue.run(async () => { started++; running++; max = Math.max(max, running); try { return await work[i].promise; } finally { running--; } }, new AbortController().signal);
  const a = run(0), b = run(1), c = run(2);
  const abort = new AbortController(); let cancelledRuns = 0;
  const cancelled = queue.run(async () => cancelledRuns++, abort.signal); abort.abort(); await assert.rejects(cancelled, { name: 'AbortError' });
  await turn(); assert.equal(started, 2); assert.equal(max, 2);
  work[0].resolve(1); await a; await turn(); assert.equal(started, 3); assert.equal(max, 2);
  work[1].resolve(2); work[2].resolve(3); assert.equal(await b, 2); assert.equal(await c, 3); assert.equal(cancelledRuns, 0);
  await assert.rejects(queue.run(async () => { throw new Error('read failed'); }, new AbortController().signal), /read failed/);
  assert.equal(await queue.run(async () => 42, new AbortController().signal), 42);
});
test('new read/disposal discard old result without freeing an in-flight IPC slot early', async () => {
  const queue = new ReadQueue(1), lifetime = new ReadLifetime(), first = lifetime.begin(), gate = deferred();
  const old = queue.run(() => gate.promise, first.signal); await turn(); const latest = lifetime.begin();
  await assert.rejects(old, { name: 'AbortError' }); assert.equal(lifetime.current(first.token), false);
  let started = false; const next = queue.run(async () => { started = true; return 'new'; }, latest.signal);
  await turn(); assert.equal(started, false); gate.resolve('old'); assert.equal(await next, 'new'); assert.equal(lifetime.current(latest.token), true);
  lifetime.dispose(); assert.equal(lifetime.current(latest.token), false); assert.equal(lifetime.begin().signal.aborted, true);
});
