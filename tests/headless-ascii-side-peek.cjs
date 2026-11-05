async (page) => {
  page.setDefaultTimeout(15000);
  const origin = new URL(page.url()).origin, checks = [], errors = [], external = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.context().route('**/*', async route => { const url = new URL(route.request().url()); if (url.origin === origin) await route.continue(); else { external.push(url.origin); await route.abort(); } });
  await page.addInitScript(() => {
    let next = 1; window.isTauri = true; window.__homeQueries = 0;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback() { return next++; }, unregisterCallback() {},
      async invoke(command, args = {}) {
        if (command === 'operate') {
          if (args.request.command.operation === 'home.query') window.__homeQueries++;
          return (await fetch('/__test__/operate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args.request) })).json();
        }
        if (['pending_edits', 'close_confirmed', 'plugin:event|listen', 'plugin:event|unlisten'].includes(command)) return next++;
        throw Error('Unexpected IPC: ' + command);
      }
    };
  });
  const control = async (step, input = {}) => { const response = await page.request.post(origin + '/__test__/control', { data: { step, input } }); if (!response.ok()) throw Error(await response.text()); return response.json(); };
  const check = (condition, label) => { if (!condition) throw Error(label); checks.push(label); };
  const canvas = page.locator('.workspace-tab-panel.active canvas.ascii-shark');
  const frame = () => canvas.getAttribute('data-frame');
  const advances = async () => { const before = await frame(); await page.waitForFunction(value => document.querySelector('.workspace-tab-panel.active canvas.ascii-shark')?.dataset.frame !== value, before); };
  const stable = async () => { await page.waitForTimeout(220); const before = await frame(); await page.waitForTimeout(300); return (await frame()) === before; };
  const search = async (name, newTab = false) => {
    await page.getByRole('button', { name: newTab ? 'New tab' : 'Search', exact: true }).click();
    await page.getByRole('combobox', { name: 'Search workspace' }).fill(name);
    await page.waitForFunction(() => document.querySelector('#context-search-results')?.getAttribute('aria-busy') === 'false');
    await page.getByRole('option').filter({ has: page.locator('strong', { hasText: name }) }).first().click();
  };
  const mode = async name => {
    await page.locator('.peek').getByRole('button', { name: 'Switch peek mode' }).click();
    await page.getByRole('menuitemradio', { name }).click();
  };
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await control('configure', { theme: 'dark', phase: 'main' }); const info = await control('init');
  await page.goto(origin); await page.evaluate(() => { localStorage.clear(); localStorage.setItem('noteka.peek-mode.v1', 'side'); localStorage.setItem('noteka.home-cover.v1', 'ascii'); }); await page.reload();
  await page.locator('#workspace-path').fill(info.path); await page.getByRole('button', { name: 'Create workspace', exact: true }).click();
  await page.locator('.workspace-tab-panel.active .home').waitFor();
  const note = await control('entity_create', { kind: 'note', title: 'Nota lateral sintética', document: { schema_version: 2, blocks: [{ id: 'original', type: 'paragraph', content: [{ type: 'text', text: 'Texto original' }] }] } });
  await control('entity_create', { kind: 'note', title: 'Segunda nota sintética', document: { schema_version: 2, blocks: [{ id: 'second', type: 'paragraph', content: [] }] } });
  await page.waitForFunction(() => Number(document.querySelector('canvas.ascii-shark')?.dataset.frame) > 2);
  const breadcrumb = await page.locator('.workspace-tab-panel.active .home-pane').evaluate(pane => {
    const bar = pane.querySelector('.overview-breadcrumb-bar'), capsule = bar.querySelector('[aria-current="page"]'), canvas = pane.querySelector('canvas.ascii-shark');
    const pill = capsule.getBoundingClientRect(), cover = canvas.getBoundingClientRect(), style = getComputedStyle(capsule);
    return { bar: getComputedStyle(bar).backgroundColor, overlap: pill.top >= cover.top && pill.bottom < cover.bottom && pill.left >= cover.left, filter: style.backdropFilter };
  });
  check(breadcrumb.bar === 'rgba(0, 0, 0, 0)' && breadcrumb.overlap && (breadcrumb.filter === 'none' || breadcrumb.filter.includes('blur(8px)')), `Home breadcrumb overlays the ASCII without a solid bar: ${JSON.stringify(breadcrumb)}`);
  await page.emulateMedia({ contrast: 'more' });
  check(await page.locator('.workspace-tab-panel.active .page-breadcrumb [aria-current="page"]').evaluate(el => getComputedStyle(el).backdropFilter === 'none'), 'breadcrumb becomes solid with increased contrast');
  await page.emulateMedia({ contrast: 'no-preference' });
  await page.locator('.desktop-sidebar nav').getByRole('button', { name: 'Tasks', exact: true }).click();
  await page.locator('.workspace-tab-panel.active .tasks-page').waitFor();
  const sectionBar = await page.locator('.workspace-tab-panel.active .overview-breadcrumb-bar').evaluate(el => ({ color: getComputedStyle(el).backgroundColor, image: getComputedStyle(el).backgroundImage }));
  check(sectionBar.color === 'rgba(0, 0, 0, 0)' && sectionBar.image === 'none', `section breadcrumb integrates with its background: ${JSON.stringify(sectionBar)}`);
  await page.locator('.workspace-tab-panel.active .overview-breadcrumb-bar').getByRole('button', { name: 'Home', exact: true }).click();
  await page.locator('.workspace-tab-panel.active .home').waitFor();
  await page.waitForFunction(() => Number(document.querySelector('.workspace-tab-panel.active canvas.ascii-shark')?.dataset.frame) > 2);
  const backgrounds = await page.locator('.workspace-tab-panel.active .home').evaluate(home => {
    const root = document.documentElement, cover = home.querySelector('.home-cover');
    const inspect = () => ({ cover: getComputedStyle(cover).backgroundColor, home: getComputedStyle(home).backgroundImage, bg: getComputedStyle(root).getPropertyValue('--bg').trim() });
    const dark = inspect(); root.dataset.theme = 'light'; const light = inspect(); root.dataset.theme = 'dark';
    cover.dataset.cover = 'photo'; const photo = getComputedStyle(cover).backgroundColor; cover.dataset.cover = 'ascii';
    return { dark, light, photo };
  });
  check(backgrounds.dark.cover === 'rgba(0, 0, 0, 0)' && backgrounds.dark.home.includes('gradient') && backgrounds.light.cover === 'rgba(0, 0, 0, 0)' && backgrounds.light.home === 'none' && backgrounds.dark.bg !== backgrounds.light.bg && backgrounds.photo !== 'rgba(0, 0, 0, 0)', `ASCII and Home share their background in both themes: ${JSON.stringify(backgrounds)}`);
  await canvas.evaluate(el => { window.__originalCanvas = el; });
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await page.getByRole('combobox', { name: 'Search workspace' }).waitFor();
  check(await stable(), 'ASCII pauses while global search blocks Home');
  const glass = await page.locator('.context-search').evaluate(el => ({ surface: getComputedStyle(el).backdropFilter, backdrop: getComputedStyle(el, '::backdrop').backdropFilter }));
  check(glass.backdrop === 'none' && (glass.surface === 'none' || glass.surface === 'blur(10px)'), `search blur is bounded to dialog: ${JSON.stringify(glass)}`);
  const contrast = await page.locator('.context-search').evaluate(el => {
    const ctx = document.createElement('canvas').getContext('2d');
    const pixel = value => { ctx.clearRect(0, 0, 1, 1); ctx.fillStyle = value; ctx.fillRect(0, 0, 1, 1); return [...ctx.getImageData(0, 0, 1, 1).data]; };
    const background = pixel(getComputedStyle(el).backgroundColor), ink = pixel(getComputedStyle(el.querySelector('.context-results-label')).color);
    const lum = rgb => rgb.map(v => v / 255).map(c => c <= .04045 ? c / 12.92 : ((c + .055) / 1.055) ** 2.4).reduce((sum, c, i) => sum + c * [.2126, .7152, .0722][i], 0);
    const blended = value => background.slice(0, 3).map(c => c * background[3] / 255 + value * (1 - background[3] / 255));
    const ratio = rgb => { const a = lum(ink.slice(0, 3)), b = lum(rgb); return (Math.max(a, b) + .05) / (Math.min(a, b) + .05); };
    return { alpha: background[3] / 255, worst: Math.min(ratio(blended(0)), ratio(blended(204))) };
  });
  check(contrast.worst >= 4.5 && (glass.surface === 'none' || contrast.alpha < .8), `dark search is translucent and legible against backdrop extremes: ${JSON.stringify(contrast)}`);
  await page.getByRole('button', { name: 'Close search' }).click();
  await advances(); check(true, 'ASCII resumes after global search closes');
  await search('Nota lateral sintética');
  const side = page.locator('.peek-side'); await side.locator('.title-input').waitFor(); await page.waitForTimeout(450);
  const detailNav = await side.locator('.editor-bar').evaluate(el => ({ bar: getComputedStyle(el).backgroundColor, image: getComputedStyle(el).backgroundImage, actions: getComputedStyle(el.querySelector(':scope > .actions')).backdropFilter }));
  check(detailNav.bar === 'rgba(0, 0, 0, 0)' && detailNav.image === 'none' && detailNav.actions === 'none', `detail breadcrumb integrates with page while actions keep menus correctly positioned: ${JSON.stringify(detailNav)}`);
  await side.locator('.title-input').fill('Borrador en Side');
  check(await page.evaluate(() => document.activeElement?.closest('.peek-side') !== null), 'note holds keyboard focus');
  await advances(); check(true, 'ASCII advances while Side peek is focused');
  check(await canvas.evaluate(el => el === window.__originalCanvas), 'same canvas and scene instance retained');
  const beforeCount = await page.evaluate(() => window.__homeQueries);
  const consumed = await page.evaluate(() => { const event = new KeyboardEvent('keydown', { key: 'r', metaKey: true, bubbles: true, cancelable: true }); document.activeElement.dispatchEvent(event); return event.defaultPrevented; });
  await page.waitForTimeout(100);
  check(!consumed && await page.evaluate(() => window.__homeQueries) === beforeCount, 'unfocused Home does not consume its refresh shortcut');
  check((await control('get', { id: note.id })).title === 'Nota lateral sintética' && await side.locator('.title-input').inputValue() === 'Borrador en Side', 'animation preserves unsaved note and stored original');
  const splitter = page.getByRole('separator', { name: 'Resize side peek' }); await splitter.focus(); await page.keyboard.press('ArrowRight'); await page.waitForTimeout(350); await advances();
  check(await side.locator('.title-input').inputValue() === 'Borrador en Side', 'resize keeps animation and draft');
  await page.locator('.workspace-tab-panel.active .home').evaluate(el => { el.scrollTop = el.scrollHeight; });
  check(await stable(), 'offscreen cover pauses with Side peek');
  await page.locator('.workspace-tab-panel.active .home').evaluate(el => { el.scrollTop = 0; }); await advances(); check(true, 'cover resumes when visible');
  await page.emulateMedia({ reducedMotion: 'reduce' }); await page.waitForFunction(() => matchMedia('(prefers-reduced-motion: reduce)').matches); await page.waitForFunction(() => document.querySelector('.workspace-tab-panel.active canvas.ascii-shark')?.dataset.frame === '0');
  check(await stable(), 'reduced motion remains static');
  await page.emulateMedia({ reducedMotion: 'no-preference' }); await page.waitForFunction(() => !matchMedia('(prefers-reduced-motion: reduce)').matches); await advances(); check(true, 'animation resumes after reduced motion');
  await page.evaluate(() => { Object.defineProperty(document, 'hidden', { configurable: true, value: true }); document.dispatchEvent(new Event('visibilitychange')); });
  check(await stable(), 'hidden document pauses animation');
  await page.evaluate(() => { delete document.hidden; document.dispatchEvent(new Event('visibilitychange')); }); await advances(); check(true, 'document visibility restores animation');
  await mode('Center peek'); await page.locator('.peek-center .title-input').waitFor();
  check(await stable(), 'Center peek keeps prior paused behavior');
  await mode('Full page'); await page.locator('.peek-full .title-input').waitFor();
  check(await stable(), 'Full page keeps prior paused behavior');
  await mode('Side peek'); await page.locator('.peek-side .title-input').waitFor(); await advances();
  check(await side.locator('.title-input').inputValue() === 'Borrador en Side', 'returning to Side resumes without losing draft');
  await search('Segunda nota sintética', true); await page.locator('.workspace-tab-panel.active .title-input').waitFor();
  await page.waitForTimeout(200); const hiddenBefore = await page.evaluate(() => window.__originalCanvas.dataset.frame); await page.waitForTimeout(300);
  check(await page.evaluate(() => window.__originalCanvas.dataset.frame) === hiddenBefore, 'inactive Home tab pauses animation');
  await page.getByRole('tab').first().click(); await side.locator('.title-input').waitFor(); await advances();
  check(await side.locator('.title-input').inputValue() === 'Borrador en Side', 'return to original tab resumes and preserves draft');
  check(errors.length === 0 && external.length === 0, `browser errors: ${JSON.stringify(errors)}; external requests: ${JSON.stringify(external)}`);
  await control('finish', { ok: true, checks });
}
