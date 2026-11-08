async page => {
 const origin=new URL(page.url()).origin,checks=[],errors=[],external=[];let stage='setup';
 try {
  page.on('pageerror',e=>errors.push(String(e)));await page.context().route('**/*',async r=>{if(new URL(r.request().url()).origin===origin)await r.continue();else{external.push(r.request().url());await r.abort();}});
  await page.addInitScript(()=>{let n=1;window.isTauri=true;window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener(){}};window.__TAURI_INTERNALS__={transformCallback(){return n++;},unregisterCallback(){},async invoke(c,a={}){if(c==='operate')return(await fetch('/__test__/operate',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(a.request)})).json();if(['pending_edits','close_confirmed','plugin:event|listen','plugin:event|unlisten'].includes(c))return n++;throw Error('Unexpected IPC '+c);}};});
  const control=async(step,input={})=>{const r=await page.request.post(origin+'/__test__/control',{data:{step,input}});if(!r.ok())throw Error(await r.text());return r.json();};const check=(v,s)=>{if(!v)throw Error(s);checks.push(s);};
  await control('configure',{theme:'light',phase:'main'});const info=await control('init');await page.setViewportSize({width:1440,height:900});await page.goto(origin);await page.getByLabel('Workspace path').fill(info.path);await page.getByRole('button',{name:'Create workspace',exact:true}).click();await page.locator('.home').waitFor();
  const today=(await control('home',{view:'tasks_calendar_week'})).date;await control('task_paging_seed',{date:today});
  const tasksRoot=()=>page.locator('.workspace-tab-panel.active .home-tasks[data-home-section="task"]').first();
  const rows=root=>root.locator('.home-item');
  const settled=async(root,count)=>{await page.waitForFunction(([el,n])=>el.querySelectorAll('.home-item').length===n,[await root.elementHandle(),count],{timeout:15000});};
  const view=async(name)=>{const root=tasksRoot();await root.getByRole('group',{name:'Tasks views'}).getByRole('button',{name,exact:true}).click();return root;};
  if(await page.locator('.desktop-sidebar').getAttribute('aria-hidden')==='true')await page.getByRole('button',{name:'Expand sidebar'}).click();
  await page.getByRole('navigation',{name:'Sections'}).getByRole('button',{name:'Tasks',exact:true}).click();

  stage='Inbox';let root=await view('Inbox');await settled(root,10);
  check(await root.getByText('Pending tasks without a local Project relation. Linking one to a project removes it from this view.').count()===0,'Inbox omits the explanatory note');

  stage='Scheduled';root=await view('Scheduled');await settled(root,14);
  check(await root.getByRole('button',{name:'Load more',exact:true}).isVisible(),'Scheduled loads fourteen tasks first');
  await root.getByRole('button',{name:'Load more',exact:true}).click();await settled(root,16);
  check(await root.getByRole('button',{name:'Load more',exact:true}).count()===0,'Scheduled accumulates the remaining page');

  stage='No Due';root=await view('No Due');await settled(root,14);
  check(await root.getByRole('button',{name:'Load more',exact:true}).isVisible(),'No Due loads fourteen tasks first');
  await root.getByRole('button',{name:'Load more',exact:true}).click();await settled(root,16);
  check(await root.getByRole('button',{name:'Load more',exact:true}).count()===0,'No Due accumulates the remaining page');

  stage='All';root=await view('All');await settled(root,14);
  check(await root.getByRole('button',{name:'Load more',exact:true}).isVisible(),'All loads fourteen tasks first');
  await root.getByRole('button',{name:'Load more',exact:true}).click();await settled(root,28);
  check(await root.getByRole('button',{name:'Load more',exact:true}).isVisible(),'All keeps Load more while another page exists');

  stage='Home Scheduled';await page.getByRole('navigation',{name:'Sections'}).getByRole('button',{name:'Home',exact:true}).click();root=await view('Scheduled');await settled(root,14);
  check(await root.getByRole('button',{name:'Load more',exact:true}).isVisible(),'Home Scheduled shares the fourteen-task page');

  check(!errors.length&&!external.length,'No page errors or external requests');await control('finish',{ok:true,checks});return{ok:true,count:checks.length};
 }catch(e){throw Error(`${stage}: ${e.stack||e}`);}
}
