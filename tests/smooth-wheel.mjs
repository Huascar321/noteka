import test from 'node:test';
import assert from 'node:assert/strict';
import { isDiscreteWheel, readSmoothWheel, saveSmoothWheel, SMOOTH_WHEEL_KEY } from '../src/lib/smooth-wheel.ts';

const event = (values = {}) => ({ deltaMode: 0, deltaX: 0, deltaY: 40, ctrlKey: false, metaKey: false, altKey: false, shiftKey: false, ...values });
test('wheel signals distinguish conservative discrete candidates from unsupported gestures', () => {
  assert.equal(isDiscreteWheel(event({ wheelDeltaY: -120 })), true);
  assert.equal(isDiscreteWheel(event({ wheelDeltaY: 240, deltaY: -80 })), true);
  assert.equal(isDiscreteWheel(event({ deltaMode: 1, deltaY: 3 })), true);
  for (const change of [{ wheelDeltaY: -37 }, { wheelDeltaY: 0 }, { wheelDeltaY: -120, ctrlKey: true }, { wheelDeltaY: -120, shiftKey: true }, { wheelDeltaY: -120, deltaX: 1 }, { wheelDeltaY: -120, deltaY: 0 }, { wheelDeltaY: -120, deltaMode: 2 }, { wheelDeltaY: -120, deltaY: Infinity }]) assert.equal(isDiscreteWheel(event(change)), false);
});
test('local preference defaults on, persists and reports unavailable storage', () => {
  let value = null;
  const store = { getItem: key => { assert.equal(key, SMOOTH_WHEEL_KEY); return value; }, setItem: (key, next) => { assert.equal(key, SMOOTH_WHEEL_KEY); value = next; } };
  assert.equal(readSmoothWheel(() => store), true);
  value = 'invalid'; assert.equal(readSmoothWheel(() => store), true);
  assert.equal(saveSmoothWheel(false, () => store), ''); assert.equal(readSmoothWheel(() => store), false);
  assert.equal(saveSmoothWheel(true, () => store), ''); assert.equal(readSmoothWheel(() => store), true);
  const unavailable = () => { throw new Error('denied'); };
  assert.equal(readSmoothWheel(unavailable), true); assert.match(saveSmoothWheel(false, unavailable), /could not be saved/);
});
