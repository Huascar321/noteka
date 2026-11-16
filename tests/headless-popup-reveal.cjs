async page => {
 const origin=new URL(page.url()).origin,stage=new URL(page.url()).hash.slice(1),browser=page.context().browser().browserType().name(),checks=[],samples=[],errors=[];
 const check=(value,message)=>{if(!value)throw Error(message);checks.push(message);};
 page.on('pageerror',error=>errors.push(error.message));
 await page.context().route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
 await page.context().tracing.start({screenshots:true,snapshots:true,sources:true});
 await page.addInitScript(()=>{let n=1;window.isTauri=true;window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener(){}};window.__TAURI_INTERNALS__={transformCallback(){return n++;},unregisterCallback(){},async invoke(command,args={}){if(command==='operate')return(await fetch('/__test__/operate',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(args.request)})).json();if(['pending_edits','close_confirmed','plugin:event|listen','plugin:event|unlisten'].includes(command))return n++;throw Error(command);}};localStorage.setItem('noteka.peek-mode.v1','side');});
 const control=async(step,input={})=>{const response=await page.request.post(origin+'/__test__/control',{data:{step,input}});if(!response.ok())throw Error(await response.text());return response.json();};
 await control('configure',{theme:'light',phase:'main'});const info=await control('init');
 await page.setViewportSize({width:1180,height:800});await page.goto(origin);await page.getByLabel('Workspace path').fill(info.path);await page.getByRole('button',{name:'Create workspace',exact:true}).click();await page.locator('.home').waitFor();
 const project=await control('entity_create',{kind:'project',title:'Example project',payload:{state:'doing'}});
 await control('entity_create',{kind:'project',title:'Another project',payload:{state:'planned'}});
 const task=await control('entity_create',{kind:'task',title:'Popup reveal review',payload:{state:'todo'},context:{target_id:project.id,kind:'project'}});
 const freeze=async(trigger,selector)=>trigger.evaluate(async(button,selector)=>{
  button.click();let host,glass;const deadline=performance.now()+3000;
  while(!(glass=(host=document.querySelector(selector))?.querySelector('.liquid-morph-glass'))){if(performance.now()>deadline)throw Error('Missing material');await new Promise(requestAnimationFrame);}
  for(const animation of host.getAnimations({subtree:true})){animation.pause();animation.currentTime=animation.id?100:0;}
  const content=[...host.children].filter(child=>child instanceof HTMLElement&&!child.classList.contains('liquid-morph-glass'));
  return{selector,duration:glass.getAnimations()[0].effect.getTiming().duration,opacities:content.map(child=>Number(getComputedStyle(child).opacity)),reveals:content.map(child=>child.getAnimations().filter(a=>a.id==='liquid-overlay-content').map(a=>a.effect.getKeyframes().find(frame=>Number(frame.opacity)===1)?.offset*a.effect.getTiming().duration))};
 },selector);
 const settle=async(surface)=>{await surface.evaluate(host=>{for(const animation of host.getAnimations({subtree:true})){try{animation.finish();}catch{}}});await page.waitForFunction(selector=>!document.querySelector(selector)?.classList.contains('liquid-morph-active'),await surface.evaluate(el=>el.classList.contains('relation-picker')?'.relation-picker':'#settings-dialog'));};
 for(const theme of ['light','dark']){
  await page.evaluate(theme=>localStorage.setItem('noteka.theme.v1',theme),theme);await page.reload();await page.locator('.home').waitFor();
  const row=page.locator(`.home-item[data-entity-id="${task.id}"] .home-open`).first();await row.waitFor();await row.evaluate(button=>button.click());
  const side=page.locator('.peek-side');await side.getByRole('button',{name:'Example project',exact:true}).waitFor();
  const picker=page.getByRole('dialog',{name:'Edit projects',exact:true});
  const menuSample=await freeze(side.getByRole('button',{name:'Edit task project',exact:true}),'.relation-picker');samples.push({theme,...menuSample});
  if(stage!=='before'){check(menuSample.duration===160,`${theme}: project picker completes in 160 ms`);check(menuSample.opacities.every(value=>value>.99),`${theme}: picker content opacity is complete at 100 ms`);check(menuSample.reveals.flat().every(time=>Math.abs(time-100)<1),`${theme}: shared reveal endpoint is 100 ms`);}
  await picker.screenshot({path:`output/playwright/popup-reveal/${stage}-${browser}-${theme}-100ms.png`,animations:'allow'});
  await settle(picker);check(await picker.locator('.liquid-morph-glass').count()===0,`${theme}: temporary layer released`);
  await page.waitForFunction(()=>document.querySelector('.relation-picker-results')?.getAttribute('aria-busy')==='false'&&document.querySelectorAll('.relation-picker-item').length===2);
  await picker.screenshot({path:`output/playwright/popup-reveal/${stage}-${browser}-${theme}-settled.png`});
  await page.keyboard.press('Escape');await picker.waitFor({state:'detached'});check(await side.getByRole('button',{name:'Edit task project',exact:true}).evaluate(button=>button===document.activeElement),`${theme}: Escape restores project trigger focus`);
  await side.getByRole('button',{name:'Close peek',exact:true}).click();await side.waitFor({state:'detached'});
  const settings=page.getByRole('dialog',{name:'Settings',exact:true});const dialogSample=await freeze(page.getByRole('button',{name:'Settings',exact:true}),'#settings-dialog');samples.push({theme,...dialogSample});
  if(stage!=='before'){check(dialogSample.duration===400,`${theme}: dialog material retains 400 ms`);check(dialogSample.opacities.every(value=>value>.99),`${theme}: dialog opacity is complete at 100 ms`);}
  await settle(settings);await settings.getByRole('button',{name:'Close Settings',exact:true}).click();await settings.waitFor({state:'detached'});
 }
 await page.emulateMedia({reducedMotion:'reduce'});await page.getByRole('button',{name:'Settings',exact:true}).click();await page.getByRole('dialog',{name:'Settings',exact:true}).waitFor();check(await page.locator('.liquid-morph-glass').count()===0,'Reduced motion skips the reveal');
 check(!errors.length,`No frontend errors: ${JSON.stringify(errors)}`);check((await control('get',{id:task.id})).title===task.title,'Opening/reveal does not mutate the original');
 await page.context().tracing.stop({path:`output/playwright/popup-reveal/${stage}-${browser}-trace.zip`});await control('finish',{ok:true,checks,samples});return{checks:checks.length,samples};
}
