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
    const f=await control('pdftable_seed',{theme}), workspace=f.workspace;
    const ctl=(step,input={})=>control(step,{workspace,...input});
    const cli=async(operation,input)=>{const r=await ctl('pdfocr_cli',{operation,input});if(!r.ok)throw new Error(JSON.stringify(r));return r.data;};
    const ops=()=>ctl('pdfocr_operations'), snapshot=()=>ctl('pdfocr_snapshot');
    const button=name=>page.getByRole('button',{name,exact:true});
    const receipt=async(name)=>{await button(name).click();await page.locator('.preparation-receipt,.preparation-error').waitFor();if(await page.locator('.preparation-error').count())throw new Error(await page.locator('.preparation-error').textContent());return JSON.parse(await page.locator('.preparation-receipt pre').textContent());};
    const inspect=async()=>{await button("Query selected job").click();await page.locator('.preparation-inspection').waitFor();};
    await page.goto(origin);await page.getByLabel("Workspace path",{exact:true}).fill(workspace);await button("Open workspace").click();await page.locator('.home [data-home-section="person"] .home-coverage').waitFor();
    await page.getByRole('button', { name: 'Settings', exact: true }).click(); await page.getByLabel("Theme",{exact:true}).selectOption(theme); await page.getByRole('button', { name: "Close Settings", exact: true }).click();await page.getByRole('navigation',{name:"Sections"}).getByRole('button',{name:'Notes',exact:true}).click();
    await page.locator('.list-items button').filter({hasText:'PDF OCR headless sintético'}).click();await page.getByLabel("Note title").fill('Buffer tabla PDF pendiente');
    const initial=await snapshot();await page.locator('.document-utilities > summary').click();await page.locator('.local-tools > summary').click();await page.locator('.preparation-panel > summary').click();await page.locator('.generation-panel > summary').click();
    expect(!(await ops()).some(o=>o.operation.startsWith('extraction.')),'Panel tabla no procesa ni consulta al montar');
    await page.getByLabel("Managed ID to prepare",{exact:true}).fill(f.attachment.id);await page.getByLabel("Preparation extractor").selectOption('pdf_ocr');
    const job=(await receipt("Enqueue extraction")).job;await inspect();await receipt("Resume global queue");await inspect();await receipt("Run selected job");
    const request={id:job.id,page_number:2,x_edges:['0','0.5','1'],y_min:'0.4',y_max:'1',header_row:1,limit:20};
    const first=await cli('extraction.pdf_ocr_table',request), second=await cli('extraction.pdf_ocr_table',{...request,cursor:first.next_cursor});
    expect(first.cell_count===22&&first.row_count===11&&first.manifest.page_count===3,'Tabla real tres páginas y once filas');expect(first.outside.length===1,'Región Footer exterior preservada como referencia');
    const golden=[['Item','Count'],...Array.from({length:10},(_,i)=>[i%2===0?'Alpha':'Beta',i%2===0?'12.50':'003'])];
    const cells=[...first.items,...second.items];for(let r=0;r<11;r++)for(let c=0;c<2;c++)expect(cells[r*2+c].regions.map(x=>x.text).join('\n')===golden[r][c],'Golden previo exacto '+r+'/'+c);
    await page.getByLabel("Reader type").selectOption('pdf_ocr_table');await page.getByLabel("Job ID to read").fill(job.id);await page.getByLabel("Physical PDF page",{exact:true}).fill('2');await page.getByLabel("PDF column boundaries").fill('0,0.5,1');await page.getByLabel("PDF minimum Y").fill('0.4');await page.getByLabel("PDF header row",{exact:true}).fill('1');
    const before=await snapshot();await button("Read from the beginning").focus();await page.keyboard.press('Enter');await page.locator('.generation-result').waitFor();
    const canonical=value=>JSON.stringify(value,(_k,v)=>v&&typeof v==='object'&&!Array.isArray(v)?Object.fromEntries(Object.entries(v).sort(([a],[b])=>a.localeCompare(b))):v);
    const compare=async(expected)=>{
      const got=await page.locator('.generation-item').evaluateAll(ns=>ns.map(n=>({ordinal:Number(n.dataset.ordinal),row:Number(n.dataset.row),column:Number(n.dataset.column),regions:[...n.querySelectorAll('.generation-locator')].map(p=>JSON.parse(p.textContent)),texts:[...n.querySelectorAll('.generation-text')].map(p=>p.textContent)})));
      expect(got.length===expected.items.length&&got.length<=20,'Página veinte reemplazada');
      for(let n=0;n<got.length;n++){const a=got[n],b=expected.items[n];expect(a.ordinal===b.ordinal&&a.row===b.row&&a.column===b.column,'Celda ordinal/fila/columna '+b.ordinal);expect(canonical(a.regions)===canonical(b.regions),'Regiones/procedencia completa '+b.ordinal);expect(canonical(a.texts)===canonical(b.regions.map(r=>r.text)),'Texto literal '+b.ordinal);}
      expect(await page.locator('[data-reader=covers]').textContent()==='false','Tabla no declara cubrir generación');expect((await page.locator('[data-reader=table-coverage]').textContent()).includes('página cubre tabla: false'),'Cobertura de página parcial visible');
      expect(canonical(JSON.parse(await page.locator('.generation-outside').textContent()))===canonical(expected.outside),'Referencias exteriores completas');
    };
    await compare(first);await button("Next reader page").click();await page.waitForFunction(()=>document.querySelector('.generation-item')?.dataset.ordinal==='20');await compare(second);
    expect(canonical(await snapshot())===canonical(before),'Lectura tabla no escribe ni indexa');
    await page.getByLabel("PDF column boundaries").fill('0,1');await button("Read from the beginning").click();await page.locator('.generation-result').waitFor();expect(await page.locator('.generation-item').count()===11,'Una columna conserva once filas');expect(await page.locator('.generation-item').first().locator('.generation-text').count()===2,'Dos regiones de la misma celda se conservan');expect(await page.locator('[data-reader=covers]').textContent()==='false','Tabla completa no acredita generación');expect((await page.locator('[data-reader=table-coverage]').textContent()).includes('página cubre tabla: true'),'Cobertura de tabla completa explícita');
    await page.getByLabel("PDF column boundaries").fill('0,0.25,0.5,1');await button("Read from the beginning").click();await page.locator('.generation-result').waitFor();expect(await page.locator('.generation-cell-empty').count()>0,'Columna vacía conserva sus celdas');expect(await page.locator('.generation-item').first().locator('.generation-text').count()===0,'Celda vacía no toma texto vecino');await page.getByLabel("PDF column boundaries").fill('0,0.5,1');

    await page.getByLabel("PDF column boundaries").fill(Array.from({length:34},(_,i)=>String(i/33)).join(','));await button("Read from the beginning").click();await page.locator('.generation-error').waitFor();expect((await page.locator('.generation-error').textContent()).includes('invalid_pdf_table_input'),'Rust rechaza grid que supera32 columnas');expect(await page.locator('.generation-item').count()===0,'Cupo inválido no devuelve celdas parciales');await page.getByLabel("PDF column boundaries").fill('0,0.5,1');
    await ctl('pdfocr_instrument',{instrument:{operation:'extraction.pdf_ocr_table',hostileText:true}});await button("Read from the beginning").click();await page.locator('.generation-result').waitFor();expect((await page.locator('.generation-text').first().textContent()).includes('<img src=x'),'Fixture de transporte hostil permanece texto literal');expect(await page.locator('.generation-text img,.generation-text script').count()===0,'Tabla no crea nodos desde markup');expect(!await page.evaluate(()=>Boolean(window.__tableExecuted)),'Tabla no ejecuta texto hostil');await ctl('pdfocr_instrument',{instrument:{}});

    await page.getByLabel("Physical PDF page",{exact:true}).fill('1');await page.getByLabel("PDF header row",{exact:true}).fill('');await button("Read from the beginning").click();await page.locator('.generation-empty').waitFor();expect(await page.locator('.generation-item').count()===0,'Página física vacía preservada');
    await page.getByLabel("Physical PDF page",{exact:true}).fill('2');await page.getByLabel("PDF header row",{exact:true}).fill('1');
    const bbox=cells[0].regions[0].bbox,crossing=Number(bbox.x)+Number(bbox.width)/2;
    await page.getByLabel("PDF column boundaries").fill('0,'+String(crossing)+',1');await button("Read from the beginning").click();await page.locator('.generation-error').waitFor();expect((await page.locator('.generation-error').textContent()).includes('pdf_table_crossing_region'),'Cruce real no divide texto ni devuelve éxito parcial');await page.getByLabel("PDF column boundaries").fill('0,0.5,1');
    for(const [step,reason] of [['pdftable_tamper','input_stale'],['pdftable_corrupt','invalid_pdf_ocr_integrity']]){await ctl(step,{enabled:true});await button("Read from the beginning").click();await page.locator('.generation-error').waitFor();expect((await page.locator('.generation-error').textContent()).includes(reason),'Revalidación íntegra '+reason);expect(await page.locator('.generation-item').count()===0,'Error retira celdas');await ctl(step,{enabled:false});}
    await ctl('pdfocr_instrument',{instrument:{operation:'extraction.pdf_ocr_table',failure:true}});await button("Read from the beginning").click();await page.locator('.generation-error').waitFor();expect(await page.locator('.generation-item').count()===0,'Error de transporte retira página');await ctl('pdfocr_instrument',{instrument:{}});
    await ctl('pdfocr_instrument',{instrument:{operation:'extraction.pdf_ocr_table',delay:400}});const late=page.waitForResponse(r=>r.url().endsWith('/__test__/operate')&&r.request().postDataJSON().command?.operation==='extraction.pdf_ocr_table');await button("Read from the beginning").click();await page.getByLabel("PDF minimum Y").fill('0.41');await late;expect(await page.locator('.generation-result,.generation-error').count()===0,'Respuesta tardía descartada al editar grid');await ctl('pdfocr_instrument',{instrument:{}});await page.getByLabel("PDF minimum Y").fill('0.4');
    await button("Read from the beginning").click();await page.locator('.generation-result').waitFor();expect(!(await page.locator('.generation-text script,.generation-locator script').count()),'Texto/regiones permanecen datos escapados');
    expect(await page.getByLabel("Note title").inputValue()==='Buffer tabla PDF pendiente','Lectura preserva buffer');await button('Search').click();await page.locator('.context-search-footer button').click();await button("Cancel").click();expect(await page.getByLabel("Note title").inputValue()==='Buffer tabla PDF pendiente','Cancelar navegación conserva buffer');
    await inspect();await receipt("Select generation for search");
    await page.locator('[data-reader=table-coverage]').scrollIntoViewIfNeeded();await page.screenshot({path:f.workspace.slice(0,f.workspace.lastIndexOf('/'))+'/pdf-table-'+theme+'.png'});
    await button('Search').click();await page.locator('.context-search-footer button').click();await button("Discard").click();await page.getByLabel("Search scope").selectOption('attachments');await page.getByLabel("Attachment extractor").selectOption('pdf_ocr');await page.getByLabel("Attachment text").fill('Alpha');await button("Search attachments").click();await page.locator('.attachment-hit').first().waitFor();
    const hits=await cli('extraction.search',{query:'Alpha',extractor:'pdf_ocr',limit:20});expect(hits.items.length===5,'Selección conserva cinco regiones Alpha sin duplicación por tabla');
    for(let n=0;n<hits.items.length;n++){const h=hits.items[n],region=cells.flatMap(c=>c.regions).find(r=>r.ordinal===h.locator.global_region_ordinal);expect(Boolean(region)&&canonical(region.bbox)===canonical(h.locator.bbox),'Hit corresponde a región/celda '+n);expect(await page.locator('.attachment-text').nth(n).textContent()===h.text,'Search individual texto exacto '+n);}
    await page.getByLabel("Search scope").selectOption('combined');await page.getByLabel("Combined text").fill('Alpha');await button("Search notes and attachments").click();await page.locator('.grouped-attachment-hit').first().waitFor();await page.locator('.grouped-note-hit').first().waitFor();expect(await page.locator('.grouped-attachment-hit').count()===5,'Agrupada no crea otro índice de tablas');
    for(let n=0;n<hits.items.length;n++)expect(canonical(JSON.parse(await page.locator('.grouped-attachment-locator').nth(n).textContent()))===canonical(hits.items[n].locator),'Agrupada conserva región de celda '+n);
    const recorded=await ops();expect(recorded.filter(o=>o.readonly_unchanged!==undefined).every(o=>o.readonly_unchanged),'Snapshot idéntico antes/después de cada lectura');expect(!recorded.some(o=>o.operation==='attachment.read'),'Sin descargar originales/base64');expect(canonical((await snapshot()).files)===canonical(initial.files),'Originales conservados');expect(external.length===0,'Cero requests externos');
    await ctl('pdftable_report',{theme,ok:true,checks,external_requests:external.length,operations:recorded.length});await page.getByRole('button', { name: 'Settings', exact: true }).click(); await page.locator('.settings-navigation').getByRole('button', { name: 'General', exact: true }).click(); await button("Change workspace").click();reports.push({theme,checks:checks.length});
  }
  console.log(JSON.stringify({reports,external_requests:external.length}));
}
