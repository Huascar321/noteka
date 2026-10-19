async (page) => {
  const origin = new URL(page.url()).origin, external = [], reports = [];
  await page.context().route('**/*', async route => {
    const url = new URL(route.request().url());
    if (url.origin === origin) await route.continue(); else { external.push(url.origin); await route.abort(); }
  });
  if (!await page.evaluate(() => Boolean(window.__TAURI_INTERNALS__))) {
    await page.addInitScript(() => {
      let next = 1; const callbacks = new Map();
      window.isTauri = true; // Transporte CLI de pruebas; no aceptación IPC nativa.
      window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
      window.__TAURI_INTERNALS__ = {
        transformCallback: callback => { const id = next++; callbacks.set(id, callback); return id; },
        unregisterCallback: id => callbacks.delete(id),
        invoke: async (command, args = {}) => {
          if (command === 'operate') {
            const response = await fetch('/__test__/operate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args.request) });
            const result = await response.json(); if (!response.ok) throw new Error(JSON.stringify(result)); return result;
          }
          if (command === 'pending_edits' || command === 'close_confirmed' || command === 'plugin:event|unlisten') return;
          if (command === 'plugin:event|listen') { window.__notekaClose = callbacks.get(args.handler); return next++; }
          throw new Error('IPC de prueba desconocido: ' + command);
        },
      };
    });
  }
  const control = async (step, input) => {
    const response = await page.request.post(origin + '/__test__/control', { data: { step, input } });
    const result = await response.json(); if (!response.ok()) throw new Error(JSON.stringify(result)); return result;
  };
  for (const theme of ['dark', 'light']) {
    const checks = [], expect = (condition, label) => { if (!condition) throw new Error(theme + ': ' + label); checks.push(label); };
    const f = await control('reader_seed', { theme }), workspace = f.workspace;
    const ctl = (step, input = {}) => control(step, { workspace, ...input });
    const cli = async (operation, input) => { const r = await ctl('reader_cli', { operation, input }); if (!r.ok) throw new Error(JSON.stringify(r)); return r.data; };
    const snapshot = () => ctl('reader_snapshot'), ops = () => ctl('reader_operations');
    const button = name => page.getByRole('button', { name, exact: true });
    if (f.isolated) await page.goto(origin + '/?workspace=' + encodeURIComponent(workspace) + '&theme=' + theme);
    else {
      await page.goto(origin); await page.getByLabel('Ruta del workspace').fill(workspace); await button('Abrir workspace').click();
      await page.locator('.home [data-home-section="person"] .home-coverage').waitFor();
      await page.getByLabel('Tema', { exact: true }).selectOption(theme);
      await page.getByRole('navigation', { name: 'Secciones' }).getByRole('button', { name: 'Notes', exact: true }).click();
      await page.locator('.list-items button').filter({ hasText: 'Lectura sintética' }).click();
      await page.getByLabel('Título de la nota').fill('Buffer de lector conservado');
    }
    const before = await snapshot(); await page.locator('.generation-panel > summary').click();
    expect(!(await ops()).some(o => o.operation.startsWith('extraction.')), 'Montar/abrir no consulta ni procesa');
    expect(await button('Leer desde el inicio').isDisabled(), 'ID vacío no se envía');
    const fill = async source => {
      await page.getByLabel('ID de trabajo para lectura').fill(source.job.id);
      await page.getByLabel('Tipo de lectura').selectOption(source.kind);
    };
    const settled = () => page.locator('.generation-result, .generation-error').waitFor();
    const read = async source => { await fill(source); await button('Leer desde el inicio').click(); await settled(); };
    const assertPage = async (source, expected, number) => {
      expect(await page.locator('[data-reader="page"]').textContent() === String(number), source.kind + ': número de página');
      expect(await page.locator('[data-reader="generation"]').textContent() === expected.generation.id, source.kind + ': generación exacta');
      expect(await page.locator('[data-reader="covers"]').textContent() === String(expected.covers_generation), source.kind + ': cobertura real de página');
      expect(JSON.stringify(JSON.parse(await page.locator('.generation-source').textContent())) === JSON.stringify(expected.generation), source.kind + ': procedencia completa sin campos omitidos');
      const rows = await page.locator('.generation-item').evaluateAll(nodes => nodes.map(node => ({
        ordinal: Number(node.dataset.ordinal), text: node.querySelector('.generation-text')?.textContent,
        locator: node.querySelector('.generation-locator')?.textContent,
        cached: node.querySelector('.generation-cached')?.textContent,
        formula: node.querySelector('.generation-formula')?.textContent,
        cell: node.querySelector('.generation-cell')?.textContent,
      })));
      expect(rows.length === expected.items.length && rows.length <= 20, source.kind + ': página reemplazada/acotada');
      for (let i = 0; i < rows.length; i++) {
        const item = expected.items[i], actual = rows[i]; expect(actual.ordinal === item.ordinal, source.kind + ': ordinal ' + i);
        if (source.kind === 'xlsx_cells') {
          expect(JSON.stringify(JSON.parse(actual.cell)) === JSON.stringify(item.cell), 'XLSX celda completa ' + i);
          expect(JSON.stringify(JSON.parse(actual.cached)) === JSON.stringify(item.cell.cached ?? null), 'XLSX caché tipada exacta ' + i);
          expect(JSON.stringify(JSON.parse(actual.formula)) === JSON.stringify(item.cell.formula ?? null), 'XLSX fórmula separada exacta ' + i);
          expect(JSON.stringify(JSON.parse(actual.locator)) === JSON.stringify(item.locator), 'XLSX localizador ' + i);
        } else {
          if (actual.text !== item.text) {
            let n=0; while(n<item.text.length && actual.text[n]===item.text[n]) n++;
            const dom = await page.locator('.generation-text').nth(i).evaluate(async node => ({ length:node.textContent.length, hash:[...new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(node.textContent)))].map(b=>b.toString(16).padStart(2,'0')).join('') }));
            const expectedHash = await page.evaluate(async text => [...new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text)))].map(b=>b.toString(16).padStart(2,'0')).join(''),item.text);
            throw new Error(JSON.stringify({mismatch_ordinal:item.ordinal,actual_length:actual.text.length,expected_length:item.text.length,actual_around:actual.text.slice(n-40,n+80),expected_around:item.text.slice(n-40,n+80),difference:n,dom,expected_hash:expectedHash}));
          }
          expect(actual.text === item.text, source.kind + ': texto canónico completo ' + i);
          const locator = source.kind === 'pdf_text' ? item.locator : Object.fromEntries(['byte_start','byte_end','line_start','line_end'].map(k => [k,item[k]]));
          expect(JSON.stringify(JSON.parse(actual.locator)) === JSON.stringify(locator), source.kind + ': localizador íntegro ' + i);
        }
      }
      if (expected.manifest) expect(JSON.stringify(JSON.parse(await page.locator('.generation-manifest').textContent())) === JSON.stringify(expected.manifest), source.kind + ': manifiesto completo');
      expect(await page.locator('.generation-item script, .generation-item img, .generation-item iframe').count() === 0, 'Contenido no interpretado');
    };
    for (const [name, source] of Object.entries(f.fixtures)) {
      await fill(source); expect(await page.locator('.generation-result').count() === 0, name + ': editar retira página sin consultar');
      await page.getByLabel('ID de trabajo para lectura').focus(); await page.keyboard.press('Enter'); await settled();
      for (let i = 0; i < source.pages.length; i++) {
        if (i) { await button('Siguiente página de lectura').focus(); await page.keyboard.press('Enter'); await settled(); }
        await assertPage(source, source.pages[i], i + 1);
      }
      expect(await button('Siguiente página de lectura').count() === 0, name + ': cursor nulo termina');
      expect(!(await page.evaluate(() => window.__readerExecuted)), name + ': contenido hostil no ejecuta');
      if (!f.isolated) expect(await page.getByLabel('Título de la nota').inputValue() === 'Buffer de lector conservado', name + ': buffer conservado');
    }
    expect(JSON.stringify(await snapshot()) === JSON.stringify(before), 'Recorrer todos los lectores no modifica SQLite ni originales');
    // ID/tipo incorrectos: error, no fallback ni página anterior.
    await read({ ...f.fixtures['cells.xlsx'], kind: 'utf8_text' });
    expect(await page.locator('.generation-error').count() === 1 && await page.locator('.generation-item').count() === 0, 'Tipo ajeno falla sin fallback');
    await page.getByLabel('ID de trabajo para lectura').fill('invalid-id'); await button('Leer desde el inicio').click(); await settled();
    expect((await page.locator('.generation-error').textContent()).includes('validation_error'), 'ID inválido conserva código Rust');
    // Fuentes/derivados alterados son errores reales del núcleo, sin reparación.
    const literal = f.fixtures['literal.txt']; await ctl('reader_tamper', { name: 'literal.txt', enabled: true }); await read(literal);
    expect(await page.locator('.generation-error').count() === 1, 'Fuente alterada retira texto'); await ctl('reader_tamper', { name: 'literal.txt', enabled: false });
    await ctl('reader_corrupt', { enabled: true }); await read(literal);
    expect(await page.locator('.generation-error').count() === 1, 'Derivado corrupto falla completo'); await ctl('reader_corrupt', { enabled: false });
    // Error de transporte sintético con HTML aparente, siempre escapado.
    await ctl('reader_instrument', { instrument: { operation: 'extraction.fragments', failure: true } }); await read(literal);
    expect((await page.locator('.generation-error').textContent()).includes('<script>Fallo lector sintético</script>'), 'Error hostil literal'); await ctl('reader_instrument', { instrument: {} });
    // Cursor después de regenerar: no mezcla ni reinicia automáticamente.
    await read(literal); const current = await cli('extraction.get', { id: literal.job.id });
    if (current.state !== 'queued') await cli('extraction.retry', { id: current.id, expected_revision: current.revision });
    await cli('extraction.run', { id: current.id }); await button('Siguiente página de lectura').click(); await settled();
    expect(await page.locator('.generation-error').count() === 1 && await page.locator('.generation-item').count() === 0, 'Generación concurrente invalida cursor sin reiniciar');
    // Descartar lectura tardía al cambiar ID, incluyendo error tardío.
    for (const failure of [false, true]) {
      await ctl('reader_instrument', { instrument: { operation: 'extraction.fragments', delay: 300, failure } }); await fill(literal);
      const response = page.waitForResponse(r => r.url().endsWith('/__test__/operate') && r.request().postDataJSON().command?.operation === 'extraction.fragments');
      await button('Leer desde el inicio').click(); await page.getByLabel('ID de trabajo para lectura').fill(f.fixtures['cells.xlsx'].job.id); await response;
      expect(await page.locator('.generation-result, .generation-error').count() === 0, 'Respuesta tardía descartada ' + failure);
    }
    await ctl('reader_instrument', { instrument: {} }); await read(f.fixtures['cells.xlsx']);
    for (const width of [1280,900]) {
      await page.setViewportSize({ width, height: 800 }); await page.locator('.generation-result').scrollIntoViewIfNeeded();
      expect(await page.locator('.generation-panel').evaluate(n => n.scrollWidth <= n.clientWidth + 1), 'Sin overflow horizontal a ' + width);
      await page.screenshot({ path: f.root + '/reader-' + theme + '-' + width + '.png', fullPage: true });
    }
    await button('Leer desde el inicio').focus(); expect(await button('Leer desde el inicio').evaluate(n => n === document.activeElement), 'Foco identificable por teclado');
    if (!f.isolated) {
      await button('Search').click(); await button('Cancelar').click(); expect(await page.getByLabel('Título de la nota').inputValue() === 'Buffer de lector conservado', 'Guardia conserva buffer al cancelar salida');
      await button('Search').click(); await button('Descartar').click();
      await page.getByRole('navigation', { name: 'Secciones' }).getByRole('button', { name: 'Notes', exact: true }).click();
      await page.locator('.list-items button').filter({ hasText: 'Lectura sintética' }).click(); await page.locator('.generation-panel > summary').click();
      expect(await page.getByLabel('ID de trabajo para lectura').inputValue() === '', 'Remount de nota retira criterios/lectura');
    }
    // Montar el editor puede mostrar el panel antes de terminar relation.list.
    // Auditar todas las llamadas terminadas; una llamada pendiente no es un write.
    // No se omiten entradas falsas ni se acepta una llamada que nunca termina.
    let operations = await ops(); const deadline = Date.now() + 5000;
    while (operations.some(o => o.readonly_unchanged === undefined)) {
      if (Date.now() >= deadline) throw new Error('Lectura UI no terminó para auditoría: ' + JSON.stringify(operations.filter(o => o.readonly_unchanged === undefined)));
      await new Promise(resolve => setTimeout(resolve, 20)); operations = await ops();
    }
    expect(operations.every(o => o.readonly_unchanged), 'Cada lectura UI conserva SQLite/originales');
    expect(operations.filter(o => o.operation.startsWith('extraction.')).every(o => ['extraction.fragments','extraction.pdf_fragments','extraction.xlsx_cells'].includes(o.operation)), 'UI sólo llama lectores, sin writes/índice/descargas');
    expect(external.length === 0, 'Cero requests externos');
    await ctl('reader_report', { theme, ok: true, checks, external_requests: external.length, isolated: f.isolated });
    reports.push({ theme, checks: checks.length });
  }
  console.log(JSON.stringify({ reports, external_requests: external.length }));
}
