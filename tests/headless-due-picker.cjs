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
  const open=async()=>{await due.click();await picker.waitFor({timeout:5000});await page.waitForTimeout(540);};
  const material=picker.locator('.date-picker-material');
  const motionId=()=>material.evaluate(el=>el.getAnimations().find(a=>a.playState==='running')?.id||'');
  const rect=()=>picker.evaluate(el=>el.getBoundingClientRect().toJSON());
  stage='native trigger and material';await page.waitForFunction(()=>{const el=document.querySelector('.task-due-trigger');return el&&!el.matches(':disabled');});check(await due.evaluate(el=>el.tagName==='BUTTON'),'Due property uses a button trigger');check(await props.locator('.task-due-property input').count()===0,'Due property has no native date input');check(await due.evaluate(el=>getComputedStyle(el).cursor)==='pointer','Due date shows a pointer cursor');
  await open();const glass=await picker.evaluate(el=>{const cs=getComputedStyle(el.querySelector('.date-picker-material'));return{blur:cs.backdropFilter,radius:cs.borderRadius,bg:cs.backgroundColor};});
  check(/blur/.test(glass.blur),`Calendar uses the glass blur (${glass.blur})`);check(glass.radius==='14px',`Calendar keeps the shared menu radius (${glass.radius})`);
  const base=await rect();check(base.width<=292&&base.left>=0&&base.right<=1440&&base.top>=0&&base.bottom<=900,`Calendar stays inside the viewport (${JSON.stringify(base)})`);
  check(await picker.locator('.date-picker-day[data-selected="true"]').getAttribute('data-date')===today,'Selected day follows the Due date');
  check(await material.evaluate(el=>el.getAnimations().length===0),`Settled glass releases its animation effects (${await material.evaluate(el=>JSON.stringify(el.getAnimations().map(a=>({id:a.id,state:a.playState,time:a.currentTime,target:a.effect.target?.className,pseudo:a.effect.pseudoElement}))))})`);
  check(await picker.evaluate(el=>getComputedStyle(el).backdropFilter)==='none','Only the material layer filters the backdrop');
  stage='close motion';await page.keyboard.press('Escape');await page.waitForTimeout(40);
  check(await picker.count()===1&&await picker.evaluate(el=>el.inert),'Escape makes the closing calendar inert');
  check(await motionId()==='date-picker-dismiss','Close animates the material back to Due');
  await page.waitForTimeout(380);check(await picker.count()===0,'Close animation unmounts the calendar');check(await active()==='Due date','Close animation returns focus to the trigger');
  stage='geometry and interruption';await due.click();await picker.waitFor();
  const frames=await picker.evaluate(el=>{
    const glass=el.querySelector('.date-picker-material'),content=el.querySelector('.date-picker-content');
    const a=glass.getAnimations()[0],text=content.getAnimations()[0];
    a.pause();text.pause();
    a.currentTime=0;text.currentTime=0;
    const source=glass.getBoundingClientRect().toJSON();
    a.currentTime=225;text.currentTime=225;
    const expanded=glass.getBoundingClientRect().toJSON();
    const textTransform=getComputedStyle(content).transform;
    a.currentTime=100;text.currentTime=100;
    const beforeClose=glass.getBoundingClientRect().toJSON();
    return{source,expanded,textTransform,beforeClose,panel:el.getBoundingClientRect().toJSON()};
  });
  const trigger=await due.boundingBox();
  check(Math.abs(frames.source.x-trigger.x)<2&&Math.abs(frames.source.y-trigger.y)<2&&Math.abs(frames.source.width-trigger.width)<2&&Math.abs(frames.source.height-trigger.height)<2,'Glass starts at the actual Due rectangle');
  check(frames.expanded.width>frames.panel.width&&frames.expanded.height>frames.panel.height,'Glass expands past its resting size before settling');
  check(frames.textTransform==='none'||/^matrix\(1, 0, 0, 1,/.test(frames.textTransform),'Calendar text never inherits the capsule compression');
  await page.keyboard.press('Escape');
  const closeStart=await material.evaluate(el=>{const a=el.getAnimations()[0];a.pause();a.currentTime=0;return el.getBoundingClientRect().toJSON();});
  check(Math.abs(closeStart.width-frames.beforeClose.width)<2&&Math.abs(closeStart.y-frames.beforeClose.y)<2,'Early close starts at the interrupted frame without jumping');
  const closeEnd=await material.evaluate(el=>{const a=el.getAnimations()[0];a.currentTime=200;const rect=el.getBoundingClientRect().toJSON();a.finish();return rect;});
  check(Math.abs(closeEnd.width-trigger.width)<2&&Math.abs(closeEnd.y-trigger.y)<2,'Closing glass returns to the Due rectangle');
  await page.waitForTimeout(80);check(await picker.count()===0,'Interrupted opening closes cleanly');
  await open();
  stage='month navigation';await picker.getByRole('button',{name:'Next month',exact:true}).click();await page.waitForTimeout(200);check((await picker.locator('.date-picker-title').textContent()).trim()==='December 2026','Next month updates the title');
  await picker.getByRole('button',{name:'Previous month',exact:true}).click();await picker.getByRole('button',{name:'Previous month',exact:true}).click();await page.waitForTimeout(200);check((await picker.locator('.date-picker-title').textContent()).trim()==='October 2026','Previous month updates the title');
  await picker.getByRole('button',{name:'Next month',exact:true}).click();await page.waitForTimeout(200);
  stage='pick and save';await picker.getByRole('button',{name:'November 12, 2026',exact:true}).click();await page.waitForTimeout(320);
  check(await picker.count()===0,'Picking a day closes the calendar');check(await due.getAttribute('data-date')==='2026-11-12',`Picking a day writes the trigger (${await due.getAttribute('data-date')})`);check(await active()==='Due date','Focus returns to the Due trigger');
  await page.locator('.workspace-tab-panel.peek:not(.peek-hidden)').getByRole('button',{name:/^Save/}).first().click();await page.waitForFunction(()=>!document.querySelector('.peek:not(.peek-hidden) .save-status.warning'));
  const saved=await control('get',{id:taskId});check(saved?.payload?.due_date==='2026-11-12',`Saved payload keeps the picked date (${saved?.payload?.due_date})`);
  stage='keyboard';await due.focus();await due.press('Alt+ArrowDown');await picker.waitFor();await page.waitForTimeout(200);
  check(await active()==='November 12, 2026',`Alt+ArrowDown focuses the selected day (${await active()})`);
  await page.keyboard.press('ArrowRight');check(await active()==='November 13, 2026',`ArrowRight moves one day (${await active()})`);
  await page.keyboard.press('ArrowDown');check(await active()==='November 20, 2026',`ArrowDown moves one week (${await active()})`);
  await page.keyboard.press('PageUp');check(await active()==='October 20, 2026',`PageUp moves one month (${await active()})`);
  await page.keyboard.press('PageDown');await page.keyboard.press('Enter');await page.waitForTimeout(320);check(await due.getAttribute('data-date')==='2026-11-20',`Enter picks the focused day (${await due.getAttribute('data-date')})`);check(await active()==='Due date','Enter returns focus to the field');check(await due.getAttribute('aria-expanded')==='false','Due trigger reports the closed calendar');
  await due.press('Alt+ArrowDown');await picker.waitFor();await page.waitForTimeout(200);check(await due.getAttribute('aria-expanded')==='true','Due trigger reports the open calendar');await page.keyboard.press('Escape');await page.waitForTimeout(320);check(await picker.count()===0&&await active()==='Due date','Escape from the grid closes and returns focus');
  stage='dismiss and actions';await open();await page.keyboard.press('Escape');await page.waitForTimeout(320);check(await picker.count()===0,'Escape closes the calendar');check(await page.locator('.peek:not(.peek-hidden)').count()>0,'Escape keeps the task peek open');
  await open();await due.click();await page.waitForTimeout(380);check(await picker.count()===0,'Clicking Due again dismisses its calendar');
  await open();await picker.getByRole('button',{name:'Clear',exact:true}).click();await page.waitForTimeout(320);check(await due.getAttribute('data-date')==='','Clear empties the Due value');
  await open();await picker.getByRole('button',{name:'Today',exact:true}).click();await page.waitForTimeout(320);check(await due.getAttribute('data-date')===today,'Today selects the civil date');
  await open();const peekBox=await page.locator('.workspace-tab-panel.peek:not(.peek-hidden)').boundingBox();await page.mouse.click(peekBox.x+24,peekBox.y+120);await page.waitForTimeout(320);check(await picker.count()===0,'Outside click dismisses the calendar');check(await due.getAttribute('data-date')===today,'Outside click keeps the Due value');
  stage='narrow';await page.setViewportSize({width:420,height:740});await page.waitForTimeout(300);await due.scrollIntoViewIfNeeded();await open();const narrow=await rect();check(narrow.left>=0&&narrow.right<=420&&narrow.top>=0&&narrow.bottom<=740,`Narrow calendar stays inside the viewport (${JSON.stringify(narrow)})`);
  await page.keyboard.press('Escape');await page.waitForTimeout(320);
  stage='above and resize';await page.setViewportSize({width:1440,height:440});await due.scrollIntoViewIfNeeded();await open();
  const above=await rect(),aboveTrigger=await due.boundingBox();check(above.bottom<=aboveTrigger.y,'Calendar flips above Due when there is no room below');
  await page.setViewportSize({width:1200,height:700});await page.waitForTimeout(80);const resized=await rect();check(resized.right<=1200&&resized.bottom<=700,'Open calendar repositions on resize');
  await page.keyboard.press('Escape');await page.waitForTimeout(380);
  stage='visual frames';await page.setViewportSize({width:1440,height:900});await due.scrollIntoViewIfNeeded();
  const browser=page.context().browser().browserType().name();
  const neutralMaterials={},selectionColors={};
  for(const theme of ['light','dark']){
    await page.evaluate(t=>{document.documentElement.dataset.theme=t;},theme);
    // Freeze in the first rendering frame, before screenshot/locator round trips
    // can let the short transition finish on a loaded machine.
    const frozen=await due.evaluate(async trigger=>{
      trigger.click();await new Promise(requestAnimationFrame);
      const panel=document.querySelector('.date-picker');
      const nodes=[panel.querySelector('.date-picker-material'),panel.querySelector('.date-picker-content'),panel.querySelector('.date-picker-sheen')];
      const animations=nodes.flatMap(node=>node.getAnimations({subtree:true}));
      for(const a of animations){a.pause();a.currentTime=0;}
      return animations.some(a=>a.id==='date-picker-arrive');
    });
    check(frozen,`${theme}: opening frames capture the live morph`);
    const box=await picker.boundingBox(),buttonBox=await due.boundingBox();
    const clip={x:Math.max(0,Math.min(box.x,buttonBox.x)-24),y:Math.max(0,Math.min(box.y,buttonBox.y)-24),width:box.width+48,height:box.y+box.height-Math.min(box.y,buttonBox.y)+48};
    for(const time of [0,50,100,150,200,250,317,400]){
      await picker.evaluate((el,t)=>{for(const node of [el.querySelector('.date-picker-material'),el.querySelector('.date-picker-content'),el.querySelector('.date-picker-sheen')])for(const a of node.getAnimations({subtree:true})){a.pause();a.currentTime=t;}},time);
      if(time===100)check(await picker.locator('.date-picker-content').evaluate(el=>Number(getComputedStyle(el).opacity)>.99),`${theme}: Due content reaches full opacity in 100 ms`);
      await page.screenshot({path:`output/playwright/due-liquid-morph/${browser}/${theme}-${String(time).padStart(3,'0')}.png`,clip,animations:'allow'});
    }
    neutralMaterials[theme]=await material.evaluate(el=>getComputedStyle(el).background);
    selectionColors[theme]=await picker.locator('[data-selected="true"]').evaluate(el=>getComputedStyle(el).backgroundColor);
    await page.keyboard.press('Escape');await page.waitForTimeout(380);
    await page.emulateMedia({contrast:'more'});await open();
    const fallback=await material.evaluate(el=>({blur:getComputedStyle(el).backdropFilter,sheen:getComputedStyle(el.querySelector('.date-picker-sheen')).display}));
    check(fallback.blur==='none'&&fallback.sheen==='none',`${theme}: high contrast removes blur and sheen`);
    await page.keyboard.press('Escape');await page.waitForTimeout(380);await page.emulateMedia({contrast:'no-preference'});
  }
  stage='alternate accent';if(await page.locator('.desktop-sidebar').getAttribute('aria-hidden')==='true')await page.getByRole('button',{name:'Expand sidebar',exact:true}).click();
  await page.getByRole('button',{name:'Settings',exact:true}).click();
  await page.getByRole('button',{name:'Violet',exact:true}).click();await page.getByRole('button',{name:'Close Settings',exact:true}).click();
  for(const theme of ['light','dark']){
    await page.evaluate(t=>{document.documentElement.dataset.theme=t;},theme);await open();
    check(await material.evaluate(el=>getComputedStyle(el).background)===neutralMaterials[theme],`${theme}: Violet keeps the glass material neutral`);
    check(await picker.locator('[data-selected="true"]').evaluate(el=>getComputedStyle(el).backgroundColor)!==selectionColors[theme],`${theme}: selected day follows the alternate accent`);
    await picker.screenshot({path:`output/playwright/due-liquid-morph/${browser}/${theme}-violet-rest.png`});
    await page.keyboard.press('Escape');await page.waitForTimeout(380);
  }
  stage='dynamic reduced motion';await due.click();await picker.waitFor();await page.emulateMedia({reducedMotion:'reduce'});await page.waitForTimeout(80);
  check(await material.evaluate(el=>el.getAnimations().length===0),'Changing Reduce Motion cancels an in-flight expansion');await page.keyboard.press('Escape');await page.waitForTimeout(80);
  stage='reduced motion';await page.emulateMedia({reducedMotion:'reduce'});await page.setViewportSize({width:1440,height:900});await page.waitForTimeout(300);
  await due.click();await picker.waitFor();check(await material.evaluate(el=>el.getAnimations().length===0),'Reduced motion removes the open animation');
  await page.keyboard.press('Escape');await page.waitForTimeout(80);check(await picker.count()===0,'Reduced motion closes immediately');
  await page.emulateMedia({reducedMotion:'no-preference'});
  check(!errors.length&&!external.length,'No page errors or external requests');await control('finish',{ok:true,checks});return{ok:true,count:checks.length};
 }catch(e){throw Error(`${stage}: ${e.stack||e}`);}
}
