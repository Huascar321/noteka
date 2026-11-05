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
  const control = async (step, input = {}) => { const r = await page.request.post(origin + '/__test__/control', { data: { step, input } }); if (!r.ok()) throw Error(step + ': ' + await r.text()); return r.json(); };
  const check = (condition, label) => { if (!condition) throw Error(label); checks.push(label); };
  const pane = () => page.locator('.workspace-tab-panel.peek:not(.peek-hidden):visible').last();
  const open = async title => {
    if (!await page.getByRole('button', { name: 'Search', exact: true }).isVisible()) await page.getByRole('button', { name: 'Expand sidebar', exact: true }).click();
    const search = page.getByRole('combobox', { name: 'Search workspace' });
    for (let i = 0; i < 12 && !await search.isVisible(); i++) { await page.getByRole('button', { name: 'Search', exact: true }).click(); await page.waitForTimeout(150); }
    await search.fill(title);
    await page.waitForFunction(() => document.querySelector('#context-search-results')?.getAttribute('aria-busy') === 'false');
    await page.getByRole('option').filter({ has: page.locator('strong', { hasText: title }) }).first().click();
    await page.waitForFunction(value => [...document.querySelectorAll('.title-input')].some(el => el.value === value && !el.disabled && el.getClientRects().length), title);
    await page.waitForTimeout(200);
  };
  try {
    await page.setViewportSize({ width: 1440, height: 900 });
    await control('configure', { theme: 'light', phase: 'main' }); const info = await control('init');
    await page.goto(origin); await page.evaluate(() => { localStorage.clear(); localStorage.setItem('noteka.peek-mode.v1', 'side'); }); await page.reload();
    await page.locator('#workspace-path').fill(info.path); await page.getByRole('button', { name: 'Create workspace', exact: true }).click();
    await page.locator('.workspace-tab-panel.active .home').waitFor();
    const empty = await control('entity_create', { kind: 'task', title: 'Tarea sin cuerpo', payload: { state: 'todo' } });
    const original = { schema_version: 2, blocks: [{ id: 'block-original', type: 'paragraph', content: [{ type: 'text', text: 'Texto original' }] }] };
    const task = await control('entity_create', { kind: 'task', title: 'Tarea con bloques', payload: { state: 'todo' }, document: original });
    await control('entity_create', { kind: 'note', title: 'Nota de control', document: original });
    await open('Tarea sin cuerpo');
    const editor = () => pane().getByRole('textbox', { name: 'Task content' });
    check(await pane().locator('.task-document .page-editor').count() === 1, 'Task uses contextual editor');
    await editor().focus();
    let stored = await control('get', { id: empty.id });
    check(stored.revision === empty.revision && !stored.document, 'open and focus do not persist empty document');
    await pane().getByRole('button', { name: 'Close peek' }).click();
    await open('Tarea con bloques');
    check(await pane().locator('.format-controls,.add-paragraph').count() === 0, 'legacy footer and toolbar absent');
    await editor().click(); await page.keyboard.press('End'); await page.keyboard.type(' /h1');
    await pane().getByRole('listbox', { name: 'Block commands' }).waitFor();
    check(await pane().getByRole('option', { name: /Heading 1/ }).isVisible(), 'slash filters heading');
    await page.keyboard.press('Enter');
    check(await editor().locator('h1').count() === 1, 'slash applies heading');
    await page.keyboard.press('End'); await page.keyboard.press('Enter'); await page.keyboard.type('Texto de prueba');
    for (let i = 0; i < 6; i++) await page.keyboard.press('Shift+ArrowLeft');
    await pane().getByRole('toolbar', { name: 'Text formatting' }).waitFor();
    await pane().getByRole('button', { name: 'Bold', exact: true }).click();
    check(await editor().locator('strong').count() === 1, 'selection formatting applies bold');
    await editor().locator('p').last().hover();
    const beforeDuplicate = await editor().locator('p').count();
    await pane().getByRole('button', { name: 'Block actions', exact: true }).click();
    await pane().getByRole('menuitem', { name: 'Duplicate', exact: true }).click();
    await page.waitForTimeout(100);
    check(await editor().locator('p').count() === beforeDuplicate + 1, 'block action duplicates paragraph');
    const ids = await editor().locator('p').evaluateAll(nodes => nodes.map(node => node.getAttribute('data-id')));
    check(ids.every(Boolean) && new Set(ids).size === ids.length, 'duplicated block receives distinct identity');
    await page.keyboard.press('Meta+z');
    check(await editor().locator('p').count() === beforeDuplicate, 'undo reverses duplicate');
    await page.keyboard.press('Meta+Shift+z');
    check(await editor().locator('p').count() === beforeDuplicate + 1, 'redo restores duplicate');
    const last = await editor().locator('p').last().boundingBox(), box = await editor().boundingBox();
    await page.mouse.click(box.x + box.width / 2, last.y + last.height + 22);
    check(await editor().locator('p').count() === beforeDuplicate + 2, 'click below content adds paragraph');
    const finalBox = await editor().locator('p').last().boundingBox();
    await page.mouse.click(box.x + box.width / 2, finalBox.y + finalBox.height + 22);
    check(await editor().locator('p').count() === beforeDuplicate + 2, 'repeated click reuses blank paragraph');
    await page.keyboard.type('Final guardado');
    check((await control('get', { id: task.id })).revision === task.revision, 'editing does not auto-save');
    await pane().getByRole('button', { name: /^Save/ }).click();
    await pane().getByText('Saved', { exact: true }).waitFor();
    stored = await control('get', { id: task.id });
    check(stored.revision > task.revision && JSON.stringify(stored.document).includes('Final guardado'), 'explicit save persists document');
    await pane().getByRole('button', { name: 'Close peek' }).click(); await open('Tarea con bloques');
    check(await editor().getByText('Final guardado').count() === 1, 'reopened task preserves written content');
    await editor().locator('p').last().click(); await page.keyboard.type(' pendiente');
    await pane().getByRole('button', { name: 'Close peek' }).click();
    const guard = page.getByRole('dialog', { name: 'You have unsaved changes' }); await guard.waitFor();
    await guard.getByRole('button', { name: 'Cancel', exact: true }).click();
    check((await editor().innerText()).includes('pendiente'), 'cancel keeps unsaved draft');
    await pane().getByRole('button', { name: 'Close peek' }).click();
    await guard.getByRole('button', { name: 'Discard', exact: true }).click();
    await open('Nota de control');
    const noteEditor = pane().getByRole('textbox', { name: 'Note content' });
    const noteLast = await noteEditor.locator('p').last().boundingBox(), noteBox = await noteEditor.boundingBox();
    await page.mouse.click(noteBox.x + noteBox.width / 2, noteLast.y + noteLast.height + 22);
    check(await noteEditor.locator('p').count() === 2, 'Note still continues below its last block');
    check(errors.length === 0 && external.length === 0, `browser errors ${JSON.stringify(errors)}; external requests ${JSON.stringify(external)}`);
    await control('finish', { ok: true, checks });
  } catch (error) { throw Error(`After ${checks.length} checks (${checks.at(-1)}): ${error.stack || error}`); }
}
