async page => {
 const origin=new URL(page.url()).origin, checks=[], samples=[], errors=[];
 page.on('pageerror',e=>errors.push(e.message));
 await page.context().route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());
 await page.addInitScript(()=>{let id=1;window.isTauri=true;window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener(){}};window.__TAURI_INTERNALS__={transformCallback(){return id++;},unregisterCallback(){},async invoke(command,args={}){if(command==='operate')return(await fetch('/__test__/operate',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(args.request)})).json();if(['pending_edits','close_confirmed','plugin:event|listen','plugin:event|unlisten'].includes(command))return id++;throw Error(command);}};const add=EventTarget.prototype.addEventListener;window.__wheelBindings=[];EventTarget.prototype.addEventListener=function(type,listener,options){if(type==='wheel')window.__wheelBindings.push({document:this===document,passive:options?.passive,capture:options?.capture});return add.call(this,type,listener,options);};});
 const control=async(step,input={})=>{const r=await page.request.post(origin+'/__test__/control',{data:{step,input}});if(!r.ok())throw Error(await r.text());return r.json();};
 const check=(v,s)=>{if(!v)throw Error(s);checks.push(s);};
 await control('configure',{theme:'light',phase:'main'});const info=await control('init');await page.setViewportSize({width:1180,height:780});await page.goto(origin);await page.getByLabel('Workspace path').fill(info.path);await page.getByRole('button',{name:'Create workspace',exact:true}).click();await page.locator('.home').waitFor();
 check(await page.evaluate(()=>window.__wheelBindings.filter(b=>b.document).every(b=>b.passive===true)),'Document wheel classification never registers a blocking listener');
 const browser=page.context().browser().browserType().name(),dialog=page.getByRole('dialog',{name:'Settings',exact:true});
 for(const theme of ['light','dark']){
  await page.getByRole('button',{name:'Settings',exact:true}).click();await dialog.getByLabel('Theme',{exact:true}).selectOption(theme);await dialog.getByRole('button',{name:'Close Settings',exact:true}).click();await dialog.waitFor({state:'detached'});
  for(const time of [0,50,100,180,224,312,399]){
   const result=await page.getByRole('button',{name:'Settings',exact:true}).evaluate(async(trigger,time)=>{
    trigger.click();let host,glass;const deadline=performance.now()+2500;while(!(glass=(host=document.querySelector('#settings-dialog'))?.querySelector('.liquid-morph-glass'))){if(performance.now()>deadline)throw Error('No material');await new Promise(requestAnimationFrame);}
    const animations=host.getAnimations({subtree:true});for(const a of animations){a.pause();a.currentTime=a.id?time:0;}
    const bounds=glass.getBoundingClientRect(), matrix=new DOMMatrixReadOnly(getComputedStyle(glass).transform);
    const rows=[...host.children].filter(e=>!e.classList.contains('liquid-morph-glass')).map(e=>{const r=e.getBoundingClientRect(),s=getComputedStyle(e);const body=s.clipPath.match(/^inset\((.*?) round/);const values=body?.[1].trim().split(/\s+/).map(parseFloat)||[];const inset=values.length===1?[values[0],values[0],values[0],values[0]]:values.length===2?[values[0],values[1],values[0],values[1]]:values.length===3?[values[0],values[1],values[2],values[1]]:values;return{error:Math.max(...inset.map((v,i)=>Math.abs(v-[bounds.top-r.top,r.right-bounds.right,r.bottom-bounds.bottom,bounds.left-r.left][i]))),opacity:Number(s.opacity),transform:s.transform,clip:s.clipPath};});
    return {time,duration:glass.getAnimations()[0].effect.getTiming().duration,rows,matrix:[matrix.a,matrix.d]};
   },time);
   samples.push({theme,...result});check(result.duration===400,`${theme}/${time}: approved 400 ms duration`);check(result.rows.every(r=>r.error<1.5&&r.transform==='none'&&r.opacity>=.23),`${theme}/${time}: unscaled content tracks glass within 1.5 px ${JSON.stringify(result.rows)}`);
   if(time===100)await dialog.screenshot({path:`output/playwright/performance/reveal-${browser}-${theme}-100.png`,animations:'allow'});
   await dialog.evaluate(host=>{for(const a of host.getAnimations({subtree:true})){try{a.finish();}catch{}}});await page.waitForFunction(()=>!document.querySelector('#settings-dialog.liquid-morph-active'));await page.keyboard.press('Escape');await dialog.waitFor({state:'detached'});
  }
 }
 const canvas=page.locator('canvas.ascii-shark').first();const before=await canvas.getAttribute('data-frame');await page.waitForFunction(value=>document.querySelector('canvas.ascii-shark').dataset.frame!==value,before);check(true,'Canvas still animates with throttled diagnostic publishing');
 check(!errors.length,`No frontend errors: ${JSON.stringify(errors)}`);await control('finish',{ok:true,checks,samples});return {checks:checks.length};
}
