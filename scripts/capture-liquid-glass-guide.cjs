async (page) => {
  page.setDefaultTimeout(15000);
  const origin = new URL(page.url()).origin;
  const browser = page.context().browser().browserType().name();
  const directory = 'output/playwright/liquid-glass-guide/' + browser;
  const captures = [], errors = [], external = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.context().route('**/*', async route => {
    if (new URL(route.request().url()).origin === origin) await route.continue();
    else { external.push(route.request().url()); await route.abort(); }
  });
  await page.addInitScript(() => {
    let next = 1;
    window.isTauri = true;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback() { return next++; }, unregisterCallback() {},
      async invoke(command, args = {}) {
        if (command === 'operate') return (await fetch('/__test__/operate', {
          method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args.request),
        })).json();
        if (['pending_edits', 'close_confirmed', 'plugin:event|listen', 'plugin:event|unlisten'].includes(command)) return next++;
        throw Error('IPC inesperado: ' + command);
      },
    };
  });
  const control = async (step, input = {}) => {
    const response = await page.request.post(origin + '/__test__/control', { data: { step, input } });
    if (!response.ok()) throw Error(await response.text());
    return response.json();
  };
  const assert = (condition, message) => { if (!condition) throw Error(message); };
  await control('configure', { theme: 'light', phase: 'main' });
  const info = await control('init');
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(origin);
  await page.evaluate(() => { localStorage.clear(); localStorage.setItem('noteka.peek-mode.v1', 'full'); });
  await page.reload();
  await page.locator('#workspace-path').fill(info.path);
  await page.getByRole('button', { name: 'Create workspace', exact: true }).click();
  await page.locator('.home').waitFor();
  const attachment = await control('import_attachment', { source_path: info.image });
  const person = await control('entity_create', { kind: 'person', title: 'Ana Torres', payload: {
    company: 'Studio Norte', relationships: ['Friend', 'Colleague'], photo: attachment.id, pipeline_status: 'contacted',
  } });
  await control('entity_create', { kind: 'person', title: 'Alex Rivera', payload: { company: 'Studio Sur', relationships: ['Colleague'], pipeline_status: 'prospect' } });
  const project = await control('entity_create', { kind: 'project', title: 'Design system', payload: { state: 'doing' } });
  const tag = await control('entity_create', { kind: 'tag', title: 'Design', payload: { classification: 'topic' } });
  const task = await control('entity_create', { kind: 'task', title: 'Review visual guidelines', payload: { state: 'todo' } });
  await control('relation_add', { source_id: task.id, target_id: project.id, kind: 'project', expected_revision: task.revision });
  let note = await control('entity_create', { kind: 'note', title: 'Product brief', payload: { note_type: 'reference', favorite: false }, document: {
    schema_version: 2, blocks: [
      { id: 'intro', type: 'paragraph', content: [{ type: 'text', text: 'Keep the content clear. Use glass for navigation and floating controls.' }] },
      { id: 'sample', type: 'code_block', attrs: { language: 'css' }, content: [{ type: 'text', text: '/* Shared material */\nbackground: var(--glass-surface);\nborder-radius: 14px;' }] },
    ],
  } });
  for (const [target, kind] of [[project, 'project'], [person, 'person'], [tag, 'tag']]) {
    await control('relation_add', { source_id: note.id, target_id: target.id, kind, expected_revision: note.revision });
    note = await control('get', { id: note.id });
  }
  const nav = page.getByRole('navigation', { name: 'Sections', exact: true });
  const pane = () => page.locator('.workspace-tab-panel.peek:not(.peek-hidden):visible').last();
  const settings = async (theme, color) => {
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await page.getByLabel('Theme', { exact: true }).selectOption(theme);
    await page.getByRole('button', { name: color, exact: true }).click();
    await page.getByRole('button', { name: 'Close Settings', exact: true }).click();
  };
  const search = async () => {
    if (!await page.locator('.desktop-sidebar .sidebar-search').isVisible()) await page.getByRole('button', { name: 'Expand sidebar', exact: true }).click();
    await page.locator('.desktop-sidebar .sidebar-search').click();
    await page.getByRole('combobox', { name: 'Search workspace' }).waitFor();
    await page.waitForFunction(() => document.querySelector('#context-search-results')?.getAttribute('aria-busy') === 'false');
  };
  const openNote = async () => {
    await search();
    await page.getByRole('combobox', { name: 'Search workspace' }).fill(note.title);
    await page.getByRole('option').filter({ has: page.locator('strong', { hasText: note.title }) }).first().click();
    await pane().locator('.title-input').waitFor();
    await pane().getByRole('button', { name: 'Remove relation to Design system', exact: true }).waitFor();
  };
  const capture = async (name, locator, theme, state, accent = 'Blue') => {
    await locator.waitFor({ state: 'visible' });
    await locator.evaluate(el => Promise.all(el.getAnimations().map(animation => animation.finished.catch(() => {}))));
    const file = `${name}-${theme}.png`;
    await locator.screenshot({ path: directory + '/' + file, animations: 'disabled' });
    const size = await locator.boundingBox();
    captures.push({ file, theme, accent, state, viewport: page.viewportSize(), cssSize: { width: size.width, height: size.height } });
  };
  for (const theme of ['light', 'dark']) {
    await settings(theme, 'Blue');
    await nav.getByRole('button', { name: 'Home', exact: true }).click();
    await page.locator('.home').waitFor();
    await page.locator('.home-tasks').first().getByRole('button', { name: 'Refresh Tasks', exact: true }).click();
    await page.locator('.home-heading h1').click();
    await page.mouse.move(800, 500);
    await capture('overview', page.locator('.app-window'), theme, 'Home, sidebar, tabs and breadcrumb');
    await capture('sidebar', page.locator('.desktop-sidebar'), theme, 'Selected Home and compact Search');
    await page.getByRole('group', { name: 'Home cover', exact: true }).getByRole('button', { name: 'ASCII', exact: true }).focus();
    await capture('cover-selector', page.locator('.home-cover-controls'), theme, 'ASCII selected, keyboard focus');
    const toolbar = page.locator('.home-tasks .task-view-toolbar').first();
    await toolbar.scrollIntoViewIfNeeded();
    await capture('toolbar', toolbar, theme, 'Sliding view indicator, grouped tools and New');
    await nav.getByRole('button', { name: 'People', exact: true }).click();
    const section = page.locator('.workspace-tab-panel.active .home-people');
    await section.getByRole('button', { name: 'Refresh People', exact: true }).click();
    const card = section.locator(`[data-person-id="${person.id}"]`);
    await card.locator('img').evaluate(async image => { await image.decode(); });
    await card.getByRole('button', { name: 'Open Ana Torres', exact: true }).focus();
    await capture('people', section, theme, 'Synthetic contacts, solid cards with glass frame and keyboard focus');
    await openNote();
    await capture('note', pane(), theme, 'Full note pane: compact Tag/Type, relation capsules, header and solid content');
    const actions = pane().getByRole('group', { name: 'Note actions', exact: true });
    await actions.getByRole('button', { name: 'Page actions', exact: true }).click();
    const menu = pane().getByRole('menu', { name: 'Page actions', exact: true });
    await capture('page-menu', menu, theme, 'Open local glass menu');
    await menu.getByRole('menuitemcheckbox', { name: 'Lock page', exact: true }).click();
    await actions.getByRole('button', { name: 'Unlock page temporarily' }).waitFor();
    assert(await pane().getByLabel('Note type', { exact: true }).isDisabled(), 'Type must be disabled while locked');
    await capture('note-locked', pane().locator('.note-page-properties'), theme, 'Locked fields retain frame and height; Tag destination remains navigable');
    await actions.getByRole('button', { name: 'Page actions', exact: true }).click();
    await menu.getByRole('menuitemcheckbox', { name: 'Lock page', exact: true }).click();
    await page.waitForFunction(() => !document.querySelector('.peek .page-lock-status'));
    await pane().getByRole('button', { name: 'Add person', exact: true }).click();
    const picker = page.getByRole('dialog', { name: 'Edit people', exact: true });
    await picker.getByLabel('Search people').fill('Ana');
    await picker.getByRole('button', { name: /Ana Torres/ }).waitFor();
    await capture('relation-picker', picker, theme, 'Mounted relation picker uses glass-menu');
    await page.keyboard.press('Escape');
    await pane().locator('.title-input').fill('Product brief — draft');
    await actions.getByRole('button', { name: /^Save / }).waitFor();
    await capture('note-header-dirty', pane().locator('.editor-bar'), theme, 'Unsaved changes, tinted Save and white shortcut');
    await pane().getByRole('button', { name: 'Close peek', exact: true }).click();
    const guard = page.getByRole('dialog', { name: 'You have unsaved changes', exact: true });
    await capture('save-guard', guard, theme, 'Default Cancel focus, neutral secondary capsules and tinted confirmation');
    await guard.getByRole('button', { name: 'Discard', exact: true }).click();
    await nav.getByRole('button', { name: 'Home', exact: true }).click();
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await capture('settings', page.getByRole('dialog', { name: 'Settings', exact: true }), theme, 'Preferences, shared popup material and compact Search');
    await page.keyboard.press('Escape');
    await search();
    assert(await page.getByRole('combobox', { name: 'Search workspace' }).getAttribute('placeholder') === 'Search', 'Search placeholder must match sidebar');
    await capture('search', page.getByRole('dialog', { name: 'Search workspace', exact: true }), theme, 'Shared popup material over actual Home content');
    await page.keyboard.press('Escape');
  }
  await settings('dark', 'Violet');
  await openNote();
  await pane().locator('.title-input').fill('Product brief — draft');
  await capture('violet-action', pane().locator('.editor-bar'), 'dark', 'Alternate accent; header remains neutral', 'Violet');
  await page.setViewportSize({ width: 420, height: 740 });
  await pane().getByRole('button', { name: 'Close peek', exact: true }).click();
  const guard = page.getByRole('dialog', { name: 'You have unsaved changes', exact: true });
  assert(await guard.getByRole('button', { name: 'Save and continue', exact: true }).isVisible(), 'Narrow confirmation must remain reachable');
  await capture('save-guard-narrow', guard, 'dark', '420px, secondary row and full-width confirmation', 'Violet');
  await guard.getByRole('button', { name: 'Discard', exact: true }).click();
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.emulateMedia({ contrast: 'more', reducedMotion: 'reduce' });
  await search();
  const fallback = page.getByRole('dialog', { name: 'Search workspace', exact: true });
  assert(await fallback.evaluate(el => getComputedStyle(el).backdropFilter === 'none'), 'Solid fallback must remove blur');
  await capture('search-solid-fallback', fallback, 'dark', 'prefers-contrast: more; solid surface, no sheen/filter', 'Violet');
  assert(errors.length === 0 && external.length === 0, 'No page errors or external requests allowed');
  await control('finish', { browser, browserVersion: page.context().browser().version(), captures, errors, externalRequests: external.length });
  return { browser, screenshots: captures.length, errors: errors.length, externalRequests: external.length };
}
