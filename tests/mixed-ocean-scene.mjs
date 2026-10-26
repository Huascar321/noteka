import test from 'node:test';
import assert from 'node:assert/strict';
import { mixedOceanActors, depthAppearance } from '../src/lib/mixed-ocean-scene.ts';

test('all families share a stable depth order with near and far individuals', () => {
  for (const width of [420, 1000, 2400]) for (const seconds of [0, 8, 42, 180]) {
    const actors = mixedOceanActors(width, 288, seconds, 17);
    assert.deepEqual(new Set(actors.map(a => a.family)), new Set(['ocean', 'koi', 'jellyfish']));
    assert.equal(new Set(actors.map(a => a.id)).size, actors.length);
    for (let i = 1; i < actors.length; i++) assert(actors[i - 1].depth >= actors[i].depth);
    for (const family of ['ocean', 'koi', 'jellyfish']) {
      const same = actors.filter(a => a.family === family);
      assert(Math.min(...same.map(a => a.depth)) < .35);
      assert(Math.max(...same.map(a => a.depth)) > .65);
    }
  }
});

test('depth supplies smaller, lower contrast silhouettes while staying visible', () => {
  const near = depthAppearance(.1), far = depthAppearance(.9);
  assert(near.scale > far.scale * 1.7);
  assert(near.opacity > far.opacity * 2);
  for (const value of [-5, 0, .5, 1, 5]) {
    const appearance = depthAppearance(value);
    assert(appearance.opacity > 0 && appearance.opacity <= 1);
    assert(appearance.scale > 0);
  }
});

test('mixed geometry is finite, seeded and less dense in narrow covers', () => {
  assert.deepEqual(mixedOceanActors(0, 288, 0, 1), []);
  assert.deepEqual(mixedOceanActors(1000, NaN, 0, 1), []);
  assert(mixedOceanActors(420, 230, 8, 1).length < mixedOceanActors(1000, 288, 8, 1).length);
  for (const [width, height] of [[320, 230], [420, 230], [1000, 288], [2400, 288]]) {
    for (const seed of [0, 1, 17, 2147483646]) for (const time of [0, 8, 42, 180]) {
      const actors = mixedOceanActors(width, height, time, seed);
      assert.deepEqual(actors, mixedOceanActors(width, height, time, seed));
      for (const actor of actors) {
        assert(Object.values(actor.pose).every(Number.isFinite));
        assert(actor.pose.x >= 0 && actor.pose.x <= width);
        assert(actor.pose.y >= 0 && actor.pose.y <= height);
        assert(actor.opacity > 0 && actor.opacity <= 1);
      }
    }
  }
  assert.notDeepEqual(mixedOceanActors(1000, 288, 8, 1), mixedOceanActors(1000, 288, 8, 2));
});

test('continuous movement crosses the former scene boundary without replacing actors', () => {
  const before = mixedOceanActors(1000, 288, 41.999, 17);
  const after = mixedOceanActors(1000, 288, 42.001, 17);
  assert.deepEqual(new Set(before.map(a => a.id)), new Set(after.map(a => a.id)));
  for (const actor of before) {
    const next = after.find(a => a.id === actor.id);
    assert(Math.hypot(actor.pose.x - next.pose.x, actor.pose.y - next.pose.y) < 1);
  }
});

test('shared actors keep their depth and paths across the responsive density boundary', () => {
  const narrow = mixedOceanActors(599, 288, 8, 17);
  const wide = mixedOceanActors(601, 288, 8, 17);
  for (const actor of narrow) {
    const next = wide.find(a => a.id === actor.id);
    assert(next);
    assert.equal(actor.depth, next.depth);
    assert(Math.abs(actor.pose.x / 599 - next.pose.x / 601) < .01);
    assert(Math.abs(actor.pose.y - next.pose.y) < 1);
  }
});
