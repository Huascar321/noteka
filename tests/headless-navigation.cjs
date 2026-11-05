async (page) => {
 page.setDefaultTimeout(12000);
 const origin=new URL(page.url()).origin, external=[],errors=[];
 page.on('pageerror',error=>errors.push(String(error)));
 await page.context().route('**/*',async route=>{const u=new URL(route.request().url());if(u.origin===origin)await route.continue();else{external.push(u.origin);await route.abort();}});
 await page.addInitScript(()=>{
  let pending=false,next=1;const callbacks=new Map();window.isTauri=true;
  window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:()=>{}};
  window.__navigationPending=()=>pending;window.__navigationCloseCount=0;
  window.__TAURI_INTERNALS__={transformCallback:callback=>{const id=next++;callbacks.set(id,callback);return id;},unregisterCallback:id=>callbacks.delete(id),invoke:async(command,args={})=>{
   if(command==='operate'){const response=await fetch('/__test__/operate',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(args.request)});return response.json();}
   if(command==='pending_edits'){pending=args.pending;return;}
   if(command==='close_confirmed'){window.__navigationCloseCount++;return;}
   if(command==='plugin:event|listen'){window.__navigationClose=callbacks.get(args.handler);return next++;}
   if(command==='plugin:event|unlisten')return;
   throw Error('Unexpected IPC: '+command);
  }};
 });
 const control=async(step,input={})=>{const response=await page.request.post(origin+'/__test__/control',{data:{step,input}});if(!response.ok())throw Error(step+': '+await response.text());return response.json();};
 const active=()=>page.locator('.workspace-tab-panel.active');
 const nav=label=>page.locator('.desktop-sidebar nav').getByRole('button',{name:label,exact:true});
 const waitPane=async()=>{await page.waitForFunction(()=>{const el=document.querySelector('.workspace-tab-panel.active');return el&&!el.querySelector(".entity-list [aria-label=\"Refresh\"]:disabled,.title-input:disabled");});};
 const search=async(query,plus=false)=>{await page.getByRole('button',{name:plus?"New tab":'Search',exact:true}).click();const field=page.getByRole('combobox',{name:"Search workspace"});await field.fill(query);await page.waitForFunction(()=>document.querySelector('#context-search-results')?.getAttribute('aria-busy')==='false');return page.locator('.context-search');};
 const choose=async(query,title,plus=false)=>{const dialog=await search(query,plus);await dialog.getByRole('option').filter({has:page.locator('strong',{hasText:title})}).first().click();await page.locator('.context-search').waitFor({state:'detached'});await waitPane();};
 let diagnosticChecks=[];
 try {
 for(const theme of ['light','dark']){
  const checks=[];diagnosticChecks=checks;const check=(condition,name)=>{if(!condition)throw Error(name);checks.push(name);};
  await control('configure',{theme,phase:'main'});const info=await control('init');
  await page.goto(origin);await page.evaluate(theme=>{localStorage.clear();localStorage.setItem('noteka.theme.v1',theme);},theme);await page.reload();
  await page.locator('#workspace-path').fill(info.path);await page.getByRole('button',{name:"Create workspace",exact:true}).click();
  await active().locator('.home').waitFor();await page.waitForFunction(()=>document.querySelectorAll('.home-coverage').length===5);
  await page.reload();await active().locator('.home').waitFor();check(await page.locator('#workspace-path').count()===0,'reload reabre el workspace guardado sin pedir ruta');
  check(JSON.stringify(await page.locator("nav[aria-label=\"Sections\"] button").allTextContents())===JSON.stringify(['Home','Tasks','Notes','Projects','Tags','People']),'exact six sidebar entries in order');
  check(await page.locator('.collections-shortcut').count()===1,'Collections icon outside navigation list');
  const a=await control('entity_create',{kind:'note',title:'Alpha navigation',document:{schema_version:2,blocks:[{id:'a-block',type:'paragraph',content:[{type:'text',text:'Alpha original'}]}]}});
  const b=await control('entity_create',{kind:'note',title:'Beta navigation',document:{schema_version:2,blocks:[{id:'b-block',type:'paragraph',content:[{type:'text',text:'Beta original'}]}]}});
  await control('entity_create',{kind:'task',title:'Task navigation',payload:{state:'todo'}});
  await control('entity_create',{kind:'project',title:'Project navigation',payload:{state:'doing'}});
  await control('entity_create',{kind:'tag',title:'Tag navigation',payload:{classification:'topic'}});
  await control('entity_create',{kind:'person',title:'Person navigation'});
  const canvas=active().locator('canvas.ascii-shark');await page.waitForFunction(()=>Number(document.querySelector('canvas.ascii-shark')?.dataset.frame)>3);
  const f1=await canvas.getAttribute('data-frame');await page.waitForFunction(before=>document.querySelector('canvas.ascii-shark')?.dataset.frame!==before,f1);check(true,'ASCII runs with changing frames by default');
  await active().locator('.home').evaluate(el=>{el.scrollTop=600;});await page.waitForTimeout(100);const offscreenFrame=await canvas.getAttribute('data-frame');await page.waitForTimeout(220);check(await canvas.getAttribute('data-frame')===offscreenFrame,'offscreen cover suspends its animation');await active().locator('.home').evaluate(el=>{el.scrollTop=0;});await page.waitForFunction(before=>document.querySelector('canvas.ascii-shark')?.dataset.frame!==before,offscreenFrame);
  await active().locator('.home-cover').hover();await active().getByRole('button',{name:"Photo",exact:true}).click();
  await page.waitForFunction(()=>document.querySelector('.home-cover-photo')?.complete&&document.querySelector('.home-cover-photo')?.naturalWidth===2048);
  check(await page.locator('canvas.ascii-shark').count()===0,'photo mode uses bundled original image and unmounts animation');
  await page.screenshot({path:info.root+`/navigation-${theme}-photo.png`});
  await active().getByRole('button',{name:'ASCII',exact:true}).click();await canvas.waitFor();
  await page.emulateMedia({reducedMotion:'reduce'});await page.waitForFunction(()=>document.querySelector('canvas.ascii-shark')?.dataset.frame==='0');
  const still=await canvas.getAttribute('data-frame');await page.waitForTimeout(220);check(await canvas.getAttribute('data-frame')===still,'reduced motion draws one still frame');
  await page.emulateMedia({reducedMotion:'no-preference'});await page.waitForFunction(()=>Number(document.querySelector('canvas.ascii-shark')?.dataset.frame)>0);
  await page.screenshot({path:info.root+`/navigation-${theme}-ascii.png`});
  await page.getByRole('button',{name:"New tab",exact:true}).click();await page.getByRole('combobox',{name:"Search workspace"}).waitFor();await page.keyboard.press('Escape');
  check(await page.getByRole('tab').count()===1,'cancel + creates no empty tab');
  check(await page.getByRole('button',{name:"New tab",exact:true}).evaluate(el=>el===document.activeElement),'Escape restores trigger focus');
  await choose('Alpha','Alpha navigation');await active().locator('.title-input').waitFor();
  check(await active().locator('.entity-list').count()===0,'entity detail contains no record list');
  await active().locator('.title-input').fill('Alpha pending');await active().locator('.noteka-document').fill('Alpha local buffer');
  await page.waitForFunction(()=>document.querySelector('.tab-dirty')&&window.__navigationPending());
  const firstId=await page.getByRole('tab',{selected:true}).getAttribute('id');
  const slowDialog=await search('Beta',true);await page.waitForFunction(()=>document.querySelector('.context-preview h2')?.textContent==='Beta navigation');
  await page.evaluate(()=>{const original=window.__TAURI_INTERNALS__.invoke;window.__navigationHeld=false;window.__TAURI_INTERNALS__.invoke=async(command,args={})=>{if(!window.__navigationHeld&&command==='operate'&&args.request.command.operation==='entity.get'){window.__navigationHeld=true;await new Promise(resolve=>window.__navigationRelease=resolve);}return original(command,args);};window.__navigationRestore=()=>{window.__TAURI_INTERNALS__.invoke=original;window.__navigationRelease?.();};});
  await slowDialog.getByRole('option').first().click();await page.waitForFunction(()=>window.__navigationHeld);check(await active().locator('.entity-list').count()===0,'slow detail opening does not flash its list');await page.locator('#'+firstId).click();
  check(await active().locator('.title-input').inputValue()==='Alpha pending','switching tabs while another original loads preserves current buffer');
  await page.evaluate(()=>window.__navigationRestore());await page.getByRole('tab').nth(1).click();await active().locator('.title-input').waitFor();await waitPane();
  check(await active().locator('.title-input').inputValue()==='Beta navigation','late original response stays in its own tab');
  check(await page.getByRole('tab').count()===2,'+ uses same dialog and opens result in a new tab');
  await active().locator('.title-input').fill('Beta pending');await active().locator('.noteka-document').fill('Beta local buffer');
  const secondId=await page.getByRole('tab',{selected:true}).getAttribute('id');
  await page.locator('#'+firstId).click();await active().locator('.noteka-document').waitFor();
  check(await active().locator('.title-input').inputValue()==='Alpha pending'&&(await active().locator('.noteka-document').innerText()).includes('Alpha local buffer'),'first tab preserves independent title and editor');
  await active().locator('.noteka-document').focus();await page.keyboard.press('Meta+z');await page.waitForFunction(()=>document.querySelector('.workspace-tab-panel.active .noteka-document')?.textContent.includes('Alpha original'));await page.keyboard.press('Meta+Shift+z');await page.waitForFunction(()=>document.querySelector('.workspace-tab-panel.active .noteka-document')?.textContent.includes('Alpha local buffer'));check(true,'editor undo and redo survive tab switches');
  await page.locator('#'+secondId).click();check(await active().locator('.title-input').inputValue()==='Beta pending','second tab keeps independent buffer');
  await page.keyboard.press('Control+Tab');check(await page.getByRole('tab',{selected:true}).getAttribute('id')===firstId,'Ctrl Tab cycles mounted panels');
  await page.keyboard.press('Meta+k');await page.getByRole('combobox',{name:"Search workspace"}).fill('notes');await page.waitForFunction(()=>document.querySelector('#context-search-results')?.getAttribute('aria-busy')==='false');await page.keyboard.press('Enter');
  await page.getByRole('dialog',{name:"You have unsaved changes"}).waitFor();await page.getByRole('button',{name:"Cancel",exact:true}).click();check(await active().locator('.title-input').inputValue()==='Alpha pending','current-tab search respects dirty buffer guard');
  await control('failure_on');await active().getByRole('button',{name:/^Guardar/}).click();await active().getByText("Error · Unsaved",{exact:true}).waitFor();
  await page.getByRole('button',{name:/Cerrar pestaña/}).first().click();await page.getByRole('button',{name:"Save and continue",exact:true}).click();
  await page.getByRole('dialog',{name:"You have unsaved changes"}).getByRole('alert').waitFor();check(await active().locator('.title-input').inputValue()==='Alpha pending','failed save keeps buffer and close guard');
  await page.getByRole('button',{name:"Cancel",exact:true}).click();await control('failure_off');
  await active().getByRole('button',{name:"Retry",exact:false}).first().click();await active().getByText("Saved",{exact:true}).waitFor();
  check((await control('get',{id:a.id})).title==='Alpha pending','explicit retry confirms real SQLite save');
  await page.locator('#'+secondId).click();const currentB=await control('get',{id:b.id});await control('cli_update',{id:b.id,expected_revision:currentB.revision,title:'Beta external'});
  await active().getByRole('button',{name:/^Guardar/}).click();await active().getByText("Conflict · Unsaved",{exact:true}).waitFor();
  check(await active().locator('.title-input').inputValue()==='Beta pending','revision conflict preserves local second buffer');
  await page.locator('#'+firstId).click();await page.evaluate(()=>window.__navigationClose({event:'noteka-close-requested',payload:null}));
  await page.getByRole('dialog',{name:"You have unsaved changes"}).waitFor();check(await page.getByRole('tab',{selected:true}).getAttribute('id')===secondId,'host close activates and checks dirty inactive tab');
  await page.getByRole('button',{name:"Cancel",exact:true}).click();check(await page.evaluate(()=>window.__navigationCloseCount)===0,'cancel prevents host close');
  await page.locator('#'+firstId).click();await page.getByRole('button', { name: 'Settings', exact: true }).click(); await page.locator('.settings-navigation').getByRole('button', { name: 'General', exact: true }).click(); await page.getByRole('button',{name:"Change workspace",exact:true}).click();await page.getByRole('dialog',{name:"You have unsaved changes"}).waitFor();check(await page.getByRole('tab',{selected:true}).getAttribute('id')===secondId,'workspace change checks inactive pending tab');await page.getByRole('button',{name:"Cancel",exact:true}).click();
  await page.getByRole('button',{name:/Cerrar pestaña/}).nth(1).click();await page.getByRole('button',{name:"Discard",exact:true}).click();await page.waitForFunction(()=>document.querySelectorAll('[role="tab"]').length===1);
  check((await control('get',{id:b.id})).title==='Beta external','discard closes only tab and keeps external revision');
  await page.getByRole('button',{name:'Collections',exact:true}).click();await active().locator('.database-panel').waitFor();check(true,'Collections shortcut opens working panel');
  await nav('Tasks').click();await page.screenshot({path:info.root+`/navigation-${theme}-section.png`});check(await active().locator('.workspace-pane').evaluate(el=>el.classList.contains('overview-pane')),'section list occupies main content area');
  await choose('archive','Archive');check(await active().locator('.entity-list').getAttribute('aria-label')==='Archive','hidden Archive route accessible by command search');
  await choose('my day','My Day');check(await active().locator('.entity-list').getAttribute('aria-label')==='My Day','hidden My Day route remains accessible');
  await page.getByRole('button',{name:'Search',exact:true}).click();await page.locator('.context-search-footer button').click();await active().getByLabel("Search scope",{exact:true}).waitFor();check(true,'Advanced search retains attachments and combined search');
  await nav('Home').click();await active().locator('.home').waitFor();await page.waitForFunction(()=>document.querySelectorAll('.home-coverage').length===5);
  await active().locator('.home').evaluate(el=>{el.scrollTop=500;});const scroll=await active().locator('.home').evaluate(el=>el.scrollTop);
  await choose('Beta','Beta external',true);await page.locator('#'+firstId).click();
  check(await active().locator('.home').evaluate(el=>el.scrollTop)===scroll,'tab switch preserves Home scroll');
  await active().locator('.home').evaluate(el=>{el.scrollTop=0;});await page.waitForTimeout(100);await page.getByRole('tab').nth(1).click();await page.waitForTimeout(100);const inactiveCanvas=page.locator(`[aria-labelledby="${firstId}"] canvas.ascii-shark`);const inactiveFrame=await inactiveCanvas.getAttribute('data-frame');await page.waitForTimeout(220);check(await inactiveCanvas.getAttribute('data-frame')===inactiveFrame,'inactive tab suspends its canvas');await page.locator('#'+firstId).click();
  await page.setViewportSize({width:420,height:800});await page.getByRole('button',{name:"Expand sidebar",exact:true}).click();await nav('Notes').click();await waitPane();
  check(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),'narrow viewport has no document overflow');
  await page.keyboard.press('Meta+k');await page.getByRole('combobox',{name:"Search workspace"}).fill('Alpha');await page.waitForFunction(()=>document.querySelector('#context-search-results')?.getAttribute('aria-busy')==='false');await page.keyboard.press('ArrowDown');
  check(await page.getByRole('combobox',{name:"Search workspace"}).getAttribute('aria-activedescendant')!==null,'keyboard selects real result');await page.keyboard.press('Escape');
  await page.setViewportSize({width:1440,height:920});await page.getByRole('button',{name:'Search',exact:true}).click();await page.getByRole('combobox',{name:"Search workspace"}).fill('Alpha');await page.waitForFunction(()=>document.querySelector('.context-preview h2')?.textContent==='Alpha pending');
  await page.screenshot({path:info.root+`/navigation-${theme}-search.png`});await page.keyboard.press('Escape');
  const importedPrefix=[{id:'prop-heading',type:'heading',attrs:{level:3},content:[{type:'text',text:'Propiedades de Notion'}]},{id:'prop-list',type:'bullet_list',content:[],children:[['Favorite','Yes'],['Type','Reference'],['Updated','January 18, 2026'],['Archived','No']].map(([key,value],i)=>({id:'prop-item-'+i,type:'list_item',content:[],children:[{id:'prop-value-'+i,type:'paragraph',content:[{type:'text',text:key+': '+value}]}]}))},{id:'prop-copy',type:'paragraph',content:[{type:'text',text:'Favorite: Yes\nType: Reference\nUpdated: January 18, 2026\nArchived: No'}]}];
  const reference=await control('entity_create',{kind:'note',title:'Reference document',payload:{icon:'key',icon_color:'#0b6e99',inbox:false},document:{schema_version:2,blocks:[...importedPrefix,{id:'ref-body',type:'paragraph',content:[{type:'text',text:'Example document content.'}]},{id:'ref-table',type:'code_block',attrs:{language:'markdown'},content:[{type:'text',text:'| Field | Value |\n| --- | --- |\n| Example | Safe local text |'}]}]}});
  await choose('Reference','Reference document',true);await active().locator('.document-table table').waitFor();
  check(await active().locator('.entity-list').count()===0,'reference detail mounts no record list');
  check(!(await active().locator('.noteka-document').innerText()).includes('Propiedades de Notion'),'imported metadata leaves document body and appears as properties');
  check(await active().locator('.page-properties').getByLabel('Favorite',{exact:true}).isChecked(),'source Favorite presented from actual imported value');
  check(await active().locator('.format-toolbar').isHidden(),'formatting controls are collapsed by default');
  check(await active().locator('.document-table td').nth(3).innerText()==='Safe local text','Markdown source has a real escaped table view');
  check(JSON.stringify((await control('get',{id:reference.id})).document)===JSON.stringify(reference.document),'opening detail performs no source mutation');
  await page.screenshot({path:info.root+`/navigation-${theme}-detail.png`});
  await active().locator('.noteka-document > p').first().fill('Edited reference body.');await active().getByRole('button',{name:/^Guardar/}).click();await active().getByText("Saved",{exact:true}).waitFor();
  const updatedReference=await control('get',{id:reference.id});check(JSON.stringify(updatedReference.document.blocks.slice(0,3))===JSON.stringify(reference.document.blocks.slice(0,3)),'body save preserves every imported metadata block and ID');
  check(updatedReference.document.blocks.at(-1).content[0].text===reference.document.blocks.at(-1).content[0].text,'table presentation preserves original Markdown on save');
  await active().locator('.document-table-source > summary').click();await active().getByLabel("Table Markdown",{exact:true}).fill('| Field | Value |\n| --- | --- |\n| Example | Edited table text |');await page.waitForFunction(()=>document.querySelector('.workspace-tab-panel.active .document-table tr:last-child td:last-child')?.textContent==='Edited table text');await active().getByRole('button',{name:/^Guardar/}).click();await active().getByText("Saved",{exact:true}).waitFor();const tableEdit=await control('get',{id:reference.id});check(tableEdit.document.blocks.at(-1).type==='code_block'&&tableEdit.document.blocks.at(-1).id==='ref-table'&&tableEdit.document.blocks.at(-1).content[0].text.includes('Edited table text'),'table source can be edited without changing its persisted block type or ID');
  const beforeTools=await control('get',{id:reference.id});await active().locator('.document-utilities > summary').click();await active().locator('.local-tools > summary').click();await active().locator('.preparation-panel > summary').click();await active().locator('.generation-panel > summary').click();check(await active().getByLabel("Managed ID to prepare",{exact:true}).isVisible()&&await active().getByLabel("Job ID to read",{exact:true}).isVisible(),'local preparation and reader remain reachable through explicit disclosure controls');check(JSON.stringify((await control('get',{id:reference.id})).document)===JSON.stringify(beforeTools.document),'opening local tools leaves original document unchanged');
  for(const [section,title] of [['Tasks','Task navigation'],['Projects','Project navigation'],['Tags','Tag navigation'],['People','Person navigation']]){await nav(section).click();await waitPane();await active().locator('.entity-list button').filter({hasText:title}).click();await active().locator('.title-input').waitFor();check(await active().locator('.entity-list').count()===0,section+' detail contains no record list');} 
  check(external.length===0,'no external requests, assets remain offline');check(errors.length===0,'no browser runtime errors');
  await waitPane();await page.waitForFunction(()=>!window.__navigationPending());
  await control('finish',{ok:true,checks,theme,transport:'headless-cli-not-native-ipc'});
 }
 } catch(error) { const status=await control('status');await page.screenshot({path:status.root+'/navigation-failure.png'});const view=await page.evaluate(()=>({active:document.querySelector('.workspace-tab-panel.active')?.innerText.slice(0,2400),classes:document.querySelector('.workspace-tab-panel.active .workspace-pane')?.className,dialog:document.querySelector('dialog[open]')?.innerText,focus:document.activeElement?.outerHTML.slice(0,300)}));throw Error(JSON.stringify({error:String(error),lastChecks:diagnosticChecks.slice(-4),view,errors,root:status.root})); }
 return {ok:true,external,errors};
}
