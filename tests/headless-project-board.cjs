async (page) => {
  const checks = [];
  const errors = [];
  page.on('pageerror', error => errors.push(String(error)));
  const origin = new URL(page.url()).origin;
  await page.addInitScript(() => {
    let next = 1;
    const callbacks = new Map();
    window.isTauri = true;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback: callback => { const id = next++; callbacks.set(id, callback); return id; },
      unregisterCallback: id => callbacks.delete(id),
      invoke: async (cmd, args = {}) => {
        if (cmd === 'operate') {
          const response = await fetch('/__test__/operate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args.request) });
          const result = await response.json();
          if (!response.ok) throw new Error(JSON.stringify(result));
          return result;
        }
        if (cmd === 'smoke_control') {
          const response = await fetch('/__test__/control', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ step: args.step, input: args.input || {} }) });
          const result = await response.json();
          if (!response.ok) throw new Error(JSON.stringify(result));
          return result;
        }
        if (cmd === 'pending_edits' || cmd === 'close_confirmed' || cmd === 'plugin:event|unlisten') return;
        if (cmd === 'plugin:event|listen') return next++;
        throw new Error('IPC de prueba desconocido: ' + cmd);
      }
    };
  });
  await page.setViewportSize({ width: 1600, height: 900 });
  await page.goto(origin);
  await page.waitForSelector('#workspace-path');
  const info = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('smoke_control', { step: 'init' }));
  await page.fill('#workspace-path', info.path);
  await page.getByRole('button', { name: 'Create workspace' }).click();
  await page.waitForSelector('.home');

  const control = (step, input = {}) => page.evaluate(([step, input]) => window.__TAURI_INTERNALS__.invoke('smoke_control', { step, input }), [step, input]);
  const create = (kind, title, payload = {}) => control('entity_create', { kind, title, payload });
  const link = async (source, target, kind) => {
    const fresh = await control('get', { id: source.id });
    return control('relation_add', { source_id: fresh.id, target_id: target.id, kind, expected_revision: fresh.revision });
  };
  const waitFor = async (action, predicate, label) => {
    const start = Date.now();
    let value;
    while (Date.now() - start < 12000) {
      value = await action();
      if (predicate(value)) return value;
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    throw new Error(`Timeout: ${label}`);
  };
  const project = await create('project', 'Synthetic board project', { state: 'doing' });
  const todo = await create('task', 'Board pending', { state: 'todo', due_date: '2026-10-20' });
  const doing = await create('task', 'Board doing', { state: 'doing', due_date: '2026-10-21' });
  const done = await create('task', 'Board done', { state: 'done', due_date: '2026-10-19' });
  for (const task of [todo, doing, done]) await link(task, project, 'project');

  await page.locator('.desktop-sidebar nav[aria-label="Sections"]').getByRole('button', { name: 'Projects', exact: true }).click();
  await page.waitForSelector('.projects-page');
  await page.waitForFunction(id => !!document.querySelector(`[data-project-id="${id}"] .task-page-project-open, [data-project-id="${id}"] .project-card-open`), project.id, { timeout: 15000 });
  await page.locator(`[data-project-id="${project.id}"] .task-page-project-open, [data-project-id="${project.id}"] .project-card-open`).first().click();
  await page.waitForSelector('.page.project-detail');
  const tasks = page.locator('section[aria-label="Project Tasks"]');
  await tasks.getByRole('button', { name: 'Board', exact: true }).click();
  await page.waitForSelector('.task-board');
  await tasks.scrollIntoViewIfNeeded();
  await page.screenshot({ path: 'output/playwright/project-task-board-20261104.png' });
  if (await page.locator('.task-board-column').count() !== 3) throw new Error('Board no muestra tres columnas');
  if (await page.locator('.task-board-card').count() !== 3) throw new Error('Board no muestra las tres tareas');
  checks.push('Board abre con columnas To Do/Doing/Done y las tareas relacionadas');

  await page.locator(`.task-board-card[data-entity-id="${todo.id}"] select`).selectOption('doing');
  const movedBySelect = await waitFor(() => control('get', { id: todo.id }), task => task.payload.state === 'doing', 'estado doing por selector');
  if (!movedBySelect) throw new Error('El selector no actualizó la tarea');
  await page.waitForFunction(() => document.querySelector('.task-board')?.getAttribute('aria-busy') === 'false');
  checks.push('El selector de estado mueve una tarjeta y persiste doing');

  await page.locator('.task-board').scrollIntoViewIfNeeded();
  const card = page.locator(`.task-board-card[data-entity-id="${doing.id}"]`);
  const cardBox = await card.boundingBox();
  const targetColumn = await page.locator('.task-board-column[data-task-state="todo"]').boundingBox();
  if (!cardBox || !targetColumn) throw new Error('No se pudieron medir las tarjetas');
  await page.mouse.move(cardBox.x + 18, cardBox.y + 14);
  await page.mouse.down();
  await page.mouse.move(targetColumn.x + targetColumn.width / 2, targetColumn.y + 48, { steps: 8 });
  await page.mouse.up();
  await waitFor(() => control('get', { id: doing.id }), task => task.payload.state === 'todo', 'estado todo por arrastre');
  checks.push('El arrastre entre columnas persiste el estado de destino');

  await tasks.getByRole('button', { name: 'List', exact: true }).click();
  await page.waitForSelector('.project-task-group');
  const groups = await page.locator('.project-task-group .project-task-state').allTextContents();
  if (!groups.includes('Doing') || !groups.includes('Done')) throw new Error('List no muestra los grupos Doing/Done');
  const checkboxRow = page.locator('.project-task-row').first();
  const checkboxTaskId = await checkboxRow.getAttribute('data-entity-id');
  await checkboxRow.locator('[role="checkbox"]').click();
  await waitFor(() => control('get', { id: checkboxTaskId }), task => task.payload.state === 'done', 'checkbox de List');
  checks.push('List usa un checkbox real y persiste done');

  await tasks.getByRole('button', { name: 'Board', exact: true }).click();
  await page.locator('.task-board-column[data-task-state="todo"] .task-board-new').click();
  await page.waitForSelector('.title-input');
  const selectedStatus = await page.locator('[aria-label="Task status"]').inputValue();
  if (selectedStatus !== 'todo') throw new Error('New task no preselecciona To Do');
  if (!(await page.locator('.page').textContent()).includes(`Captura vinculada a project: ${project.id}`)) throw new Error('New task no conserva el contexto del Project');
  checks.push('New task por columna prepara la captura con estado y Project correctos');

  if (errors.length) throw new Error('Errores de página: ' + errors.join(' | '));
  await control('finish', { ok: true, checks });
  return { ok: true, checks };
}
