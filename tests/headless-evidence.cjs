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
    const checks = [], expect = (condition, message) => { if (!condition) throw new Error(`${theme}: ${message}`); checks.push(message); };
    const equal = (a, b) => JSON.stringify(a) === JSON.stringify(b);
    const fixture = await control('evidence_seed', { theme });
    let workspace = fixture.workspace;
    const snapshot = () => control('evidence_snapshot', { workspace });
    const operations = () => control('evidence_operations', { workspace });
    const query = async input => {
      const response = await control('evidence_query', { workspace, query: { ...input, limit: 20 } });
      if (!response.ok) throw new Error(JSON.stringify(response)); return response.data;
    };
    const open = async path => {
      await page.getByLabel('Ruta del workspace', { exact: true }).fill(path);
      await page.getByRole('button', { name: 'Abrir workspace', exact: true }).click();
      await page.locator('.home [data-home-section="person"] .home-coverage').waitFor();
      workspace = path;
    };
    const changeWorkspace = async path => { await page.getByRole('button', { name: 'Cambiar workspace', exact: true }).click(); await open(path); };
    const enter = async () => {
      await page.getByRole('button', { name: 'Search', exact: true }).click();
      await page.getByLabel('Ámbito de búsqueda', { exact: true }).selectOption('attachments');
      await page.getByLabel('Texto de adjuntos', { exact: true }).waitFor();
    };
    const fill = async (text, filters = {}) => {
      await page.getByLabel('Texto de adjuntos', { exact: true }).fill(text);
      await page.getByLabel('Extractor de adjuntos', { exact: true }).selectOption(filters.extractor || '');
      await page.getByLabel('Trabajo de adjuntos', { exact: true }).fill(filters.job_id || '');
      await page.getByLabel('ID de adjunto', { exact: true }).fill(filters.attachment_id || '');
    };
    const settled = () => page.waitForFunction(() => Boolean(document.querySelector('.attachment-coverage, .attachment-error')));
    const submit = async (text, filters = {}) => { await fill(text, filters); await page.getByRole('button', { name: 'Buscar adjuntos', exact: true }).click(); await settled(); };
    const assertPage = async (expected, label) => {
      const actual = await page.locator('.attachment-hit').evaluateAll(nodes => nodes.map(node => ({
        job: node.dataset.job, generation: node.dataset.generation, ordinal: Number(node.dataset.ordinal),
        text: node.querySelector('.attachment-text').textContent,
        source: Object.fromEntries([...node.querySelectorAll('[data-source]')].map(n => [n.dataset.source, n.textContent])),
        locator: Object.fromEntries([...node.querySelectorAll('[data-locator]')].map(n => [n.dataset.locator, n.textContent])),
      })));
      expect(equal(actual.map(a => [a.job, a.generation, a.ordinal, a.text]), expected.items.map(h => [h.source.job_id, h.source.generation_id, h.ordinal, h.text])), label + ': orden/identidad/texto canónico exactos');
      expect(actual.length <= 20, label + ': página acotada');
      for (let i = 0; i < actual.length; i++) {
        const source = { ...expected.items[i].source, ordinal: expected.items[i].ordinal };
        expect(Object.entries(source).every(([key, value]) => actual[i].source[key] === String(value)), label + ': fuente íntegra ' + i);
        expect(Object.entries(expected.items[i].locator).every(([key, value]) => actual[i].locator[key] === (Array.isArray(value) ? value.join(', ') : String(value))), label + ': localizador íntegro ' + i);
      }
      for (const [key, value] of Object.entries(expected.coverage)) {
        if (typeof value === 'number') expect(await page.locator(`[data-coverage="${key}"]`).textContent() === String(value), label + ': cobertura ' + key);
      }
      expect(await page.locator('[data-page="profile"]').textContent() === expected.profile && await page.locator('[data-page="revision"]').textContent() === String(expected.revision), label + ': perfil/revisión');
    };
    await page.goto(origin); await open(workspace);
    await page.getByLabel('Tema', { exact: true }).selectOption(theme);
    const before = await snapshot(); await enter();
    expect((await operations()).filter(op => op.operation === 'extraction.search').length === 0, 'Montar ámbito no consulta ni prepara');
    expect(await page.locator('.attachment-status').textContent() === 'Ingresá texto y buscá adjuntos. Todavía no se consultó el índice.', 'Estado inicial no es cero hits');
    expect(await page.getByRole('button', { name: 'Buscar adjuntos', exact: true }).isDisabled(), 'Consulta vacía no se envía');
    expect(await page.getByLabel('Tipo de búsqueda', { exact: true }).count() === 0 && await page.getByRole('button', { name: 'Reconstruir índice', exact: true }).count() === 0, 'Sin filtros de entidad ni reconstrucción en adjuntos');

    await fill('nkprobe'); await page.getByLabel('Texto de adjuntos', { exact: true }).focus(); await page.keyboard.press('Enter'); await settled();
    expect(await page.locator('.attachment-hit').count() === 20, 'Enter envía consulta de adjuntos por teclado');
    await assertPage(fixture.pages[0], 'Primera página');
    await page.getByRole('button', { name: 'Siguiente página de adjuntos', exact: true }).focus(); await page.keyboard.press('Enter'); await settled();
    expect(await page.locator('.attachment-hit').count() === 11, 'Enter pagina adjuntos por teclado');
    await assertPage(fixture.pages[1], 'Segunda página sin acumulación');
    expect(await page.getByRole('button', { name: 'Siguiente página de adjuntos', exact: true }).count() === 0, 'Cursor nulo termina paginación');
    await page.getByRole('button', { name: 'Volver al inicio de adjuntos', exact: true }).click(); await settled();
    await assertPage(fixture.pages[0], 'Reinicio explícito');
    const count = (await operations()).filter(op => op.operation === 'extraction.search').length;
    await page.getByRole('button', { name: 'Actualizar', exact: true }).click(); await settled();
    expect((await operations()).filter(op => op.operation === 'extraction.search').length === count + 1, 'Actualizar consulta una vez, no reconstruye');

    for (const extractor of ['utf8_text', 'pdf_text', 'xlsx_cells']) {
      const input = { query: 'nkprobe', extractor };
      await submit(input.query, input); await assertPage(await query(input), extractor);
      if (extractor === 'utf8_text') {
        expect(await page.locator('.attachment-previous').count() === 1, 'Intento anterior visible');
        expect(await page.locator('.attachment-hit img, .attachment-hit script, .attachment-hit a').count() === 0 && !await page.evaluate(() => window.__evidenceExecuted), 'HTML/scripts/links inertes, sin interpretación');
        const summary = page.locator('.attachment-hit summary').first();
        await summary.focus(); await page.keyboard.press('Enter');
        expect(await summary.evaluate(n => n.parentElement.open), 'Enter abre procedencia');
        await page.keyboard.press('Space');
        expect(!await summary.evaluate(n => n.parentElement.open), 'Space cierra procedencia');
        await page.keyboard.press('Tab');
        expect(await page.evaluate(() => document.activeElement.matches(':focus-visible') && getComputedStyle(document.activeElement).outlineStyle !== 'none'), 'Tab conserva foco visible');
      }
    }
    for (const term of ['145', 'true', 'DIV', '2026', 'DO_NOT_EXECUTE', 'longtail']) {
      await submit(term, { extractor: 'xlsx_cells' }); await assertPage(await query({ query: term, extractor: 'xlsx_cells' }), 'Campo XLSX ' + term);
      if (term === 'longtail') expect(await page.locator('.attachment-partial').count() > 0, 'Porción XLSX y caché no recalculada explícitas');
    }
    const filtered = { extractor: 'xlsx_cells', job_id: fixture.jobs['cells.xlsx'].id, attachment_id: fixture.jobs['cells.xlsx'].attachment_id };
    await submit('nkprobe', filtered); await assertPage(await query({ query: 'nkprobe', ...filtered }), 'Tres filtros AND');
    await submit('zzsincoincidencias'); await assertPage(await query({ query: 'zzsincoincidencias' }), 'Cero hits con cobertura');
    expect(await page.locator('.attachment-empty').count() === 1 && (await page.locator('.attachment-coverage').textContent()).includes('no prueba ausencia'), 'Cero hits no es ausencia universal');
    for (const [text, filters] of [['🚀'.repeat(100), {}], [Array(17).fill('nkprobe').join(' '), {}], ['nkprobe\u0001', {}], ['nkprobe', { job_id: 'no-es-uuid' }]]) {
      await submit(text, filters);
      expect((await page.locator('.attachment-error').textContent()).includes('validation_error'), 'Input inválido conservado para validación Rust');
      expect(await page.getByLabel('Texto de adjuntos', { exact: true }).inputValue() === text && await page.locator('.attachment-coverage, .attachment-hit').count() === 0, 'Error no trunca input ni conserva evidencia anterior');
    }
    await submit('nkprobe', { extractor: 'utf8_text' });
    const opsBeforeEdit = (await operations()).length;
    await page.getByLabel('Texto de adjuntos', { exact: true }).fill('evidencia');
    expect(await page.locator('.attachment-coverage, .attachment-hit').count() === 0 && (await operations()).length === opsBeforeEdit, 'Editar retira página/cursor sin consulta automática');
    expect(equal(await snapshot(), before), 'Todas las consultas/inspecciones previas conservan SQL/fuentes/generaciones/índice');

    const restored = await control('evidence_roundtrip', { workspace });
    await control('evidence_tamper', { workspace, altered: true }); const tampered = await snapshot();
    await submit('nkprobe', { extractor: 'utf8_text' });
    expect((await page.locator('.attachment-error').textContent()).includes('storage_error') && await page.locator('.attachment-hit').count() === 0, 'Fuente alterada falla sin omitir/refill');
    expect(equal(await snapshot(), tampered), 'Error de fuente no repara ni muta');
    await control('evidence_tamper', { workspace, altered: false });
    expect(equal(await snapshot(), before), 'Restauración sintética deliberada restituye fuente exacta');

    const hold = async (operation = 'extraction.search') => page.evaluate(operation => {
      const original = window.__TAURI_INTERNALS__.invoke;
      window.__evidenceHeld = false; window.__evidenceActive = 0; window.__evidenceMax = 0;
      let taken = false;
      window.__TAURI_INTERNALS__.invoke = async (command, args = {}) => {
        if (command !== 'operate' || args.request.command.operation !== operation) return original(command, args);
        const selected = !taken; taken = true;
        window.__evidenceMax = Math.max(window.__evidenceMax, ++window.__evidenceActive);
        try {
          const result = await original(command, args); // CLI real ya respondió; sólo se retiene entrega.
          if (selected) { window.__evidenceResponse = result; await new Promise(resolve => { window.__evidenceRelease = resolve; window.__evidenceHeld = true; }); }
          return result;
        } finally { window.__evidenceActive--; }
      };
      window.__evidenceRestore = () => { window.__TAURI_INTERNALS__.invoke = original; window.__evidenceRelease?.(); };
    }, operation);
    for (const reject of [false, true]) {
      for (const axis of ['texto', 'filtro', 'ámbito', 'sección', 'workspace']) {
        await fill(reject ? '🚀'.repeat(100) : 'nkprobe'); await hold();
        try {
          await page.getByRole('button', { name: 'Buscar adjuntos', exact: true }).click();
          await page.waitForFunction(() => window.__evidenceHeld);
          expect(await page.evaluate(() => window.__evidenceResponse.ok) === !reject, 'Respuesta retenida real ' + axis + '/' + reject);
          if (axis === 'ámbito') {
            await page.getByLabel('Ámbito de búsqueda', { exact: true }).selectOption('notes');
            await page.getByLabel('Ámbito de búsqueda', { exact: true }).selectOption('attachments');
          } else if (axis === 'sección') {
            await page.getByRole('button', { name: 'Home', exact: true }).click();
            await page.locator('.home [data-home-section="person"] .home-coverage').waitFor(); await enter();
          } else if (axis === 'workspace') {
            await changeWorkspace(workspace === fixture.workspace ? restored.path : fixture.workspace); await enter();
          }
          await fill('evidencia', axis === 'filtro' ? { extractor: 'pdf_text' } : {});
          await page.getByRole('button', { name: 'Buscar adjuntos', exact: true }).click();
          expect(await page.locator('.attachment-hit, .attachment-error').count() === 0 && await page.evaluate(() => window.__evidenceActive) === 1, 'Nueva lectura espera cupo sin resultado viejo ' + axis);
          await page.evaluate(() => window.__evidenceRelease()); await settled();
          await assertPage(await query({ query: 'evidencia', ...(axis === 'filtro' ? { extractor: 'pdf_text' } : {}) }), 'Intercalado ' + axis + '/' + reject);
          expect(await page.evaluate(() => window.__evidenceMax) === 1 && await page.locator('.attachment-error').count() === 0, 'Éxito/error/finally obsoletos descartados y un IPC ' + axis);
        } finally { await page.evaluate(() => window.__evidenceRestore?.()); }
      }
    }
    await changeWorkspace(restored.path); await enter();
    const restoredBefore = await snapshot(); await submit('nkprobe'); await assertPage(fixture.pages[0], 'Restaurado UI');
    expect(equal(await snapshot(), restoredBefore), 'Restaurado UI conserva snapshot');
    await page.reload(); await open(workspace); await enter();
    expect(await page.locator('.attachment-coverage, .attachment-hit').count() === 0, 'Reapertura no recupera resultados ni cursores');
    await submit('nkprobe'); await assertPage(fixture.pages[0], 'Reapertura UI');
    await control('evidence_reset', { workspace }); const reset = await snapshot();
    await page.getByRole('button', { name: 'Siguiente página de adjuntos', exact: true }).click(); await settled();
    expect((await page.locator('.attachment-error').textContent()).includes('index_revision_changed') && await page.locator('.attachment-hit, .attachment-coverage').count() === 0, 'Cursor vencido no mezcla revisión ni reinicia');
    expect(equal(await snapshot(), reset), 'Error de cursor no muta ni repara');
    await page.getByRole('button', { name: 'Buscar de nuevo', exact: true }).click(); await settled();
    expect(await page.locator('.attachment-empty').count() === 1, 'Buscar de nuevo explícito refleja reset');

    await changeWorkspace(fixture.workspace); await enter();
    await page.getByLabel('Ámbito de búsqueda', { exact: true }).selectOption('notes');
    await page.getByLabel('Texto de búsqueda', { exact: true }).fill('nknotefixture');
    await page.getByRole('button', { name: 'Buscar', exact: true }).click();
    await page.locator('.search-hit').first().click(); await page.getByLabel('Título de la nota', { exact: true }).waitFor();
    expect((await page.locator('.source-locator').textContent()).includes('evidence-paragraph'), 'Notas abre entidad/bloque originales');
    await page.getByLabel('Título de la nota', { exact: true }).fill('Buffer protegido evidencia');
    await page.getByLabel('Ámbito de búsqueda', { exact: true }).selectOption('attachments');
    await submit('nkprobe', { extractor: 'utf8_text' });
    expect(await page.getByLabel('Título de la nota', { exact: true }).inputValue() === 'Buffer protegido evidencia', 'Alternar/buscar conserva buffer pendiente');
    await page.locator('.attachment-hit summary').click();
    await page.getByRole('button', { name: 'Home', exact: true }).click(); await page.getByRole('button', { name: 'Cancelar', exact: true }).click();
    expect(await page.getByLabel('Título de la nota', { exact: true }).inputValue() === 'Buffer protegido evidencia' && await page.locator('.attachment-hit').count() === 1, 'Cancelar navegación conserva buffer y selección de evidencia');
    await page.getByRole('button', { name: 'Home', exact: true }).click(); await page.getByRole('button', { name: 'Guardar y continuar', exact: true }).click();
    await page.locator('.home [data-home-section="person"] .home-coverage').waitFor();
    expect(await page.locator('.home').count() === 1, 'Guardar y continuar respeta guardia');
    await enter(); await page.getByLabel('Ámbito de búsqueda', { exact: true }).selectOption('notes');
    await page.getByRole('button', { name: 'Buscar', exact: true }).click(); await page.locator('.search-hit').first().click();
    await page.waitForFunction(() => document.querySelector('.title-input')?.value === 'Buffer protegido evidencia');
    await page.getByLabel('Título de la nota', { exact: true }).fill('Descartable evidencia');
    await page.getByLabel('Ámbito de búsqueda', { exact: true }).selectOption('attachments');
    await page.getByRole('button', { name: 'Home', exact: true }).click(); await page.getByRole('button', { name: 'Descartar', exact: true }).click();
    await page.locator('.home [data-home-section="person"] .home-coverage').waitFor();
    expect(await page.locator('.home-cover, .home').count() >= 1, 'Descartar respeta guardia sin cambiar Home');
    await enter();
    for (const reject of [false, true]) {
      await page.getByLabel('Ámbito de búsqueda', { exact: true }).selectOption('notes');
      await page.getByLabel('Texto de búsqueda', { exact: true }).fill(reject ? Array(17).fill('nkprobe').join(' ') : 'nknotefixture');
      await hold('search.query');
      try {
        await page.getByRole('button', { name: 'Buscar', exact: true }).click(); await page.waitForFunction(() => window.__evidenceHeld);
        expect(await page.evaluate(() => window.__evidenceResponse.ok) === !reject, 'Notas: respuesta retenida real ' + reject);
        await page.getByLabel('Ámbito de búsqueda', { exact: true }).selectOption('attachments');
        await submit('nkprobe', { extractor: 'utf8_text' });
        await page.evaluate(() => window.__evidenceRelease()); await page.waitForFunction(() => window.__evidenceActive === 0);
        expect(await page.locator('.attachment-hit').count() === 1 && await page.locator('.search-hit, .entity-list > .error').count() === 0, 'Notas: éxito/error tardío no aparece en adjuntos ' + reject);
      } finally { await page.evaluate(() => window.__evidenceRestore?.()); }
    }
    await page.locator('.attachment-hit summary').click();
    expect(await page.locator('.attachment-search').evaluate(n => n.scrollWidth <= n.clientWidth + 1), 'Texto/hashes completos ajustan al ancho de lista');
    await page.screenshot({ path: fixture.root + '/search-' + theme + '.png' });
    const seen = [...await control('evidence_operations', { workspace: fixture.workspace }), ...await control('evidence_operations', { workspace: restored.path })];
    expect(seen.filter(op => op.operation.startsWith('extraction.')).every(op => op.operation === 'extraction.search') && !seen.some(op => op.operation.startsWith('attachment.')), 'Sólo extraction.search desde UI, sin preparar/descargar/reparar');
    expect(seen.filter(op => op.operation === 'extraction.search').every(op => op.readonly_unchanged === true), 'Cada lectura UI conserva su snapshot lógico y fuentes, también al fallar');
    expect(external.length === 0, 'Sin intentos de requests remotos');
    await control('evidence_report', { workspace, theme, ok: true, checks, transport: 'headless-cli-not-native', external_requests: external.length });
    reports.push({ theme, checks: checks.length, root: fixture.root });
  }
  return { reports, external_requests: external.length };
}
