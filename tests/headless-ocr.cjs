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
    const checks=[], expect=(ok,label)=>{if(!ok)throw new Error(theme+': '+label);checks.push(label);};
    const f=await control('ocr_seed',{theme}), workspace=f.workspace;
    const ctl=(step,input={})=>control(step,{workspace,...input});
    const cli=async(operation,input)=>{const r=await ctl('ocr_cli',{operation,input});if(!r.ok)throw new Error(JSON.stringify(r));return r.data;};
    const ops=()=>ctl('ocr_operations'), snapshot=()=>ctl('ocr_snapshot');
    const button=name=>page.getByRole('button',{name,exact:true});
    const receipt=async(name)=>{await button(name).click();await page.locator('.preparation-receipt,.preparation-error').waitFor();if(await page.locator('.preparation-error').count())throw new Error(await page.locator('.preparation-error').textContent());return JSON.parse(await page.locator('.preparation-receipt pre').textContent());};
    const inspect=async()=>{await button('Consultar trabajo elegido').click();await page.locator('.preparation-inspection').waitFor();};
    await page.goto(origin);await page.getByLabel('Ruta del workspace',{exact:true}).fill(workspace);await button('Abrir workspace').click();await page.locator('.home [data-home-section="person"] .home-coverage').waitFor();
    await page.getByLabel('Tema',{exact:true}).selectOption(theme);await page.getByRole('navigation',{name:'Secciones'}).getByRole('button',{name:'Notes',exact:true}).click();
    await page.locator('.list-items button').filter({hasText:'OCR headless sintético'}).click();await page.getByLabel('Título de la nota').fill('Buffer OCR pendiente');
    const initial=await snapshot();await page.locator('.preparation-panel > summary').click();await page.locator('.generation-panel > summary').click();
    expect(!(await ops()).some(o=>o.operation.startsWith('extraction.')),'Abrir nota/paneles no importa ni procesa/lee/indexa');
    await page.getByLabel('ID administrado para preparar',{exact:true}).fill(f.attachment.id);await page.getByLabel('Extractor de preparación').selectOption('image_ocr');
    const enqueue=await receipt('Encolar extracción');const job=enqueue.job;expect(job.extractor==='image_ocr'&&job.state==='queued'&&!job.generation,'Enqueue OCR explícito sin ejecución');
    await inspect();const paused=await receipt('Ejecutar trabajo elegido');expect(paused.outcome==='idle'&&paused.reason==='queue_paused','OCR pausado no ejecuta');
    await inspect();await receipt('Reanudar cola global');await inspect();const ran=await receipt('Ejecutar trabajo elegido');expect(ran.job.state==='succeeded'&&ran.job.generation.complete,'OCR real publica generación completa');
    const pages=[];let cursor=null;do{const p=await cli('extraction.ocr_regions',{id:job.id,limit:20,...(cursor?{cursor}:{})});pages.push(p);cursor=p.next_cursor;}while(cursor);
    expect(pages.length>=2&&pages[0].manifest.region_count>20,'Fixture OCR real supera veinte regiones');
    expect((await cli('extraction.index',{action:'get'})).coverage.indexed_generations===0,'Extraer OCR no selecciona para búsqueda');
    await page.getByLabel('ID de trabajo para lectura').fill(job.id);await page.getByLabel('Tipo de lectura').selectOption('image_ocr');
    const beforeRead=await snapshot();await button('Leer desde el inicio').focus();await page.keyboard.press('Enter');await page.locator('.generation-result').waitFor();
    const checkPage=async(p,n)=>{
      expect(await page.locator('[data-reader="page"]').textContent()===String(n),'Número de página OCR '+n);
      expect(JSON.stringify(JSON.parse(await page.locator('.generation-manifest').textContent()))===JSON.stringify(p.manifest),'Manifiesto OCR íntegro '+n);
      expect(JSON.stringify(JSON.parse(await page.locator('.generation-source').textContent()))===JSON.stringify(p.generation),'Fuente/generación OCR completas '+n);
      const rows=await page.locator('.generation-item').evaluateAll(ns=>ns.map(x=>({ordinal:Number(x.dataset.ordinal),text:x.querySelector('.generation-text').textContent,locator:JSON.parse(x.querySelector('.generation-locator').textContent)})));
      expect(rows.length===p.items.length&&rows.length<=20,'Página OCR reemplazada y acotada '+n);
      for(let i=0;i<rows.length;i++){const expected=p.items[i];expect(rows[i].text===expected.text&&rows[i].ordinal===expected.ordinal,'Texto/ordinal OCR literal '+n+'/'+i);expect(JSON.stringify(rows[i].locator)===JSON.stringify({bbox:expected.bbox,confidence:expected.confidence,coordinate_space:p.manifest.coordinate_space}),'Decimales y coordenadas OCR '+n+'/'+i);}
    };
    for(let i=0;i<pages.length;i++){if(i){await button('Siguiente página de lectura').click();await page.locator('.generation-result').waitFor();}await checkPage(pages[i],i+1);}
    expect(JSON.stringify(await snapshot())===JSON.stringify(beforeRead),'Lectura completa no escribe');
    expect((await page.locator('.generation-result').textContent()).includes('no garantiza transcripción completa'),'Límite de cobertura OCR visible');
    await ctl('ocr_instrument',{instrument:{operation:'extraction.ocr_regions',failure:true}});await button('Leer desde el inicio').click();await page.locator('.generation-error').waitFor();expect(await page.locator('.generation-item').count()===0,'Error OCR retira página/cursor');expect(!(await page.evaluate(()=>Boolean(window.__ocrExecuted))),'Datos OCR no ejecutan markup');await ctl('ocr_instrument',{instrument:{}});
    await ctl('ocr_instrument',{instrument:{operation:'extraction.ocr_regions',delay:400}});const late=page.waitForResponse(r=>r.url().endsWith('/__test__/operate')&&r.request().postDataJSON().command?.operation==='extraction.ocr_regions');await button('Leer desde el inicio').click();await page.getByLabel('ID de trabajo para lectura').fill('otro');await late;expect(await page.locator('.generation-result,.generation-error').count()===0,'Respuesta OCR tardía descartada al cambiar ID');await ctl('ocr_instrument',{instrument:{}});
    await inspect();await receipt('Seleccionar generación para búsqueda');expect((await cli('extraction.index',{action:'get'})).coverage.indexed_generations===1,'Selección OCR es acción independiente');
    expect(await page.getByLabel('Título de la nota').inputValue()==='Buffer OCR pendiente','Preparar/leer conserva buffer');await button('Search').click();await button('Cancelar').click();expect(await page.getByLabel('Título de la nota').inputValue()==='Buffer OCR pendiente','Cancelar navegación conserva buffer OCR');
    // Error real independiente: imagen inválida, sin fallback ni éxito parcial.
    await page.getByLabel('ID administrado para preparar',{exact:true}).fill(f.invalid.id);const invalid=await receipt('Encolar extracción');await inspect();await button('Ejecutar trabajo elegido').click();await page.locator('.preparation-error').waitFor();expect((await page.locator('.preparation-error').textContent()).includes('ocr_invalid_image'),'Error nativo real explícito en preparación');await inspect();expect(await page.locator('[data-preparation="state"]').textContent()==='failed','Error OCR persistido failed');
    await button('Search').click();await button('Descartar').click();await page.getByLabel('Ámbito de búsqueda').selectOption('attachments');
    await page.getByLabel('Extractor de adjuntos').selectOption('image_ocr');await page.getByLabel('Texto de adjuntos').fill('notekaocr');const searchBefore=await snapshot();await button('Buscar adjuntos').click();await page.locator('.attachment-hit').first().waitFor();
    const golden=await cli('extraction.search',{query:'notekaocr',extractor:'image_ocr',limit:20});expect(await page.locator('.attachment-hit').count()===golden.items.length,'Search OCR página veinte');
    for(let i=0;i<golden.items.length;i++){const row=page.locator('.attachment-hit').nth(i),hit=golden.items[i];expect(await row.locator('.attachment-text').textContent()===hit.text,'Search OCR texto literal '+i);expect((await row.locator('h3').textContent()).includes('OCR inferido'),'OCR etiquetado inferido '+i);expect(await row.locator('[data-locator="confidence"]').textContent()===hit.locator.confidence,'Search confianza decimal '+i);expect(await row.locator('[data-locator="bbox"]').textContent()===JSON.stringify(hit.locator.bbox),'Search bbox decimal '+i);}
    expect(JSON.stringify(await snapshot())===JSON.stringify(searchBefore),'Buscar OCR no procesa ni indexa');
    await page.getByLabel('Ámbito de búsqueda').selectOption('combined');await page.getByLabel('Texto conjunto').fill('notekaocr');await button('Buscar notas y adjuntos').click();await page.locator('.grouped-attachment-hit').first().waitFor();await page.locator('.grouped-note-hit').first().waitFor();expect((await page.locator('.grouped-attachments').textContent()).includes('OCR inferido'),'Agrupada mantiene evidencia inferida y grupos separados');
    expect(await page.locator('.grouped-attachment-hit').count()===golden.items.length,'Agrupada conserva página de adjuntos');
    const recorded=await ops();expect(recorded.filter(o=>o.readonly_unchanged!==undefined).every(o=>o.readonly_unchanged),'Todas las lecturas preservan snapshot');expect(recorded.filter(o=>o.operation==='extraction.run').every(o=>o.input.id),'Run OCR siempre por ID explícito');expect(!recorded.some(o=>o.operation==='attachment.read'),'Sin descarga adicional base64');expect(JSON.stringify((await snapshot()).files)===JSON.stringify(initial.files),'Originales administrados intactos');expect(external.length===0,'Cero requests externos');
    await ctl('ocr_report',{theme,ok:true,checks,external_requests:external.length,operations:recorded.length});await button('Cambiar workspace').click();reports.push({theme,checks:checks.length});
  }
  console.log(JSON.stringify({reports,external_requests:external.length}));
}
