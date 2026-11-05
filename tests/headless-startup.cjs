// Verifica el arranque: primer uso pide workspace, la ruta se guarda y un
// reinicio reabre sin pasar por la selección. Sólo transporte headless.
async (page) => {
  page.setDefaultTimeout(15000);
  const origin = new URL(page.url()).origin, errors = [], checks = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.addInitScript(() => {
    let next = 1; window.isTauri = true;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback() { return next++; }, unregisterCallback() {},
      async invoke(command, args = {}) {
        if (command === 'operate') {
          const response = await fetch('/__test__/operate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args.request) });
          return response.json();
        }
        if (['pending_edits', 'close_confirmed', 'plugin:event|listen', 'plugin:event|unlisten'].includes(command)) return next++;
        throw Error('Unexpected IPC: ' + command);
      }
    };
  });
  const control = async (step, input = {}) => {
    const response = await page.request.post(origin + '/__test__/control', { data: { step, input } });
    if (!response.ok()) throw Error(await response.text());
    return response.json();
  };
  const check = (value, message) => { if (!value) throw Error(message); checks.push(message); };
  const sections = () => page.locator('nav[aria-label="Sections"]');
  const home = () => page.locator('.workspace-tab-panel.active .home');
  await control('configure', { theme: 'dark', phase: 'main' });
  const info = await control('init');
  await page.goto(origin); await page.evaluate(() => localStorage.clear()); await page.reload();
  await page.locator('#workspace-path').waitFor();
  check(await sections().count() === 0, 'primer arranque sin workspace muestra la selección explícita');
  await page.locator('#workspace-path').fill(info.path);
  await page.getByRole('button', { name: 'Create workspace', exact: true }).click();
  await home().waitFor();
  check(await page.evaluate(() => localStorage.getItem('noteka.workspace-path.v1')) === info.path, 'la ruta se guarda al crear el workspace');
  await page.reload();
  await home().waitFor();
  check(await page.locator('#workspace-path').count() === 0, 'un reinicio reabre el workspace guardado sin pedir ruta');
  check(await sections().count() === 1, 'el reinicio monta el lateral del workspace abierto');
  await page.evaluate(() => localStorage.removeItem('noteka.workspace-path.v1'));
  await page.reload();
  await page.locator('#workspace-path').waitFor();
  check(await sections().count() === 0, 'sin preferencia ni workspace predeterminado vuelve a la selección');
  await page.locator('#workspace-path').fill(info.path + '-missing');
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByText('La ruta no contiene').waitFor();
  check(await sections().count() === 0, 'una ruta inválida no presenta datos de otro workspace');
  check(errors.length === 0, 'sin errores de página durante el arranque');
  const result = { ok: true, checks, errors, transport: 'headless-cli-not-native-ipc' };
  await control('finish', result);
  return result;
}
