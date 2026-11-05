async (page) => {
  page.setDefaultTimeout(15000);
  const origin = new URL(page.url()).origin;
  const auditParams = new URLSearchParams(new URL(page.url()).hash.slice(1));
  const auditAccent = auditParams.get('contrast'), selectedAuditScenes = auditParams.get('scenes')?.split(',');
  const contrast = [], contrastFailures = [], contrastSkipped = [], fallbackChecks = [], fallbackFailures = [];
  const browser = page.context().browser().browserType().name();
  const directory = 'output/playwright/liquid-glass-guide/' + browser;
  const captures = [], errors = [], external = [];
  // Each scene declares all three axes. null means the axis is not presented
  // within that capture, rather than an inherited, undocumented selection.
  const sceneContexts = {
    welcome:[null,null,null], overview:[null,'Tasks / Today',null], sidebar:[null,null,null],
    'cover-selector':[null,'Home / ASCII',null], toolbar:[null,'Tasks / Today',null],
    people:[null,'People / Contacts',null], 'birthday-calendar':[null,'People / Birthdays',null],
    'move-project':[null,'Projects / Board',null], collections:['Studio collection','Studio list',null],
    'detail-task':[null,'Task detail',null], 'detail-project':[null,'Project detail',null],
    'detail-goal':[null,'Goal detail',null], 'detail-tag':[null,'Tag detail',null],
    'detail-person':[null,'Person detail',null], 'detail-milestone':[null,'Milestone detail',null],
    note:[null,'Note detail',null], 'page-menu':[null,'Note detail',null],
    'export-tools':[null,'Note detail',null], 'attachment-tools':[null,'Note detail',null], 'resource-tools':[null,'Note detail',null],
    'note-locked':[null,'Note detail',null], 'relation-picker':[null,'Note detail',null],
    'insert-view':['Studio collection','Studio list',null],
    'note-header-dirty':[null,'Note detail',null], 'save-guard':[null,'Note detail',null],
    settings:[null,null,null], 'settings-color':[null,null,null], 'settings-workspace':[null,null,null],
    search:[null,null,'Local search / all types'], 'search-solid':[null,null,'Local search / all types'],
    'search-solid-fallback':[null,null,'Local search / all types'],
    'advanced-search':[null,null,'Notes and entities'], 'advanced-search-narrow':[null,null,'Notes and entities'],
    'attachment-search':[null,null,'Attachments'], 'combined-search':[null,null,'Notes and attachments (groups)'],
    'violet-action':[null,'Note detail',null], 'save-guard-narrow':[null,'Note detail',null],
  };
  const auditSelectors = {
    welcome:'.glass-field, .glass-action, .glass-secondary',
    toolbar:'.task-new, .glass-tool-group button',
    people:'.task-new, .glass-tool-group button',
    'birthday-calendar':'.birthday-tools button',
    'move-project':'button, select',
    collections:'.collection-actions button, .collection-tabs button, .collection-templates button, select',
    'detail-task':'.entity-fields input:not([type=checkbox]), .entity-fields select, .task-page-properties select, .task-page-properties input:not([type=checkbox]), .task-project-value button, .note-page-actions button',
    'detail-project':'.entity-fields input:not([type=checkbox]), .entity-fields select, .project-page-properties select, .project-page-properties input:not([type=checkbox]), .project-contents header button, .note-page-actions button',
    'detail-goal':'.entity-fields input:not([type=checkbox]), .entity-fields select, .goal-page-properties select, .goal-tag-value button, .goal-journal-add, .goal-project-add, .note-page-actions button',
    'detail-tag':'.entity-fields input:not([type=checkbox]), .entity-fields select, .tag-page-properties select, .tag-contents header button, .note-page-actions button',
    'detail-person':'.entity-fields input:not([type=checkbox]), .entity-fields select, .person-contents header button, .note-page-actions button',
    'detail-milestone':'.entity-fields input:not([type=checkbox]), .entity-fields select, .note-page-actions button',
    note:'.note-page-properties select, .note-tag-value button, .note-page-actions button',
    'page-menu':'button', 'export-tools':'.glass-field, .glass-secondary',
    'attachment-tools':'.glass-field, .glass-secondary', 'resource-tools':'.glass-secondary', 'relation-picker':'input,button',
    'insert-view':'select,button', 'note-header-dirty':'.glass-action, .page-refresh, .page-actions-trigger',
    'save-guard':'button', settings:'.settings-search input, .settings-theme-row select, .settings-change-workspace',
    'settings-color':'#primary-color-hex, .primary-color-hex button', 'settings-workspace':'.settings-change-workspace',
    search:'.context-search-input input, .context-search-input button, .context-search-filters button',
    'advanced-search':'.glass-field, .glass-secondary', 'attachment-search':'.glass-field, .glass-secondary',
    'combined-search':'.glass-field, .glass-secondary', 'advanced-search-narrow':'.glass-field, .glass-secondary',
    'search-solid':'.context-search-input input, .context-search-input button, .context-search-filters button',
    'search-solid-fallback':'.context-search-input input, .context-search-input button, .context-search-filters button',
    'save-guard-narrow':'button', 'violet-action':'.glass-action, .page-refresh, .page-actions-trigger',
  };
  const auditScene = async (name, locator, theme) => {
    const selector = auditSelectors[name];
    if (!auditAccent || !selector || (selectedAuditScenes && !selectedAuditScenes.includes(name))) return;
    if (auditAccent !== 'Violet' && ['save-guard-narrow','violet-action','search-solid-fallback'].includes(name)) return;
    const startedInHighContrast = await page.evaluate(() => matchMedia('(prefers-contrast: more)').matches);
    const scrollState = await locator.evaluate(el=>{ const saved=[]; for(let n=el;n;n=n.parentElement) if(n.scrollHeight>n.clientHeight||n.scrollWidth>n.clientWidth) saved.push([n,n.scrollTop,n.scrollLeft]); for(const n of el.querySelectorAll('*')) if(n.scrollHeight>n.clientHeight||n.scrollWidth>n.clientWidth) saved.push([n,n.scrollTop,n.scrollLeft]); window.__glassAuditScroll=saved; });
    const targets = locator.locator(selector);
    for (let i=0, count=await targets.count(); i<count; i++) {
      const target = targets.nth(i);
      if (!await target.isVisible()) { contrastSkipped.push({scene:name,theme,index:i,reason:'hidden'}); continue; }
      if (await target.isDisabled()) { contrastSkipped.push({scene:name,theme,index:i,reason:'disabled'}); continue; }
      const readMeta = () => target.evaluate(el => {
        const r=el.getBoundingClientRect();
        if (r.width<8 || r.height<8 || r.left<0 || r.top<0 || r.right>innerWidth || r.bottom>innerHeight) return null;
        const input=el instanceof HTMLInputElement;
        const placeholder=input && !el.value && !!el.placeholder;
        const style=getComputedStyle(el, placeholder ? '::placeholder' : null);
        const text=el.getAttribute('aria-label') || (input ? (el.value || el.placeholder) : el.textContent?.trim()) || el.getAttribute('title') || el.tagName;
        let opacity=1; for(let node=el;node;node=node.parentElement) opacity*=Number(getComputedStyle(node).opacity);
        return {color:style.color, opacity, text:text.slice(0,100), placeholder};
      });
      let meta = await readMeta();
      if (!meta) { await target.scrollIntoViewIfNeeded(); meta = await readMeta(); }
      if (!meta) { contrastSkipped.push({scene:name,theme,index:i,reason:'outside viewport after scrolling'}); continue; }
      for (const interaction of ['rest','hover']) {
        if (interaction === 'rest') await page.mouse.move(0,0); else await target.hover();
        await target.evaluate(el=>el.setAttribute('data-glass-contrast-probe',''));
        const regions = await target.evaluate((el,{placeholder}) => {
          const box=el.getBoundingClientRect();
          const opacityOf=node=>{let a=1;for(let p=node;p;p=p.parentElement)a*=Number(getComputedStyle(p).opacity);return a;};
          const region=(rect,color,opacity)=>({x:Math.max(0,rect.left-box.left),y:Math.max(0,rect.top-box.top),w:Math.min(rect.right,box.right)-Math.max(rect.left,box.left),h:Math.min(rect.bottom,box.bottom)-Math.max(rect.top,box.top),color,opacity});
          if(el instanceof HTMLInputElement || el instanceof HTMLSelectElement){
            const style=getComputedStyle(el,placeholder?'::placeholder':null), base=getComputedStyle(el);
            const pad=parseFloat(base.paddingLeft)||0, value=el instanceof HTMLSelectElement ? el.selectedOptions[0]?.textContent||'' : el.value||el.placeholder||'00/00/0000';
            const canvas=document.createElement('canvas'),ctx=canvas.getContext('2d');ctx.font=`${base.fontWeight} ${base.fontSize} ${base.fontFamily}`;
            const width=Math.min(box.width-pad-24,Math.max(12,ctx.measureText(value).width+2));
            const height=Math.min(box.height-4,parseFloat(base.lineHeight)||parseFloat(base.fontSize)*1.3);
            return [region({left:box.left+pad,top:box.top+(box.height-height)/2,right:box.left+pad+width,bottom:box.top+(box.height+height)/2},style.color,opacityOf(el)*(placeholder?Number(style.opacity):1))];
          }
          const nodes=[], walker=document.createTreeWalker(el,NodeFilter.SHOW_TEXT);
          while(walker.nextNode()){const node=walker.currentNode;if(!node.textContent.trim())continue;const range=document.createRange();range.selectNodeContents(node);const st=getComputedStyle(node.parentElement);for(const rect of range.getClientRects()) if(rect.width>1 && rect.height>1) nodes.push(region(rect,st.color,opacityOf(node.parentElement)));}
          if(nodes.length) return nodes;
          const icon=el.querySelector('svg');if(icon){const st=getComputedStyle(icon);return [region(icon.getBoundingClientRect(),st.color,opacityOf(icon))];}
          const st=getComputedStyle(el);return [region(box,st.color,opacityOf(el))];
        },meta);
        const hideInk='[data-glass-contrast-probe], [data-glass-contrast-probe] * { color:transparent !important; -webkit-text-fill-color:transparent !important; text-shadow:none !important; caret-color:transparent !important; } [data-glass-contrast-probe]::placeholder, [data-glass-contrast-probe]::-webkit-datetime-edit, [data-glass-contrast-probe]::-webkit-datetime-edit-fields-wrapper, [data-glass-contrast-probe]::-webkit-datetime-edit-text, [data-glass-contrast-probe]::-webkit-datetime-edit-year-field, [data-glass-contrast-probe]::-webkit-datetime-edit-month-field, [data-glass-contrast-probe]::-webkit-datetime-edit-day-field { color:transparent !important; -webkit-text-fill-color:transparent !important; } [data-glass-contrast-probe] svg { visibility:hidden !important; }';
        const shot = await target.screenshot({ animations:'disabled', style:hideInk });
        await target.evaluate(el=>el.removeAttribute('data-glass-contrast-probe'));
        const ratio=await page.evaluate(async ({png,regions})=>{
          const img=new Image(); img.src='data:image/png;base64,'+png; await img.decode();
          const c=document.createElement('canvas'); c.width=img.naturalWidth; c.height=img.naturalHeight; const x=c.getContext('2d',{willReadFrequently:true});x.drawImage(img,0,0);const pixels=x.getImageData(0,0,c.width,c.height).data;
          const k=document.createElement('canvas').getContext('2d');
          const lum=a=>a.map(v=>v/255).map(v=>v<=.04045?v/12.92:((v+.055)/1.055)**2.4).reduce((v,n,i)=>v+n*[.2126,.7152,.0722][i],0);
          let worst=Infinity;
          for(const region of regions){
            k.clearRect(0,0,1,1);k.fillStyle=region.color;k.fillRect(0,0,1,1);const rgba=[...k.getImageData(0,0,1,1).data],sx=devicePixelRatio,sy=devicePixelRatio;
            const x0=Math.max(Math.ceil(3*sx),Math.ceil(region.x*sx)),x1=Math.min(c.width-Math.ceil(3*sx),Math.floor((region.x+region.w)*sx));
            const y0=Math.max(1,Math.ceil((region.y+region.h*.2)*sy)),y1=Math.min(c.height-1,Math.floor((region.y+region.h*.8)*sy));
            for(let y=y0;y<y1;y+=3)for(let xx=x0;xx<x1;xx+=4){const offset=(y*c.width+xx)*4,bg=[pixels[offset],pixels[offset+1],pixels[offset+2]];const alpha=rgba[3]/255*region.opacity;const fg=bg.map((b,i)=>rgba[i]*alpha+b*(1-alpha));const a=lum(fg),b=lum(bg);worst=Math.min(worst,(Math.max(a,b)+.05)/(Math.min(a,b)+.05));}
          } return worst;
        },{png:shot.toString('base64'),regions});
        const result={scene:name,index:i,theme,accent:auditAccent,interaction,control:meta.text,placeholder:meta.placeholder,regions,ratio:Number.isFinite(ratio)?Number(ratio.toFixed(3)):null};
        contrast.push(result); if (!Number.isFinite(ratio) || ratio<4.5) contrastFailures.push(result);
      }
    }
    await locator.evaluate(()=>{for(const [el,top,left] of window.__glassAuditScroll||[]) {el.scrollTop=top;el.scrollLeft=left;} delete window.__glassAuditScroll;});
    // The same media rules implement both preferences. Exercise contrast via
    // the browser and reduced transparency by enabling those exact rules in
    // place; WebKit does not expose the latter preference through Playwright.
    const checkFallback = async preference => {
      if(preference==='contrast') await page.emulateMedia({contrast:'more',reducedMotion:'reduce'});
      else await page.evaluate(()=>{const saved=[];const visit=rules=>{for(const rule of rules){if(rule instanceof CSSMediaRule&&rule.conditionText.includes('prefers-reduced-transparency')){saved.push([rule,rule.media.mediaText]);rule.media.mediaText='all';}else if(rule.cssRules)visit(rule.cssRules);}};for(const sheet of document.styleSheets)try{visit(sheet.cssRules);}catch{}window.__glassRestoreMedia=()=>saved.forEach(([r,v])=>r.media.mediaText=v);});
      const values=await locator.evaluate(el=>{
        const selector='.glass-popup,.glass-menu,.glass-action.primary,.glass-secondary,.glass-field,.glass-tool-group,.note-page-actions,.task-new.primary,.task-project-value,.goal-tag-value,.note-tag-value,.task-status-control,.project-detail-state,.goal-detail-state,.tag-detail-type,.context-search-input,.settings-search';
        const nodes=[...(el.matches(selector)?[el]:[]),...el.querySelectorAll(selector)];
        return nodes.filter(n=>n.getClientRects().length).map(n=>{const s=getComputedStyle(n),p=getComputedStyle(n,'::before');return {className:String(n.className),tag:n.tagName,background:s.backgroundImage,shadow:s.boxShadow,filter:s.backdropFilter,pseudo:p.content};});
      });
      for(const value of values){const ok=value.background==='none'&&value.shadow==='none'&&value.filter==='none'&&['none','normal'].includes(value.pseudo);const item={scene:name,theme,accent:auditAccent,preference,...value,ok};fallbackChecks.push(item);if(!ok)fallbackFailures.push(item);}
      if(preference==='contrast') await page.emulateMedia({contrast:'no-preference',reducedMotion:'reduce'});
      else await page.evaluate(()=>{window.__glassRestoreMedia?.();delete window.__glassRestoreMedia;});
    };
    await checkFallback('contrast'); await checkFallback('reduced-transparency-rules');
    if (startedInHighContrast) await page.emulateMedia({contrast:'more',reducedMotion:'reduce'});
  };
  const capture = async (name, locator, theme, state, accent = 'Blue') => {
    await locator.waitFor({ state: 'visible' });
    if (!Object.hasOwn(sceneContexts, name)) throw Error(`Scene has no declared context: ${name}`);
    // Wait for bridge operations and a quiet window, not merely an attached node.
    await page.waitForFunction(() => (window.__glassPending ?? 0) === 0 && Date.now() - (window.__glassLastOp ?? 0) >= 350);
    await page.waitForFunction(el => [el,...el.querySelectorAll('[aria-busy="true"]')].every(node=>node.getAttribute('aria-busy') !== 'true' || !node.getClientRects().length), await locator.elementHandle());
    const [collection, view, searchScope] = sceneContexts[name];
    if (collection) {
      const collectionControl = name === 'insert-view' ? page.getByLabel('Collection to embed') : page.locator('.collection-tabs button.active');
      const actualCollection = name === 'insert-view' ? await collectionControl.locator('option:checked').textContent() : await collectionControl.textContent();
      if (actualCollection?.trim() !== collection) throw Error(`${name}: unexpected collection ${actualCollection}`);
      const viewControl = name === 'insert-view' ? page.getByLabel('View to embed') : page.getByLabel('Collection view');
      const actualView = (await viewControl.locator('option:checked').textContent())?.trim();
      if (!actualView?.startsWith(view)) throw Error(`${name}: unexpected view ${actualView}`);
    }
    if (['Notes and entities','Attachments','Notes and attachments (groups)'].includes(searchScope)) {
      const actual = (await page.getByLabel('Search scope', {exact:true}).locator('option:checked').textContent())?.trim();
      if (actual !== searchScope) throw Error(`${name}: unexpected search scope ${actual}`);
    } else if (searchScope === 'Local search / all types') {
      const dialog = page.getByRole('dialog',{name:'Search workspace',exact:true});
      if (await dialog.getByRole('button',{name:/Filters/}).getAttribute('aria-expanded') !== 'false' || await dialog.getByRole('combobox').inputValue() !== '') throw Error(`${name}: search is not in its explicit default scope`);
    }
    for (const [expected, group, button] of [['Tasks / Today','Tasks views','Today'],['People / Contacts','People views','Contacts'],['People / Birthdays','People views','Birthdays'],['Projects / Board','Projects views','Board']]) {
      if (view === expected && await page.getByRole('group',{name:group,exact:true}).getByRole('button',{name:button,exact:true}).getAttribute('aria-pressed') !== 'true') throw Error(`${name}: view ${expected} not selected`);
    }
    if (view === 'Home / ASCII' && await page.getByRole('group',{name:'Home cover'}).getByRole('button',{name:'ASCII'}).getAttribute('aria-pressed') !== 'true') throw Error(`${name}: ASCII not selected`);
    const detailTitles={'Task detail':'Review visual guidelines','Project detail':'Design system','Goal detail':'Studio goal','Tag detail':'Design','Person detail':'Ana Torres','Milestone detail':'Studio milestone','Note detail':'Product brief'};
    if (Object.hasOwn(detailTitles,view)) {
      const actual=await pane().locator('.title-input').inputValue();
      const expected=detailTitles[view];
      if (actual!==expected && !(view==='Note detail' && actual==='Product brief — draft')) throw Error(`${name}: unexpected detail ${actual}`);
    }
    await locator.evaluate(el => Promise.all(el.getAnimations().map(animation => animation.finished.catch(() => {}))));
    await auditScene(name, locator, theme);
    if (auditAccent) await page.mouse.move(0,0);
    const file = `${name}-${theme}.png`;
    await locator.screenshot({ path: directory + '/' + file, animations: 'disabled' });
    const size = await locator.boundingBox();
    const selection = await page.evaluate(() => {
      const visible = el => el && el.getClientRects().length > 0 && getComputedStyle(el).visibility !== 'hidden';
      const selected = selector => { const el = [...document.querySelectorAll(selector)].find(visible); return el instanceof HTMLSelectElement ? el.selectedOptions[0]?.textContent?.trim() ?? null : el?.textContent?.trim() ?? null; };
      return { collection: selected('.collection-tabs button.active, [aria-label="Collection to embed"]'),
        view: selected('[aria-label="Collection view"], [aria-label="View to embed"]'),
        searchScope: selected('[aria-label="Search scope"]') ?? (visible(document.querySelector('.context-search[open]')) ? 'Local search: all types, unscoped' : null) };
    });
    captures.push({ file, theme, accent, state, sceneContext: { collection, view, searchScope }, selection, viewport: page.viewportSize(), cssSize: { width: size.width, height: size.height } });
  };
  page.on('pageerror', error => errors.push(String(error)));
  await page.context().route('**/*', async route => {
    if (new URL(route.request().url()).origin === origin) await route.continue();
    else { external.push(route.request().url()); await route.abort(); }
  });
  await page.addInitScript(() => {
    let next = 1;
    window.__glassPending = 0; window.__glassLastOp = 0;
    window.isTauri = true;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      transformCallback() { return next++; }, unregisterCallback() {},
      async invoke(command, args = {}) {
        if (command === 'operate') {
          window.__glassPending++; window.__glassLastOp = Date.now();
          try { return await (await fetch('/__test__/operate', {
            method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args.request),
          })).json(); } finally { window.__glassPending--; window.__glassLastOp = Date.now(); }
        }
        if (['pending_edits', 'close_confirmed', 'plugin:event|listen', 'plugin:event|unlisten'].includes(command)) return next++;
        throw Error('IPC inesperado: ' + command);
      },
    };
  });
  const control = async (step, input = {}) => {
    const response = await page.request.post(origin + '/__test__/control', { data: { step, input } });
    if (!response.ok()) throw Error(await response.text());
    return response.json();
  };
  const assert = (condition, message) => { if (!condition) throw Error(message); };
  await control('configure', { theme: 'light', phase: 'main' });
  const info = await control('init');
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(origin);
  await page.evaluate(accent => { localStorage.clear(); localStorage.setItem('noteka.peek-mode.v1', 'full'); if (accent === 'Violet') localStorage.setItem('noteka.primary-color.v1', '#8254d4'); }, auditAccent);
  await page.reload();
  for (const theme of ['light', 'dark']) {
    await page.evaluate(theme => localStorage.setItem('noteka.theme.v1', theme), theme);
    await page.reload();
    await page.waitForFunction(theme => document.documentElement.dataset.theme === theme, theme);
    await capture('welcome', page.locator('.welcome'), theme, 'Empty synthetic workspace path; no workspace selected');
  }
  await page.evaluate(() => localStorage.setItem('noteka.theme.v1', 'light'));
  await page.reload();
  await page.locator('#workspace-path').fill(info.path);
  await page.getByRole('button', { name: 'Create workspace', exact: true }).click();
  await page.locator('.home').waitFor();
  const attachment = await control('import_attachment', { source_path: info.image });
  const person = await control('entity_create', { kind: 'person', title: 'Ana Torres', payload: {
    company: 'Studio Norte', relationships: ['Friend', 'Colleague'], photo: attachment.id, pipeline_status: 'contacted',
  } });
  await control('entity_create', { kind: 'person', title: 'Alex Rivera', payload: { company: 'Studio Sur', relationships: ['Colleague'], pipeline_status: 'prospect' } });
  const project = await control('entity_create', { kind: 'project', title: 'Design system', payload: { state: 'doing' } });
  const tag = await control('entity_create', { kind: 'tag', title: 'Design', payload: { classification: 'topic' } });
  const task = await control('entity_create', { kind: 'task', title: 'Review visual guidelines', payload: { state: 'todo' } });
  const goal = await control('entity_create', { kind: 'note', title: 'Studio goal', payload: { inbox: false, goal: { state: 'active' } } });
  const milestone = await control('entity_create', { kind: 'note', title: 'Studio milestone', payload: { inbox: false, milestone: { goal_id: goal.id } } });
  const operate = async (operation, input) => {
    const response = await page.request.post(origin + '/__test__/operate', { data: { workspace: info.path, command: { operation, input } } });
    const result = await response.json();
    if (!response.ok() || !result.ok) throw Error(`${operation}: ${JSON.stringify(result)}`);
    return result.data;
  };
  const collection = (await operate('collection.create', { name: 'Studio collection' })).collection;
  await operate('collection.create', { name: 'Archive collection' });
  const savedView = await operate('view.save', { collection_id: collection.id, name: 'Studio list', layout: 'list', config: {} });
  await control('relation_add', { source_id: task.id, target_id: project.id, kind: 'project', expected_revision: task.revision });
  let note = await control('entity_create', { kind: 'note', title: 'Product brief', payload: { note_type: 'reference', favorite: false }, document: {
    schema_version: 2, blocks: [
      { id: 'intro', type: 'paragraph', content: [{ type: 'text', text: 'Keep the content clear. Use glass for navigation and floating controls.' }] },
      { id: 'sample', type: 'code_block', attrs: { language: 'css' }, content: [{ type: 'text', text: '/* Shared material */\nbackground: var(--glass-surface);\nborder-radius: 14px;' }] },
    ],
  } });
  for (const [target, kind] of [[project, 'project'], [person, 'person'], [tag, 'tag']]) {
    await control('relation_add', { source_id: note.id, target_id: target.id, kind, expected_revision: note.revision });
    note = await control('get', { id: note.id });
  }
  const nav = page.getByRole('navigation', { name: 'Sections', exact: true });
  const pane = () => page.locator('.workspace-tab-panel.peek:not(.peek-hidden):visible').last();
  const settings = async (theme, color) => {
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await page.getByLabel('Theme', { exact: true }).selectOption(theme);
    await page.getByRole('button', { name: color, exact: true }).click();
    await page.getByRole('button', { name: 'Close Settings', exact: true }).click();
  };
  const search = async () => {
    if (!await page.locator('.desktop-sidebar .sidebar-search').isVisible()) await page.getByRole('button', { name: 'Expand sidebar', exact: true }).click();
    await page.locator('.desktop-sidebar .sidebar-search').click();
    await page.getByRole('combobox', { name: 'Search workspace' }).waitFor();
    await page.waitForFunction(() => document.querySelector('#context-search-results')?.getAttribute('aria-busy') === 'false');
  };
  const openRecord = async title => {
    await search();
    await page.getByRole('combobox', { name: 'Search workspace' }).fill(title);
    await page.getByRole('option').filter({ has: page.locator('strong', { hasText: title }) }).first().click();
    await pane().locator('.title-input').waitFor();
  };
  const openNote = async () => {
    await openRecord(note.title);
    await pane().getByRole('button', { name: 'Remove relation to Design system', exact: true }).waitFor();
  };

  for (const theme of ['light', 'dark']) {
    await settings(theme, auditAccent || 'Blue');
    await nav.getByRole('button', { name: 'Home', exact: true }).click();
    await page.locator('.home').waitFor();
    await page.getByRole('group',{name:'Tasks views',exact:true}).getByRole('button',{name:'Today',exact:true}).click();
    await page.locator('.home-tasks').first().getByRole('button', { name: 'Refresh Tasks', exact: true }).click();
    await page.locator('.home-heading h1').click();
    await page.mouse.move(800, 500);
    await capture('overview', page.locator('.app-window'), theme, 'Home, sidebar, tabs and breadcrumb');
    await capture('sidebar', page.locator('.desktop-sidebar'), theme, 'Selected Home and compact Search');
    await page.getByRole('group', { name: 'Home cover', exact: true }).getByRole('button', { name: 'ASCII', exact: true }).click();
    await page.keyboard.press('Tab');
    await page.getByRole('group', { name: 'Home cover', exact: true }).getByRole('button', { name: 'ASCII', exact: true }).focus();
    await capture('cover-selector', page.locator('.home-cover-controls'), theme, 'ASCII selected, keyboard focus');
    const toolbar = page.locator('.home-tasks .task-view-toolbar').first();
    await toolbar.scrollIntoViewIfNeeded();
    await capture('toolbar', toolbar, theme, 'Sliding view indicator, grouped tools and New');
    await nav.getByRole('button', { name: 'People', exact: true }).click();
    const section = page.locator('.workspace-tab-panel.active .home-people');
    await section.getByRole('group',{name:'People views',exact:true}).getByRole('button',{name:'Contacts',exact:true}).click();
    await section.getByRole('button', { name: 'Refresh People', exact: true }).click();
    const card = section.locator(`[data-person-id="${person.id}"]`);
    await card.locator('img').evaluate(async image => { await image.decode(); });
    await page.keyboard.press('Tab');
    await card.getByRole('button', { name: 'Open Ana Torres', exact: true }).focus();
    await capture('people', section, theme, 'Synthetic contacts, solid cards with glass frame and keyboard focus');
    await section.getByRole('group', { name: 'People views' }).getByRole('button', { name: 'Birthdays', exact: true }).click();
    await section.locator('.people-calendar').waitFor();
    await page.waitForFunction(() => document.querySelector('.workspace-tab-panel.active .home-people')?.getAttribute('aria-busy') === 'false');
    await capture('birthday-calendar', section.locator('.people-calendar-toolbar'), theme, 'People / Birthdays; calendar navigation grouped');
    await nav.getByRole('button', { name: 'Projects', exact: true }).click();
    const projects = page.locator('.workspace-tab-panel.active .home-projects');
    await projects.getByRole('group', { name: 'Projects views' }).getByRole('button', { name: 'Board', exact: true }).click();
    await projects.getByRole('button', { name: 'Move Design system', exact: true }).click();
    const move = page.getByRole('dialog', { name: 'Move Design system', exact: true });
    await capture('move-project', move, theme, 'Projects / Board; confirmation and native selectors');
    await move.getByRole('button', { name: 'Cancel', exact: true }).click();
    await page.locator('.desktop-sidebar .collections-shortcut').click();
    const database = page.locator('.workspace-tab-panel.active .database-panel');
    await database.getByRole('button', { name: 'Studio collection', exact: true }).click();
    await database.getByLabel('Collection view').selectOption(savedView.id);
    await database.getByLabel('Collection layout').waitFor();
    await capture('collections', database, theme, 'Studio collection / Studio list; loaded local results');
    for (const [name, entity] of [['task', task], ['project', project], ['goal', goal], ['tag', tag], ['person', person], ['milestone', milestone]]) {
      await openRecord(entity.title);
      if (!await pane().locator('.page-details').evaluate(el => el.open)) await pane().locator('.page-details summary').click();
      await pane().locator('.page-details[open]').waitFor();
      await capture(`detail-${name}`, pane(), theme, `${name} detail; View details open`);
      await pane().getByRole('button', { name: 'Close peek', exact: true }).click();
    }
    await openNote();
    await capture('note', pane(), theme, 'Full note pane: compact Tag/Type, relation capsules, header and solid content');
    const actions = pane().getByRole('group', { name: 'Note actions', exact: true });
    await actions.getByRole('button', { name: 'Page actions', exact: true }).focus();
    await page.keyboard.press('Enter');
    const menu = pane().getByRole('menu', { name: 'Page actions', exact: true });
    await capture('page-menu', menu, theme, 'Open local glass menu');
    for (const [label, name] of [['Export','export-tools'], ['Attachments','attachment-tools'], ['External resources','resource-tools']]) {
      await menu.getByRole('menuitem', { name:label, exact:true }).click();
      const tool = pane().locator('.page-tool-panel:not([hidden])'); await tool.waitFor();
      if (name === 'export-tools') await tool.getByLabel('Export destination').fill('/synthetic/noteka-export');
      if (name === 'attachment-tools') { await tool.locator('.preparation-panel > summary').click(); await tool.locator('.generation-panel > summary').click(); }
      await capture(name, tool, theme, `${label} tool; synthetic note`);
      await page.keyboard.press('Escape');
      await actions.getByRole('button', { name: 'Page actions', exact: true }).click();
    }
    await menu.getByRole('menuitemcheckbox', { name: 'Lock page', exact: true }).click();
    await actions.getByRole('button', { name: 'Unlock page temporarily' }).waitFor();
    assert(await pane().getByLabel('Note type', { exact: true }).isDisabled(), 'Type must be disabled while locked');
    await capture('note-locked', pane().locator('.note-page-properties'), theme, 'Locked fields retain frame and height; Tag destination remains navigable');
    await actions.getByRole('button', { name: 'Page actions', exact: true }).click();
    await menu.getByRole('menuitemcheckbox', { name: 'Lock page', exact: true }).click();
    await page.waitForFunction(() => !document.querySelector('.peek .page-lock-status'));
    await pane().getByRole('button', { name: 'Add person', exact: true }).click();
    const picker = page.getByRole('dialog', { name: 'Edit people', exact: true });
    await picker.getByLabel('Search people').fill('Ana');
    await picker.getByRole('button', { name: /Ana Torres/ }).waitFor();
    await capture('relation-picker', picker, theme, 'Mounted relation picker uses glass-menu');
    await page.keyboard.press('Escape');
    const paragraph = pane().locator('.ProseMirror p').first();
    await paragraph.click();
    await page.keyboard.press('End');
    await page.keyboard.press('Enter');
    await page.keyboard.type('/embedded');
    await pane().getByRole('option', { name: /Embedded database/ }).click();
    const viewPicker = pane().getByRole('region', { name: 'Insert local view' });
    await viewPicker.getByLabel('Collection to embed').selectOption(collection.id);
    await viewPicker.getByLabel('View to embed').locator(`option[value="${savedView.id}"]`).waitFor({ state: 'attached' });
    await viewPicker.getByLabel('View to embed').selectOption(savedView.id);
    await capture('insert-view', viewPicker, theme, 'Studio collection / Studio list selected; floating picker');
    await viewPicker.getByRole('button', { name: 'Insert saved view' }).click();
    await pane().locator('.collection-view-node').waitFor();
    await pane().locator('.title-input').fill('Product brief — draft');
    await actions.getByRole('button', { name: /^Save / }).waitFor();
    await capture('note-header-dirty', pane().locator('.editor-bar'), theme, 'Unsaved changes, tinted Save and white shortcut');
    await pane().getByRole('button', { name: 'Close peek', exact: true }).focus();
    await page.keyboard.press('Enter');
    const guard = page.getByRole('dialog', { name: 'You have unsaved changes', exact: true });
    await capture('save-guard', guard, theme, 'Default Cancel focus, neutral secondary capsules and tinted confirmation');
    await guard.getByRole('button', { name: 'Discard', exact: true }).click();
    await nav.getByRole('button', { name: 'Home', exact: true }).click();
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await capture('settings', page.getByRole('dialog', { name: 'Settings', exact: true }), theme, 'Preferences, shared popup material and compact Search');
    await page.getByRole('button', { name:'Custom', exact:true }).click();
    await page.locator('#primary-color-hex').waitFor();
    await capture('settings-color', page.getByRole('dialog', {name:'Settings', exact:true}), theme, 'Custom color controls, theme and Apply');
    await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'General',exact:true}).click();
    // The workspace is temporary, but its OS path must not enter the atlas.
    await page.locator('.settings-path').evaluate(el => { el.textContent = '/synthetic/noteka-workspace'; el.title = '/synthetic/noteka-workspace'; });
    await capture('settings-workspace', page.getByRole('dialog',{name:'Settings',exact:true}), theme, 'Workspace controls, including Change workspace');
    await page.keyboard.press('Escape');
    await search();
    assert(await page.getByRole('combobox', { name: 'Search workspace' }).getAttribute('placeholder') === 'Search', 'Search placeholder must match sidebar');
    await capture('search', page.getByRole('dialog', { name: 'Search workspace', exact: true }), theme, 'Shared popup material over actual Home content');
    await page.getByRole('button', { name: 'Advanced search' }).click();
    const scope = page.getByLabel('Search scope', { exact: true });
    for (const [value, name] of [['notes', 'advanced-search'], ['attachments', 'attachment-search'], ['combined', 'combined-search']]) {
      await scope.selectOption(value);
      const area = page.locator('.workspace-tab-panel.active .entity-list');
      const input = area.locator(value === 'notes' ? '[aria-label="Search text"]' : value === 'attachments' ? '[aria-label="Attachment text"]' : '[aria-label="Combined text"]');
      await input.waitFor();
      await capture(name, area, theme, `Advanced search / ${value}; empty query`);
    }
    await scope.selectOption('notes');
    await page.setViewportSize({width:420,height:740});
    const narrowSearch = page.locator('.workspace-tab-panel.active .entity-list');
    await narrowSearch.getByLabel('Search text').waitFor();
    assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Narrow advanced search must not overflow globally');
    await capture('advanced-search-narrow', narrowSearch, theme, '420px, notes and entities scope');
    await page.setViewportSize({width:1440,height:900});
    await page.emulateMedia({contrast:'more',reducedMotion:'reduce'});
    await search();
    const solid = page.getByRole('dialog',{name:'Search workspace',exact:true});
    assert(await solid.evaluate(el=>getComputedStyle(el).backdropFilter === 'none'), 'High contrast removes popup blur');
    await capture('search-solid', solid, theme, 'High contrast fallback; local search');
    await page.keyboard.press('Escape');
    await page.emulateMedia({contrast:'no-preference',reducedMotion:'reduce'});
  }
  await settings('dark', 'Violet');
  await openNote();
  await pane().locator('.title-input').fill('Product brief — draft');
  await capture('violet-action', pane().locator('.editor-bar'), 'dark', 'Alternate accent; header remains neutral', 'Violet');
  await page.setViewportSize({ width: 420, height: 740 });
  await pane().getByRole('button', { name: 'Close peek', exact: true }).click();
  const guard = page.getByRole('dialog', { name: 'You have unsaved changes', exact: true });
  assert(await guard.getByRole('button', { name: 'Save and continue', exact: true }).isVisible(), 'Narrow confirmation must remain reachable');
  await capture('save-guard-narrow', guard, 'dark', '420px, secondary row and full-width confirmation', 'Violet');
  await guard.getByRole('button', { name: 'Discard', exact: true }).click();
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.emulateMedia({ contrast: 'more', reducedMotion: 'reduce' });
  await search();
  const fallback = page.getByRole('dialog', { name: 'Search workspace', exact: true });
  assert(await fallback.evaluate(el => getComputedStyle(el).backdropFilter === 'none'), 'Solid fallback must remove blur');
  await capture('search-solid-fallback', fallback, 'dark', 'prefers-contrast: more; solid surface, no sheen/filter', 'Violet');
  assert(errors.length === 0 && external.length === 0, 'No page errors or external requests allowed');
  await control('finish', { browser, browserVersion: page.context().browser().version(), captures, errors, externalRequests: external.length, contrast, contrastFailures, contrastSkipped, fallbackChecks, fallbackFailures });
  return { browser, screenshots: captures.length, errors: errors.length, externalRequests: external.length, contrast: contrast.length, contrastFailures, fallbackFailures };
}
