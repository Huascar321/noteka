async (page) => {
  const origin = new URL(page.url()).origin;
  const browser = page.context().browser();
  const directory = 'output/playwright/brand/' + browser.browserType().name();
  const checks = [], captures = [], errors = [], external = [];
  page.setDefaultTimeout(15000);
  page.on('pageerror', error => errors.push(String(error)));
  await page.route('**/*', route => {
    if (new URL(route.request().url()).origin === origin) return route.continue();
    external.push(route.request().url()); return route.abort();
  });
  await page.addInitScript(() => {
    let next = 1;
    window.isTauri = true;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback() { return next++; }, unregisterCallback() {},
      async invoke(command, args = {}) {
        if (command === 'operate') {
          const response = await fetch('/__test__/operate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args.request) });
          return response.json();
        }
        if (['pending_edits', 'close_confirmed', 'plugin:event|listen', 'plugin:event|unlisten'].includes(command)) return next++;
        throw Error('Unexpected IPC: ' + command);
      }
    };
  });
  const check = (value, message) => { if (!value) throw Error(message); checks.push(message); };
  const control = async (step, input = {}) => {
    const response = await page.request.post(origin + '/__test__/control', { data: { step, input } });
    if (!response.ok()) throw Error(await response.text());
    return response.json();
  };
  const capture = async (name, locator) => {
    await locator.screenshot({ path: directory + '/' + name, animations: 'disabled' }); captures.push(name);
  };
  for (const theme of ['light', 'dark']) {
    await control('configure', { theme, phase: 'main' });
    const info = await control('init');
    await page.setViewportSize({ width: 1180, height: 780 });
    await page.goto(origin);
    await page.evaluate(theme => { localStorage.clear(); localStorage.setItem('noteka.theme.v1', theme); }, theme);
    await page.reload();
    await page.locator('#workspace-path').waitFor();
    check(await page.locator('.welcome [data-noteka-mark]').count() === 1, theme + ': welcome uses the SVG mark');
    check(await page.locator('.welcome .noteka-wordmark').evaluate(node => getComputedStyle(node).fontWeight) === '450', theme + ': lighter wordmark');
    await capture(`welcome-${theme}.png`, page.locator('.welcome'));
    await page.locator('#workspace-path').fill(info.path);
    await page.getByRole('button', { name: 'Create workspace', exact: true }).click();
    await page.locator('.workspace-tab-panel.active .home').waitFor();
    const brand = page.locator('.workspace-brand');
    check(await brand.locator('[data-noteka-mark]').count() === 1 && (await brand.innerText()) === 'Noteka', theme + ': sidebar identity');
    check(await brand.locator('svg').getAttribute('aria-hidden') === 'true' && await brand.locator('svg').getAttribute('focusable') === 'false', theme + ': decorative mark does not add a keyboard stop');
    await capture(`sidebar-${theme}.png`, page.locator('.desktop-sidebar'));
    await capture(`header-${theme}.png`, brand);
    await page.locator('nav[aria-label="Sections"]').getByRole('button', { name: 'Notes', exact: true }).click();
    await page.locator('nav[aria-label="Sections"] button[aria-current="page"]').filter({ hasText: 'Notes' }).waitFor();
    check(true, theme + ': Notes navigation preserved');
    await page.getByRole('button', { name: 'Search', exact: true }).click();
    await page.getByRole('dialog').waitFor();
    await page.keyboard.press('Escape');
    check(await page.getByRole('dialog').count() === 0, theme + ': Search opens and closes');
    // Use the existing keyboard-accessible resize handle, preserving its behavior.
    const handle = page.getByRole('separator', { name: 'Resize sidebar', exact: true });
    await handle.focus();
    for (let i = 0; i < 8; i++) await page.keyboard.press('ArrowLeft');
    check(await brand.locator('.noteka-wordmark').evaluate(node => node.clientWidth >= node.scrollWidth), theme + ': compact sidebar keeps the name');
    await capture(`sidebar-compact-${theme}.png`, page.locator('.desktop-sidebar'));
    await page.emulateMedia({ contrast: 'more', reducedMotion: 'reduce' });
    const colors = await brand.locator('svg').evaluate(svg => ({ body: getComputedStyle(svg.querySelector('.noteka-mark-body')).fill, text: getComputedStyle(svg.parentElement).color }));
    check(colors.body === colors.text, theme + ': high contrast uses theme ink');
    await capture(`header-contrast-${theme}.png`, brand);
    await page.emulateMedia({ contrast: 'no-preference', reducedMotion: 'reduce' });
    await page.setViewportSize({ width: 420, height: 740 });
    await page.getByRole('button', { name: 'Expand sidebar', exact: true }).click();
    await brand.waitFor({ state: 'visible' });
    check(await brand.locator('.noteka-wordmark').isVisible(), theme + ': brand visible at 420px');
  }
  const favicon = await page.request.get(origin + '/noteka.svg');
  check(favicon.ok() && !(await favicon.text()).includes('<circle'), 'local favicon uses the new identity without the old dot');
  check(errors.length === 0 && external.length === 0, 'no page errors or external requests');
  const result = { ok: true, checks, captures, errors, external, browserVersion: browser.version() };
  await control('finish', result);
  return result;
}
