async (page) => {
  const origin = new URL(page.url()).origin;
  const errors = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.addInitScript(() => {
    let next = 1; const callbacks = new Map(); window.isTauri = true;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback(callback) { const id = next++; callbacks.set(id, callback); return id; },
      unregisterCallback(id) { callbacks.delete(id); },
      async invoke(command, args = {}) {
        if (command === 'operate') return (await fetch('/__test__/operate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args.request) })).json();
        if (command === 'pending_edits' || command === 'close_confirmed' || command === 'plugin:event|unlisten') return;
        if (command === 'plugin:event|listen') return next++;
        throw Error('Unexpected IPC: ' + command);
      }
    };
  });
  const control = async (step, input = {}) => {
    const response = await page.request.post(origin + '/__test__/control', { data: { step, input } });
    if (!response.ok()) throw Error(step + ': ' + await response.text());
    return response.json();
  };
  const expect = (condition, message) => { if (!condition) throw Error(message); };
  const themeIs = async value => page.waitForFunction(expected => document.documentElement.dataset.theme === expected, value);
  const checkChrome = async theme => {
    await page.waitForTimeout(250); // Let the existing color transition finish before measuring contrast.
    const result = await page.evaluate(() => {
      const sidebar = document.querySelector('.desktop-sidebar'), bar = document.querySelector('.window-titlebar');
      const active = sidebar.querySelector('nav button.active');
      const context = document.createElement('canvas').getContext('2d');
      const color = value => { context.clearRect(0, 0, 1, 1); context.fillStyle = value; context.fillRect(0, 0, 1, 1); return [...context.getImageData(0, 0, 1, 1).data].slice(0, 3); };
      const lum = value => color(value).map(c => c / 255).map(c => c <= .04045 ? c / 12.92 : ((c + .055) / 1.055) ** 2.4).reduce((sum, c, i) => sum + c * [.2126, .7152, .0722][i], 0);
      const indicator = sidebar.querySelector('.sidebar-nav-indicator');
      const activeStyle = getComputedStyle(active), surface = indicator ? getComputedStyle(indicator).backgroundColor : activeStyle.backgroundColor, a = lum(activeStyle.color), b = lum(surface);
      return { sidebar: getComputedStyle(sidebar).backgroundImage, bar: getComputedStyle(bar).backgroundImage, sidebarBlur: getComputedStyle(sidebar).backdropFilter, barBlur: getComputedStyle(bar).backdropFilter, foreground: activeStyle.color, background: surface, contrast: (Math.max(a, b) + .05) / (Math.min(a, b) + .05) };
    });
    expect(result.sidebar.includes('gradient') && result.bar.includes('gradient') && result.sidebarBlur === 'none' && result.barBlur === 'none' && result.contrast >= 4.5, `${theme} chrome appearance, contrast and no backdrop filter: ${JSON.stringify(result)}`);
  };
  await control('configure', { theme: 'dark', phase: 'main' });
  const info = await control('init');
  await page.emulateMedia({ colorScheme: 'light' });
  await page.goto(origin);
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await themeIs('dark');
  await page.locator('#workspace-path').fill(info.path);
  await page.getByRole('button', { name: 'Create workspace', exact: true }).click();
  await page.locator('.workspace-tab-panel.active .home').waitFor();
  await checkChrome('dark');
  await page.emulateMedia({ contrast: 'more' });
  expect(await page.locator('.desktop-sidebar').evaluate(el => getComputedStyle(el).backgroundImage === 'none'), 'increased contrast uses a solid sidebar');
  await page.emulateMedia({ contrast: 'no-preference' });
  await control('entity_create', { kind: 'note', title: 'Theme draft', document: { schema_version: 2, blocks: [{ id: 'draft-block', type: 'paragraph', content: [{ type: 'text', text: 'Original' }] }] } });
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await page.getByRole('combobox', { name: 'Search workspace' }).fill('Theme draft');
  await page.waitForFunction(() => document.querySelector('#context-search-results')?.getAttribute('aria-busy') === 'false');
  await page.getByRole('option').filter({ has: page.locator('strong', { hasText: 'Theme draft' }) }).first().click();
  const title = page.locator('.title-input:visible').first();
  await title.waitFor();
  await title.fill('Unsaved theme draft');
  if (!await page.getByRole('button', { name: 'Settings', exact: true }).isVisible()) await page.locator('.sidebar-toggle').click();
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  const surfaces = () => page.evaluate(() => ({ sidebar: getComputedStyle(document.querySelector('.desktop-sidebar')).backgroundImage, home: getComputedStyle(document.querySelector('.home')).backgroundImage, background: getComputedStyle(document.documentElement).backgroundColor }));
  const blue = await surfaces();
  await page.getByRole('button', { name: 'Violet', exact: true }).click();
  await page.waitForFunction(() => document.documentElement.style.getPropertyValue('--primary-color') === '#8254d4');
  const violet = await surfaces();
  await page.getByRole('button', { name: 'Green', exact: true }).click();
  await page.waitForFunction(() => document.documentElement.style.getPropertyValue('--primary-color') === '#298560');
  const green = await surfaces();
  expect(['sidebar', 'home', 'background'].every(key => blue[key] === violet[key] && violet[key] === green[key]), `glass backgrounds stay neutral across primary colors: ${JSON.stringify({ blue, violet, green })}`);
  await checkChrome('dark green');
  await page.getByRole('button', { name: 'Blue', exact: true }).click();
  const select = page.getByLabel('Theme', { exact: true });
  expect(JSON.stringify(await select.locator('option').allTextContents()) === JSON.stringify(['System', 'Dark', 'Light']), 'option order');
  await select.selectOption('system');
  await themeIs('light');
  await checkChrome('light');
  const lightBlue = (await surfaces()).sidebar;
  await page.getByRole('button', { name: 'Violet', exact: true }).click();
  await page.waitForFunction(() => document.documentElement.style.getPropertyValue('--primary-color') === '#8254d4');
  expect(lightBlue === (await surfaces()).sidebar, 'light sidebar stays neutral across primary colors');
  await checkChrome('light violet');
  await page.getByRole('button', { name: 'Blue', exact: true }).click();
  const lightAccent = await page.evaluate(() => document.documentElement.style.getPropertyValue('--section-bg'));
  expect(await page.evaluate(() => localStorage.getItem('noteka.theme.v1')) === 'system', 'system saved');
  await page.emulateMedia({ colorScheme: 'dark' });
  await themeIs('dark');
  await checkChrome('dark system');
  expect(await page.locator('.title-input').first().inputValue() === 'Unsaved theme draft', 'system change preserves draft');
  const darkAccent = await page.evaluate(() => document.documentElement.style.getPropertyValue('--section-bg'));
  expect(lightAccent !== darkAccent, 'accent updates with effective theme');
  await select.selectOption('light');
  await themeIs('light');
  await page.emulateMedia({ colorScheme: 'light' });
  await page.emulateMedia({ colorScheme: 'dark' });
  await themeIs('light');
  await select.selectOption('system');
  await themeIs('dark');
  await page.evaluate(() => { Storage.prototype.setItem = () => { throw new Error('quota'); }; });
  await select.selectOption('light');
  await themeIs('light');
  expect(await page.getByText('Theme applied. Your preference could not be saved on this device.').isVisible(), 'storage error visible');
  const reopened = await page.context().newPage();
  await reopened.emulateMedia({ colorScheme: 'dark' });
  await reopened.goto(origin);
  await reopened.waitForFunction(() => document.documentElement.dataset.theme === 'dark');
  expect(await reopened.evaluate(() => localStorage.getItem('noteka.theme.v1')) === 'system', 'system restored');
  await reopened.close();
  expect(errors.length === 0, 'no page errors: ' + errors.join('; '));
  return 'System theme: option order, default, live light/dark, draft, accent, fixed theme, storage error and reload passed';
}
