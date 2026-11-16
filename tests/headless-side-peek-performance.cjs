async page => {
  const origin = new URL(page.url()).origin, stage = new URL(page.url()).hash.slice(1), checks = [], samples = [], errors = [];
  const browser = page.context().browser().browserType().name();
  const check = (value, message) => { if (!value) throw Error(message); checks.push(message); };
  page.on('pageerror', error => errors.push(error.message));
  await page.context().route('**/*', route => new URL(route.request().url()).origin === origin ? route.continue() : route.abort());
  await page.context().tracing.start({ screenshots: true, snapshots: true, sources: true });
  await page.addInitScript(() => {
    let next = 1; window.isTauri = true; window.__peekReads = []; window.__coverResizes = 0;
    const width = Object.getOwnPropertyDescriptor(HTMLCanvasElement.prototype, 'width');
    Object.defineProperty(HTMLCanvasElement.prototype, 'width', { ...width, set(value) { if (window.__countPeek && this.matches('.ascii-shark')) window.__coverResizes++; width.set.call(this, value); } });
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = { transformCallback() { return next++; }, unregisterCallback() {}, async invoke(command, args = {}) {
      if (command === 'operate') {
        const request = args.request.command;
        if (window.__countPeek) window.__peekReads.push({ operation: request.operation, id: request.input.id, time: performance.now() });
        if (window.__countPeek && request.operation === 'entity.get') await new Promise(resolve => setTimeout(resolve, 80));
        if (window.__countPeek && request.operation === 'relation.list' && !window.__releasePeekMetadata) await new Promise(resolve => window.__releasePeekMetadata = resolve);
        if (window.__holdLateExternal && request.operation === 'external.list' && !window.__releaseLateExternal) {
          await new Promise(resolve => window.__releaseLateExternal = resolve);
          return { ok: false, error: { code: 'storage_error', message: 'Stale external failure', details: {} } };
        }
        return (await fetch('/__test__/operate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args.request) })).json();
      }
      if (['pending_edits', 'close_confirmed', 'plugin:event|listen', 'plugin:event|unlisten'].includes(command)) return next++;
      throw Error(command);
    } };
    localStorage.setItem('noteka.peek-mode.v1', 'side');
    localStorage.setItem('noteka.theme.v1', 'light');
  });
  const control = async (step, input = {}) => { const response = await page.request.post(origin + '/__test__/control', { data: { step, input } }); if (!response.ok()) throw Error(await response.text()); return response.json(); };
  await control('configure', { theme: 'light', phase: 'main' }); const info = await control('init');
  await page.setViewportSize({ width: 1440, height: 900 }); await page.goto(origin);
  await page.getByLabel('Workspace path').fill(info.path); await page.getByRole('button', { name: 'Create workspace', exact: true }).click(); await page.locator('.home').waitFor();
  const entities = [];
  for (const kind of ['note', 'task']) entities.push(await control('entity_create', { kind, title: `Performance ${kind}`, payload: kind === 'task' ? { state: 'todo' } : {}, document: { schema_version: 2, blocks: [{ id: 'body', type: 'paragraph', content: [{ type: 'text', text: `Original synthetic ${kind} body` }] }] } }));
  await page.reload(); await page.locator('.home').waitFor();
  for (const entity of entities) {
    await page.locator(`[data-home-section="${entity.kind}"]`).getByRole('button', { name: 'All', exact: true }).evaluate(button => button.click());
    const row = page.locator(`.home-item[data-entity-id="${entity.id}"] .home-open`).first(); await row.waitFor();
    await page.evaluate(() => { window.__peekReads = []; window.__coverResizes = 0; window.__releasePeekMetadata = null; window.__countPeek = true; window.__peekStarted = performance.now(); });
    // Keep the cover visible while opening a row farther down Home.
    await row.evaluate(button => button.click());
    const side = page.locator('.peek-side'); await side.locator('.title-input').waitFor();
    await page.waitForFunction(() => !!window.__releasePeekMetadata);
    await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
    const sample = await side.evaluate(panel => ({ titleMs: performance.now() - window.__peekStarted, editableBeforeMetadata: !panel.querySelector('.title-input').disabled && panel.querySelector('.noteka-document')?.getAttribute('contenteditable') === 'true', reads: window.__peekReads.filter(r => r.operation === 'entity.get').length, transition: getComputedStyle(panel).transitionProperty, duration: getComputedStyle(panel).transitionDuration }));
    if (stage !== 'before') {
      check(sample.editableBeforeMetadata, `${entity.kind}: fresh original editable before held metadata resolves`);
      check(sample.reads === 1, `${entity.kind}: one fresh original read from Home row`);
      check(sample.transition.includes('transform') && !sample.transition.includes('right') && sample.duration.includes('0.26s'), `${entity.kind}: compositor slide retains 260 ms`);
      await side.locator('.title-input').fill(`Pending ${entity.kind}`);
      await side.getByRole('button', { name: 'Close peek', exact: true }).click();
      const guard = page.getByRole('dialog', { name: 'You have unsaved changes' }); await guard.waitFor();
      await guard.getByRole('button', { name: 'Cancel', exact: true }).click(); await guard.waitFor({ state: 'detached' });
      check(await side.locator('.title-input').inputValue() === `Pending ${entity.kind}`, `${entity.kind}: pending metadata keeps close guard and draft`);
    }
    await page.evaluate(() => window.__releasePeekMetadata());
    await page.waitForFunction(() => !document.querySelector('.peek-side .title-input')?.disabled);
    await page.waitForTimeout(650);
    sample.coverResizes = await page.evaluate(() => window.__coverResizes);
    sample.cover = await page.locator('canvas.ascii-shark').evaluate(canvas => ({ css: canvas.getBoundingClientRect().width, pixels: canvas.width, dpr: Math.min(2,devicePixelRatio) }));
    if (stage !== 'before') {
      check(sample.coverResizes <= 3, `${entity.kind}: buffers rebuilt at most three times during opening (${sample.coverResizes})`);
      check(Math.abs(sample.cover.pixels - sample.cover.css * sample.cover.dpr) <= 1, `${entity.kind}: settled cover retains pixel resolution`);
      check(await side.evaluate(panel => getComputedStyle(panel).transform === 'none'), `${entity.kind}: settled panel keeps viewport positioning for fixed menus`);
    }
    samples.push({ kind: entity.kind, ...sample });
    await page.screenshot({ path: `output/playwright/side-peek-performance/${stage}-${browser}-${entity.kind}.png` });
    await page.evaluate(() => window.__countPeek = false);
    await side.getByRole('button', { name: 'Close peek', exact: true }).click();
    if (stage !== 'before') { const guard = page.getByRole('dialog', { name: 'You have unsaved changes' }); await guard.waitFor(); await guard.getByRole('button', { name: 'Discard', exact: true }).click(); }
    await side.waitFor({ state: 'detached' }); await page.waitForTimeout(450);
    check((await control('get', { id: entity.id })).title === entity.title, `${entity.kind}: stored original unchanged`);
  }
  if (stage !== 'before') {
    const open = async entity => {
      await page.locator(`.home-item[data-entity-id="${entity.id}"] .home-open`).first().evaluate(button => button.click());
      await page.waitForFunction(title => document.querySelector('.peek-side .title-input')?.value === title && !document.querySelector('.peek-side .title-input')?.disabled, entity.title);
    };
    await page.evaluate(() => { window.__holdLateExternal = true; });
    await open(entities[0]); await page.waitForFunction(() => !!window.__releaseLateExternal);
    await open(entities[1]); await open(entities[0]);
    await page.evaluate(() => window.__releaseLateExternal()); await page.waitForTimeout(120);
    check(!(await page.locator('.peek-side').innerText()).includes('Stale external failure'), 'A → B → A ignores an obsolete metadata error');
    check((await page.locator('.peek-side .noteka-document').innerText()).includes('Original synthetic note body'), 'Late metadata keeps the current fresh document');
    await page.locator('.peek-side').getByRole('button', { name: 'Close peek', exact: true }).click(); await page.locator('.peek-side').waitFor({ state: 'detached' });
  }
  check(!errors.length, `No frontend errors: ${JSON.stringify(errors)}`);
  await page.context().tracing.stop({ path: `output/playwright/side-peek-performance/${stage}-${browser}-trace.zip` });
  await control('finish', { ok: true, checks, samples }); return { checks: checks.length, samples };
}
