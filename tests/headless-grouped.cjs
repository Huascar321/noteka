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
  for (const theme of ['dark','light']) {
    const checks=[], expect=(ok,label)=>{if(!ok)throw new Error(theme+': '+label);checks.push(label);};
    const f=await control('evidence_seed',{theme}), workspace=f.workspace;
    const direct=async(operation,input)=>{const r=await page.request.post(origin+'/__test__/operate',{data:{workspace,command:{operation,input}}});const v=await r.json();if(!v.ok)throw new Error(JSON.stringify(v));return v.data;};
    for(let i=0;i<23;i++) await direct('entity.create',{kind:'note',title:'nkprobe Nota '+i+' <script>dato</script>',document:{schema_version:2,blocks:[{id:'combined-'+i,type:'paragraph',content:[{type:'text',text:'nkprobe bloque '+i+' café 🚀'}]}]}});
    const goldenNotes=[];let cursor;
    do{const p=await direct('search.query',{query:'nkprobe',archived:false,limit:20,...(cursor?{cursor}:{} )});goldenNotes.push(p);cursor=p.next_cursor;}while(cursor);
    const snapshot=()=>control('evidence_snapshot',{workspace}), operations=()=>control('evidence_operations',{workspace});
    const button=name=>page.getByRole('button',{name,exact:true});
    await page.goto(origin); await page.getByLabel('Ruta del workspace').fill(workspace); await button('Abrir workspace').click();await page.locator('.home [data-home-section="person"] .home-coverage').waitFor();
    await page.getByLabel('Tema',{exact:true}).selectOption(theme);await button('Search').click();await page.getByLabel('Ámbito de búsqueda').selectOption('combined');
    const start=(await operations()).length, before=await snapshot(); expect(await page.locator('.grouped-note-hit,.grouped-attachment-hit').count()===0,'Montar no consulta ni prepara');
    const settled=()=>page.waitForFunction(()=>!document.querySelector('.grouped-search [role="status"]'));
    await page.getByLabel('Texto conjunto').fill('nkprobe');await page.getByLabel('Texto conjunto').focus();await page.keyboard.press('Enter');await settled();
    const noteRows=()=>page.locator('.grouped-note-hit details pre').evaluateAll(nodes=>nodes.map(n=>JSON.parse(n.textContent)));
    const attRows=()=>page.locator('.grouped-attachment-hit').evaluateAll(nodes=>nodes.map(n=>({text:n.querySelector('.grouped-attachment-text').textContent,source:JSON.parse(n.querySelector('.grouped-attachment-source').textContent),locator:JSON.parse(n.querySelector('.grouped-attachment-locator').textContent)})));
    const assertNotes=async expected=>expect(JSON.stringify(await noteRows())===JSON.stringify(expected.items),'Notas exactas/orden/fuente/página de CLI');
    const assertAtt=async expected=>{const rows=await attRows();expect(rows.length===expected.items.length&&rows.length<=20,'Adjuntos página acotada/reemplazada');for(let i=0;i<rows.length;i++){expect(rows[i].text===expected.items[i].text,'Texto canónico '+i);expect(JSON.stringify(rows[i].source)===JSON.stringify(expected.items[i].source),'Fuente '+i);expect(JSON.stringify(rows[i].locator)===JSON.stringify(expected.items[i].locator),'Localizador '+i);}};
    await assertNotes(goldenNotes[0]);await assertAtt(f.pages[0]);
    const cov=JSON.parse(await page.locator('.grouped-coverage').textContent());expect(Object.entries(f.pages[0].coverage).every(([k,v])=>cov[k]===v),'Cobertura completa real sin total inventado');
    await page.locator('.grouped-note-hit button').first().click();
    // entity.get/openOriginal terminan antes de editar el buffer de esa nota.
    await page.waitForFunction(id=>document.querySelector('.source-locator')?.textContent.includes(id),goldenNotes[0].items[0].entity_id,{timeout:5000});
    await page.getByLabel('Título de la nota').fill('Buffer conjunto conservado');
    const attFirst=await attRows();
    for(let i=1;i<goldenNotes.length;i++){await button('Siguiente página de entidades').focus();await page.keyboard.press('Enter');await settled();await assertNotes(goldenNotes[i]);expect(JSON.stringify(await attRows())===JSON.stringify(attFirst),'Paginar notas conserva adjuntos');}
    const noteLast=await noteRows();await button('Siguiente página de adjuntos conjuntos').click();await settled();await assertAtt(f.pages[1]);expect(JSON.stringify(await noteRows())===JSON.stringify(noteLast),'Paginar adjuntos conserva notas');
    expect(await page.getByLabel('Título de la nota').inputValue()==='Buffer conjunto conservado','Consultar/paginar conserva buffer');expect(JSON.stringify(await snapshot())===JSON.stringify(before),'Lecturas y apertura no escriben SQLite/originales');
    await page.locator('.grouped-note-hit button').first().click();await button('Cancelar').click();expect(await page.getByLabel('Título de la nota').inputValue()==='Buffer conjunto conservado','Abrir otro hit mantiene guardia/buffer');
    await button('Inicio de adjuntos conjuntos').click();await settled();await control('evidence_reset',{workspace});await button('Siguiente página de adjuntos conjuntos').click();await settled();
    expect((await page.locator('.grouped-attachments').textContent()).includes('revision_conflict')&&await page.locator('.grouped-attachment-hit').count()===0,'Cursor vencido retira sólo adjuntos');expect(JSON.stringify(await noteRows())===JSON.stringify(noteLast),'Conflicto conserva snapshot de notas');
    let fail=true, delay=false;
    await page.route('**/__test__/operate',async route=>{const body=route.request().postDataJSON();if(delay&&body.command.operation==='search.query')await new Promise(r=>setTimeout(r,300));if(fail&&body.command.operation==='search.query')await route.fulfill({status:200,contentType:'application/json',body:JSON.stringify({api_version:1,request_id:'headless',ok:false,error:{code:'storage_error',message:'<script>fallo de notas</script>',details:{reason:'injected'}}})});else await route.continue();});
    await button('Buscar notas y adjuntos').click();await settled();expect((await page.locator('.grouped-notes').textContent()).includes('storage_error'),'Error parcial de notas explícito');expect(await page.locator('.grouped-coverage').count()===1,'Adjuntos válido pese al error de notas');
    const countBefore=(await operations()).filter(o=>o.operation==='extraction.search').length;fail=false;await button('Reintentar entidades').click();await settled();await assertNotes(goldenNotes[0]);expect((await operations()).filter(o=>o.operation==='extraction.search').length===countBefore,'Reintentar notas no repite adjuntos');
    await page.getByLabel('Texto conjunto').fill('otra consulta');expect(await page.locator('.grouped-note-hit,.grouped-attachment-hit,.grouped-coverage').count()===0,'Editar retira ambos grupos sin consultar');
    const overlong='🚀'.repeat(100);
    const invalidResponses=Promise.all(['search.query','extraction.search'].map(operation=>page.waitForResponse(response=>{
      if(!response.url().endsWith('/__test__/operate'))return false;
      const command=response.request().postDataJSON().command;
      return command?.operation===operation&&command.input.query===overlong;
    })));
    await page.getByLabel('Texto conjunto').fill(overlong);await button('Buscar notas y adjuntos').click();
    const invalid=await Promise.all((await invalidResponses).map(response=>response.json()));
    await page.waitForFunction(()=>['.grouped-notes','.grouped-attachments'].every(selector=>document.querySelector(selector+' [role="alert"] strong')?.textContent.includes('validation_error')),{},{timeout:5000});
    expect(invalid.every(response=>response.ok===false&&response.error.code==='validation_error')&&(await page.locator('.grouped-notes').textContent()).includes('validation_error')&&(await page.locator('.grouped-attachments').textContent()).includes('validation_error'),'Core rechaza exceso UTF-8 sin truncamiento');
    delay=true;await page.getByLabel('Texto conjunto').fill('nkprobe');const late=page.waitForResponse(r=>r.url().endsWith('/__test__/operate')&&r.request().postDataJSON().command?.operation==='search.query');await button('Buscar notas y adjuntos').click();await page.getByLabel('Texto conjunto').fill('consulta nueva');await late;delay=false;expect(await page.locator('.grouped-note-hit,.grouped-attachment-hit,.grouped-coverage,.grouped-search [role="alert"]').count()===0,'Respuesta tardía no publica bajo criterios nuevos');
    const seen=(await operations()).slice(start);expect(seen.filter(o=>o.operation.startsWith('extraction.')).every(o=>o.operation==='extraction.search'),'Sin preparación/lectores/reset/indexación desde UI');expect(seen.filter(o=>o.operation==='extraction.search').every(o=>o.readonly_unchanged),'Cada búsqueda de adjuntos conserva snapshot');
    await page.setViewportSize({width:900,height:800});expect(await page.locator('.grouped-search').evaluate(n=>n.scrollWidth<=n.clientWidth+1),'Viewport estrecho sin overflow');expect(external.length===0,'Sin requests remotos');
    await control('evidence_report',{workspace,theme,ok:true,checks,external_requests:external.length});await button('Cambiar workspace').click();await button('Descartar').click();reports.push({theme,checks:checks.length});
  }
  console.log(JSON.stringify({reports,external_requests:external.length}));
}
