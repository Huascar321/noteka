async (page) => {
  page.setDefaultTimeout(12000);
  const origin = new URL(page.url()).origin, errors = [], checks = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.addInitScript(() => {
    let next = 1; window.isTauri = true;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback() { return next++; }, unregisterCallback() {},
      async invoke(command, args = {}) {
        if (command === 'operate') return (await fetch('/__test__/operate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args.request) })).json();
        if (['pending_edits', 'close_confirmed', 'plugin:event|listen', 'plugin:event|unlisten'].includes(command)) return next++;
        throw Error('Unexpected IPC: ' + command);
      }
    };
  });
  const control = async (step, input = {}) => { const response = await page.request.post(origin + '/__test__/control', { data: { step, input } }); if (!response.ok()) throw Error(await response.text()); return response.json(); };
  const check = (value, message) => { if (!value) throw Error(message); checks.push(message); };
  await control('configure', { theme: 'dark', phase: 'main' }); const info = await control('init');
  await page.goto(origin); await page.evaluate(() => localStorage.clear()); await page.reload();
  await page.locator('#workspace-path').fill(info.path); await page.getByRole('button', { name: 'Create workspace', exact: true }).click();
  await page.locator('.workspace-tab-panel.active .home').waitFor();
  await page.evaluate(() => {
    const outer = document.createElement('div'); outer.id = 'wheel-test-outer'; outer.style.cssText = 'position:fixed;z-index:150;top:60px;left:5px;width:200px;height:200px;overflow:auto;background:var(--bg)';
    const inner = document.createElement('div'); inner.id = 'wheel-test-inner'; inner.style.cssText = 'height:120px;overflow:auto;line-height:20px';
    const contents = document.createElement('div'); contents.style.height = '800px'; contents.textContent = 'Synthetic scroll test'; inner.append(contents);
    const below = document.createElement('div'); below.style.height = '1000px'; outer.append(inner, below); document.querySelector('.app-window').append(outer);
    window.__wheelTest = { calls: [], inner, outer, fire(target, options = {}) {
      const e = new WheelEvent('wheel', { bubbles: true, cancelable: true, deltaY: 40, ...options });
      if (options.legacy !== undefined) Object.defineProperty(e, 'wheelDeltaY', { value: options.legacy });
      target.dispatchEvent(e); return e.defaultPrevented;
    }};
    for (const el of [inner, outer]) { const native = el.scrollTo.bind(el); el.scrollTo = options => { window.__wheelTest.calls.push({ id: el.id, top: options.top, behavior: options.behavior }); native(options); }; }
  });
  const run = fn => page.evaluate(fn);
  const first = await run(() => { const t = window.__wheelTest; return { prevented: t.fire(t.inner, { legacy: -120 }), calls: t.calls.slice() }; });
  check(first.prevented && first.calls.at(-1).id === 'wheel-test-inner' && first.calls.at(-1).behavior === 'smooth' && first.calls.at(-1).top === 40, 'discrete mouse step is smoothed in nearest scroller');
  const accumulated = await run(() => { const t = window.__wheelTest; t.fire(t.inner, { legacy: -120 }); return t.calls.at(-1); });
  check(accumulated.top === 80, 'rapid steps accumulate distance');
  const reverse = await run(() => { const t = window.__wheelTest; t.inner.scrollTop = 100; document.dispatchEvent(new Event('keydown', { bubbles: true })); t.inner.scrollTop = 100; t.fire(t.inner, { legacy: -120 }); t.fire(t.inner, { deltaY: -40, legacy: 120 }); return t.calls.slice(-2); });
  check(reverse.at(-1).behavior === 'smooth' && reverse.at(-1).top < 100, 'reversal cancels the old target');
  const excluded = await run(() => { const t = window.__wheelTest; const area = document.createElement('textarea'), select = document.createElement('select'), number = document.createElement('input'), range = document.createElement('input'); number.type = 'number'; range.type = 'range'; t.inner.append(area, select, number, range); const cases = [t.fire(t.inner, { deltaY: 4, legacy: -12 }), t.fire(t.inner, { deltaX: 12, legacy: -120 }), t.fire(t.inner, { ctrlKey: true, legacy: -120 }), t.fire(t.inner, { shiftKey: true, legacy: -120 }), t.fire(area, { legacy: -120 }), t.fire(select, { legacy: -120 }), t.fire(number, { legacy: -120 }), t.fire(range, { legacy: -120 }), t.fire(t.inner, { deltaMode: 2, legacy: -120 })]; area.remove(); select.remove(); number.remove(); range.remove(); return cases; });
  check(excluded.every(x => !x), 'precise, horizontal, zoom, modified, form controls and page input remain native');
  const lineMode = await run(() => { const t = window.__wheelTest; document.dispatchEvent(new Event('keydown')); t.inner.scrollTop = 100; const prevented = t.fire(t.inner, { deltaMode: 1, deltaY: 2 }); return { prevented, call: t.calls.at(-1) }; });
  check(lineMode.prevented && lineMode.call.top === 140 && lineMode.call.behavior === 'smooth', 'line-based wheel uses local line height');
  const boundary = await run(() => { const t = window.__wheelTest; document.dispatchEvent(new Event('keydown')); t.inner.scrollTop = t.inner.scrollHeight; return t.fire(t.inner, { legacy: -120 }); });
  check(!boundary, 'nested scroller edge keeps native propagation');
  await page.emulateMedia({ reducedMotion: 'reduce' }); await page.waitForFunction(() => matchMedia('(prefers-reduced-motion: reduce)').matches); await page.waitForTimeout(150);
  check(!await run(() => { const t = window.__wheelTest; t.inner.scrollTop = 0; return t.fire(t.inner, { legacy: -120 }); }), 'reduced motion keeps native scrolling');
  await page.emulateMedia({ reducedMotion: 'no-preference' }); await page.waitForFunction(() => !matchMedia('(prefers-reduced-motion: reduce)').matches); await page.waitForTimeout(150);
  const noPreference = await run(() => { const t = window.__wheelTest; t.inner.scrollTop = 0; return t.fire(t.inner, { legacy: -120 }); });
  check(noPreference, 'mouse smoothing resumes when reduced motion is removed');
  await run(() => { document.getElementById('wheel-test-outer').style.display = 'none'; });
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  const toggle = page.getByRole('checkbox', { name: 'Smooth mouse scrolling' });
  check(await toggle.isChecked(), 'preference is enabled initially');
  await page.getByLabel('Search settings').fill('scroll');
  check(await toggle.isVisible() && await page.getByRole('heading', { name: 'Appearance', exact: true }).count() === 0, 'settings search finds only matching group');
  await toggle.uncheck();
  check(await page.evaluate(() => localStorage.getItem('noteka.smooth-wheel.v1')) === 'off', 'disabled preference is stored');
  await page.getByRole('button', { name: 'Close Settings' }).click();
  await run(() => { const t = window.__wheelTest; t.outer.style.display = ''; t.inner.scrollTop = 0; });
  check(!await run(() => window.__wheelTest.fire(window.__wheelTest.inner, { legacy: -120 })), 'disabled preference restores native scrolling');
  await run(() => { const t = window.__wheelTest; t.inner.scrollTop = t.inner.scrollHeight; t.outer.scrollTop = 0; });
  await page.mouse.move(100, 100); await page.mouse.wheel(0, 120); await page.waitForTimeout(500);
  const nativeBoundary = await run(() => window.__wheelTest.outer.scrollTop);
  const reopened = await page.context().newPage(); await reopened.goto(origin);
  check(await reopened.evaluate(() => localStorage.getItem('noteka.smooth-wheel.v1')) === 'off', 'disabled preference persists in a new app session');
  const restartedNative = await reopened.evaluate(async () => { await new Promise(resolve => setTimeout(resolve, 100)); const box = document.createElement('div'); box.style.cssText = 'height:100px;overflow:auto'; const child = document.createElement('div'); child.style.height = '500px'; box.append(child); document.querySelector('.app-window').append(box); const event = new WheelEvent('wheel', { bubbles: true, cancelable: true, deltaY: 40 }); Object.defineProperty(event, 'wheelDeltaY', { value: -120 }); box.dispatchEvent(event); box.remove(); return !event.defaultPrevented; });
  check(restartedNative, 'disabled preference restores native behavior after app restart');
  await reopened.close();
  await run(() => { document.getElementById('wheel-test-outer').style.display = 'none'; });
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  check(!await page.getByRole('checkbox', { name: 'Smooth mouse scrolling' }).isChecked(), 'preference remains disabled when reopened');
  await page.evaluate(() => { Storage.prototype.setItem = () => { throw Error('quota'); }; });
  await page.getByRole('checkbox', { name: 'Smooth mouse scrolling' }).check();
  check(await page.getByText('Scrolling preference applied. It could not be saved on this device.').isVisible(), 'storage error is visible and session choice remains active');
  await page.getByRole('button', { name: 'Close Settings' }).click();
  await page.getByRole('dialog', { name: 'Settings', exact: true }).waitFor({ state: 'detached' });
  await run(() => { const t = window.__wheelTest; t.outer.style.display = ''; t.inner.scrollTop = 0; });
  check(await run(() => window.__wheelTest.fire(window.__wheelTest.inner, { legacy: -120 })), 'session preference still works after failed save');
  await run(() => { const t = window.__wheelTest; document.dispatchEvent(new Event('keydown')); t.outer.scrollTop = 0; t.inner.scrollTop = 0; t.calls = []; });
  await page.mouse.move(100, 100);
  // The first event can be non-cancelable while the global classifier is passive.
  // It identifies the device/scroller; the following mouse step is cancelable locally.
  await page.mouse.wheel(0, 120); await page.waitForTimeout(250);
  await run(() => { const t = window.__wheelTest; t.inner.scrollTop = 0; t.calls = []; });
  await page.mouse.wheel(0, 120); await page.waitForTimeout(40);
  const during = await run(() => window.__wheelTest.inner.scrollTop);
  await page.waitForTimeout(500); const after = await run(() => window.__wheelTest.inner.scrollTop);
  const wheelCall = await run(() => window.__wheelTest.calls.find(call => call.behavior === 'smooth'));
  check(wheelCall?.id === 'wheel-test-inner' && after > during && after > 0, `browser mouse wheel animates the target over time (${during} → ${after})`);
  await run(() => { const t = window.__wheelTest; document.dispatchEvent(new Event('keydown')); t.inner.scrollTop = t.inner.scrollHeight; t.outer.scrollTop = 0; });
  await page.mouse.wheel(0, 120); await page.waitForTimeout(500);
  const smoothBoundary = await run(() => window.__wheelTest.outer.scrollTop);
  check(smoothBoundary >= nativeBoundary - 1, `nested-edge wheel preserves native behavior (${nativeBoundary} → ${smoothBoundary})`);
  check(errors.length === 0, 'no browser errors: ' + errors.join('; '));
  await control('finish', { ok: true, checks });
}
