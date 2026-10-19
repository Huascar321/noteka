async (page) => {
  const origin=new URL(page.url()).origin;
  const external=[];
  await page.context().route('**/*',async route=>{const u=new URL(route.request().url());if(u.origin===origin)await route.continue();else {external.push(u.origin);await route.abort();}});
  await page.exposeFunction('notekaViewport',({width,height})=>page.setViewportSize({width,height}));
  await page.exposeFunction('notekaCapture',async({name})=>{if(typeof name!=='string'||!/^[a-zA-Z0-9-]+$/.test(name))throw new Error('Nombre de captura inválido');const r=await page.request.get(origin+'/__test__/status');const {root}=await r.json();const theme=name.includes('-light-')?'light':'dark';const path=root+'/'+theme+'/'+name+'.png';await page.screenshot({path});return path;});
  await page.addInitScript(() => {
    let pending=false,next=1;const callbacks=new Map();
    window.isTauri=true; // Sólo transporte de pruebas, no host Tauri real.
    window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:()=>{}};
    const control=async(step,input={})=>{
      if(step==='focus_host')return true; // Headless browser ya tiene foco DOM real; no se falsea hasFocus.
      if(step==='viewport')return window.notekaViewport(input);
      if(step==='capture')return window.notekaCapture(input);
      if(step==='pending')return pending;
      if(step==='close_request'){window.__notekaClose?.({event:'noteka-close-requested',payload:null});return true;}
      const response=await fetch('/__test__/control',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({step,input})});const result=await response.json();if(!response.ok)throw new Error(JSON.stringify(result));
      if(step==='finish'||step==='close_confirmed')window.__notekaHeadlessResult=result;
      return result;
    };
    window.__TAURI_INTERNALS__={
      transformCallback:callback=>{const id=next++;callbacks.set(id,callback);return id;},unregisterCallback:id=>callbacks.delete(id),
      invoke:async(cmd,args={})=>{
        if(cmd==='operate'){const response=await fetch('/__test__/operate',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(args.request)});const result=await response.json();if(!response.ok)throw new Error(JSON.stringify(result));return result;}
        if(cmd==='smoke_control')return control(args.step,args.input);
        if(cmd==='pending_edits'){pending=args.pending;return;}
        if(cmd==='close_confirmed')return control('close_confirmed');
        if(cmd==='plugin:event|listen'){window.__notekaClose=callbacks.get(args.handler);return next++;}
        if(cmd==='plugin:event|unlisten')return;
        throw new Error('IPC de prueba desconocido: '+cmd);
      }
    };
  });
  for(const theme of ['dark','light']) {
    for(const phase of ['main','reopen','restored']) {
      await page.request.post(origin+'/__test__/control',{data:{step:'configure',input:{theme,phase}}});
      await page.goto(origin);
      await page.evaluate(async()=>{window.eval(await (await fetch('/__test__/home.js')).text());window.eval(await (await fetch('/__test__/smoke.js')).text());});
      await page.waitForFunction(()=>!!window.__notekaHeadlessResult,undefined,{timeout:480000});
      const after=await (await page.request.get(origin+'/__test__/status')).json();const result=after.results.at(-1);if(!result.ok)throw new Error(JSON.stringify({root:after.root,result}));
    }
    // Eventos de teclado del browser, además de las regresiones DOM compartidas.
    const checks=[];
    const home=page.getByRole('button',{name:'Home',exact:true});
    await home.focus();await page.keyboard.press('Enter');
    await page.locator('.home [data-home-section="person"] .home-coverage').waitFor();
    checks.push('Enter navega a Home desde el shell');
    const people=page.locator('[data-home-section="person"]');
    const next=people.getByRole('button',{name:'New',exact:true});
    await next.focus();await page.keyboard.press('Enter');
    await people.locator('.home-coverage').waitFor();
    if(await next.getAttribute('aria-pressed')!=='true')throw new Error('Enter no cambió vista People');
    await page.keyboard.press('Tab');
    if(!await page.evaluate(()=>document.activeElement?.getAttribute('aria-pressed')==='false'&&document.activeElement.matches(':focus-visible')&&getComputedStyle(document.activeElement).outlineStyle!=='none'))throw new Error('Tab/foco visible de vista People');
    checks.push('Enter cambia vista y Tab conserva foco visible de control nombrado');
    const summary=page.locator('[data-home-section="project"] summary').first();
    const original=await summary.evaluate(el=>el.parentElement.open);
    await summary.focus();await page.keyboard.press('Space');
    if(await summary.evaluate(el=>el.parentElement.open)===original)throw new Error('Space no plegó grupo');
    await page.keyboard.press('Enter');
    if(await summary.evaluate(el=>el.parentElement.open)!==original)throw new Error('Enter no reabrió grupo');
    checks.push('Space/Enter pliegan y reabren grupo sin reemplazar IDs');
    const card=people.locator('.home-open').first();
    const title=(await card.getAttribute('aria-label')).slice('Abrir '.length);
    await card.focus();await page.keyboard.press('Enter');
    await page.waitForFunction(()=>document.querySelector('.title-input')?.disabled===false);
    if(await page.locator('.title-input').inputValue()!==title)throw new Error('Enter abrió otra persona');
    checks.push('Enter abre persona original desde galería textual');
    await page.keyboard.press('Meta+n');
    await page.waitForFunction(()=>document.activeElement?.getAttribute('aria-label')==='Título de la nota');
    await page.keyboard.type('Captura teclado descartable');
    await page.getByRole('button',{name:'Volver a Home',exact:true}).click();
    await page.getByRole('button',{name:'Cancelar',exact:true}).click();
    if(await page.locator('.title-input').inputValue()!=='Captura teclado descartable')throw new Error('Cancelar perdió buffer teclado');
    await page.getByRole('button',{name:'Volver a Home',exact:true}).click();
    await page.getByRole('button',{name:'Descartar',exact:true}).click();
    await page.locator('.home').waitFor();
    checks.push('Meta+n/tecleo real: captura y guardia cancelar/descartar');
    // Retener sólo transporte: la respuesta sigue viniendo de CLI/SQLite reales.
    // Una apertura/captura Home en curso no debe permitir otro buffer que luego se borre.
    const hold = operation => page.evaluate(operation => {
      const original = window.__TAURI_INTERNALS__.invoke;
      window.__notekaHeld = false;
      window.__notekaRelease = null;
      window.__TAURI_INTERNALS__.invoke = async (command, args = {}) => {
        if (!window.__notekaHeld && command === 'operate' && args.request.command.operation === operation) {
          window.__notekaHeld = true;
          await new Promise(resolve => window.__notekaRelease = resolve);
        }
        return original(command, args);
      };
      window.__notekaRestore = () => { window.__TAURI_INTERNALS__.invoke = original; window.__notekaRelease?.(); };
    }, operation);
    const release = () => page.evaluate(() => window.__notekaRestore?.());
    await page.waitForFunction(() => ['task','note','project','tag','person'].every(kind => document.querySelector(`[data-home-section="${kind}"] .home-coverage`)));
    const originalCard = people.locator('.home-open').first();
    const originalTitle = (await originalCard.getAttribute('aria-label')).slice('Abrir '.length);
    await hold('entity.get');
    try {
      await originalCard.click();
      await page.waitForFunction(() => window.__notekaHeld);
      await page.keyboard.press('Meta+n');
      if (!await page.locator('.home').count() || await page.locator('.title-input').count()) throw new Error('Atajo inició un buffer durante apertura Home pendiente');
    } finally { await release(); }
    await page.waitForFunction(() => document.querySelector('.title-input')?.disabled === false);
    if (await page.locator('.title-input').inputValue() !== originalTitle) throw new Error('Apertura Home pendiente terminó en otro original');
    checks.push('Apertura Home lenta bloquea captura por atajo y conserva el original');
    await page.getByRole('button', { name: 'Volver a Home', exact: true }).click();
    await page.locator('.home').waitFor();
    await hold('entity.list');
    try {
      await page.getByRole('button', { name: 'Crear en Tasks', exact: true }).click();
      await page.waitForFunction(() => window.__notekaHeld);
      await page.keyboard.press('Meta+n');
      if (await page.locator('.title-input').count()) throw new Error('Atajo inició otro buffer durante captura Home pendiente');
    } finally { await release(); }
    await page.waitForFunction(() => document.activeElement?.getAttribute('aria-label') === 'Título de la entidad' && document.querySelector('.title-input')?.disabled === false);
    if (!await page.getByLabel('Vencimiento', { exact: true }).count()) throw new Error('Captura Home pendiente cambió de tipo');
    await page.getByRole('button', { name: 'Volver a Home', exact: true }).click();
    await page.getByRole('button', { name: 'Descartar', exact: true }).click();
    await page.locator('.home').waitFor();
    checks.push('Captura Home lenta bloquea atajo competidor y enfoca el buffer del tipo elegido');
    await page.request.post(origin+'/__test__/control',{data:{step:'keyboard_report',input:{ok:true,theme,checks}}});
  }
  if(external.length)throw new Error('Requests remotos intentados: '+external.join(','));
  await page.request.post(origin+'/__test__/control',{data:{step:'network_report',input:{external_origins:external,service_workers:'blocked',browser:'headless',user_agent:await page.evaluate(()=>navigator.userAgent)}}});
  const status=await (await page.request.get(origin+'/__test__/status')).json();
  return {root:status.root,sessions:status.results.map(r=>({theme:r.theme,phase:r.phase,ok:r.ok,checks:r.checks.length})),keyboard_checks:14,external_requests:external.length};
}
