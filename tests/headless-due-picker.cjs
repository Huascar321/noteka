async page => {
 const origin=new URL(page.url()).origin,checks=[],errors=[],external=[];let stage='setup';
 try {
  page.on('pageerror',e=>errors.push(String(e)));await page.context().route('**/*',async r=>{if(new URL(r.request().url()).origin===origin)await r.continue();else{external.push(r.request().url());await r.abort();}});
  await page.addInitScript(()=>{let n=1;window.isTauri=true;window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener(){}};window.__TAURI_INTERNALS__={transformCallback(){return n++;},unregisterCallback(){},async invoke(c,a={}){if(c==='operate')return(await fetch('/__test__/operate',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(a.request)})).json();if(['pending_edits','close_confirmed','plugin:event|listen','plugin:event|unlisten'].includes(c))return n++;throw Error('Unexpected IPC '+c);}};});
  const control=async(step,input={})=>{const r=await page.request.post(origin+'/__test__/control',{data:{step,input}});if(!r.ok())throw Error(await r.text());return r.json();};const check=(v,s)=>{if(!v)throw Error(s);checks.push(s);};
  await control('configure',{theme:'light',phase:'main'});const info=await control('init');await page.setViewportSize({width:1440,height:900});await page.goto(origin);await page.getByLabel('Workspace path').fill(info.path);await page.getByRole('button',{name:'Create workspace',exact:true}).click();await page.locator('.home').waitFor();
  const today=(await control('home',{view:'tasks_calendar_week'})).date;await control('home_seed',{date:today});
  if(await page.locator('.desktop-sidebar').getAttribute('aria-hidden')==='true')await page.getByRole('button',{name:'Expand sidebar'}).click();
  await page.getByRole('navigation',{name:'Sections'}).getByRole('button',{name:'Tasks',exact:true}).click();
  const root=page.locator('.workspace-tab-panel.active .home-tasks[data-home-section="task"]').first();const row=root.locator('.home-item').first();await row.waitFor({timeout:20000});const taskId=await row.getAttribute('data-entity-id');await row.locator('.home-open').click();
  const props=page.locator('.task-page-properties');await props.waitFor();await page.waitForTimeout(300);
  const due=props.getByLabel('Due date',{exact:true}),picker=page.locator('.date-picker');
  const active=()=>page.evaluate(()=>document.activeElement?.getAttribute('aria-label')??document.activeElement?.tagName??'');
  const open=async()=>{await due.click();await picker.waitFor({timeout:5000});await page.waitForTimeout(220);};
  const rect=()=>picker.evaluate(el=>el.getBoundingClientRect().toJSON());
  stage='native trigger and material';check(await due.evaluate(el=>el.tagName==='BUTTON'),'Due property uses a button trigger');check(await props.locator('.task-due-property input').count()===0,'Due property has no native date input');check(await due.evaluate(el=>getComputedStyle(el).cursor)==='pointer','Due date shows a pointer cursor');
  await open();const glass=await picker.evaluate(el=>{const cs=getComputedStyle(el);return{blur:cs.backdropFilter,radius:cs.borderRadius,bg:cs.backgroundColor};});
  check(/blur/.test(glass.blur),`Calendar uses the glass blur (${glass.blur})`);check(glass.radius==='14px',`Calendar keeps the shared menu radius (${glass.radius})`);
  const base=await rect();check(base.width<=292&&base.left>=0&&base.right<=1440&&base.top>=0&&base.bottom<=900,`Calendar stays inside the viewport (${JSON.stringify(base)})`);
  check(await picker.locator('.date-picker-day[data-selected="true"]').getAttribute('data-date')===today,'Selected day follows the Due date');
  stage='month navigation';await picker.getByRole('button',{name:'Next month',exact:true}).click();await page.waitForTimeout(200);check((await picker.locator('.date-picker-title').textContent()).trim()==='December 2026','Next month updates the title');
  await picker.getByRole('button',{name:'Previous month',exact:true}).click();await picker.getByRole('button',{name:'Previous month',exact:true}).click();await page.waitForTimeout(200);check((await picker.locator('.date-picker-title').textContent()).trim()==='October 2026','Previous month updates the title');
  await picker.getByRole('button',{name:'Next month',exact:true}).click();await page.waitForTimeout(200);
  stage='pick and save';await picker.getByRole('button',{name:'November 12, 2026',exact:true}).click();await page.waitForTimeout(200);
  check(await picker.count()===0,'Picking a day closes the calendar');check(await due.getAttribute('data-date')==='2026-11-12',`Picking a day writes the trigger (${await due.getAttribute('data-date')})`);check(await active()==='Due date','Focus returns to the Due trigger');
  await page.locator('.workspace-tab-panel.peek:not(.peek-hidden)').getByRole('button',{name:/^Save/}).first().click();await page.waitForFunction(()=>!document.querySelector('.peek:not(.peek-hidden) .save-status.warning'));
  const saved=await control('get',{id:taskId});check(saved?.payload?.due_date==='2026-11-12',`Saved payload keeps the picked date (${saved?.payload?.due_date})`);
  stage='keyboard';await due.focus();await due.press('Alt+ArrowDown');await picker.waitFor();await page.waitForTimeout(200);
  check(await active()==='November 12, 2026',`Alt+ArrowDown focuses the selected day (${await active()})`);
  await page.keyboard.press('ArrowRight');check(await active()==='November 13, 2026',`ArrowRight moves one day (${await active()})`);
  await page.keyboard.press('ArrowDown');check(await active()==='November 20, 2026',`ArrowDown moves one week (${await active()})`);
  await page.keyboard.press('PageUp');check(await active()==='October 20, 2026',`PageUp moves one month (${await active()})`);
  await page.keyboard.press('PageDown');await page.keyboard.press('Enter');await page.waitForTimeout(200);check(await due.getAttribute('data-date')==='2026-11-20',`Enter picks the focused day (${await due.getAttribute('data-date')})`);check(await active()==='Due date','Enter returns focus to the field');check(await due.getAttribute('aria-expanded')==='false','Due trigger reports the closed calendar');
  await due.press('Alt+ArrowDown');await picker.waitFor();await page.waitForTimeout(200);check(await due.getAttribute('aria-expanded')==='true','Due trigger reports the open calendar');await page.keyboard.press('Escape');await page.waitForTimeout(200);check(await picker.count()===0&&await active()==='Due date','Escape from the grid closes and returns focus');
  stage='dismiss and actions';await open();await page.keyboard.press('Escape');await page.waitForTimeout(200);check(await picker.count()===0,'Escape closes the calendar');check(await page.locator('.peek:not(.peek-hidden)').count()>0,'Escape keeps the task peek open');
  await open();await picker.getByRole('button',{name:'Clear',exact:true}).click();await page.waitForTimeout(200);check(await due.getAttribute('data-date')==='','Clear empties the Due value');
  await open();await picker.getByRole('button',{name:'Today',exact:true}).click();await page.waitForTimeout(200);check(await due.getAttribute('data-date')===today,'Today selects the civil date');
  await open();const peekBox=await page.locator('.workspace-tab-panel.peek:not(.peek-hidden)').boundingBox();await page.mouse.click(peekBox.x+24,peekBox.y+120);await page.waitForTimeout(200);check(await picker.count()===0,'Outside click dismisses the calendar');check(await due.getAttribute('data-date')===today,'Outside click keeps the Due value');
  stage='narrow';await page.setViewportSize({width:420,height:740});await page.waitForTimeout(300);await due.scrollIntoViewIfNeeded();await open();const narrow=await rect();check(narrow.left>=0&&narrow.right<=420&&narrow.top>=0&&narrow.bottom<=740,`Narrow calendar stays inside the viewport (${JSON.stringify(narrow)})`);
  await page.keyboard.press('Escape');
  check(!errors.length&&!external.length,'No page errors or external requests');await control('finish',{ok:true,checks});return{ok:true,count:checks.length};
 }catch(e){throw Error(`${stage}: ${e.stack||e}`);}
}
