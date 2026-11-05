async (page) => {
  page.setDefaultTimeout(20000);
  const origin = new URL(page.url()).origin, checks = [], errors = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.context().route('**/*', route => new URL(route.request().url()).origin === origin ? route.continue() : route.abort());
  await page.addInitScript(() => {
    let next = 1;
    window.isTauri = true;
    window.__focusReads = { active: 0, maximum: 0, targetId: null, entityReads: 0, holdUpdate: false, updateHeld: false };
    const reads = new Set(['home.query','entity.get','entity.list','relation.list','workflow.query','search.query','collection.get','collection.list','database.query','template.list','extraction.search']);
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback() { return next++; }, unregisterCallback() {},
      async invoke(command, args = {}) {
        if (command === 'operate') {
          const op = args.request.command;
          const reading = reads.has(op.operation);
          if (op.operation === 'entity.get' && op.input.id === window.__focusReads.targetId) window.__focusReads.entityReads++;
          if (op.operation === 'entity.update' && op.input.id === window.__focusReads.targetId && window.__focusReads.holdUpdate) {
            window.__focusReads.holdUpdate = false; window.__focusReads.updateHeld = true;
            await new Promise(resolve => { window.__focusReads.releaseUpdate = resolve; });
            window.__focusReads.updateHeld = false;
          }
          if (reading) { window.__focusReads.active++; window.__focusReads.maximum = Math.max(window.__focusReads.maximum, window.__focusReads.active); }
          try { return await (await fetch('/__test__/operate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args.request) })).json(); }
          finally { if (reading) window.__focusReads.active--; }
        }
        if (['pending_edits','close_confirmed','plugin:event|listen','plugin:event|unlisten'].includes(command)) return next++;
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
  const settled = async section => page.waitForFunction(selector => document.querySelector(selector)?.getAttribute('aria-busy') === 'false', section);
  const notes = page.locator('.home [data-home-section="note"]');
  const projects = page.locator('.home [data-home-section="project"]');
  const noteRows = () => notes.locator('.home-item');
  try {
    await page.setViewportSize({ width: 1440, height: 900 });
    await control('configure', { theme: 'dark', phase: 'main' }); const info = await control('init');
    await page.goto(origin); await page.evaluate(() => localStorage.clear()); await page.reload();
    await page.locator('#workspace-path').fill(info.path);
    await page.getByRole('button', { name: 'Create workspace', exact: true }).click();
    await page.locator('.workspace-tab-panel.active .home').waitFor();
    await settled('.home [data-home-section="note"]');
    await control('refresh_benchmark_seed', { total: 1000 });
    await page.waitForFunction(() => [...document.querySelectorAll('.home [data-home-section]')].every(section => section.getAttribute('aria-busy') === 'false'));
    await control('home_read', { view: 'notes_library', delay: 700, failure: false });
    const beforeRefresh = await page.evaluate(() => {
      const root = document.querySelector('.workspace-tab-panel.active .home');
      const projects = root.querySelector('[data-home-section="project"]');
      root.scrollTop += projects.getBoundingClientRect().top - root.getBoundingClientRect().top - 90;
      return { top: projects.getBoundingClientRect().top, scroll: root.scrollTop };
    });
    await page.evaluate(() => window.dispatchEvent(new Event('focus')));
    await notes.getByText('Updating; showing previous results.').waitFor();
    const duringRefresh = await page.evaluate(() => ({ top: document.querySelector('.workspace-tab-panel.active .home [data-home-section="project"]').getBoundingClientRect().top, scroll: document.querySelector('.workspace-tab-panel.active .home').scrollTop }));
    check(Math.abs(beforeRefresh.top - duringRefresh.top) <= 1 && Math.abs(beforeRefresh.scroll - duringRefresh.scroll) <= 1, `focus refresh keeps the visible position while loading: ${JSON.stringify({ beforeRefresh, duringRefresh })}`);
    await settled('.home [data-home-section="note"]');
    await page.waitForFunction(() => [...document.querySelectorAll('.home [data-home-section]')].every(section => section.getAttribute('aria-busy') === 'false'));
    await notes.getByRole('button', { name: 'Refresh Notes' }).focus();
    const focusScroll = await page.evaluate(() => { const root = document.querySelector('.workspace-tab-panel.active .home'); const projects = root.querySelector('[data-home-section="project"]'); root.scrollTop += projects.getBoundingClientRect().top - root.getBoundingClientRect().top - 90; return root.scrollTop; });
    await page.evaluate(() => window.dispatchEvent(new Event('focus')));
    await notes.getByText('Updating; showing previous results.').waitFor();
    await page.waitForFunction(() => [...document.querySelectorAll('.home [data-home-section]')].every(section => section.getAttribute('aria-busy') === 'false'));
    await page.waitForFunction(() => document.activeElement?.getAttribute('aria-label') === 'Refresh Notes');
    check(Math.abs((await page.evaluate(() => document.querySelector('.workspace-tab-panel.active .home').scrollTop)) - focusScroll) <= 1, 'restoring focus to an offscreen control does not scroll');
    await control('home_read', { view: 'notes_library', delay: 0, failure: false });
    await notes.getByRole('button', { name: 'All', exact: true }).click();
    await page.waitForFunction(() => document.querySelectorAll('.home [data-home-section="note"] .home-item').length === 10);
    await notes.getByRole('button', { name: 'Load more', exact: true }).click();
    await page.waitForFunction(() => document.querySelectorAll('.home [data-home-section="note"] .home-item').length === 20);
    check(await noteRows().count() === 20, 'two pages loaded');

    await control('home_read', { view: 'notes_all', delay: 700, failure: false });
    await noteRows().last().locator('.home-open').focus();
    await page.evaluate(() => window.dispatchEvent(new Event('focus')));
    await notes.getByText('Updating; showing previous results.').waitFor();
    check(await noteRows().count() === 20, 'refresh keeps both pages visible');
    check(await notes.getByRole('button', { name: 'Load more', exact: true }).isDisabled(), 'old cursor disabled while refreshing');
    await notes.getByText('Additional pages were reset. Load more if available.').waitFor();
    check(await noteRows().count() === 10, 'successful refresh returns to the first page');
    await page.waitForFunction(() => document.activeElement?.matches('.home [data-home-section="note"] .home-tabs [aria-pressed="true"]'));
    check(true, 'keyboard focus returns to the current view when its row is removed');

    await control('home_read', { view: 'notes_all', delay: 0, failure: true });
    await notes.getByRole('button', { name: 'Refresh Notes' }).click();
    await notes.getByText('Showing previous results; they may be out of date.').waitFor();
    check(await noteRows().count() === 10, 'failure retains visibly stale rows');
    check(await notes.getByRole('button', { name: 'Load more', exact: true }).isDisabled(), 'stale cursor disabled after failure');
    await control('home_read', { view: 'notes_all', delay: 0, failure: false });
    await notes.getByRole('button', { name: 'Retry Notes' }).click();
    await settled('.home [data-home-section="note"]');
    check(await notes.getByText('Showing previous results; they may be out of date.').count() === 0, 'retry marks rows current');

    await control('home_read', { view: 'projects_board', delay: 0, failure: false });
    await projects.getByRole('button', { name: 'Refresh Projects' }).click();
    await page.waitForFunction(() => document.querySelectorAll('.home [data-home-section="project"] .project-move-trigger').length > 0);
    const cardCount = await projects.locator('.project-card').count();
    await control('home_read', { view: 'projects_board', delay: 700, failure: true });
    await projects.getByRole('button', { name: 'Refresh Projects' }).click();
    await projects.getByText('Updating; showing previous results.').waitFor();
    check(await projects.locator('.project-card').count() === cardCount, 'board cards remain visible during refresh');
    check(await projects.locator('.project-move-trigger').count() === 0, 'board movement disabled while updating');
    await projects.getByText('Showing previous results; they may be out of date.').waitFor();
    check(await projects.locator('.project-move-trigger').count() === 0, 'board movement disabled after failed refresh');
    await control('home_read', { view: 'projects_board', delay: 0, failure: false });
    await projects.getByRole('button', { name: 'Retry Projects' }).click();
    await page.waitForFunction(() => document.querySelectorAll('.home [data-home-section="project"] .project-move-trigger').length > 0);
    check(true, 'board movement returns after a verified refresh');

    const original = { schema_version: 2, blocks: [{ id: 'focus-block', type: 'paragraph', content: [{ type: 'text', text: 'Original text' }] }] };
    const note = await control('entity_create', { kind: 'note', title: 'Focus draft', document: original });
    await notes.getByRole('button', { name: 'Refresh Notes' }).click();
    await notes.getByRole('button', { name: 'Open Focus draft', exact: true }).waitFor();
    await notes.getByRole('button', { name: 'Open Focus draft', exact: true }).click();
    const pane = page.locator('.workspace-tab-panel.peek:not(.peek-hidden):visible').last();
    const editor = pane.getByRole('textbox', { name: 'Note content' });
    await editor.waitFor(); await editor.locator('p').first().click(); await page.keyboard.press('End'); await page.keyboard.type(' pending');
    await pane.getByText('Unsaved changes', { exact: true }).waitFor();
    await page.evaluate(id => { window.__focusReads.targetId = id; window.__focusReads.holdUpdate = true; window.__focusReads.entityReads = 0; }, note.id);
    await pane.getByRole('button', { name: /^Save/ }).click();
    await page.waitForFunction(() => window.__focusReads.updateHeld);
    await page.evaluate(() => { for (let i = 0; i < 20; i++) window.dispatchEvent(new Event('focus')); });
    const readsBeforeSave = await page.evaluate(() => window.__focusReads.entityReads);
    await page.evaluate(() => window.__focusReads.releaseUpdate());
    await pane.getByText('Saved', { exact: true }).waitFor();
    await page.waitForFunction(before => window.__focusReads.entityReads > before, readsBeforeSave);
    check(true, 'focus received during save is refreshed after the save finishes');
    await editor.locator('p').first().click(); await page.keyboard.press('End'); await page.keyboard.type(' conflict');
    await pane.getByText('Unsaved changes', { exact: true }).waitFor();
    const saved = await control('get', { id: note.id });
    await control('cli_update', { id: note.id, expected_revision: saved.revision, title: 'Focus external' });
    await page.waitForFunction(() => window.__focusReads.active === 0);
    await page.evaluate(() => { window.__focusReads.maximum = 0; for (let i = 0; i < 20; i++) window.dispatchEvent(new Event('focus')); });
    await pane.getByText('Conflict · Unsaved', { exact: true }).waitFor();
    check((await editor.innerText()).includes('conflict'), 'external revision preserves the unsaved document');
    check((await control('get', { id: note.id })).title === 'Focus external', 'focus never overwrites the external revision');
    await page.waitForFunction(() => window.__focusReads.active === 0);
    check(await page.evaluate(() => window.__focusReads.maximum <= 2), 'visible panels share at most two physical reads');
    await editor.locator('p').first().click(); await page.keyboard.press('Meta+z');
    check(!(await editor.innerText()).includes('conflict'), 'undo history survives refresh and conflict');
    await page.keyboard.press('Meta+Shift+z');
    check((await editor.innerText()).includes('conflict'), 'redo history survives refresh and conflict');
    check(errors.length === 0, `browser errors: ${errors.join(', ')}`);
    await control('finish', { ok: true, checks });
  } catch (error) { throw Error(`After ${checks.length} checks (${checks.at(-1)}): ${error.stack || error}`); }
}
