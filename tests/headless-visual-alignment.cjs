async page => {
 const origin=new URL(page.url()).origin,checks=[],errors=[],external=[];let stage='setup';
 try {
  page.on('pageerror',e=>errors.push(String(e)));await page.context().route('**/*',async r=>{if(new URL(r.request().url()).origin===origin)await r.continue();else{external.push(r.request().url());await r.abort();}});
  await page.addInitScript(()=>{let n=1;window.isTauri=true;window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener(){}};window.__TAURI_INTERNALS__={transformCallback(){return n++;},unregisterCallback(){},async invoke(c,a={}){if(c==='operate')return(await fetch('/__test__/operate',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(a.request)})).json();if(['pending_edits','close_confirmed','plugin:event|listen','plugin:event|unlisten'].includes(c))return n++;throw Error('Unexpected IPC '+c);}};});
  const control=async(step,input={})=>{const r=await page.request.post(origin+'/__test__/control',{data:{step,input}});if(!r.ok())throw Error(await r.text());return r.json();};
  const check=(v,s)=>{if(!v)throw Error(s);checks.push(s);};
  await control('configure',{theme:'light',phase:'main'});const info=await control('init');await page.setViewportSize({width:1440,height:900});await page.goto(origin);await page.evaluate(()=>localStorage.setItem('noteka.peek-mode.v1','full'));await page.reload();
  await page.getByLabel('Workspace path').fill(info.path);await page.getByRole('button',{name:'Create workspace',exact:true}).click();await page.locator('.home').waitFor();
  const task=await control('entity_create',{kind:'task',title:'Alignment task',payload:{state:'doing',due_date:'2026-09-30'}});
  const project=await control('entity_create',{kind:'project',title:'Alignment project',payload:{state:'doing',end_date:'2026-09-30'}});
  const note=await control('entity_create',{kind:'note',title:'Alignment note'});
  const tag=await control('entity_create',{kind:'tag',title:'Alignment tag with a long synthetic title',payload:{classification:'topic'}});
  const person=await control('entity_create',{kind:'person',title:'Alignment person with a long synthetic name'});
  for(const entity of [task,project,note]){let fresh=await control('get',{id:entity.id});await control('relation_add',{source_id:entity.id,target_id:person.id,kind:'person',expected_revision:fresh.revision});if(entity.id!==note.id){fresh=await control('get',{id:entity.id});await control('relation_add',{source_id:entity.id,target_id:tag.id,kind:'tag',expected_revision:fresh.revision});}}
  const expand=async()=>{if(await page.locator('.desktop-sidebar').getAttribute('aria-hidden')==='true')await page.getByRole('button',{name:'Expand sidebar'}).click();};
  const pane=()=>page.locator('.workspace-tab-panel.peek:not(.peek-hidden):visible').last();
  const open=async title=>{await expand();await page.locator('.desktop-sidebar .sidebar-search').click();await page.getByRole('combobox',{name:'Search workspace'}).fill(title);await page.getByRole('option').filter({has:page.locator('strong',{hasText:title})}).first().click();await pane().locator('.title-input').waitFor();};
  const close=async()=>{await pane().getByRole('button',{name:'Close peek'}).click();};
  for(const theme of ['light','dark']){
   await expand();await page.getByRole('button',{name:'Settings',exact:true}).click();await page.getByLabel('Theme',{exact:true}).selectOption(theme);await page.getByRole('button',{name:'Close Settings',exact:true}).click();
   for(const width of [1440,900,420]){
    await page.setViewportSize({width,height:900});await page.waitForTimeout(300);
    for(const entity of [task,project,note,tag,person]){
     stage=`${theme}/${width}/${entity.title}`;await open(entity.title);const groups=pane().locator('.page-relations .relation-group');await groups.first().waitFor();
     for(let i=0;i<await groups.count();i++){
      const group=groups.nth(i),result=await group.evaluate(el=>{const label=el.querySelector('.relation-group-label'),container=el.querySelector('.relation-chips'),target=container?.querySelector('.relation-chip, .relation-add');if(!label||!target)return null;const range=document.createRange();range.selectNodeContents(label);const lr=range.getClientRects()[0];let node=[...target.childNodes].find(n=>n.nodeType===Node.TEXT_NODE&&n.textContent.trim());if(!node)node=[...target.childNodes].find(n=>n.nodeType===Node.TEXT_NODE);if(!node)return null;const tr=document.createRange();tr.selectNodeContents(node);const r=tr.getClientRects()[0];const a=target.getBoundingClientRect(),c=container.getBoundingClientRect();return {delta:Math.abs((lr.top+lr.bottom-r.top-r.bottom)/2),inside:a.left>=c.left-1&&a.right<=c.right+1,remove:[...container.querySelectorAll('.relation-remove')].every(x=>getComputedStyle(x).marginTop==='0px'&&getComputedStyle(x).display==='grid')};});
      check(result && result.delta<=2.5 && result.inside && result.remove,`${stage} relation row ${i}: first text aligned and controls fit ${JSON.stringify(result)}`);
     }
     const dates=pane().locator('.task-due-property input, .project-page-properties input[type="date"]');for(let i=0;i<await dates.count();i++){const d=dates.nth(i),st=await d.evaluate(el=>({left:parseFloat(getComputedStyle(el).paddingLeft),right:parseFloat(getComputedStyle(el).paddingRight),type:el.type,value:el.value}));check(st.left===8&&st.right===8&&st.type==='date'&&st.value==='2026-09-30',`${stage} date padding and value ${JSON.stringify(st)}`);}
     check(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),`${stage}: no global overflow`);await close();
    }
   }
  }
  stage='attachment preparation alignment';await page.setViewportSize({width:1440,height:900});await page.waitForTimeout(300);await open(note.title);await pane().getByRole('button',{name:'Page actions',exact:true}).click();await pane().getByRole('menu',{name:'Page actions'}).getByRole('menuitem',{name:'Attachments',exact:true}).click();const prep=pane().locator('.preparation-panel');await prep.locator(':scope > summary').click();
  const aligned=await prep.locator('.preparation-actions').evaluate(el=>{const s=el.querySelector('select').getBoundingClientRect();return [...el.querySelectorAll(':scope > button')].map(b=>{const r=b.getBoundingClientRect();return Math.abs((r.top+r.bottom-s.top-s.bottom)/2)});});check(aligned.every(d=>d<=1.5),'Preparation buttons centered on selector '+JSON.stringify(aligned));
  check(!errors.length&&!external.length,'No page errors or external requests');await control('finish',{ok:true,checks});return{ok:true,count:checks.length};
 }catch(e){throw Error(`${stage}: ${e.stack||e}`);}
}
