// Inbox derivado de relaciones Project (Tasks y Notes) y convivencia con
// Notes Unclassified. Sólo transporte headless con CLI/SQLite sintéticos.
async page => {
 const origin=new URL(page.url()).origin,checks=[],errors=[],external=[];let stage='setup';
 try {
  page.on('pageerror',e=>errors.push(String(e)));await page.context().route('**/*',async r=>{if(new URL(r.request().url()).origin===origin)await r.continue();else{external.push(r.request().url());await r.abort();}});
  await page.addInitScript(()=>{let n=1;window.isTauri=true;window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener(){}};window.__TAURI_INTERNALS__={transformCallback(){return n++;},unregisterCallback(){},async invoke(c,a={}){if(c==='operate')return(await fetch('/__test__/operate',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(a.request)})).json();if(['pending_edits','close_confirmed','plugin:event|listen','plugin:event|unlisten'].includes(c))return n++;throw Error('Unexpected IPC '+c);}};});
  const control=async(step,input={})=>{const r=await page.request.post(origin+'/__test__/control',{data:{step,input}});if(!r.ok())throw Error(await r.text());return r.json();};
  const check=(v,s)=>{if(!v)throw Error(s);checks.push(s);};
  await control('configure',{theme:'light',phase:'main'});const info=await control('init');
  await page.goto(origin);await page.getByLabel('Workspace path').fill(info.path);await page.getByRole('button',{name:'Create workspace',exact:true}).click();await page.locator('.home').waitFor();
  const task=await control('entity_create',{kind:'task',title:'Synthetic loose task',payload:{inbox:false}});
  const project=await control('entity_create',{kind:'project',title:'Synthetic classifier project',payload:{state:'doing'}});
  const note=await control('entity_create',{kind:'note',title:'Synthetic loose note',payload:{inbox:false}});
  const tag=await control('entity_create',{kind:'tag',title:'Synthetic classifier tag',payload:{classification:'topic'}});
  let tasks=page.locator('.workspace-tab-panel.active .home-tasks[data-home-section="task"]').first(),notes=page.locator('.workspace-tab-panel.active .home-notes[data-home-section="note"]').first();
  const pane=()=>page.locator('.workspace-tab-panel.peek:not(.peek-hidden):visible').last();
  const refresh=async scope=>{await scope.getByRole('button',{name:scope===tasks?'Refresh Tasks':'Refresh Notes'}).click();};
  stage='Home task';
  await tasks.getByRole('group',{name:'Tasks views'}).getByRole('button',{name:'Inbox',exact:true}).click();
  await tasks.getByRole('button',{name:'Open Synthetic loose task'}).waitFor();
  check(await tasks.getByText(/without a local Project relation/).isVisible(),'Task Inbox explica la derivación desde Project');
  check(await tasks.getByRole('group',{name:'Tasks views'}).getByRole('button',{name:'Unclassified',exact:true}).count()===0,'Tasks ya no ofrece la tab Unclassified');
  await tasks.getByRole('button',{name:'Open Synthetic loose task'}).click();
  check(await pane().getByLabel('In Inbox').count()===0,'El detalle de tarea ya no ofrece el control Inbox manual');
  await control('relation_add',{source_id:task.id,target_id:project.id,kind:'project',expected_revision:task.revision});
  await pane().getByRole('button',{name:'Close peek'}).click();
  await refresh(tasks);
  await tasks.getByRole('button',{name:'Open Synthetic loose task'}).waitFor({state:'hidden'});
  check(true,'Vincular la tarea a un proyecto la saca de Inbox');
  stage='Home note';
  await page.getByRole('navigation',{name:'Sections'}).getByRole('button',{name:'Notes',exact:true}).click();
  notes=page.locator('.workspace-tab-panel.active .home-notes[data-home-section="note"]').first();
  await notes.getByRole('group',{name:'Notes views'}).getByRole('button',{name:'Inbox',exact:true}).click();
  await notes.getByRole('button',{name:'Open Synthetic loose note'}).waitFor();
  check(await notes.getByText(/Tag or People links do not remove/).isVisible(),'Note Inbox explica que Tag/People no la sacan');
  check(await notes.getByRole('group',{name:'Notes views'}).getByRole('button',{name:'Unclassified',exact:true}).count()===1,'Notes conserva la tab Unclassified');
  await control('relation_add',{source_id:note.id,target_id:tag.id,kind:'tag',expected_revision:note.revision});
  await refresh(notes);
  await notes.getByRole('button',{name:'Open Synthetic loose note'}).waitFor();
  check(true,'Un vínculo Tag no saca la nota de Inbox');
  await notes.getByRole('group',{name:'Notes views'}).getByRole('button',{name:'Unclassified',exact:true}).click();
  await notes.getByRole('button',{name:'Open Synthetic loose note'}).waitFor({state:'hidden'});
  check(true,'Unclassified de Notes sigue excluyendo por Tag');
  await notes.getByRole('group',{name:'Notes views'}).getByRole('button',{name:'Inbox',exact:true}).click();
  await notes.getByRole('button',{name:'Open Synthetic loose note'}).waitFor();
  const current=await control('get',{id:note.id});
  await control('relation_add',{source_id:note.id,target_id:project.id,kind:'project',expected_revision:current.revision});
  await refresh(notes);
  await notes.getByRole('button',{name:'Open Synthetic loose note'}).waitFor({state:'hidden'});
  check(true,'Vincular la nota a un proyecto la saca de Inbox');
  stage='Pages';
  await page.getByRole('navigation',{name:'Sections'}).getByRole('button',{name:'Tasks',exact:true}).click();
  tasks=page.locator('.workspace-tab-panel.active .home-tasks[data-home-section="task"]').first();
  await tasks.getByRole('group',{name:'Tasks views'}).getByRole('button',{name:'Inbox',exact:true}).waitFor();
  check(await tasks.getByRole('group',{name:'Tasks views'}).getByRole('button',{name:'Unclassified',exact:true}).count()===0,'La página Tasks tampoco ofrece Unclassified');
  await page.getByRole('navigation',{name:'Sections'}).getByRole('button',{name:'Notes',exact:true}).click();
  notes=page.locator('.workspace-tab-panel.active .home-notes[data-home-section="note"]').first();
  check(await notes.getByRole('group',{name:'Notes views'}).getByRole('button',{name:'Unclassified',exact:true}).count()===1,'La página Notes conserva Unclassified');
  check(!errors.length&&!external.length,'Sin errores de página ni solicitudes externas');
  await control('finish',{ok:true,checks});return{ok:true,count:checks.length};
 }catch(e){throw Error(`${stage}: ${e.stack||e}`);}
}
