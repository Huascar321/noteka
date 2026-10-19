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
    const checks = [], expect = (ok, label) => { if (!ok) throw new Error(theme + ': ' + label); checks.push(label); };
    const f = await control('prep_seed', { theme }), workspace = f.workspace;
    const ctl = (step, input = {}) => control(step, { workspace, ...input });
    const cli = async (operation, input) => { const r = await ctl('prep_cli', { operation, input }); if (!r.ok) throw new Error(JSON.stringify(r)); return r.data; };
    const ops = () => ctl('prep_operations');
    const button = name => page.getByRole('button', { name, exact: true });
    const receipt = async name => {
      await button(name).click();
      await page.locator('.preparation-receipt, .preparation-error').waitFor();
      if (await page.locator('.preparation-error').count()) throw new Error(await page.locator('.preparation-error').textContent());
      return JSON.parse(await page.locator('.preparation-receipt pre').textContent());
    };
    const inspect = async () => { await button('Consultar trabajo elegido').click(); await page.locator('.preparation-inspection').waitFor(); };
    const list = async () => { await button('Consultar trabajos').click(); await page.locator('.preparation-page').waitFor(); };
    const select = async id => page.getByLabel('ID administrado para preparar', { exact: true }).fill(id);
    await page.goto(origin);
    await page.getByLabel('Ruta del workspace', { exact: true }).fill(workspace); await button('Abrir workspace').click();
    await page.locator('.home [data-home-section="person"] .home-coverage').waitFor().catch(async e => { throw new Error(e.message + '\nUI: ' + await page.locator('body').innerText()); });
    await page.getByLabel('Tema', { exact: true }).selectOption(theme); await page.getByRole('navigation', { name: 'Secciones' }).getByRole('button', { name: 'Notes', exact: true }).click();
    await page.locator('.list-items button').filter({ hasText: 'Preparación sintética' }).click();
    await page.getByLabel('Título de la nota', { exact: true }).fill('Buffer pendiente conservado');
    const before = await ctl('prep_snapshot');
    await page.locator('.preparation-panel > summary').click();
    expect((await ops()).filter(o => o.operation.startsWith('extraction.')).length === 0, 'Montar/abrir panel no consulta ni procesa');
    expect(await page.getByLabel('Referencia para preparar').locator('option').count() === 7, 'Seis IDs únicos desde siete referencias, rótulos de bloque literales');
    expect(!await page.evaluate(() => window.__evidenceExecuted), 'Rótulo hostil no ejecuta');
    const jobs = {};
    for (const name of ['literal.txt', 'text.pdf', 'cells.xlsx', 'empty.txt', 'bad.txt']) {
      const source = f.sources[name];
      await page.getByLabel('Referencia para preparar').selectOption(source.attachment.id);
      await page.getByLabel('Extractor de preparación').selectOption(source.extractor);
      await list(); expect(await page.locator('.preparation-job').count() === 0, name + ': sin trabajo antes de encolar');
      await button('Encolar extracción').focus(); await page.keyboard.press('Enter'); await page.locator('.preparation-receipt').waitFor();
      const enqueued = JSON.parse(await page.locator('.preparation-receipt pre').textContent()); jobs[name] = enqueued.job;
      expect(enqueued.job.state === 'queued' && enqueued.job.attempts === 0 && !enqueued.deduplicated, name + ': enqueue explícito por teclado sin ejecución');
      await inspect();
      if (name === 'literal.txt') {
        const paused = await receipt('Ejecutar trabajo elegido'); expect(paused.outcome === 'idle' && paused.reason === 'queue_paused', 'Run pausado no ejecuta');
        await inspect(); const resumed = await receipt('Reanudar cola global'); expect(!resumed.paused, 'Reanudar es acción global separada'); await inspect();
      }
      if (name === 'bad.txt') {
        await button('Ejecutar trabajo elegido').click(); await page.locator('.preparation-error').waitFor();
        expect((await page.locator('.preparation-error').textContent()).includes('nul_in_text'), 'UTF-8 inválido devuelve error explícito, no éxito parcial');
        const failed = await cli('extraction.get', { id: enqueued.job.id }); jobs[name] = failed;
        expect(failed.state === 'failed' && failed.error, 'Error de ejecución confirma estado failed consultable'); await inspect();
        expect(await page.locator('.preparation-inspection').textContent().then(t => t.includes(failed.error.code)), 'Error persistido literal visible');
        continue;
      }
      const ran = await receipt('Ejecutar trabajo elegido');
      jobs[name] = ran.job;
      expect(ran.outcome === 'succeeded' && ran.job.attempts === 1, name + ': un intento explícito');
      const unselected = await cli('extraction.index', { action: 'get', job_id: ran.job.id });
      expect(unselected.items.length === 0 && unselected.coverage.unindexed_supported_generations === 1, name + ': extraer no selecciona');
      await inspect();
      expect(await page.locator('[data-preparation="generation"]').textContent() === ran.job.generation.id, name + ': generación observada exacta');
      const put = await receipt('Seleccionar generación para búsqueda');
      expect(put.action === 'put' && put.changed && put.entry.source.generation_id === ran.job.generation.id, name + ': put explícito CAS');
      if (name === 'empty.txt') expect(put.entry.fragment_count === 0, 'Generación vacía compatible seleccionada con cero fragmentos');
      await inspect(); const noop = await receipt('Seleccionar generación para búsqueda'); expect(!noop.changed && noop.revision === put.revision, name + ': no-op revisado');
      expect(await page.getByLabel('Título de la nota').inputValue() === 'Buffer pendiente conservado', name + ': buffer conservado');
    }
    // Deduplicación terminal: no ejecuta ni selecciona nuevamente.
    await select(f.sources['literal.txt'].attachment.id); await list();
    await page.getByLabel('Extractor de preparación').selectOption('utf8_text');
    const dedupe = await receipt('Encolar extracción'); expect(dedupe.deduplicated && dedupe.job.id === jobs['literal.txt'].id && dedupe.job.attempts === 1, 'Dedupe conserva ID e intentos');
    await inspect(); const retried = await receipt('Reencolar trabajo elegido'); expect(retried.state === 'queued' && retried.result_from_previous_attempt, 'Retry conserva generación anterior sin ejecutar');
    await inspect(); const cancelled = await receipt('Cancelar trabajo elegido'); expect(cancelled.state === 'cancelled' && cancelled.result_from_previous_attempt, 'Cancel conserva generación anterior');
    await inspect();
    const latest = await cli('extraction.retry', { id: cancelled.id, expected_revision: cancelled.revision });
    await button('Reencolar trabajo elegido').click(); await page.locator('.preparation-error').waitFor();
    expect((await page.locator('.preparation-error').textContent()).includes('revision_conflict'), 'CAS job obsoleto falla sin rebase');
    expect((await cli('extraction.get', { id: latest.id })).revision === latest.revision, 'No se repite retry');
    await inspect();
    const oldGen = await page.locator('[data-preparation="generation"]').textContent();
    const regenerated = await cli('extraction.run', { id: latest.id });
    expect(regenerated.job.generation.id !== oldGen, 'Otro cliente regenera antes de put');
    await button('Seleccionar generación para búsqueda').click(); await page.locator('.preparation-error').waitFor();
    expect((await page.locator('.preparation-error').textContent()).includes('revision_conflict'), 'CAS generación/índice concurrente rechaza put');
    await inspect();
    const otherIndex = await cli('extraction.index', { action: 'get' });
    const otherJob = f.jobs['unselected.txt'];
    await cli('extraction.index', { action: 'put', job_id: otherJob.id, expected_generation_id: otherJob.generation.id, expected_revision: otherIndex.revision });
    await button('Seleccionar generación para búsqueda').click(); await page.locator('.preparation-error').waitFor();
    expect((await page.locator('.preparation-error').textContent()).includes('revision_conflict'), 'CAS índice concurrente rechaza put con generación intacta');
    await inspect();
    const ctrl = await cli('extraction.control', { action: 'get' }); await cli('extraction.control', { action: 'set', paused: !ctrl.paused, expected_revision: ctrl.revision });
    await button('Pausar cola global').click(); await page.locator('.preparation-error').waitFor();
    expect((await page.locator('.preparation-error').textContent()).includes('revision_conflict'), 'CAS control global no se rebasa');
    await inspect(); await receipt('Reanudar cola global');
    // Página sintética de configuraciones futuras; sin hits/generaciones fabricados.
    await select(f.jobs['queued.txt'].attachment_id); await list();
    const ids1 = await page.locator('.preparation-job').allTextContents(); expect(ids1.length === 20, 'Primera página 20 trabajos');
    await page.locator('.preparation-job').first().click(); await inspect();
    expect(await button('Ejecutar trabajo elegido').isDisabled() && await button('Reencolar trabajo elegido').isDisabled(), 'Config futura visible sin ejecutar ni convertir');
    await button('Siguiente página de trabajos').click(); await page.locator('.preparation-page').filter({ hasText: 'Página 2' }).waitFor();
    const ids2 = await page.locator('.preparation-job').allTextContents(); expect(ids2.length === 2 && new Set([...ids1, ...ids2]).size === 22, 'Segunda página reemplaza sin pérdidas/duplicación');
    await select('invalid-id'); await button('Consultar trabajos').click(); await page.locator('.preparation-error').waitFor();
    expect(await page.locator('.preparation-job').count() === 0 && (await page.locator('.preparation-error').textContent()).includes('validation_error'), 'Entrada inválida retira página, no es vacío');
    // Lectura con error y contenido no confiable sólo en transporte de prueba.
    await select(f.sources['literal.txt'].attachment.id);
    await ctl('prep_instrument', { instrument: { operation: 'extraction.list', failure: true } });
    await button('Consultar trabajos').click(); await page.locator('.preparation-error').waitFor();
    expect((await page.locator('.preparation-error').textContent()).includes('<script>nunca ejecutar</script>'), 'Error hostil mostrado como texto');
    await ctl('prep_instrument', { instrument: {} });
    // Respuesta tardía descartada al cambiar ID; no segunda consulta implícita.
    await ctl('prep_instrument', { instrument: { operation: 'extraction.list', delay: 400 } });
    const late = page.waitForResponse(r => r.url().endsWith('/__test__/operate') && r.request().postDataJSON().command?.operation === 'extraction.list');
    await button('Consultar trabajos').click(); await select(f.sources['text.pdf'].attachment.id); await late;
    expect(await page.locator('.preparation-page, .preparation-error').count() === 0, 'Lectura tardía no publica tras cambiar ID');
    await ctl('prep_instrument', { instrument: {} });
    // Fallo de transporte después de una mutación confirmada: no repetir.
    await select(f.sources['bad.txt'].attachment.id); await list(); await page.locator('.preparation-job').first().click(); await inspect();
    await receipt('Reencolar trabajo elegido'); await inspect();
    const bad = await cli('extraction.get', { id: jobs['bad.txt'].id });
    await ctl('prep_instrument', { instrument: { operation: 'extraction.cancel', transport_after: true } });
    await button('Cancelar trabajo elegido').click(); await page.locator('.preparation-error').waitFor();
    expect((await page.locator('.preparation-error').textContent()).includes('El estado puede haber cambiado'), 'Transporte incierto no promete rollback');
    expect((await cli('extraction.get', { id: bad.id })).state === 'cancelled', 'Mutación real confirmada pese a transporte perdido');
    expect(await button('Encolar extracción').isDisabled(), 'Exige consultar tras incertidumbre'); await ctl('prep_instrument', { instrument: {} });
    // Guardias y buffer: cancelar navegación no guarda ni descarta.
    await button('Search').click(); await button('Cancelar').click();
    expect(await page.getByLabel('Título de la nota').inputValue() === 'Buffer pendiente conservado', 'Cancelar navegación preserva buffer tras preparación');
    await ctl('prep_fail_save', { enabled: true });
    await button('Search').click(); await button('Guardar y continuar').click();
    await page.locator('dialog .error').waitFor();
    expect(await page.getByLabel('Título de la nota').inputValue() === 'Buffer pendiente conservado' && await page.locator('.preparation-panel').count() === 1, 'Fallo de guardar conserva buffer y panel sin navegar');
    await button('Cancelar').click(); await ctl('prep_fail_save', { enabled: false });
    expect((await ctl('prep_snapshot')).original === before.original, 'Todos los pasos preservan entidades/documentos/revisiones/FTS/historial');
    expect(JSON.stringify((await ctl('prep_snapshot')).files) === JSON.stringify(before.files), 'Originales administrados idénticos');
    const recorded = await ops();
    expect(recorded.filter(o => o.readonly_unchanged !== undefined).every(o => o.readonly_unchanged), 'Cada lectura UI conserva snapshot SQLite');
    expect(recorded.filter(o => o.operation.startsWith('extraction.')).every(o => o.originals_unchanged !== false), 'Cada operación derivada conserva originales/entidades');
    expect(recorded.filter(o => o.operation === 'extraction.run').every(o => typeof o.input.id === 'string'), 'Todos los run envían ID explícito');
    expect(!recorded.some(o => o.operation === 'extraction.search'), 'Preparación no ejecuta búsquedas');
    expect(!recorded.some(o => o.operation === 'extraction.index' && o.input.action === 'reset'), 'Sin reset/reparación');
    expect(recorded.filter(o => o.operation === 'attachment.read').length === 0, 'Preparación no descarga base64');
    for (const width of [1280, 900]) {
      await page.setViewportSize({ width, height: 800 }); await page.locator('.preparation-panel').scrollIntoViewIfNeeded();
      expect(await page.locator('.preparation-panel').evaluate(n => n.scrollWidth <= n.clientWidth + 1), 'Panel sin overflow horizontal a ' + width);
      await page.screenshot({ path: f.root + '/preparation-' + theme + '-' + width + '.png', fullPage: true });
    }
    await button('Consultar trabajo elegido').focus();
    expect(await button('Consultar trabajo elegido').evaluate(n => n === document.activeElement), 'Control conserva foco por teclado');
    expect(external.length === 0, 'Sin requests externos');
    await inspect();
    await ctl('prep_instrument', { instrument: { operation: 'extraction.cancel', delay: 400 } });
    const finalResponse = page.waitForResponse(r => r.url().endsWith('/__test__/operate') && r.request().postDataJSON().command?.operation === 'extraction.cancel');
    await button('Cancelar trabajo elegido').click();
    await button('Search').click(); await button('Descartar').click();
    await finalResponse; await ctl('prep_instrument', { instrument: {} });
    expect(await page.locator('.preparation-receipt, .preparation-error').count() === 0, 'Mutación tardía no publica en Search tras desmontar');
    const countBeforeRemount = (await ops()).filter(o => o.operation.startsWith('extraction.')).length;
    await page.getByRole('navigation', { name: 'Secciones' }).getByRole('button', { name: 'Notes', exact: true }).click();
    await page.locator('.list-items button').filter({ hasText: 'Preparación sintética' }).click();
    await page.locator('.preparation-panel > summary').click();
    expect(await page.getByLabel('ID administrado para preparar').inputValue() === '' && (await ops()).filter(o => o.operation.startsWith('extraction.')).length === countBeforeRemount, 'Remount no reutiliza inspección ni repite mutación');
    await ctl('prep_report', { theme, ok: true, checks, external_requests: external.length, operation_count: recorded.length });
    // Guardar no forma parte de la preparación; abandonar con descarte explícito.
    await button('Cambiar workspace').click();
    reports.push({ theme, checks: checks.length });
  }
  console.log(JSON.stringify({ reports, external_requests: external.length }));
}
