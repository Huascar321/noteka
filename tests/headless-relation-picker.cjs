async (page) => {
  page.setDefaultTimeout(12000);
  const origin = new URL(page.url()).origin, errors = [], checks = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.addInitScript(() => {
    let next = 1; const callbacks = new Map(); window.isTauri = true;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__relationTest = { mode: '', person: null, failAdd: false, failRelationList: false, updateCalls: 0, completedUpdates: 0, holdAdd: false, holdEntityId: null, holdRelationList: false };
    window.__TAURI_INTERNALS__ = {
      transformCallback(callback) { const id = next++; callbacks.set(id, callback); return id; },
      unregisterCallback(id) { callbacks.delete(id); },
      async invoke(command, args = {}) {
        if (command === 'operate') {
          const op = args.request.command;
          if (op.operation === 'entity.update') window.__relationTest.updateCalls++;
          if (op.operation === 'relation.add' && window.__relationTest.failAdd) {
            window.__relationTest.failAdd = false;
            return { ok: false, error: { code: 'revision_conflict', message: 'Injected revision conflict', details: { current_revision: 999 } } };
          }
          if (op.operation === 'relation.list' && window.__relationTest.failRelationList) {
            window.__relationTest.failRelationList = false;
            return { ok: false, error: { code: 'validation', message: 'Injected relation refresh error', details: null } };
          }
          if (op.operation === 'relation.add' && window.__relationTest.holdAdd) {
            window.__relationTest.holdAdd = false;
            await new Promise(resolve => { window.__relationTest.releaseAdd = resolve; });
          }
          if (op.operation === 'entity.list' && op.input.kind === 'person' && window.__relationTest.mode === 'error') {
            window.__relationTest.mode = '';
            return { ok: false, error: { code: 'validation', message: 'Injected list error', details: null } };
          }
          if (op.operation === 'entity.list' && op.input.kind === 'person' && window.__relationTest.mode === 'pages') {
            const entity = window.__relationTest.person;
            return { ok: true, data: op.input.cursor ? { items: [entity], next_cursor: null } : { items: [{ ...entity, id: 'fake-unrelated', title: 'Another person' }], next_cursor: 'synthetic-page' } };
          }
          if (op.operation === 'entity.list' && op.input.kind === 'person' && window.__relationTest.mode === 'race') {
            if (!window.__relationTest.held) {
              await new Promise(resolve => { window.__relationTest.held = resolve; });
              return { ok: true, data: { items: [{ ...window.__relationTest.person, id: 'old-result', title: 'Old result' }], next_cursor: null } };
            }
            return { ok: true, data: { items: [window.__relationTest.person], next_cursor: null } };
          }
          const response = await (await fetch('/__test__/operate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args.request) })).json();
          if (op.operation === 'entity.update') window.__relationTest.completedUpdates++;
          if (op.operation === 'relation.list' && window.__relationTest.holdRelationList) {
            window.__relationTest.holdRelationList = false;
            await new Promise(resolve => { window.__relationTest.releaseRelations = resolve; });
          }
          if (op.operation === 'entity.get' && op.input.id === window.__relationTest.holdEntityId) {
            window.__relationTest.holdEntityId = null;
            await new Promise(resolve => { window.__relationTest.releaseRead = resolve; });
          }
          return response;
        }
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
  const check = (value, message) => { if (!value) throw Error(message); checks.push(message); };
  const open = async title => {
    for (let attempt = 0; attempt < 12 && !await page.getByRole('combobox', { name: 'Search workspace' }).isVisible(); attempt++) {
      await page.getByRole('button', { name: 'Search', exact: true }).click();
      await page.waitForTimeout(150);
    }
    await page.getByRole('combobox', { name: 'Search workspace' }).fill(title).catch(() => { throw Error('Could not open search for ' + title); });
    await page.waitForFunction(() => document.querySelector('#context-search-results')?.getAttribute('aria-busy') === 'false');
    await page.getByRole('option').filter({ has: page.locator('strong', { hasText: title }) }).first().click();
    const guard = page.getByRole('dialog', { name: 'You have unsaved changes' });
    if (await guard.isVisible().catch(() => false)) await guard.getByRole('button', { name: 'Discard', exact: true }).click();
    await page.locator('.title-input:visible').first().waitFor();
    await page.waitForFunction(expected => [...document.querySelectorAll('.title-input')].some(el => el.value === expected && el.getClientRects().length), title).catch(() => { throw Error('Opening ' + title); });
  };
  await control('configure', { theme: 'dark', phase: 'main' });
  const info = await control('init');
  await page.goto(origin); await page.evaluate(() => { localStorage.clear(); localStorage.setItem('noteka.peek-mode.v1', 'full'); }); await page.reload();
  await page.locator('#workspace-path').fill(info.path);
  await page.getByRole('button', { name: 'Create workspace', exact: true }).click();
  await page.locator('.workspace-tab-panel.active .home').waitFor();
  const person = await control('entity_create', { kind: 'person', title: 'Ángela Local', payload: {} });
  const tag = await control('entity_create', { kind: 'tag', title: 'Trabajo Local', payload: { classification: 'topic' } });
  await control('entity_create', { kind: 'project', title: 'Proyecto Local', payload: { state: 'planned' } });
  const task = await control('entity_create', { kind: 'task', title: 'Tarea relaciones', payload: { state: 'todo' } });
  await open('Tarea relaciones');
  check(await page.getByText('Local destination ID').count() === 0, 'no technical ID form');
  await page.getByRole('button', { name: 'Add person', exact: true }).click();
  const picker = page.getByRole('dialog', { name: 'Edit people' });
  const search = picker.getByLabel('Search people'); await search.waitFor();
  check(await search.evaluate(el => el === document.activeElement), 'search receives focus');
  await search.fill('angela');
  const row = picker.getByRole('button', { name: /Ángela Local/ }); await row.waitFor();
  await page.keyboard.press('ArrowDown'); check(await row.evaluate(el => el === document.activeElement), 'keyboard moves to results');
  await page.keyboard.press('Enter');
  await page.waitForFunction(() => document.querySelector('.relation-picker-item[aria-pressed="true"]'));
  await page.waitForFunction(() => !document.querySelector('.relation-picker-item[aria-pressed="true"]')?.disabled);
  await search.focus();
  check(await page.getByRole('button', { name: 'Ángela Local', exact: true }).count() >= 1, 'selected person linked');
  await page.keyboard.press('Escape'); check(await picker.count() === 0, 'Escape closes selector');
  await page.waitForFunction(() => document.activeElement?.textContent?.trim() === 'Add person').catch(() => { throw Error('Focus restoration'); });
  check(true, 'focus returns to trigger');
  await page.getByRole('button', { name: 'Remove relation to Ángela Local' }).click();
  await page.waitForFunction(() => !document.querySelector('[aria-label="Remove relation to Ángela Local"]'));
  await page.evaluate(() => { window.__relationTest.failAdd = true; });
  await page.getByRole('button', { name: 'Add person', exact: true }).click();
  await page.getByRole('dialog', { name: 'Edit people' }).getByRole('button', { name: /Ángela Local/ }).click();
  await page.getByText('Injected revision conflict').waitFor();
  check(await page.getByRole('button', { name: 'Remove relation to Ángela Local' }).count() === 0, 'revision conflict keeps relation unchanged');
  await page.keyboard.press('Escape');
  const title = page.locator('.title-input:visible').first(); await title.fill('Tarea borrador');
  const editor = page.locator('.noteka-document:visible').first();
  await editor.fill('Texto pendiente sin guardar');
  await page.evaluate(() => { window.__relationTest.updateCalls = 0; });
  await page.evaluate(id => { window.__relationTest.holdEntityId = id; }, task.id);
  await page.getByRole('button', { name: 'Refresh page' }).click();
  await page.waitForFunction(() => typeof window.__relationTest.releaseRead === 'function');
  await page.getByRole('button', { name: 'Add tag', exact: true }).click();
  const tags = page.getByRole('dialog', { name: 'Edit tags' });
  const tagItem = tags.getByRole('button', { name: /Trabajo Local/ });
  await tagItem.waitFor();
  check(await tagItem.isEnabled() && await tags.getByText('Save changes to edit relations').count() === 0, 'dirty draft allows linking without extra save');
  await page.evaluate(() => { window.__relationTest.holdAdd = true; });
  await tagItem.click();
  await page.waitForFunction(() => typeof window.__relationTest.releaseAdd === 'function');
  await page.keyboard.press('Meta+s');
  check(await page.evaluate(() => window.__relationTest.updateCalls) === 0, 'save shortcut cannot race a relation mutation');
  await page.evaluate(() => window.__relationTest.releaseAdd());
  await page.waitForFunction(() => document.querySelector('.relation-picker-item[aria-pressed="true"]') && !document.querySelector('.relation-picker-item[aria-pressed="true"]').disabled);
  await page.evaluate(() => window.__relationTest.releaseRead());
  await page.waitForTimeout(150);
  check(await page.getByText('Unsaved changes', { exact: true }).isVisible() && await page.getByText('Conflict · Unsaved').count() === 0, 'stale refresh cannot replace confirmed revision or invent a conflict');
  const storedBeforeSave = await control('get', { id: task.id });
  check(storedBeforeSave.title === 'Tarea relaciones' && !JSON.stringify(storedBeforeSave.document).includes('Texto pendiente sin guardar') && await page.evaluate(() => window.__relationTest.updateCalls) === 0, 'relation persisted without saving title or body');
  check(await title.inputValue() === 'Tarea borrador' && (await editor.innerText()).includes('Texto pendiente sin guardar') && await page.getByText('Unsaved changes', { exact: true }).isVisible(), 'title, body and dirty status survive link');
  await page.keyboard.press('Escape');
  await page.evaluate(() => { window.__relationTest.holdRelationList = true; });
  await page.getByRole('button', { name: 'Refresh page' }).click();
  await page.waitForFunction(() => typeof window.__relationTest.releaseRelations === 'function');
  await page.getByRole('button', { name: 'Remove relation to Trabajo Local' }).click();
  await page.waitForFunction(() => !document.querySelector('[aria-label="Remove relation to Trabajo Local"]'));
  await page.evaluate(() => window.__relationTest.releaseRelations());
  await page.waitForTimeout(150);
  check(await page.getByRole('button', { name: 'Remove relation to Trabajo Local' }).count() === 0, 'stale relation listing cannot restore a removed link');
  check(await title.inputValue() === 'Tarea borrador' && (await editor.innerText()).includes('Texto pendiente sin guardar'), 'unlink preserves draft');
  await page.getByRole('button', { name: 'Add tag', exact: true }).click();
  await page.getByRole('dialog', { name: 'Edit tags' }).getByRole('button', { name: /Trabajo Local/ }).click();
  await page.waitForFunction(() => document.querySelector('.relation-picker-item[aria-pressed="true"]') && !document.querySelector('.relation-picker-item[aria-pressed="true"]').disabled);
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: /^Save/ }).click();
  await page.getByText('Saved', { exact: true }).waitFor();
  const storedAfterSave = await control('get', { id: task.id });
  check(storedAfterSave.title === 'Tarea borrador' && JSON.stringify(storedAfterSave.document).includes('Texto pendiente sin guardar') && await page.getByRole('button', { name: 'Remove relation to Trabajo Local' }).count() === 1, 'later save keeps confirmed relation and saves draft');
  await page.getByRole('button', { name: 'Edit task project' }).click();
  await page.getByRole('dialog', { name: 'Edit projects' }).getByRole('button', { name: /Proyecto Local/ }).click();
  await page.waitForFunction(() => document.querySelector('.task-detail-project')?.textContent.includes('Proyecto Local'));
  await page.waitForFunction(() => !document.querySelector('.relation-picker-item[aria-pressed="true"]')?.disabled);
  await page.keyboard.press('Escape');
  check(await page.getByText('Proyecto Local', { exact: true }).count() === 1, 'project property is not duplicated');
  await title.fill('Tarea descartable');
  await page.getByRole('button', { name: 'Remove relation to Trabajo Local' }).click();
  await page.waitForFunction(() => !document.querySelector('[aria-label="Remove relation to Trabajo Local"]'));
  await page.getByRole('button', { name: 'Proyecto Local', exact: true }).click();
  const discardDialog = page.getByRole('dialog', { name: 'You have unsaved changes' });
  await discardDialog.getByRole('button', { name: 'Discard', exact: true }).click();
  await open('Tarea borrador');
  check(await page.getByRole('button', { name: 'Remove relation to Trabajo Local' }).count() === 0 && (await control('get', { id: task.id })).title === 'Tarea borrador', 'discarding draft does not undo confirmed unlink');
  await page.getByRole('button', { name: 'Add tag', exact: true }).click();
  await page.getByRole('dialog', { name: 'Edit tags' }).getByRole('button', { name: /Trabajo Local/ }).click();
  await page.waitForFunction(() => document.querySelector('.relation-picker-item[aria-pressed="true"]') && !document.querySelector('.relation-picker-item[aria-pressed="true"]').disabled);
  await page.keyboard.press('Escape');
  await page.evaluate(p => { window.__relationTest.person = p; window.__relationTest.mode = 'pages'; }, person);
  await page.getByRole('button', { name: 'Add person', exact: true }).click();
  await page.getByRole('button', { name: 'Load more people' }).waitFor();
  await page.getByRole('button', { name: 'Load more people' }).click();
  await page.getByRole('dialog', { name: 'Edit people' }).getByRole('button', { name: /Ángela Local/ }).waitFor();
  check(true, 'unfiltered list offers additional pages');
  await page.getByLabel('Search people').fill('angela');
  await page.getByRole('dialog', { name: 'Edit people' }).getByRole('button', { name: /Ángela Local/ }).waitFor();
  check(true, 'search continues past an unmatched page');
  await page.keyboard.press('Escape');
  await page.evaluate(() => { window.__relationTest.mode = 'race'; window.__relationTest.held = null; });
  await page.getByRole('button', { name: 'Add person', exact: true }).click();
  await page.waitForFunction(() => typeof window.__relationTest.held === 'function');
  await page.getByLabel('Search people').fill('angela');
  await page.getByRole('dialog', { name: 'Edit people' }).getByRole('button', { name: /Ángela Local/ }).waitFor();
  await page.evaluate(() => window.__relationTest.held());
  await page.waitForTimeout(100);
  check(await page.getByRole('dialog', { name: 'Edit people' }).getByText('Old result').count() === 0, 'obsolete search result is discarded');
  await page.keyboard.press('Escape');
  await page.evaluate(() => { window.__relationTest.mode = 'error'; });
  await page.getByRole('button', { name: 'Add person', exact: true }).click();
  await page.getByText('Injected list error').waitFor();
  await page.getByRole('button', { name: 'Retry', exact: true }).click();
  await page.getByRole('dialog', { name: 'Edit people' }).getByRole('button', { name: /Ángela Local/ }).waitFor();
  check(true, 'load error is visible and retry works');
  await page.setViewportSize({ width: 420, height: 740 });
  for (const theme of ['light', 'dark']) {
    await page.evaluate(value => { document.documentElement.dataset.theme = value; }, theme);
    await page.waitForTimeout(50);
    const bounds = await page.locator('.relation-picker').boundingBox();
    check(bounds && bounds.x >= 0 && bounds.x + bounds.width <= 420 && bounds.y >= 0 && bounds.y + bounds.height <= 740, `${theme} narrow viewport keeps picker on screen`);
    await page.screenshot({ path: info.root + `/relations-${theme}-narrow.png` });
  }
  await page.setViewportSize({ width: 1280, height: 720 });
  await page.keyboard.press('Escape');
  await open('Trabajo Local');
  await page.getByRole('button', { name: 'Remove relation to Tarea borrador' }).waitFor();
  check(await page.getByRole('button', { name: 'Remove relation to Tarea borrador' }).count() === 1, 'incoming relation visible');
  const tagTitle = page.locator('.title-input:visible').first();
  await tagTitle.fill('Trabajo pendiente');
  await page.getByRole('button', { name: 'Remove relation to Tarea borrador' }).click();
  await page.waitForFunction(() => !document.querySelector('[aria-label="Remove relation to Tarea borrador"]'));
  check(await tagTitle.inputValue() === 'Trabajo pendiente' && (await control('get', { id: tag.id })).title === 'Trabajo Local', 'incoming unlink preserves target draft');
  await tagTitle.fill('Trabajo Local');
  await page.getByRole('button', { name: /^Save/ }).click();
  await page.getByText('Saved', { exact: true }).waitFor();
  check((await control('get', { id: tag.id })).id === tag.id, 'unlink preserves destination');
  await open('Tarea borrador');
  await page.getByRole('button', { name: 'Add person', exact: true }).click();
  await page.getByRole('dialog', { name: 'Edit people' }).getByRole('button', { name: /Ángela Local/ }).click();
  await page.waitForFunction(() => document.querySelector('.relation-picker-item[aria-pressed="true"]') && !document.querySelector('.relation-picker-item[aria-pressed="true"]').disabled);
  await page.keyboard.press('Escape');
  await control('entity_archive', { id: person.id, expected_revision: person.revision });
  await open('Trabajo Local');
  await open('Tarea borrador');
  await page.getByText('Ángela Local · archived', { exact: true }).waitFor();
  await page.getByRole('button', { name: 'Add person', exact: true }).click();
  const archived = page.getByRole('dialog', { name: 'Edit people' }).getByRole('button', { name: /Ángela Local.*Archived/ });
  await archived.waitFor();
  check(await archived.getAttribute('aria-pressed') === 'true', 'archived linked person remains identifiable and selected');
  await archived.click();
  await page.waitForFunction(() => !document.querySelector('[aria-label="Remove relation to Ángela Local"]'));
  await page.waitForFunction(() => [...document.querySelectorAll('.relation-add')].some(el => el.textContent.trim() === 'Add person' && !el.disabled));
  check((await control('get', { id: person.id })).archived_at != null, 'unlink preserves archived entity');
  await page.keyboard.press('Escape');
  await title.fill('Conflicting pending draft');
  await page.getByRole('button', { name: 'Add tag', exact: true }).click();
  await page.getByRole('dialog', { name: 'Edit tags' }).getByRole('button', { name: /Trabajo Local/ }).waitFor();
  await page.evaluate(() => { window.__relationTest.failRelationList = true; });
  await page.getByRole('dialog', { name: 'Edit tags' }).getByRole('button', { name: /Trabajo Local/ }).click();
  await page.getByText(/Relation saved, but the view could not be refreshed/).waitFor();
  const relationResponse = await page.request.post(origin + '/__test__/operate', { data: { workspace: info.path, command: { operation: 'relation.list', input: { entity_id: task.id, limit: 100 } } } });
  const confirmedRelations = await relationResponse.json();
  check(confirmedRelations.data.items.some(item => item.target_id === tag.id) && await title.inputValue() === 'Conflicting pending draft', 'refresh failure reports confirmed relation and preserves draft');
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: 'Refresh page' }).click();
  await page.getByRole('button', { name: 'Remove relation to Trabajo Local' }).waitFor();
  await page.getByRole('button', { name: 'Remove relation to Trabajo Local' }).click();
  await page.waitForFunction(() => !document.querySelector('[aria-label="Remove relation to Trabajo Local"]'));
  const currentTask = await control('get', { id: task.id });
  await control('cli_update', { id: task.id, expected_revision: currentTask.revision, title: 'External update' });
  await page.getByRole('button', { name: 'Add tag', exact: true }).click();
  await page.getByRole('dialog', { name: 'Edit tags' }).getByRole('button', { name: /Trabajo Local/ }).click();
  await page.getByRole('dialog', { name: 'Edit tags' }).getByRole('alert').waitFor();
  check(await title.inputValue() === 'Conflicting pending draft' && await page.getByText('Conflict · Unsaved').isVisible() && (await control('get', { id: task.id })).title === 'External update', 'real revision conflict preserves draft and external write');
  await page.keyboard.press('Escape');
  const completedBefore = await page.evaluate(() => window.__relationTest.completedUpdates);
  await page.getByRole('button', { name: /^Save/ }).click();
  await page.waitForFunction(before => window.__relationTest.completedUpdates > before && document.querySelector('.save-status')?.textContent.includes('Conflict'), completedBefore);
  check((await control('get', { id: task.id })).title === 'External update' && await title.inputValue() === 'Conflicting pending draft', 'subsequent save cannot overwrite conflict');
  await control('entity_create', { kind: 'note', title: 'Locked relations note', payload: { page_locked: true }, document: { schema_version: 2, blocks: [{ id: 'locked-block', type: 'paragraph', content: [] }] } });
  await open('Locked relations note');
  check(await page.getByRole('button', { name: 'Edit note tags' }).isDisabled() && await page.getByRole('button', { name: 'Add person', exact: true }).isDisabled(), 'locked note cannot change relations');
  check(errors.length === 0, 'no browser errors: ' + errors.join('; '));
  await control('finish', { ok: true, checks });
}
