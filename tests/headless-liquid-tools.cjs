async page => {
  const origin=new URL(page.url()).origin,checks=[],errors=[],external=[];let stage='setup';
  try {
    page.on('pageerror',e=>errors.push(String(e)));
    await page.context().route('**/*',async route=>{if(new URL(route.request().url()).origin===origin)await route.continue();else {external.push(route.request().url());await route.abort();}});
    await page.addInitScript(()=>{let next=1;window.isTauri=true;window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener(){}};window.__TAURI_INTERNALS__={transformCallback(){return next++;},unregisterCallback(){},async invoke(cmd,args={}){if(cmd==='operate')return (await fetch('/__test__/operate',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(args.request)})).json();if(['pending_edits','close_confirmed','plugin:event|listen','plugin:event|unlisten'].includes(cmd))return next++;throw Error('Unexpected IPC '+cmd);}};});
    const control=async(step,input={})=>{const r=await page.request.post(origin+'/__test__/control',{data:{step,input}});if(!r.ok())throw Error(await r.text());return r.json();};
    const check=(v,s)=>{if(!v)throw Error(s);checks.push(s);};
    await control('configure',{theme:'light',phase:'main'});const info=await control('init');await page.setViewportSize({width:1440,height:900});await page.goto(origin);
    await page.evaluate(()=>{localStorage.clear();localStorage.setItem('noteka.peek-mode.v1','full');});await page.reload();
    await page.getByLabel('Workspace path').fill(info.path);await page.getByRole('button',{name:'Create workspace',exact:true}).click();await page.locator('.home').waitFor();
    const original=await control('import_attachment',{source_path:info.file});
    const note=await control('entity_create',{kind:'note',title:'Synthetic search fixture',document:{schema_version:2,blocks:[{id:'p',type:'paragraph',content:[{type:'text',text:'Uniquetoken synthetic evidence'}]}]}});
    const expand=async()=>{if(await page.locator('.desktop-sidebar').getAttribute('aria-hidden')==='true')await page.getByRole('button',{name:'Expand sidebar'}).click();};
    const search=async()=>{await expand();await page.locator('.desktop-sidebar .sidebar-search').click();await page.getByRole('dialog',{name:'Search workspace',exact:true}).waitFor();};
    stage='advanced search';await search();await page.getByRole('button',{name:'Advanced search'}).click();const scope=page.getByLabel('Search scope',{exact:true});
    await scope.selectOption('notes');await page.getByLabel('Search text',{exact:true}).fill('uniquetoken');await page.locator('.search-form').getByRole('button',{name:'Search',exact:true}).click();
    await page.locator('.search-hit').filter({hasText:note.title}).first().waitFor();check(true,'Advanced text search returns synthetic note');
    await scope.selectOption('attachments');const attachments=page.locator('.attachment-search');const attachmentSubmit=attachments.getByRole('button',{name:'Search attachments'});
    check(await attachmentSubmit.isDisabled(),'Empty attachment search remains disabled');await attachments.getByLabel('Attachment text').fill('uniquetoken');await attachmentSubmit.click();
    await attachments.getByRole('region',{name:'Attachment coverage'}).waitFor();check(true,'Attachment search returns explicit coverage');
    await scope.selectOption('combined');const combined=page.locator('.grouped-search');const combinedSubmit=combined.getByRole('button',{name:'Search notes and attachments'});
    check(await combinedSubmit.isDisabled(),'Empty combined search remains disabled');await combined.getByLabel('Combined text').fill('uniquetoken');await combinedSubmit.click();
    await combined.locator('.grouped-note-hit').filter({hasText:note.title}).waitFor();await combined.locator('.grouped-coverage').waitFor({state:'attached'});check(true,'Combined search returns note and attachment coverage');
    stage='local tools';await search();await page.getByRole('combobox',{name:'Search workspace'}).fill(note.title);await page.getByRole('option').filter({has:page.locator('strong',{hasText:note.title})}).first().click();
    const pane=()=>page.locator('.workspace-tab-panel.peek:not(.peek-hidden):visible').last();await pane().locator('.title-input').waitFor();
    const menu=async name=>{await pane().getByRole('button',{name:'Page actions',exact:true}).click();await pane().getByRole('menu',{name:'Page actions'}).getByRole('menuitem',{name,exact:true}).click();return pane().locator('.page-tool-panel:not([hidden])');};
    let panel=await menu('Export');const destination=panel.getByLabel('Export destination');await destination.fill(info.export);await panel.getByRole('button',{name:'Export JSON and originals'}).click();
    await panel.getByRole('status').waitFor();const manifest=await control('check_export');check(!!manifest,'Export action writes a synthetic export');await page.keyboard.press('Escape');
    panel=await menu('Attachments');const prep=panel.locator('.preparation-panel');await prep.locator(':scope > summary').click();const query=prep.getByRole('button',{name:'Query jobs'});check(await query.isDisabled(),'Preparation query requires managed ID');
    await prep.getByLabel('Managed ID to prepare').fill(original.id);await query.click();await prep.locator('.preparation-page').waitFor();check((await prep.locator('.preparation-page').innerText()).includes('0 jobs'),'Preparation query reports empty jobs without enqueueing');
    const generation=panel.locator('.generation-panel');await generation.locator(':scope > summary').click();check(await generation.getByRole('button',{name:'Read from the beginning'}).isDisabled(),'Reader remains disabled without a job');
    await page.keyboard.press('Escape');panel=await menu('External resources');check(await panel.getByRole('button',{name:'Link from Google Drive'}).count()===1,'External resource action remains available');await page.keyboard.press('Escape');
    await pane().getByRole('button',{name:'Close peek'}).click();
    stage='settings and reopen';await expand();await page.getByRole('button',{name:'Settings',exact:true}).click();const theme=page.getByLabel('Theme',{exact:true});await theme.selectOption('dark');await page.getByRole('button',{name:'Custom',exact:true}).click();
    const hex=page.locator('#primary-color-hex');await hex.fill('#8254d4');await page.getByRole('button',{name:'Apply',exact:true}).click();
    await page.waitForFunction(()=>document.documentElement.style.getPropertyValue('--primary-color')==='#8254d4');check(true,'Settings applies custom hexadecimal color');
    await page.getByRole('button',{name:'General',exact:true}).click();await page.getByRole('button',{name:'Change workspace',exact:true}).click();await page.getByLabel('Workspace path').waitFor();
    await page.getByLabel('Workspace path').fill(info.path);await page.getByRole('button',{name:'Open workspace',exact:true}).click();await page.locator('.home').waitFor();check(true,'Change workspace and Open reopen the synthetic workspace');
    check(!errors.length&&!external.length,'No page errors or external requests');await control('finish',{ok:true,checks});return{ok:true,count:checks.length};
  } catch(error){throw Error(`${stage}: ${error.stack||error}`);}
}
