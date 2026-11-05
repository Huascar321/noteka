async (page) => {
  page.setDefaultTimeout(60000);
  const origin = new URL(page.url()).origin;
  const size = Number(new URLSearchParams(new URL(page.url()).hash.slice(1)).get('refreshBenchmark'));
  await page.route('**/*', route => new URL(route.request().url()).origin === origin ? route.continue() : route.abort());
  await page.addInitScript(() => {
    let next = 1;
    window.isTauri = true;
    window.__refreshBench = { calls: 0, active: 0, maximum: 0 };
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback() { return next++; }, unregisterCallback() {},
      async invoke(command, args = {}) {
        if (command === 'operate') {
          const home = args.request.command.operation === 'home.query';
          if (home) {
            window.__refreshBench.calls++;
            window.__refreshBench.active++;
            window.__refreshBench.maximum = Math.max(window.__refreshBench.maximum, window.__refreshBench.active);
          }
          try { return await (await fetch('/__test__/operate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args.request) })).json(); }
          finally { if (home) window.__refreshBench.active--; }
        }
        if (['pending_edits', 'close_confirmed', 'plugin:event|listen', 'plugin:event|unlisten'].includes(command)) return next++;
        throw Error('Unexpected IPC: ' + command);
      }
    };
  });
  const control = async (step, input = {}) => {
    const response = await page.request.post(origin + '/__test__/control', { data: { step, input } });
    if (!response.ok()) throw Error(step + ': ' + await response.text());
    return response.json();
  };
  const settled = async () => {
    await page.waitForFunction(() => {
      const sections = [...document.querySelectorAll('.home [data-home-section]')];
      return sections.length === 6 && sections.every(section => section.getAttribute('aria-busy') === 'false') && window.__refreshBench.active === 0;
    });
    await page.waitForTimeout(40);
    await page.waitForFunction(() => window.__refreshBench.active === 0 && [...document.querySelectorAll('.home [data-home-section]')].every(section => section.getAttribute('aria-busy') === 'false'));
  };
  await control('configure', { theme: 'dark', phase: 'main' });
  const info = await control('init');
  await page.goto(`${origin}/#refreshBenchmark=${size}`);
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await page.locator('#workspace-path').fill(info.path);
  await page.getByRole('button', { name: 'Create workspace', exact: true }).click();
  await page.locator('.workspace-tab-panel.active .home').waitFor();
  await settled();
  const fixture = await control('refresh_benchmark_seed', { total: size });
  async function cycle() {
    const before = await page.evaluate(() => {
      window.__refreshBench.maximum = 0;
      const before = window.__refreshBench.calls;
      for (let i = 0; i < 20; i++) window.dispatchEvent(new Event('focus'));
      return before;
    });
    const start = Date.now();
    await page.waitForFunction(before => window.__refreshBench.calls > before, before);
    await settled();
    const end = await page.evaluate(() => ({ ...window.__refreshBench, heap: performance.memory?.usedJSHeapSize ?? null }));
    return { elapsed_ms: Date.now() - start, calls: end.calls - before, max_in_flight: end.maximum, js_heap_bytes: end.heap };
  }
  for (let i = 0; i < 3; i++) await cycle();
  const samples = [];
  for (let i = 0; i < 30; i++) samples.push(await cycle());
  const sorted = samples.map(sample => sample.elapsed_ms).sort((a, b) => a - b);
  await control('finish', { ok: true, checks: ['synthetic workspace', '30 focus bursts settled'], benchmark: {
    ...fixture, warmups: 3, events_per_cycle: 20, p50_ms: sorted[14], p95_ms: sorted[28],
    min_calls: Math.min(...samples.map(sample => sample.calls)), max_calls: Math.max(...samples.map(sample => sample.calls)),
    max_in_flight: Math.max(...samples.map(sample => sample.max_in_flight)), samples
  } });
}
