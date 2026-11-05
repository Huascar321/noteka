async page => {
  const origin=new URL(page.url()).origin,checks=[],errors=[];page.on('pageerror',e=>errors.push(String(e)));
  await page.addInitScript(()=>{let next=1;window.isTauri=true;window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener(){}};window.__TAURI_INTERNALS__={transformCallback(){return next++;},unregisterCallback(){},async invoke(cmd,args={}){if(cmd==='operate')return (await fetch('/__test__/operate',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(args.request)})).json();if(['pending_edits','close_confirmed','plugin:event|listen','plugin:event|unlisten'].includes(cmd))return next++;throw Error('Unexpected IPC '+cmd);}};});
  const control=async(step,input={})=>{const r=await page.request.post(origin+'/__test__/control',{data:{step,input}});if(!r.ok())throw Error(await r.text());return r.json();};
  const check=(v,s)=>{if(!v)throw Error(s);checks.push(s);};
  const style=locator=>locator.evaluate(el=>({outline:getComputedStyle(el).outlineStyle,width:parseFloat(getComputedStyle(el).outlineWidth),modality:document.documentElement.dataset.focusModality}));
  await control('configure',{theme:'light',phase:'main'});const info=await control('init');await page.setViewportSize({width:1440,height:900});await page.goto(origin);
  const path=page.getByLabel('Workspace path');await path.click();let state=await style(path);check(state.modality==='pointer'&&state.outline==='none','Pointer focus on workspace field has no ring');
  await page.keyboard.press('a');state=await style(path);check(state.modality==='keyboard'&&state.outline!=='none'&&state.width>=2,'Typing restores visible keyboard focus');
  await path.fill(info.path);await page.getByRole('button',{name:'Create workspace',exact:true}).click();await page.locator('.home').waitFor();
  for(const theme of ['light','dark']){
    await page.getByRole('button',{name:'Settings',exact:true}).click();await page.getByLabel('Theme',{exact:true}).selectOption(theme);await page.getByRole('button',{name:'Close Settings'}).click();
    const tab=page.locator('.window-titlebar [role="tab"][aria-selected="true"]');await tab.click();await tab.focus();state=await style(tab);check(state.modality==='pointer'&&state.outline==='none',`${theme}: clicking tab has no ring`);
    await page.keyboard.press('ArrowRight');state=await style(tab);check(state.modality==='keyboard'&&state.outline!=='none'&&state.width>=2,`${theme}: keyboard tab focus is visible`);
    await page.locator('.desktop-sidebar .sidebar-search').click();const search=page.getByRole('dialog',{name:'Search workspace',exact:true});await search.waitFor();const wrapper=search.locator('.context-search-input');
    state=await style(wrapper);check(state.modality==='pointer'&&state.outline==='none',`${theme}: pointer-opened search has no ring`);
    await page.keyboard.type('a');state=await style(wrapper);check(state.modality==='keyboard'&&state.outline!=='none'&&state.width>=2,`${theme}: typing shows search focus`);
    await page.keyboard.press('Escape');
    await page.getByRole('button',{name:'Settings',exact:true}).click();const blue=page.getByRole('button',{name:'Blue',exact:true});await blue.click();
    check(await blue.getAttribute('aria-pressed')==='true' && await blue.evaluate(el=>getComputedStyle(el).borderColor!==getComputedStyle(el.parentElement).borderColor && getComputedStyle(el).outlineStyle==='none'),`${theme}: selected color indicator persists without pointer ring`);
    await page.keyboard.press('Tab');const focused=page.locator(':focus').first();state=await style(focused);check(state.modality==='keyboard'&&state.outline!=='none'&&state.width>=2,`${theme}: settings keyboard focus is visible`);
    await page.keyboard.press('Escape');
  }
  await page.emulateMedia({reducedMotion:'reduce'});
  const tabIndicator=page.locator('.home [data-home-section="task"] .sliding-tabs-indicator').first();
  check(await tabIndicator.evaluate(el=>getComputedStyle(el).transitionDuration.split(',').every(v=>parseFloat(v)===0)),'Reduced motion disables indicator transition');
  const newAction=page.locator('.home [data-home-section="task"] .task-new').first();await newAction.scrollIntoViewIfNeeded();await newAction.hover();
  const shadowBefore=await newAction.evaluate(el=>getComputedStyle(el).boxShadow);await page.mouse.down();const shadowPressed=await newAction.evaluate(el=>getComputedStyle(el).boxShadow);await page.mouse.move(0,0);await page.mouse.up();
  check(shadowPressed!==shadowBefore,'Press gives New a distinct material state');
  const note=await control('entity_create',{kind:'note',title:'Focus synthetic note'});
  await page.locator('.desktop-sidebar .sidebar-search').click();await page.getByRole('combobox',{name:'Search workspace'}).fill(note.title);
  await page.getByRole('option').filter({has:page.locator('strong',{hasText:note.title})}).first().click();
  const pane=page.locator('.workspace-tab-panel.peek:not(.peek-hidden):visible').last(),title=pane.locator('.title-input');await title.waitFor();await title.click();state=await style(title);
  check(state.modality==='pointer'&&state.outline==='none','Pointer focus on note title is quiet');
  await page.keyboard.press('ArrowRight');state=await style(title);check(state.modality==='keyboard'&&state.outline!=='none'&&state.width>=2,'Keyboard focus on note title is visible');
  await pane.locator('.icon-picker .icon-trigger').click();const swatch=pane.locator('.icon-picker .color-grid button').nth(1);await swatch.click();
  check(await swatch.getAttribute('aria-pressed')==='true' && await swatch.evaluate(el=>getComputedStyle(el).outlineStyle!=='none'),'Selected icon color retains its selection outline after pointer input');
  await pane.locator('.icon-picker').getByRole('button',{name:'Done',exact:true}).click();
  await pane.getByRole('button',{name:'Close peek'}).click();const guard=page.getByRole('dialog',{name:'You have unsaved changes'});await guard.getByRole('button',{name:'Discard',exact:true}).click();
  await page.emulateMedia({contrast:'more',reducedMotion:'reduce'});await page.getByRole('button',{name:'Settings',exact:true}).click();await page.keyboard.press('Tab');
  let focused=page.locator(':focus').first();state=await style(focused);check(state.modality==='keyboard'&&state.outline!=='none'&&state.width>=2,'High contrast retains visible keyboard focus');
  await page.keyboard.press('Escape');check(!errors.length,'No page errors: '+errors.join('; '));await control('finish',{ok:true,checks});return{ok:true,count:checks.length};
}
