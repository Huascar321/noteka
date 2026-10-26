import test from 'node:test';
import assert from 'node:assert/strict';
import { PANEL_WIDTHS_KEY, panelLimits, clampPanelWidth, effectivePanelWidth, sidePeekExcludesSidebar, readPanelWidths, savePanelWidths } from '../src/lib/panel-widths.ts';

test('caps apply at every viewport, including narrower than the preferred minimum', () => {
  for (const [viewport, sidebarMax, peekMax] of [[0,0,0],[320,112,208],[420,147,273],[760,266,494],[1280,448,832],[1440,504,936],[2560,896,1664]]) for (const kind of ['sidebar','sidePeek']) {
    const limits = panelLimits(kind, viewport), width = clampPanelWidth(kind, 100000, viewport);
    assert.ok(limits.min <= limits.max); assert.equal(width, limits.max);
    assert.equal(width, kind === 'sidebar' ? sidebarMax : peekMax);
    assert.equal(clampPanelWidth(kind, -100, viewport), limits.min);
  }
});
test('exactly 35 percent may coexist; the next pixel requires exclusion', () => {
  assert.equal(sidePeekExcludesSidebar(504, 1440), false);
  assert.equal(sidePeekExcludesSidebar(505, 1440), true);
  assert.equal(sidePeekExcludesSidebar(147, 420), false);
  assert.equal(sidePeekExcludesSidebar(148, 420), true);
});
test('window clamp does not replace the preferred width', () => {
  const preferred = { sidebar: 480, sidePeek: 900 };
  assert.equal(effectivePanelWidth('sidebar', preferred, 420), 147);
  assert.equal(effectivePanelWidth('sidePeek', preferred, 420), 273);
  assert.equal(effectivePanelWidth('sidebar', preferred, 1440), 480);
  assert.equal(effectivePanelWidth('sidePeek', preferred, 1440), 900);
  assert.deepEqual(preferred, { sidebar: 480, sidePeek: 900 });
});
test('defaults and invalid stored fields recover independently', () => {
  const from = value => readPanelWidths(() => ({ getItem: () => value }));
  for (const value of [null, 'broken', '[]', 'true', 'null']) assert.deepEqual(from(value), { sidebar: 250, sidePeek: null });
  assert.deepEqual(from('{"sidebar":-1,"sidePeek":520}'), { sidebar: 250, sidePeek: 520 });
  assert.deepEqual(from('{"sidebar":300,"sidePeek":"600"}'), { sidebar: 300, sidePeek: null });
  assert.equal(effectivePanelWidth('sidePeek', from(null), 1440), 648);
  assert.deepEqual(readPanelWidths(() => { throw Error('denied'); }), { sidebar: 250, sidePeek: null });
});
test('roundtrip is global presentation data with recoverable write failure', () => {
  let raw;
  const store = { getItem: key => { assert.equal(key, PANEL_WIDTHS_KEY); return raw; }, setItem: (key,value) => { assert.equal(key, PANEL_WIDTHS_KEY); raw = value; } };
  const preferred = { sidebar: 340, sidePeek: 720 };
  assert.equal(savePanelWidths(preferred, () => store), '');
  assert.deepEqual(readPanelWidths(() => store), preferred);
  assert.match(savePanelWidths(preferred, () => { throw Error('denied'); }), /could not be saved/);
  assert.deepEqual(preferred, { sidebar: 340, sidePeek: 720 });
});
