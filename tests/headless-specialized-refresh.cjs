async (page) => {
  page.setDefaultTimeout(12000);
  const origin = new URL(page.url()).origin, checks = [], errors = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.context().route('**/*', route => new URL(route.request().url()).origin === origin ? route.continue() : route.abort());
  await page.addInitScript(() => {
    let next = 1;
    window.isTauri = true;
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
  const control = async (step, input = {}) => {
    const response = await page.request.post(origin + '/__test__/control', { data: { step, input } });
    if (!response.ok()) throw Error(step + ': ' + await response.text());
    return response.json();
  };
  const check = (value, name) => { if (!value) throw Error(name); checks.push(name); };
  const button = name => page.getByRole('button', { name, exact: true });
  const burst = () => page.evaluate(() => { for (let i = 0; i < 20; i++) window.dispatchEvent(new Event('focus')); });
  let delayedNotes = 0, delayedAttachments = 0;
  await page.route('**/__test__/operate', async route => {
    const operation = route.request().postDataJSON().command?.operation;
    if (operation === 'search.query' || operation === 'extraction.search') {
      if (operation === 'search.query') delayedNotes++; else delayedAttachments++;
      await new Promise(resolve => setTimeout(resolve, 300));
    }
    await route.continue();
  });
  const waitCounts = async (notes, attachments) => {
    for (let i = 0; i < 100 && (delayedNotes < notes || delayedAttachments < attachments); i++) await new Promise(resolve => setTimeout(resolve, 50));
  };
  try {
    const fixture = await control('evidence_seed', { theme: 'dark' });
    await page.goto(origin);
    await page.getByLabel('Workspace path').fill(fixture.workspace);
    await button('Open workspace').click();
    await page.waitForFunction(() => document.querySelector('.home [data-home-section="person"]')?.getAttribute('aria-busy') === 'false');
    await button('Search').click(); await page.locator('.context-search-footer button').click();
    await page.getByLabel('Search scope').selectOption('combined');

    await page.getByLabel('Combined text').fill('nkprobe');
    await button('Search notes and attachments').click();
    await page.locator('.grouped-notes [role="status"]').waitFor();
    await burst(); await waitCounts(2, 2);
    await page.waitForFunction(() => !document.querySelector('.grouped-search [role="status"]'));
    check(delayedNotes === 2 && delayedAttachments === 2, 'focus during grouped search runs one trailing pass per group');

    delayedNotes = 0; delayedAttachments = 0;
    await button('Search notes and attachments').click();
    await page.locator('.grouped-notes [role="status"]').waitFor();
    await waitCounts(1, 1); await burst(); await page.getByLabel('Combined text').fill('new criteria');
    await new Promise(resolve => setTimeout(resolve, 500));
    check(delayedNotes === 1 && delayedAttachments === 1 && await page.locator('.grouped-note-hit,.grouped-attachment-hit').count() === 0, 'editing cancels pending grouped refresh');

    await page.getByLabel('Search scope').selectOption('attachments');
    await page.getByLabel('Attachment text').fill('nkprobe'); delayedAttachments = 0;
    await button('Search attachments').click();
    await page.getByText('Searching attachments…', { exact: true }).waitFor();
    await burst(); await waitCounts(0, 2);
    await page.waitForFunction(() => !document.querySelector('.attachment-search [role="status"]')?.textContent.includes('Searching'));
    check(delayedAttachments === 2, 'focus during attachment search runs one trailing pass');

    delayedAttachments = 0; await button('Search attachments').click();
    await page.getByText('Searching attachments…', { exact: true }).waitFor();
    await waitCounts(0, 1); await burst(); await page.getByLabel('Attachment text').fill('new criteria');
    await new Promise(resolve => setTimeout(resolve, 500));
    check(delayedAttachments === 1 && await page.locator('.attachment-hit').count() === 0, 'editing cancels pending attachment refresh');
    check(errors.length === 0, `browser errors: ${errors.join(', ')}`);
    await control('finish', { ok: true, checks });
  } catch (error) { throw Error(`After ${checks.length} checks (${checks.at(-1)}): ${error.stack || error}`); }
}
