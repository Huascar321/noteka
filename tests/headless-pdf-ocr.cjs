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
    const f=await control('pdfocr_seed',{theme}), workspace=f.workspace;
    const ctl=(step,input={})=>control(step,{workspace,...input});
    const cli=async(operation,input)=>{const r=await ctl('pdfocr_cli',{operation,input});if(!r.ok)throw new Error(JSON.stringify(r));return r.data;};
    const ops=()=>ctl('pdfocr_operations'), snapshot=()=>ctl('pdfocr_snapshot');
    const button=name=>page.getByRole('button',{name,exact:true});
    const receipt=async(name)=>{await button(name).click();await page.locator('.preparation-receipt,.preparation-error').waitFor();if(await page.locator('.preparation-error').count())throw new Error(await page.locator('.preparation-error').textContent());return JSON.parse(await page.locator('.preparation-receipt pre').textContent());};
    const inspect=async()=>{await button("Query selected job").click();await page.locator('.preparation-inspection').waitFor();};
    await page.goto(origin);await page.getByLabel("Workspace path",{exact:true}).fill(workspace);await button("Open workspace").click();await page.locator('.home [data-home-section="person"] .home-coverage').waitFor();
    await page.getByRole('button', { name: 'Settings', exact: true }).click(); await page.getByLabel("Theme",{exact:true}).selectOption(theme); await page.getByRole('button', { name: "Close Settings", exact: true }).click();await page.getByRole('navigation',{name:"Sections"}).getByRole('button',{name:'Notes',exact:true}).click();
    await page.locator('.list-items button').filter({hasText:'PDF OCR headless sintético'}).click();await page.getByLabel("Note title").fill('Buffer OCR pendiente');
    const initial=await snapshot();await page.locator('.document-utilities > summary').click();await page.locator('.local-tools > summary').click();await page.locator('.preparation-panel > summary').click();await page.locator('.generation-panel > summary').click();
    expect(!(await ops()).some(o=>o.operation.startsWith('extraction.')),'Abrir nota/paneles no importa ni procesa/lee/indexa');
    await page.getByLabel("Managed ID to prepare",{exact:true}).fill(f.attachment.id);await page.getByLabel("Preparation extractor").selectOption('pdf_ocr');
    const enqueue=await receipt("Enqueue extraction");const job=enqueue.job;expect(job.extractor==='pdf_ocr'&&job.state==='queued'&&!job.generation,'Enqueue OCR explícito sin ejecución');
    await inspect();const paused=await receipt("Run selected job");expect(paused.outcome==='idle'&&paused.reason==='queue_paused','OCR pausado no ejecuta');
    await inspect();await receipt("Resume global queue");await inspect();const ran=await receipt("Run selected job");expect(ran.job.state==='succeeded'&&ran.job.generation.complete,'OCR real publica generación completa');
    const pages=[];let cursor=null;do{const p=await cli('extraction.pdf_ocr_regions',{id:job.id,limit:20,...(cursor?{cursor}:{})});pages.push(p);cursor=p.next_cursor;}while(cursor);
    expect(pages.length>=2&&pages[0].manifest.region_count>20,'Fixture OCR real supera veinte regiones');
    expect((await cli('extraction.index',{action:'get'})).coverage.indexed_generations===0,'Extraer OCR no selecciona para búsqueda');
    await page.getByLabel("Job ID to read").fill(job.id);await page.getByLabel("Reader type").selectOption('pdf_ocr');
    const beforeRead=await snapshot();await button("Read from the beginning").focus();await page.keyboard.press('Enter');await page.locator('.generation-result').waitFor();
    const checkPage=async(p,n)=>{
      expect(await page.locator('[data-reader="page"]').textContent()===String(n),'Número de página OCR '+n);
      expect(JSON.stringify(JSON.parse(await page.locator('.generation-manifest').textContent()))===JSON.stringify(p.manifest),'Manifiesto OCR íntegro '+n);
      expect(JSON.stringify(JSON.parse(await page.locator('.generation-source').textContent()))===JSON.stringify(p.generation),'Fuente/generación OCR completas '+n);
      const rows=await page.locator('.generation-item').evaluateAll(ns=>ns.map(x=>({ordinal:Number(x.dataset.ordinal),text:x.querySelector('.generation-text').textContent,locator:JSON.parse(x.querySelector('.generation-locator').textContent)})));
      expect(rows.length===p.items.length&&rows.length<=20,'Página OCR reemplazada y acotada '+n);
      for(let i=0;i<rows.length;i++){const expected=p.items[i];expect(rows[i].text===expected.text&&rows[i].ordinal===expected.ordinal,'Texto/ordinal OCR literal '+n+'/'+i);expect(JSON.stringify(rows[i].locator)===JSON.stringify({page_number:expected.page_number,region_ordinal:expected.region_ordinal,bbox:expected.bbox,confidence:expected.confidence,coordinate_space:'rendered_page_normalized_bottom_left'}),'Decimales y coordenadas OCR '+n+'/'+i);}
    };
    for(let i=0;i<pages.length;i++){if(i){await button("Next reader page").click();await page.locator('.generation-result').waitFor();}await checkPage(pages[i],i+1);}
    expect(JSON.stringify(await snapshot())===JSON.stringify(beforeRead),'Lectura completa no escribe');
    expect((await page.locator('.generation-result').textContent()).includes('no garantiza transcripción completa'),'Límite de cobertura OCR visible');
    await ctl('pdfocr_instrument',{instrument:{operation:'extraction.pdf_ocr_regions',failure:true}});await button("Read from the beginning").click();await page.locator('.generation-error').waitFor();expect(await page.locator('.generation-item').count()===0,'Error OCR retira página/cursor');expect(!(await page.evaluate(()=>Boolean(window.__ocrExecuted))),'Datos OCR no ejecutan markup');await ctl('pdfocr_instrument',{instrument:{}});
    await ctl('pdfocr_instrument',{instrument:{operation:'extraction.pdf_ocr_regions',delay:400}});const late=page.waitForResponse(r=>r.url().endsWith('/__test__/operate')&&r.request().postDataJSON().command?.operation==='extraction.pdf_ocr_regions');await button("Read from the beginning").click();await page.getByLabel("Job ID to read").fill('otro');await late;expect(await page.locator('.generation-result,.generation-error').count()===0,'Respuesta OCR tardía descartada al cambiar ID');await ctl('pdfocr_instrument',{instrument:{}});
    await inspect();await receipt("Select generation for search");expect((await cli('extraction.index',{action:'get'})).coverage.indexed_generations===1,'Selección OCR es acción independiente');
    expect(await page.getByLabel("Note title").inputValue()==='Buffer OCR pendiente','Preparar/leer conserva buffer');await button('Search').click();await page.locator('.context-search-footer button').click();await button("Cancel").click();expect(await page.getByLabel("Note title").inputValue()==='Buffer OCR pendiente','Cancelar navegación conserva buffer OCR');
    // Error real independiente: imagen inválida, sin fallback ni éxito parcial.
    await page.getByLabel("Managed ID to prepare",{exact:true}).fill(f.invalid.id);const invalid=await receipt("Enqueue extraction");await inspect();await button("Run selected job").click();await page.locator('.preparation-error').waitFor();expect((await page.locator('.preparation-error').textContent()).includes('pdf_invalid_framing'),'Error nativo real explícito en preparación');await inspect();expect(await page.locator('[data-preparation="state"]').textContent()==='failed','Error OCR persistido failed');
    await button('Search').click();await page.locator('.context-search-footer button').click();await button("Discard").click();await page.getByLabel("Search scope").selectOption('attachments');
    await page.getByLabel("Attachment extractor").selectOption('pdf_ocr');await page.getByLabel("Attachment text").fill('noteka');const searchBefore=await snapshot();await button("Search attachments").click();await page.locator('.attachment-hit').first().waitFor();
    const golden=await cli('extraction.search',{query:'noteka',extractor:'pdf_ocr',limit:20});expect(await page.locator('.attachment-hit').count()===golden.items.length,'Search OCR página veinte');
    for(let i=0;i<golden.items.length;i++){const row=page.locator('.attachment-hit').nth(i),hit=golden.items[i];expect(await row.locator('.attachment-text').textContent()===hit.text,'Search OCR texto literal '+i);expect((await row.locator('h3').textContent()).includes('OCR inferido'),'OCR etiquetado inferido '+i);expect(await row.locator('[data-locator="confidence"]').textContent()===hit.locator.confidence,'Search confianza decimal '+i);expect(await row.locator('[data-locator="bbox"]').textContent()===JSON.stringify(hit.locator.bbox),'Search bbox decimal '+i);}
    expect(JSON.stringify(await snapshot())===JSON.stringify(searchBefore),'Buscar OCR no procesa ni indexa');
    for(let i=0;i<golden.items.length;i++){const row=page.locator('.attachment-hit').nth(i),hit=golden.items[i];expect(JSON.stringify(JSON.parse(await row.locator('[data-locator="raster"]').textContent()))===JSON.stringify(hit.locator.raster),'Raster completo de Search '+i);expect(JSON.stringify(JSON.parse(await row.locator('[data-locator="engine"]').textContent()))===JSON.stringify(hit.locator.engine),'Motor completo de Search '+i);}
    expect(await page.locator('.generation-text script,.attachment-text script').count()===0,'Texto inferido permanece texto DOM');
    expect(Boolean(golden.next_cursor),'Search PDF OCR supera veinte hits');
    const next=await cli('extraction.search',{query:'noteka',extractor:'pdf_ocr',limit:20,cursor:golden.next_cursor});
    await button("Next attachment page").click();await page.waitForFunction(n=>document.querySelectorAll('.attachment-hit').length===n,next.items.length);
    expect(JSON.stringify(await page.locator('.attachment-text').allTextContents())===JSON.stringify(next.items.map(h=>h.text)),'Search segunda página exacta y reemplazada');
    await page.getByLabel("Search scope").selectOption('combined');await page.getByLabel("Combined text").fill('noteka');await button("Search notes and attachments").click();await page.locator('.grouped-attachment-hit').first().waitFor();await page.locator('.grouped-note-hit').first().waitFor();expect((await page.locator('.grouped-attachments').textContent()).includes('OCR inferido'),'Agrupada mantiene evidencia inferida y grupos separados');
    expect(await page.locator('.grouped-attachment-hit').count()===golden.items.length,'Agrupada conserva página de adjuntos');
    const noteGroup=await page.locator('.grouped-note-hit').allTextContents();
    const groupGolden=await cli('extraction.search',{query:'noteka',limit:20});
    for(let i=0;i<groupGolden.items.length;i++)expect(JSON.stringify(JSON.parse(await page.locator('.grouped-attachment-locator').nth(i).textContent()))===JSON.stringify(groupGolden.items[i].locator),'Agrupada localizador PDF OCR íntegro '+i);
    const groupNext=await cli('extraction.search',{query:'noteka',limit:20,cursor:groupGolden.next_cursor});
    await button("Next combined attachment page").click();await page.waitForFunction(n=>document.querySelectorAll('.grouped-attachment-hit').length===n,groupNext.items.length);
    expect(JSON.stringify(await page.locator('.grouped-attachment-text').allTextContents())===JSON.stringify(groupNext.items.map(h=>h.text)),'Cursor agrupado PDF OCR exacto');
    expect(JSON.stringify(await page.locator('.grouped-note-hit').allTextContents())===JSON.stringify(noteGroup),'Paginar PDF OCR conserva grupo de notas');
    await ctl('pdfocr_instrument',{instrument:{operation:'extraction.search',failure:true}});await button("First combined attachment page").click();await page.locator('.grouped-attachments [role=alert]').waitFor();
    expect(await page.locator('.grouped-attachment-hit').count()===0,'Fallo PDF OCR retira sólo grupo de adjuntos');expect(JSON.stringify(await page.locator('.grouped-note-hit').allTextContents())===JSON.stringify(noteGroup),'Fallo PDF OCR conserva notas');
    await ctl('pdfocr_instrument',{instrument:{}});await button("Retry attachments").click();await page.locator('.grouped-attachment-hit').first().waitFor();expect(await page.locator('.grouped-attachment-hit').count()===groupGolden.items.length,'Reintento PDF OCR explícito por grupo');

    const recorded=await ops();expect(recorded.filter(o=>o.readonly_unchanged!==undefined).every(o=>o.readonly_unchanged),'Todas las lecturas preservan snapshot');expect(recorded.filter(o=>o.operation==='extraction.run').every(o=>o.input.id),'Run OCR siempre por ID explícito');expect(!recorded.some(o=>o.operation==='attachment.read'),'Sin descarga adicional base64');expect(JSON.stringify((await snapshot()).files)===JSON.stringify(initial.files),'Originales administrados intactos');expect(external.length===0,'Cero requests externos');
    await ctl('pdfocr_report',{theme,ok:true,checks,external_requests:external.length,operations:recorded.length});await page.getByRole('button', { name: 'Settings', exact: true }).click(); await page.locator('.settings-navigation').getByRole('button', { name: 'General', exact: true }).click(); await button("Change workspace").click();reports.push({theme,checks:checks.length});
  }
  console.log(JSON.stringify({reports,external_requests:external.length}));
}
