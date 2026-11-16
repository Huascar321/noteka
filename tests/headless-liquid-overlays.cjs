async page => {
  page.setDefaultTimeout(12000);
  const origin = new URL(page.url()).origin, checks = [], errors = [], external = [];
  let stage = 'setup';
  const check = (condition, message) => { if (!condition) throw Error(message); checks.push(message); };
  page.on('pageerror', error => errors.push(String(error)));
  await page.context().route('**/*', async route => {
    if (new URL(route.request().url()).origin === origin) await route.continue();
    else { external.push(route.request().url()); await route.abort(); }
  });
  await page.addInitScript(() => {
    let next = 1;
    localStorage.setItem('noteka.peek-mode.v1', 'side');
    window.isTauri = true;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = { transformCallback() { return next++; }, unregisterCallback() {}, async invoke(command, args = {}) {
      if (command === 'operate') return (await fetch('/__test__/operate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args.request) })).json();
      if (['pending_edits', 'close_confirmed', 'plugin:event|listen', 'plugin:event|unlisten'].includes(command)) return next++;
      throw Error('Unexpected IPC ' + command);
    } };
    const animate = Element.prototype.animate;
    window.__overlayAnimations = [];
    Element.prototype.animate = function (...args) { const animation = animate.apply(this, args); window.__overlayAnimations.push(animation); return animation; };
  });
  const control = async (step, input = {}) => { const result = await page.request.post(origin + '/__test__/control', { data: { step, input } }); if (!result.ok()) throw Error(await result.text()); return result.json(); };
  const wait = () => page.waitForTimeout(500);
  const sidebar = async () => { if (await page.locator('.desktop-sidebar').getAttribute('aria-hidden') === 'true') await page.getByRole('button', { name: 'Expand sidebar', exact: true }).click(); };
  const pane = () => page.locator('.workspace-tab-panel.peek:not(.peek-hidden):visible').last();
  const close = async () => { await page.keyboard.press('Escape'); await page.waitForTimeout(300); };
  const probe = async (surface, label) => {
    await surface.waitFor();
    const result = await surface.evaluate(node => {
      const glass = node.querySelector(':scope > .liquid-morph-glass'), animation = glass?.getAnimations()[0];
      return { exists: !!glass, topLayer: glass?.matches(':popover-open'), inline: !!glass?.classList.contains('liquid-morph-inline'), duration: animation?.effect.getTiming().duration, transform: getComputedStyle(node).transform, first: animation?.effect.getKeyframes()[0]?.transform };
    });
    const expectedDuration = result.inline ? 400 : 160;
    check(result.exists && (result.topLayer || result.inline) && result.duration === expectedDuration, `${label}: ${expectedDuration} ms glass morph in its surface layer (${JSON.stringify(result)})`);
    check(result.transform === 'none', `${label}: content owner never scales`);
    await surface.evaluate(async node => { const start=performance.now(); while(node.querySelector('.liquid-morph-glass') && performance.now()-start<2500) await new Promise(resolve=>setTimeout(resolve,30)); });
    check(await surface.locator('.liquid-morph-glass').count() === 0, `${label}: releases glass after settling (${await surface.evaluate(node=>JSON.stringify([...node.querySelectorAll('.liquid-morph-glass')].map(el=>el.getAnimations().map(a=>({id:a.id,state:a.playState,time:a.currentTime})))) )})`);
  };
  try {
    await control('configure', { theme: 'light', phase: 'main' });
    const info = await control('init');
    await page.setViewportSize({ width: 1440, height: 900 }); await page.goto(origin);
    await page.getByLabel('Workspace path').fill(info.path); await page.getByRole('button', { name: 'Create workspace', exact: true }).click(); await page.locator('.home').waitFor();
    const note = await control('entity_create', { kind: 'note', title: 'Liquid fixture', document: { schema_version: 2, blocks: [{ id: 'liquid-paragraph', type: 'paragraph', content: [{ type: 'text', text: 'Liquid text for editor controls' }] }] } });
    await control('entity_create', { kind: 'tag', title: 'Liquid tag', payload: { classification: 'topic' } });

    stage = 'search'; await sidebar(); await page.getByRole('button', { name: 'Search', exact: true }).click();
    const search = page.getByRole('dialog', { name: 'Search workspace', exact: true });
    await probe(search, 'Search');
    await search.getByRole('combobox').fill('Liquid fixture'); await page.waitForFunction(() => document.querySelector('#context-search-results')?.getAttribute('aria-busy') === 'false');
    await search.getByRole('option').filter({ hasText: 'Liquid fixture' }).first().click();
    await pane().getByLabel('Note title').waitFor(); await wait();
    check(await page.locator('.context-search').count() === 0, 'Search selection completes its modal outro');
    check(await pane().evaluate(node=>node.contains(document.activeElement)), 'Search selection gives focus to the destination after modal teardown');

    stage = 'page actions'; const actions = pane().getByRole('button', { name: 'Page actions', exact: true });
    await actions.click(); const menu = pane().getByRole('menu', { name: 'Page actions', exact: true });
    await probe(menu, 'Page actions');
    await menu.getByRole('menuitem', { name: 'External resources', exact: true }).click();
    const resources = pane().locator('.page-tool-panel:not([hidden])'); await probe(resources, 'PageTools');
    await resources.evaluate(node => { node.dataset.liquidIdentity = 'preserved'; });
    const draft = resources.locator('input').first();
    if (await draft.count()) await draft.fill('liquid draft preserved');
    await resources.getByRole('button', { name: 'Close External resources', exact: true }).click(); await page.waitForTimeout(300);
    check(await pane().locator('.page-tool-panel:not([hidden])').count() === 0, 'Persistent PageTools closes visually and semantically');
    await actions.click(); await pane().getByRole('menuitem', { name: 'External resources', exact: true }).click(); await wait();
    check(await resources.getAttribute('data-liquid-identity') === 'preserved', 'PageTools retains its mounted DOM instance');
    if (await draft.count()) check(await draft.inputValue() === 'liquid draft preserved', 'PageTools preserves its input buffer across the morph');
    await resources.getByRole('button', { name: 'Close External resources', exact: true }).click(); await page.waitForTimeout(300);

    stage = 'peek menu'; await pane().getByRole('button', { name: 'Switch peek mode', exact: true }).click();
    const peek = pane().getByRole('menu', { name: 'Peek mode', exact: true }); await probe(peek, 'Peek modes'); await close();
    check(await pane().locator('.peek-mode-menu').count() === 0, 'Peek menu closes without closing the detail');

    stage = 'early close';
    const interrupted = await actions.evaluate(async trigger => {
      trigger.click(); await new Promise(requestAnimationFrame); await new Promise(requestAnimationFrame);
      const host = document.querySelector('.page-actions-menu');
      for (const a of window.__overlayAnimations.filter(a => a.effect?.target === host && !a.id)) a.pause();
      for (const a of host.getAnimations({ subtree: true })) a.pause();
      await new Promise(requestAnimationFrame);
      const glass = host.querySelector(':scope > .liquid-morph-glass');
      glass.getAnimations()[0].currentTime = 90;
      return glass.getBoundingClientRect().toJSON();
    });
    await page.keyboard.press('Escape');
    const departing = await menu.evaluate(async host => {
      await new Promise(requestAnimationFrame);
      for (const a of window.__overlayAnimations.filter(a => a.effect?.target === host && !a.id)) a.pause();
      for (const a of host.getAnimations({ subtree: true })) a.pause();
      await new Promise(requestAnimationFrame);
      const glass = host.querySelector(':scope > .liquid-morph-glass'), a = glass.getAnimations()[0]; a.currentTime = 0;
      return { rect: glass.getBoundingClientRect().toJSON(), id: a.id, duration: a.effect.getTiming().duration, inert: host.inert };
    });
    check(departing.id === 'liquid-overlay-out' && departing.duration === 200 && departing.inert, 'Closing menu is inert and uses the 200 ms outro');
    check(Math.abs(departing.rect.width - interrupted.width) < 2, 'Early close begins at the interrupted glass geometry');
    await menu.evaluate(host => { for (const a of window.__overlayAnimations.filter(a => a.effect?.target === host && !a.id)) { try { a.finish(); } catch {} } });
    await page.waitForTimeout(100);

    stage = 'icon'; await pane().getByRole('button', { name: 'Choose icon and color', exact: true }).click();
    const icon = pane().getByRole('dialog', { name: 'Page icon', exact: true }); await probe(icon, 'Icon picker'); await close();

    stage = 'relations'; await pane().getByRole('button', { name: 'Edit note tags', exact: true }).click();
    const relations = pane().getByRole('dialog', { name: 'Edit tags', exact: true }); await probe(relations, 'Relations'); await close();
    check(await pane().getByLabel('Note title').inputValue() === 'Liquid fixture', 'Picker motion preserves the entity draft');

    stage = 'editor'; const editor = pane().getByRole('textbox', { name: 'Note content', exact: true });
    await editor.click(); await page.keyboard.press('End'); await page.keyboard.type('/');
    const commands = pane().getByRole('listbox', { name: 'Block commands', exact: true }); await probe(commands, 'Block commands'); await close();
    await page.keyboard.press('Backspace');

    stage = 'settings'; await sidebar(); await page.getByRole('button', { name: 'Settings', exact: true }).click();
    const settings = page.getByRole('dialog', { name: 'Settings', exact: true }); await probe(settings, 'Settings');
    await page.keyboard.press('Tab'); check(await settings.evaluate(node => node.contains(document.activeElement)), 'Settings keeps keyboard focus inside the modal');
    await settings.getByRole('button', { name: 'Violet', exact: true }).click(); await settings.getByRole('button', { name: 'Close Settings', exact: true }).click();
    const modalExit = await settings.evaluate(node => ({ open: node.open, inert: node.inert, id: node.querySelector(':scope > .liquid-morph-glass')?.getAnimations()[0]?.id }));
    check(modalExit.open && modalExit.inert && modalExit.id === 'liquid-overlay-out', `Modal remains in top layer and inert through its outro (${JSON.stringify(modalExit)})`);
    await page.waitForTimeout(300);
    check(await page.locator('#settings-dialog').count() === 0, 'Settings closes after its transition');

    stage = 'guard'; await pane().getByLabel('Note title').fill('Liquid fixture unsaved');
    const guard = page.getByRole('dialog', { name: 'You have unsaved changes', exact: true });
    const previewFrames = [];
    for (const sample of [0, 100]) {
      await pane().getByRole('button', { name: 'Close peek', exact: true }).click(); await guard.waitFor();
      const frame = await guard.evaluate(async (host, targetTime) => {
        const deadline = performance.now() + 2500;
        let glass = host.querySelector(':scope > .liquid-morph-glass');
        while (!glass || Number(glass.getAnimations()[0]?.currentTime ?? 0) < targetTime) {
          if (performance.now() > deadline) throw Error(`Preview frame ${targetTime} ms was not reached`);
          await new Promise(requestAnimationFrame); glass = host.querySelector(':scope > .liquid-morph-glass');
        }
        for (const a of window.__overlayAnimations.filter(a => a.effect?.target === host && !a.id)) a.pause();
        const heading = host.querySelector('h2'), style = getComputedStyle(heading);
        return { time: glass.getAnimations()[0].currentTime, opacity: Number(style.opacity), transform: style.transform, clip: style.clipPath, layer: getComputedStyle(glass).zIndex, contentLayers: host.querySelectorAll('h2').length };
      }, sample);
      previewFrames.push(frame);
      check(frame.opacity > .1 && frame.transform === 'none' && frame.clip !== 'none', `Dialog reveals real, unscaled content during entry (${JSON.stringify(frame)})`);
      check(frame.layer === '-1' && frame.contentLayers === 1, 'Preview glass stays behind one copy of the content');
      const browser = page.context().browser().browserType().name();
      await guard.screenshot({ path: `output/playwright/liquid-overlays/${browser}/guard-preview-${sample}.png`, animations: 'allow' });
      await guard.evaluate(host => { for (const a of window.__overlayAnimations.filter(a => a.effect?.target === host && !a.id)) { try { a.finish(); } catch {} } });
      await guard.getByRole('button', { name: 'Cancel', exact: true }).click(); await page.waitForTimeout(300);
    }
    check(previewFrames[1].opacity > previewFrames[0].opacity, 'Dialog content gains clarity continuously before the material settles');
    await pane().getByRole('button', { name: 'Close peek', exact: true }).click(); await probe(guard, 'Save guard');
    check(await guard.locator('h2').evaluate(node => node.style.clipPath === '' && getComputedStyle(node).opacity === '1'), 'Settled content restores its original clipping and full opacity');
    await guard.getByRole('button', { name: 'Cancel', exact: true }).click(); await page.waitForTimeout(300);
    check(await pane().getByLabel('Note title').inputValue() === 'Liquid fixture unsaved', 'Cancel guard preserves the unsaved buffer');

    stage = 'center peek'; await pane().getByRole('button', { name: 'Switch peek mode', exact: true }).click();
    await pane().getByRole('menuitemradio', { name: 'Center peek', exact: true }).click();
    await probe(page.locator('.peek-center'), 'Center peek');
    check(await pane().getByLabel('Note title').inputValue() === 'Liquid fixture unsaved', 'Center peek preserves the same editor buffer');

    await pane().getByRole('button', { name: 'Close peek', exact: true }).click();
    await guard.getByRole('button', { name: 'Discard', exact: true }).click(); await page.waitForFunction(()=>!document.querySelector('.peek-center'),null,{timeout:3000});
    check(await page.locator('.peek-center').count() === 0, 'Center peek closes after discarding the synthetic draft');

    stage = 'section menus'; await sidebar();
    for (const section of ['Tasks', 'Notes', 'Projects', 'Tags', 'Goals', 'People']) {
      await page.getByRole('navigation', { name: 'Sections', exact: true }).getByRole('button', { name: section, exact: true }).click();
      const root = page.locator('.workspace-tab-panel.active');
      if (!['Tasks', 'Notes'].includes(section)) {
        const nav = root.locator('.tasks-page-nav').first(); await nav.locator('summary').click();
        await probe(nav.locator('.quick-nav-options'), `${section} Nav`); await close();
        check(await nav.locator('.quick-nav-options').count() === 0, `${section}: Nav closes without navigation`);
      }
    }

    stage = 'themes and materials';
    const browser = page.context().browser().browserType().name();
    for (const theme of ['light', 'dark']) for (const accent of ['Blue', 'Violet']) {
      await page.getByRole('button', { name: 'Settings', exact: true }).click(); await wait();
      await settings.getByLabel('Theme', { exact: true }).selectOption(theme);
      await settings.getByRole('button', { name: accent, exact: true }).click();
      await settings.screenshot({ path: `output/playwright/liquid-overlays/${browser}/settings-${theme}-${accent}.png` });
      check(await settings.evaluate(node => getComputedStyle(node).transform === 'none'), `${theme}/${accent}: settled dialog keeps its natural geometry`);
      await settings.getByRole('button', { name: 'Close Settings', exact: true }).click(); await page.waitForTimeout(300);
    }
    stage = 'narrow'; await page.setViewportSize({ width: 420, height: 740 });
    await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
    await sidebar();
    await page.getByRole('button', { name: 'Settings', exact: true }).click(); await wait();
    check(await settings.evaluate(node => { const r = node.getBoundingClientRect(); return r.x >= 0 && r.right <= innerWidth + 1 && r.y >= 0 && r.bottom <= innerHeight + 1; }), 'Settings fits at 420 px');
    await settings.screenshot({ path: `output/playwright/liquid-overlays/${browser}/settings-narrow.png` }); await close();
    await page.setViewportSize({ width: 1440, height: 900 }); await sidebar();
    stage = 'contrast'; await page.emulateMedia({ contrast: 'more' });
    await page.getByRole('button', { name: 'Settings', exact: true }).click(); await wait();
    check(await settings.evaluate(node => { const s = getComputedStyle(node); return s.backdropFilter === 'none' && s.backgroundImage === 'none'; }), 'High contrast retains the solid material without blur or sheen'); await close();
    await page.emulateMedia({ contrast: 'no-preference' });
    stage = 'dynamic preference'; await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await page.emulateMedia({ reducedMotion: 'reduce' }); await page.waitForTimeout(50);
    check(await settings.locator('.liquid-morph-glass').count() === 0, 'Changing Reduce Motion cancels an active morph'); await close();

    stage = 'reduced motion'; await page.emulateMedia({ reducedMotion: 'reduce' });
    await sidebar(); await page.getByRole('button', { name: 'Settings', exact: true }).click(); await settings.waitFor();
    check(await settings.locator('.liquid-morph-glass').count() === 0, 'Reduce Motion omits decorative material');
    await settings.getByRole('button', { name: 'Close Settings', exact: true }).click(); await page.waitForTimeout(40);
    check(await page.locator('#settings-dialog').count() === 0, 'Reduce Motion closes immediately');
    await page.emulateMedia({ reducedMotion: 'no-preference' });

    check(await page.locator('.liquid-morph-glass').count() === 0, 'No decorative top-layer surfaces remain after closing');
    check(errors.length === 0 && external.length === 0, `No page errors or external requests: ${JSON.stringify({ errors, external })}`);
    await control('finish', { ok: true, checks }); return { ok: true, count: checks.length };
  } catch (error) { const state=await page.evaluate(()=>({errors:[...document.querySelectorAll('.error')].map(el=>el.textContent),panes:[...document.querySelectorAll('.workspace-tab-panel')].map(el=>({classes:el.className,hidden:el.hidden,inert:el.inert,rect:el.getBoundingClientRect().toJSON()})),dialogs:[...document.querySelectorAll('dialog')].map(el=>({classes:el.className,open:el.open})),titles:[...document.querySelectorAll('.title-input')].map(el=>({value:el.value,disabled:el.disabled}))}));throw Error(`${stage}: ${error.stack || error}\n${JSON.stringify({state,errors})}`); }
}
