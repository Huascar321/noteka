async (page) => {
  page.setDefaultTimeout(15000);
  const origin = new URL(page.url()).origin, checks = [], errors = [], external = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.context().route('**/*', async route => { const url = new URL(route.request().url()); if (url.origin === origin) await route.continue(); else { external.push(url.origin); await route.abort(); } });
  await page.addInitScript(() => {
    let next = 1; window.isTauri = true;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = { transformCallback() { return next++; }, unregisterCallback() {}, async invoke(command, args = {}) {
      if (command === 'operate') return (await fetch('/__test__/operate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args.request) })).json();
      if (['pending_edits', 'close_confirmed', 'plugin:event|listen', 'plugin:event|unlisten'].includes(command)) return next++;
      throw Error('Unexpected IPC: ' + command);
    } };
  });
  const control = async (step, input = {}) => { const response = await page.request.post(origin + '/__test__/control', { data: { step, input } }); if (!response.ok()) throw Error(`${step}: ${await response.text()}`); return response.json(); };
  const check = (value, label) => { if (!value) throw Error(label); checks.push(label); };
  const sidebar = page.locator('.desktop-sidebar');
  const side = page.locator('.peek-side');
  const splitter = page.getByRole('separator', { name: 'Resize side peek' });
  const search = async () => {
    await page.getByRole('button', { name: 'Search', exact: true }).click();
    await page.getByRole('combobox', { name: 'Search workspace' }).fill('Nota de umbral');
    await page.waitForFunction(() => document.querySelector('#context-search-results')?.getAttribute('aria-busy') === 'false');
    await page.getByRole('option').filter({ has: page.locator('strong', { hasText: 'Nota de umbral' }) }).first().click();
  };
  try {
    await page.setViewportSize({ width: 1440, height: 900 }); await page.emulateMedia({ reducedMotion: 'reduce' });
    await control('configure', { theme: 'light', phase: 'main' }); const info = await control('init');
    await page.goto(origin); await page.evaluate(() => { localStorage.clear(); localStorage.setItem('noteka.peek-mode.v1', 'side'); }); await page.reload();
    await page.locator('#workspace-path').fill(info.path); await page.getByRole('button', { name: 'Create workspace', exact: true }).click(); await page.locator('.home').waitFor();
    const note = await control('entity_create', { kind: 'note', title: 'Nota de umbral', document: { schema_version: 2, blocks: [{ id: 'initial', type: 'paragraph', content: [] }] } });
    await search(); await side.locator('.title-input').waitFor();
    check(await splitter.getAttribute('aria-valuenow') === '648', 'default is exactly 45 percent');
    check(await sidebar.getAttribute('aria-hidden') === 'false', 'sidebar remains open at exactly 45 percent');
    await side.locator('.title-input').fill('Borrador conservado');
    await side.locator('.title-input').evaluate(el => { window.__thresholdEditor = el; });
    await splitter.focus(); await page.keyboard.press('Alt+ArrowLeft');
    await page.waitForFunction(() => document.querySelector('.desktop-sidebar')?.getAttribute('aria-hidden') === 'true');
    check(await splitter.getAttribute('aria-valuenow') === '649', 'one extra pixel exceeds 45 percent');
    check(await side.locator('.title-input').inputValue() === 'Borrador conservado', 'closing sidebar preserves draft');
    await page.getByRole('button', { name: 'Expand sidebar', exact: true }).click();
    await page.waitForFunction(() => document.querySelector('.peek-side')?.classList.contains('peek-hidden'));
    check(await sidebar.getAttribute('aria-hidden') === 'false', 'reopening sidebar suppresses the wide peek');
    check(await side.locator('.title-input').evaluate(el => el === window.__thresholdEditor), 'hidden peek keeps editor instance');
    await search(); await side.locator('.title-input').waitFor();
    check(await sidebar.getAttribute('aria-hidden') === 'true' && await side.locator('.title-input').inputValue() === 'Borrador conservado', 'reopening same note restores draft and hides sidebar');
    await splitter.focus(); await page.keyboard.press('Alt+ArrowRight');
    check(await splitter.getAttribute('aria-valuenow') === '648', 'keyboard reduces side peek to exactly 45 percent');
    check(await sidebar.getAttribute('aria-hidden') === 'true', 'reducing width does not open sidebar automatically');
    await page.getByRole('button', { name: 'Expand sidebar', exact: true }).click();
    check(await sidebar.getAttribute('aria-hidden') === 'false' && !await side.evaluate(el => el.classList.contains('peek-hidden')), 'both panels coexist again at 45 percent');
    check(await side.locator('.title-input').inputValue() === 'Borrador conservado' && (await control('get', { id: note.id })).title === 'Nota de umbral', 'no implicit save or loss of draft');
    check(errors.length === 0 && external.length === 0, `browser errors ${JSON.stringify(errors)}; external requests ${JSON.stringify(external)}`);
    await control('finish', { ok: true, checks });
  } catch (error) { throw Error(`After ${checks.length} checks (${checks.at(-1)}): ${error.stack || error}`); }
}
