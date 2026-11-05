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
  const control = async (step, input = {}) => { const response = await page.request.post(origin + '/__test__/control', { data: { step, input } }); if (!response.ok()) throw Error(step + ': ' + await response.text()); return response.json(); };
  const check = (condition, label) => { if (!condition) throw Error(label); checks.push(label); };
  const open = async title => {
    const search = page.getByRole('combobox', { name: 'Search workspace' });
    if (!await page.getByRole('button', { name: 'Search', exact: true }).isVisible()) await page.getByRole('button', { name: 'Expand sidebar', exact: true }).click();
    for (let attempt = 0; attempt < 12 && !await search.isVisible(); attempt++) { await page.getByRole('button', { name: 'Search', exact: true }).click(); await page.waitForTimeout(150); }
    if (title === 'Menu Nota') {
      const result = await page.locator('.context-search').evaluate(el => {
        const ctx = document.createElement('canvas').getContext('2d');
        const pixel = value => { ctx.clearRect(0, 0, 1, 1); ctx.fillStyle = value; ctx.fillRect(0, 0, 1, 1); return [...ctx.getImageData(0, 0, 1, 1).data]; };
        const bg = pixel(getComputedStyle(el).backgroundColor), ink = pixel(getComputedStyle(el.querySelector('.context-results-label')).color);
        const lum = rgb => rgb.map(v => v / 255).map(c => c <= .04045 ? c / 12.92 : ((c + .055) / 1.055) ** 2.4).reduce((sum, c, i) => sum + c * [.2126, .7152, .0722][i], 0);
        const ratio = value => { const rgb = bg.slice(0, 3).map(c => c * bg[3] / 255 + value * (1 - bg[3] / 255)); const a = lum(ink.slice(0, 3)), b = lum(rgb); return (Math.max(a, b) + .05) / (Math.min(a, b) + .05); };
        return { alpha: bg[3] / 255, worst: Math.min(ratio(0), ratio(204)), filter: getComputedStyle(el).backdropFilter };
      });
      check(result.worst >= 4.5 && (result.filter === 'none' || result.alpha < .9), `light search is translucent and legible against backdrop extremes: ${JSON.stringify(result)}`);
    }
    await search.fill(title);
    await page.waitForFunction(() => document.querySelector('#context-search-results')?.getAttribute('aria-busy') === 'false');
    await page.getByRole('option').filter({ has: page.locator('strong', { hasText: title }) }).first().click();
    await page.waitForFunction(value => [...document.querySelectorAll('.title-input')].some(el => el.value === value && !el.disabled && el.getClientRects().length), title);
    await page.waitForTimeout(250);
  };
  const pane = () => page.locator('.workspace-tab-panel.peek:not(.peek-hidden):visible').last();
  const menu = () => pane().getByRole('menu', { name: 'Page actions' });
  const trigger = () => pane().getByRole('button', { name: 'Page actions', exact: true });
  const visibleInsideViewport = async locator => locator.evaluate(el => { const r = el.getBoundingClientRect(); return r.left >= 0 && r.top >= 0 && r.right <= innerWidth + 1 && r.bottom <= innerHeight + 1; });
  try {
  await page.setViewportSize({ width: 1440, height: 900 });
  await control('configure', { theme: 'light', phase: 'main' }); const info = await control('init');
  await page.goto(origin); await page.evaluate(() => { localStorage.clear(); localStorage.setItem('noteka.peek-mode.v1', 'side'); localStorage.setItem('noteka.theme.v1', 'light'); }); await page.reload();
  await page.locator('#workspace-path').fill(info.path); await page.getByRole('button', { name: 'Create workspace', exact: true }).click();
  await page.locator('.workspace-tab-panel.active .home').waitFor();
  const doc = { schema_version: 2, blocks: [{ id: 'block', type: 'paragraph', content: [{ type: 'text', text: 'Contenido original' }] }] };
  const fixtures = [
    { kind: 'note', title: 'Menu Nota', document: doc, lock: true, export: true, attachment: true },
    { kind: 'task', title: 'Menu Tarea', payload: { state: 'todo' }, document: doc, export: true },
    { kind: 'project', title: 'Menu Proyecto', payload: { state: 'planned' } },
    { kind: 'tag', title: 'Menu Etiqueta', payload: { classification: 'topic' } },
    { kind: 'person', title: 'Menu Persona', payload: {} },
    { kind: 'note', title: 'Menu Objetivo', payload: { goal: {} }, document: doc, export: true, attachment: true },
    { kind: 'note', title: 'Menu Hito', payload: { milestone: {} }, document: doc, export: true, attachment: true },
  ];
  for (const fixture of fixtures) {
    const { lock, export: exportable, attachment, ...input } = fixture;
    fixture.entity = await control('entity_create', input);
  }
  for (const fixture of fixtures) {
    await open(fixture.title);
    check(await trigger().count() === 1 && await pane().locator('.page-tools:not(.page-tools-header)').count() === 0, `${fixture.title}: one header menu and no footer`);
    check(await pane().locator('.editor-bar > .actions > button').filter({ hasText: 'Archive' }).count() === 0, `${fixture.title}: no duplicate archive`);
    await trigger().focus(); await page.keyboard.press('Enter');
    await menu().waitFor();
    if (fixture.title === 'Menu Nota') {
      const filter = await menu().evaluate(el => getComputedStyle(el).backdropFilter);
      check(filter === 'none' || filter === 'blur(10px)', 'menu uses a bounded blur or solid fallback');
      await page.emulateMedia({ contrast: 'more' });
      check(await menu().evaluate(el => getComputedStyle(el).backdropFilter === 'none'), 'high contrast removes menu blur');
      await page.emulateMedia({ contrast: 'no-preference' });
    }
    check(await visibleInsideViewport(menu()), `${fixture.title}: menu fits viewport`);
    check(await menu().getByRole('menuitem', { name: 'External resources' }).isEnabled(), `${fixture.title}: resources available`);
    check(await menu().getByRole('menuitem', { name: 'Archive', exact: true }).isEnabled(), `${fixture.title}: archive available`);
    check(await menu().getByRole('menuitemcheckbox', { name: 'Lock page' }).count() === (fixture.lock ? 1 : 0), `${fixture.title}: lock capability preserved`);
    check(await menu().getByRole('menuitem', { name: 'Export', exact: true }).count() === (fixture.export ? 1 : 0), `${fixture.title}: export capability preserved`);
    check(await menu().getByRole('menuitem', { name: 'Attachments', exact: true }).count() === (fixture.attachment ? 1 : 0), `${fixture.title}: attachments capability preserved`);
    await page.waitForFunction(() => document.activeElement?.closest('.page-actions-menu'));
    await page.keyboard.press('End'); check(await menu().getByRole('menuitem', { name: 'Archive', exact: true }).evaluate(el => el === document.activeElement), `${fixture.title}: keyboard navigation`);
    await page.keyboard.press('Escape'); check(await trigger().evaluate(el => el === document.activeElement), `${fixture.title}: Escape returns focus`);
    await trigger().click(); await menu().getByRole('menuitem', { name: 'External resources' }).click();
    const panel = pane().locator('.page-tool-panel:not([hidden])'); await panel.waitFor(); check(await visibleInsideViewport(panel), `${fixture.title}: tool panel fits viewport`);
    await page.keyboard.press('Escape'); check(await trigger().evaluate(el => el === document.activeElement), `${fixture.title}: panel Escape restores focus`);
    check((await control('get', { id: fixture.entity.id })).revision === fixture.entity.revision, `${fixture.title}: menu interaction doesn't write`);
    await pane().getByRole('button', { name: 'Close peek' }).click();
  }
  await open('Menu Tarea'); await pane().locator('.title-input').fill('Tarea pendiente');
  await trigger().click(); check(await menu().getByRole('menuitem', { name: 'Export', exact: true }).isDisabled(), 'dirty task disables export');
  await menu().getByRole('menuitem', { name: 'Archive', exact: true }).click();
  const guard = page.getByRole('dialog', { name: 'You have unsaved changes' }); await guard.waitFor();
  await guard.getByRole('button', { name: 'Cancel', exact: true }).click();
  check(await pane().locator('.title-input').inputValue() === 'Tarea pendiente' && !(await control('get', { id: fixtures[1].entity.id })).archived_at, 'cancel archive keeps draft and entity');
  await trigger().click(); await menu().getByRole('menuitem', { name: 'Archive', exact: true }).click();
  await guard.getByRole('button', { name: 'Discard', exact: true }).click();
  await page.waitForFunction(() => !document.querySelector('.peek .title-input'));
  check(!!(await control('get', { id: fixtures[1].entity.id })).archived_at, 'archive applies after discard');
  await pane().getByRole('button', { name: 'Close peek' }).click();
  if (!await page.getByRole('button', { name: 'Search', exact: true }).isVisible()) await page.getByRole('button', { name: 'Expand sidebar', exact: true }).click();
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await page.getByRole('combobox', { name: 'Search workspace' }).fill('Archive');
  await page.waitForFunction(() => document.querySelector('#context-search-results')?.getAttribute('aria-busy') === 'false');
  await page.getByRole('option').filter({ has: page.locator('strong', { hasText: 'Archive' }) }).first().click();
  await page.locator('.entity-list').getByRole('button', { name: /Menu Tarea/ }).click();
  await page.waitForFunction(() => [...document.querySelectorAll('.title-input')].some(el => el.value === 'Menu Tarea' && !el.disabled && el.getClientRects().length));
  await page.waitForTimeout(250); await trigger().click();
  check(await menu().getByRole('menuitem', { name: 'Restore', exact: true }).isEnabled(), 'archived page offers Restore');
  await menu().getByRole('menuitem', { name: 'Restore', exact: true }).click();
  await page.waitForFunction(() => !document.querySelector('.peek .title-input'));
  check(!(await control('get', { id: fixtures[1].entity.id })).archived_at, 'restore updates archived entity');
  await pane().getByRole('button', { name: 'Close peek' }).click();
  if (!await page.getByRole('button', { name: 'Tasks', exact: true }).isVisible()) await page.getByRole('button', { name: 'Expand sidebar', exact: true }).click();
  await page.locator('.desktop-sidebar nav').getByRole('button', { name: 'Tasks', exact: true }).click();
  await page.getByRole('button', { name: 'Create in Tasks' }).click();
  await pane().locator('.title-input').waitFor(); await page.waitForTimeout(250); await trigger().click();
  check(await menu().getByRole('menuitem', { name: 'Export', exact: true }).isDisabled() && await menu().getByRole('menuitem', { name: 'External resources' }).isDisabled() && await menu().getByRole('menuitem', { name: 'Archive', exact: true }).count() === 0, 'new task exposes only applicable disabled tools');
  await page.keyboard.press('Escape'); await pane().getByRole('button', { name: 'Close peek' }).click();
  const newGuard = page.getByRole('dialog', { name: 'You have unsaved changes' }); await newGuard.getByRole('button', { name: 'Discard', exact: true }).click();
  await open('Menu Proyecto');
  for (const [name, klass] of [['Center peek', 'peek-center'], ['Full page', 'peek-full'], ['Side peek', 'peek-side']]) {
    await pane().getByRole('button', { name: 'Switch peek mode' }).click(); await page.getByRole('menuitemradio', { name }).click();
    await page.locator('.' + klass + ' .title-input').waitFor(); await trigger().click();
    check(await menu().getByRole('menuitem', { name: 'External resources' }).isEnabled() && await visibleInsideViewport(menu()), `${name}: menu works`);
    await page.keyboard.press('Escape');
  }
  await page.setViewportSize({ width: 420, height: 800 }); await trigger().click();
  check(await visibleInsideViewport(menu()), 'narrow viewport: menu fits');
  await menu().getByRole('menuitem', { name: 'External resources' }).click();
  check(await visibleInsideViewport(pane().locator('.page-tool-panel:not([hidden])')), 'narrow viewport: tool panel fits');
  await page.keyboard.press('Escape');
  check(errors.length === 0 && external.length === 0, `browser errors: ${JSON.stringify(errors)}; external requests: ${JSON.stringify(external)}`);
  await control('finish', { ok: true, checks });
  } catch (error) { throw Error(`After ${checks.length} checks (${checks.at(-1)}): ${error.stack || error}`); }
}
