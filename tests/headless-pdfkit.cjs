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
    const f=await control('pdfkit_seed',{theme}), workspace=f.workspace;
    const ctl=(step,input={})=>control(step,{workspace,...input});
    const cli=async(operation,input)=>{const r=await ctl('pdfkit_cli',{operation,input});if(!r.ok)throw new Error(JSON.stringify(r));return r.data;};
    const ops=()=>ctl('pdfkit_operations'), snapshot=()=>ctl('pdfkit_snapshot');
    const button=name=>page.getByRole('button',{name,exact:true});
    const receipt=async(name)=>{await button(name).click();await page.locator('.preparation-receipt,.preparation-error').waitFor();if(await page.locator('.preparation-error').count())throw new Error(await page.locator('.preparation-error').textContent());return JSON.parse(await page.locator('.preparation-receipt pre').textContent());};
    const inspect=async()=>{await button("Query selected job").click();await page.locator('.preparation-inspection').waitFor();};
    await page.goto(origin);await page.getByLabel("Workspace path",{exact:true}).fill(workspace);await button("Open workspace").click();await page.locator('.home [data-home-section="person"] .home-coverage').waitFor();
    await page.getByRole('button', { name: 'Settings', exact: true }).click(); await page.getByLabel("Theme",{exact:true}).selectOption(theme); await page.getByRole('button', { name: "Close Settings", exact: true }).click();await page.getByRole('navigation',{name:"Sections"}).getByRole('button',{name:'Notes',exact:true}).click();
    await page.locator('.list-items button').filter({hasText:'PDFKit headless sintético'}).click();await page.getByLabel("Note title").fill('Buffer PDFKit pendiente');
    const initial=await snapshot();await page.locator('.document-utilities > summary').click();await page.locator('.local-tools > summary').click();await page.locator('.preparation-panel > summary').click();await page.locator('.generation-panel > summary').click();
    expect(!(await ops()).some(o=>o.operation.startsWith('extraction.')),'Abrir nota/paneles no importa ni procesa/lee/indexa');
    await page.getByLabel("Managed ID to prepare",{exact:true}).fill(f.attachment.id);await page.getByLabel("Preparation extractor").selectOption('pdfkit_text');
    const enqueue=await receipt("Enqueue extraction");const job=enqueue.job;expect(job.extractor==='pdfkit_text'&&job.state==='queued'&&!job.generation,'Enqueue PDFKit explícito sin ejecución');
    await inspect();const paused=await receipt("Run selected job");expect(paused.outcome==='idle'&&paused.reason==='queue_paused','PDFKit pausado no ejecuta');
    await inspect();await receipt("Resume global queue");await inspect();const ran=await receipt("Run selected job");expect(ran.job.state==='succeeded'&&ran.job.generation.complete,'PDFKit real publica generación completa');
    const pages=[];let cursor=null;do{const p=await cli('extraction.pdfkit_pages',{id:job.id,limit:20,...(cursor?{cursor}:{})});pages.push(p);cursor=p.next_cursor;}while(cursor);
    const native=pages.flatMap(p=>p.items);
    expect(pages.length===2&&native.length===25,'PDF nativo real: 25 páginas, paginadas 20/5');
    for(let i=0;i<native.length;i++)expect(native[i].number===i+1&&native[i].text===f.expected[i],'Golden CoreText previo al lector '+i);
    expect(native.filter(p=>p.status==='no_text').length===2,'Páginas físicas sin texto conservadas');
    expect((await cli('extraction.index',{action:'get'})).coverage.indexed_generations===0,'Extraer PDFKit no selecciona para búsqueda');
    await page.getByLabel("Job ID to read").fill(job.id);await page.getByLabel("Reader type").selectOption('pdfkit_text');
    const beforeRead=await snapshot();await button("Read from the beginning").focus();await page.keyboard.press('Enter');await page.locator('.generation-result').waitFor();
    const checkPage=async(p,n)=>{
      expect(await page.locator('[data-reader="page"]').textContent()===String(n),'Número de página del lector '+n);
      expect(JSON.stringify(JSON.parse(await page.locator('.generation-manifest').textContent()))===JSON.stringify(p.manifest),'Manifiesto PDFKit íntegro '+n);
      expect(JSON.stringify(JSON.parse(await page.locator('.generation-source').textContent()))===JSON.stringify(p.generation),'Fuente/generación completas '+n);
      const rows=await page.locator('.generation-item').evaluateAll(ns=>ns.map(x=>({ordinal:Number(x.dataset.ordinal),physical:Number(x.dataset.physicalPage),text:x.querySelector('.generation-text').textContent,locator:JSON.parse(x.querySelector('.generation-locator').textContent)})));
      expect(rows.length===p.items.length&&rows.length<=20,'Página reemplazada y acotada '+n);
      for(let i=0;i<rows.length;i++){const expected=p.items[i];expect(rows[i].text===expected.text&&rows[i].ordinal===expected.number-1&&rows[i].physical===expected.number,'Texto y página física literal '+n+'/'+i);expect(JSON.stringify(rows[i].locator)===JSON.stringify({page_number:expected.number,status:expected.status}),'Estado/localizador PDFKit '+n+'/'+i);}
    };
    for(let i=0;i<pages.length;i++){if(i){await button("Next reader page").click();await page.locator('.generation-result').waitFor();}await checkPage(pages[i],i+1);}
    expect(JSON.stringify(await snapshot())===JSON.stringify(beforeRead),'Lectura completa no escribe');
    expect((await page.locator('.generation-result').textContent()).includes('no_text no prueba ausencia visual'),'Límite de cobertura PDFKit visible');
    expect(!(await page.evaluate(()=>Boolean(window.__pdfkitExecuted))),'Texto nativo no ejecuta markup');
    await ctl('pdfkit_instrument',{instrument:{operation:'extraction.pdfkit_pages',failure:true}});await button("Read from the beginning").click();await page.locator('.generation-error').waitFor();expect(await page.locator('.generation-item').count()===0,'Error retira página/cursor');await ctl('pdfkit_instrument',{instrument:{}});
    await ctl('pdfkit_instrument',{instrument:{operation:'extraction.pdfkit_pages',delay:400}});const late=page.waitForResponse(r=>r.url().endsWith('/__test__/operate')&&r.request().postDataJSON().command?.operation==='extraction.pdfkit_pages');await button("Read from the beginning").click();await page.getByLabel("Job ID to read").fill('otro');await late;expect(await page.locator('.generation-result,.generation-error').count()===0,'Respuesta tardía descartada al cambiar ID');await ctl('pdfkit_instrument',{instrument:{}});
    await inspect();await receipt("Select generation for search");const selected=await cli('extraction.index',{action:'get'});expect(selected.coverage.indexed_generations===1&&selected.coverage.fragments===23,'Selección explícita: 23 páginas con texto, sin fragmentos vacíos');
    expect(await page.getByLabel("Note title").inputValue()==='Buffer PDFKit pendiente','Preparar/leer conserva buffer');await button('Search').click();await page.locator('.context-search-footer button').click();await button("Cancel").click();expect(await page.getByLabel("Note title").inputValue()==='Buffer PDFKit pendiente','Cancelar navegación conserva buffer');
    await page.getByLabel("Managed ID to prepare",{exact:true}).fill(f.invalid.id);await receipt("Enqueue extraction");await inspect();await button("Run selected job").click();await page.locator('.preparation-error').waitFor();expect((await page.locator('.preparation-error').textContent()).includes('pdf_invalid_framing'),'Error nativo real explícito, sin fallback');await inspect();expect(await page.locator('[data-preparation="state"]').textContent()==='failed','Error PDFKit persistido failed');
    await button('Search').click();await page.locator('.context-search-footer button').click();await button("Discard").click();await page.getByLabel("Search scope").selectOption('attachments');
    await page.getByLabel("Attachment extractor").selectOption('pdfkit_text');await page.getByLabel("Attachment text").fill('notekapdfkit');const searchBefore=await snapshot();await button("Search attachments").click();await page.locator('.attachment-hit').first().waitFor();
    let golden=await cli('extraction.search',{query:'notekapdfkit',extractor:'pdfkit_text',limit:20});
    const checkHits=async(p)=>{
      expect(await page.locator('.attachment-hit').count()===p.items.length,'Search reemplaza página: '+p.items.length);
      for(let i=0;i<p.items.length;i++){
        const row=page.locator('.attachment-hit').nth(i),hit=p.items[i],loc=hit.locator;
        expect(await row.locator('.attachment-text').textContent()===hit.text,'Search texto literal '+loc.page_number);
        expect((await row.locator('h3').textContent())===`PDFKit · página ${loc.page_number}`,'Título tipado '+loc.page_number);
        for(const field of ['kind','projection','profile','page_number','page_bytes','text_byte_start','text_byte_end'])expect(await row.locator(`[data-locator="${field}"]`).textContent()===String(loc[field]),'Localizador '+field+'/'+loc.page_number);
        expect(await row.locator('[data-locator="engine"]').textContent()===JSON.stringify(loc.engine),'Motor/OS íntegros '+loc.page_number);
        expect(await row.locator('[data-locator="limitations"]').textContent()===loc.limitations.join(', '),'Limitaciones íntegras '+loc.page_number);
        const original=native.find(p=>p.number===loc.page_number).text;
        const sliced=await page.evaluate(({text,start,end})=>new TextDecoder('utf-8',{fatal:true}).decode(new TextEncoder().encode(text).slice(start,end)),{text:original,start:loc.text_byte_start,end:loc.text_byte_end});
        expect(sliced===hit.text,'Rango UTF-8 independiente '+loc.page_number);
      }
    };
    await checkHits(golden);expect(golden.items.length===20&&golden.next_cursor,'Primera página Search 20 con cursor');await button("Next attachment page").click();await page.locator('.attachment-status').filter({hasText:'Página 2'}).waitFor();golden=await cli('extraction.search',{query:'notekapdfkit',extractor:'pdfkit_text',limit:20,cursor:golden.next_cursor});await checkHits(golden);expect(golden.items.length===3&&!golden.next_cursor,'Segunda página Search 3 sin cursor');
    expect(JSON.stringify(await snapshot())===JSON.stringify(searchBefore),'Buscar no procesa ni indexa');
    await page.getByLabel("Search scope").selectOption('combined');await page.getByLabel("Combined text").fill('notekapdfkit');await button("Search notes and attachments").click();await page.locator('.grouped-attachment-hit').first().waitFor();await page.locator('.grouped-note-hit').first().waitFor();
    expect((await page.locator('.grouped-attachments').textContent()).includes('PDFKit: página/rango del texto derivado'),'Agrupada distingue PDFKit de XLSX/OCR');expect(await page.locator('.grouped-attachment-hit').count()===20,'Agrupada mantiene páginas separadas');
    const groupedGolden=await cli('extraction.search',{query:'notekapdfkit',limit:20});for(let i=0;i<groupedGolden.items.length;i++)expect(JSON.stringify(JSON.parse(await page.locator('.grouped-attachment-locator').nth(i).textContent()))===JSON.stringify(groupedGolden.items[i].locator),'Agrupada conserva localizador íntegro '+i);
    const notes=await page.locator('.grouped-notes').textContent(), index=await cli('extraction.index',{action:'get'});await cli('extraction.index',{action:'reset',expected_revision:index.revision});await button("Next combined attachment page").click();await page.locator('.grouped-attachments [role="alert"]').waitFor();expect((await page.locator('.grouped-attachments [role="alert"]').textContent()).includes('revision'),'Cursor obsoleto falla explícitamente');expect(await page.locator('.grouped-notes').textContent()===notes,'Error de adjuntos conserva grupo de notas');
    expect(!(await page.evaluate(()=>Boolean(window.__pdfkitExecuted))),'Search individual/agrupada no ejecuta datos');
    const recorded=await ops();expect(recorded.filter(o=>o.readonly_unchanged!==undefined).every(o=>o.readonly_unchanged),'Todas las lecturas preservan snapshot');expect(recorded.filter(o=>o.operation==='extraction.run').every(o=>o.input.id),'Run siempre por ID explícito');expect(!recorded.some(o=>o.operation==='attachment.read'),'Sin descarga adicional base64');expect(JSON.stringify((await snapshot()).files)===JSON.stringify(initial.files),'Originales administrados intactos');expect(external.length===0,'Cero requests externos');
    await ctl('pdfkit_report',{theme,ok:true,checks,external_requests:external.length,operations:recorded.length});await page.getByRole('button', { name: 'Settings', exact: true }).click(); await page.locator('.settings-navigation').getByRole('button', { name: 'General', exact: true }).click(); await button("Change workspace").click();reports.push({theme,checks:checks.length});
  }
  console.log(JSON.stringify({reports,external_requests:external.length}));
}
