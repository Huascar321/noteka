import test from 'node:test';
import assert from 'node:assert/strict';
import { koiPalette } from '../src/lib/ascii-koi-palette.ts';
import { DEFAULT_PRIMARY_COLOR, PRIMARY_COLORS, hexToHsl } from '../src/lib/primary-color.ts';

const hueDistance = (a, b) => { const d = Math.abs(a - b) % 360; return Math.min(d, 360 - d); };

test('koi marks keep the chosen primary hue in both themes', () => {
  for (const { name, value } of PRIMARY_COLORS) {
    const base = hexToHsl(value);
    for (const dark of [true, false]) {
      const [low, high] = koiPalette(value, dark);
      for (const color of [low, high]) {
        assert.match(color, /^#[0-9a-f]{6}$/, `${name}: ${color}`);
        assert(hueDistance(hexToHsl(color).h, base.h) < 5, `${name} hue drift at ${color}`);
      }
      const [lowLight, highLight] = [hexToHsl(low).l, hexToHsl(high).l];
      assert(highLight - lowLight > 25, `${name}: ${lowLight} -> ${highLight}`);
    }
  }
});

test('koi tint stays discreet compared with the accent', () => {
  for (const { name, value } of PRIMARY_COLORS) {
    const base = hexToHsl(value);
    for (const dark of [true, false]) for (const color of koiPalette(value, dark)) {
      const tint = hexToHsl(color);
      assert(tint.s >= base.s * .35 && tint.s <= base.s * .65, `${name}: tint saturation ${tint.s} vs accent ${base.s}`);
      assert(tint.s <= 36, `${name}: tint is too vivid (${tint.s})`);
    }
  }
});

test('koi palette is deterministic and follows color or theme changes', () => {
  assert.deepEqual(koiPalette(DEFAULT_PRIMARY_COLOR, true), koiPalette(DEFAULT_PRIMARY_COLOR, true));
  assert.notDeepEqual(koiPalette('#3183a6', true), koiPalette('#8254d4', true));
  assert.notDeepEqual(koiPalette('#3183a6', true), koiPalette('#c97927', true));
  assert.notDeepEqual(koiPalette('#3183a6', true), koiPalette('#3183a6', false));
});

test('invalid or missing primary recovers the default blue', () => {
  for (const value of ['', '#12345', 'orange', '#gggggg']) {
    for (const dark of [true, false]) assert.deepEqual(koiPalette(value, dark), koiPalette(DEFAULT_PRIMARY_COLOR, dark));
  }
});

test('grayscale primaries stay neutral instead of borrowing an accent hue', () => {
  for (const value of ['#808080', '#ffffff', '#000000']) for (const dark of [true, false]) {
    for (const color of koiPalette(value, dark)) {
      const channels = [1, 3, 5].map(offset => parseInt(color.slice(offset, offset + 2), 16));
      assert(Math.max(...channels) - Math.min(...channels) <= 2, color);
    }
  }
});

test('the koi pair keeps readable light and dark ends per theme', () => {
  assert(hexToHsl(koiPalette('#3183a6', true)[0]).l < 30);
  assert(hexToHsl(koiPalette('#3183a6', true)[1]).l > 65);
  assert(hexToHsl(koiPalette('#3183a6', false)[0]).l < 45);
  assert(hexToHsl(koiPalette('#3183a6', false)[1]).l > 60);
});
