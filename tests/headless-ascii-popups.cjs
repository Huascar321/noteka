async (page) => {
  page.setDefaultTimeout(15000);
  const origin = new URL(page.url()).origin, checks = [], errors = [], external = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.context().route('**/*', async route => { const url = new URL(route.request().url()); if (url.origin === origin) await route.continue(); else { external.push(url.origin); await route.abort(); } });
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
  const check = (condition, label) => { if (!condition) throw Error(label); checks.push(label); };
  const canvas = page.locator('.workspace-tab-panel.active canvas.ascii-shark');
  const frame = () => canvas.getAttribute('data-frame');
  const advances = async () => { const before = await frame(); await page.waitForFunction(value => { const element = document.querySelector('.workspace-tab-panel.active canvas.ascii-shark'); return !!element && element.dataset.frame !== value; }, before); };
  const stable = async () => { await page.waitForTimeout(220); const before = await frame(); await page.waitForTimeout(300); return (await frame()) === before; };
  const switchMode = async (name) => {
    await page.locator('.peek').getByRole('button', { name: 'Switch peek mode' }).click();
    await page.getByRole('menuitemradio', { name }).click();
  };
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await control('configure', { theme: 'dark', phase: 'main' }); const info = await control('init');
  await page.goto(origin); await page.evaluate(() => { localStorage.clear(); localStorage.setItem('noteka.peek-mode.v1', 'side'); localStorage.setItem('noteka.home-cover.v1', 'ascii'); }); await page.reload();
  await page.locator('#workspace-path').fill(info.path); await page.getByRole('button', { name: 'Create workspace', exact: true }).click();
  await page.locator('.workspace-tab-panel.active .home').waitFor();
  await control('entity_create', { kind: 'note', title: 'Popup animation synthetic note', document: { schema_version: 2, blocks: [{ id: 'body', type: 'paragraph', content: [{ type: 'text', text: 'Synthetic body' }] }] } });
  await page.waitForFunction(() => Number(document.querySelector('.workspace-tab-panel.active canvas.ascii-shark')?.dataset.frame) > 2);

  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.locator('#settings-dialog').waitFor();
  await advances(); check(true, 'ASCII advances while Settings is open over Home');
  await page.getByRole('button', { name: 'Close Settings', exact: true }).click();
  await page.locator('#settings-dialog').waitFor({ state: 'detached' });
  await advances(); check(true, 'ASCII continues after Settings closes');

  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await page.getByRole('combobox', { name: 'Search workspace' }).waitFor();
  await advances(); check(true, 'ASCII advances while Search is open over Home');
  await page.getByRole('button', { name: 'Close search' }).click();
  await page.locator('.context-search').waitFor({ state: 'detached' });
  await advances(); check(true, 'ASCII continues after Search closes');

  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await page.getByRole('combobox', { name: 'Search workspace' }).fill('Popup animation synthetic note');
  await page.waitForFunction(() => document.querySelector('#context-search-results')?.getAttribute('aria-busy') === 'false');
  await page.getByRole('option').filter({ has: page.locator('strong', { hasText: 'Popup animation synthetic note' }) }).first().click();
  await page.locator('.peek-side .title-input').waitFor(); await page.waitForTimeout(400);
  await advances(); check(true, 'ASCII advances while Side peek is focused');
  await switchMode('Center peek'); await page.locator('.peek-center .title-input').waitFor();
  await advances(); check(true, 'ASCII advances behind Center peek while Home remains visible');
  await switchMode('Full page'); await page.locator('.peek-full .title-input').waitFor();
  check(await stable(), 'Full page keeps the cover suspended while it completely covers Home');
  await switchMode('Center peek'); await page.locator('.peek-center .title-input').waitFor();
  await advances(); check(true, 'Center peek resumes the cover after Full page');

  check(await page.locator('.workspace-tab-panel.active canvas.ascii-shark').evaluate(element => element.isConnected), 'the same Home cover stays mounted through every popup');
  check(errors.length === 0 && external.length === 0, `browser errors: ${JSON.stringify(errors)}; external requests: ${JSON.stringify(external)}`);
  await control('finish', { ok: true, checks });
}
