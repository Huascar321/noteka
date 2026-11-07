async (page) => {
  const origin = new URL(page.url()).origin, errors = [], checks = [];
  let stage = 'setup';
  try {
  page.on('pageerror', error => errors.push(String(error)));
  await page.addInitScript(() => {
    let next = 1; window.isTauri = true;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = { transformCallback() { return next++; }, unregisterCallback() {},
      async invoke(command, args = {}) {
        if (command === 'operate') return (await fetch('/__test__/operate', { method:'POST', headers:{'content-type':'application/json'}, body:JSON.stringify(args.request) })).json();
        if (['pending_edits','close_confirmed','plugin:event|listen','plugin:event|unlisten'].includes(command)) return next++;
        throw Error('Unexpected IPC: ' + command);
      } };
  });
  const control = async (step, input = {}) => { const r = await page.request.post(origin + '/__test__/control', { data: { step, input } }); if (!r.ok()) throw Error(await r.text()); return r.json(); };
  const assert = (value, message) => { if (!value) throw Error(message); checks.push(message); };
  const measureContrast = async (locator, label) => {
    await locator.evaluate(el => el.setAttribute('data-contrast-probe', ''));
    const ink = await locator.evaluate(el => getComputedStyle(el).color);
    const png = await locator.screenshot({ animations:'disabled', style:'[data-contrast-probe], [data-contrast-probe] * { color:transparent !important; text-shadow:none !important; } [data-contrast-probe] svg { visibility:hidden !important; }' });
    const ratio = await page.evaluate(async ({png,ink}) => {
      const image = new Image(); image.src = 'data:image/png;base64,' + png; await image.decode();
      const canvas = document.createElement('canvas'); canvas.width=image.naturalWidth; canvas.height=image.naturalHeight;
      const ctx=canvas.getContext('2d'); ctx.drawImage(image,0,0);
      const color=document.createElement('canvas').getContext('2d'); color.fillStyle=ink; color.fillRect(0,0,1,1);
      const rgb=[...color.getImageData(0,0,1,1).data].slice(0,3);
      const lum=a=>a.map(v=>v/255).map(v=>v<=.04045?v/12.92:((v+.055)/1.055)**2.4).reduce((v,c,i)=>v+c*[.2126,.7152,.0722][i],0);
      const a=lum(rgb); let worst=Infinity;
      for(let y=Math.ceil(canvas.height*.25);y<canvas.height*.75;y+=2) for(let x=Math.ceil(canvas.height*.55);x<canvas.width-canvas.height*.55;x+=3) {const b=lum([...ctx.getImageData(x,y,1,1).data].slice(0,3));worst=Math.min(worst,(Math.max(a,b)+.05)/(Math.min(a,b)+.05));}
      return worst;
    }, {png:png.toString('base64'),ink});
    await locator.evaluate(el => el.removeAttribute('data-contrast-probe'));
    assert(Number.isFinite(ratio) && ratio >= 4.5, `${label}: composed contrast ${ratio.toFixed(2)}:1`);
  };
  await control('configure', { theme:'light', phase:'main' });
  const info = await control('init'); await page.goto(origin);
  await page.getByLabel('Workspace path').fill(info.path);
  await page.getByRole('button', { name:'Create workspace', exact:true }).click();
  await page.locator('.home').waitFor();
  const project = await control('entity_create', { kind:'project', title:'Synthetic project', payload:{state:'doing'} });
  await control('entity_create', {kind:'person',title:'Synthetic contact',payload:{pipeline_status:'contacted'}});
  await control('entity_create', {kind:'tag',title:'Synthetic tag',payload:{classification:'resource'}});
  const nav = page.getByRole('navigation', { name:'Sections', exact:true });
  const expand = async () => { if (await page.locator('.desktop-sidebar').getAttribute('aria-hidden') === 'true') { await page.getByRole('button', { name:'Expand sidebar' }).click(); await page.waitForFunction(() => document.querySelector('.desktop-sidebar')?.getAttribute('aria-hidden') === 'false'); } };
  const go = async name => { await expand(); await nav.getByRole('button', { name, exact:true }).click(); };
  const database = () => page.locator('.workspace-tab-panel.active .database-panel');
  const openCollections = async () => { await expand(); await page.locator('.desktop-sidebar .collections-shortcut').click(); await database().waitFor(); };
  const saveCollectionForm = async name => {
    const form=database().locator('.collection-buffer'); await form.getByLabel('Collection draft name').fill(name);
    await form.getByRole('button',{name:'Save collection',exact:true}).click(); await form.waitFor({state:'hidden'});
    await page.waitForFunction(() => !document.querySelector('.database-panel [aria-busy="true"]'));
  };
  stage='create collections and view'; await openCollections();
  await database().getByRole('button',{name:'New collection',exact:true}).click(); await saveCollectionForm('Synthetic collection A');
  await database().getByRole('heading',{name:'Synthetic collection A',exact:true}).waitFor();
  await database().getByRole('button',{name:'New view',exact:true}).click(); await saveCollectionForm('Synthetic saved view');
  assert((await database().getByLabel('Collection view').locator('option:checked').textContent()).includes('Synthetic saved view'),'Create and select a saved collection view');
  await database().getByRole('button',{name:'New collection',exact:true}).click(); await saveCollectionForm('Synthetic collection B');
  await database().getByRole('button',{name:'Synthetic collection A',exact:true}).click();
  await page.waitForFunction(() => document.querySelector('.database-panel .collection-tabs button.active')?.textContent.trim()==='Synthetic collection A');
  assert(await database().getByRole('heading',{name:'Synthetic collection A',exact:true}).isVisible(),'Create and switch collections through UI');
  for (const theme of ['light','dark']) for (const accent of ['Blue','Violet']) {
    await expand(); await page.getByRole('button', { name:'Settings', exact:true }).click();
    await page.getByLabel('Theme', {exact:true}).selectOption(theme); await page.getByRole('button', {name:accent, exact:true}).click();
    await page.getByRole('button', {name:'Close Settings', exact:true}).click();
    for (const width of [1440,900,420]) {
      await page.setViewportSize({width, height:900});
      await page.waitForTimeout(300); // Esperar el evento matchMedia del lateral tras redimensionar.
      for (const section of ['People','Tags','Projects','Tasks','Notes','Goals']) {
        stage = `${theme}/${accent}/${width}/${section}`; await go(section);
        const kind = {People:'person',Tags:'tag',Projects:'project',Tasks:'task',Notes:'note',Goals:'goals'}[section];
        const root = page.locator(`.workspace-tab-panel.active [data-home-section="${kind}"]`);
        await root.waitFor(); await page.waitForFunction(el => el.getAttribute('aria-busy') === 'false', await root.elementHandle());
        assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `${theme}/${accent}/${width}/${section}: no global overflow`);
        const create = root.locator('.task-view-actions').getByRole('button', {name:'New',exact:true});
        assert(await create.count() === 1, `${section}: one toolbar New`);
        const icon = create.locator('svg[data-app-icon="plus"]');
        assert(await icon.count() === 1 && await icon.getAttribute('width') === '16' && await icon.getAttribute('height') === '16', `${section}: New has one 16px plus and accessible name`);
        const inside = await create.evaluate(el => { const r=el.getBoundingClientRect(); return r.left>=0 && r.right<=innerWidth && r.top>=0 && r.bottom<=innerHeight; });
        assert(inside, `${theme}/${accent}/${width}/${section}: New fits viewport`);
        if (section === 'People' || section === 'Tags') {
          assert(await root.locator('.task-capture').count() === 0, `${section}: no duplicate capture`);
          await page.mouse.move(0,0); await measureContrast(create, `${theme}/${accent}/${width}/${section} New rest`);
          await create.hover(); await measureContrast(create, `${theme}/${accent}/${width}/${section} New hover`);
        }
        if (section === 'Goals') {
          const milestone=page.locator('.workspace-tab-panel.active .milestones-section .task-new'); await milestone.scrollIntoViewIfNeeded();
          assert(await milestone.getAttribute('aria-label') === null && (await milestone.innerText()).trim() === 'New', 'Milestones retains accessible New text');
          assert(await milestone.locator('svg[data-app-icon="plus"][width="16"][height="16"]').count()===1, 'Milestones New has 16px plus');
          assert(await milestone.evaluate(el=>{const r=el.getBoundingClientRect();return r.left>=0&&r.right<=innerWidth;}),'Milestones New fits horizontally');
        }
      }
      stage = `${theme}/${accent}/${width}/Collections`; await openCollections();
      await database().getByRole('button',{name:'Synthetic collection A',exact:true}).click();
      await page.waitForFunction(() => document.querySelector('.database-panel .collection-tabs button.active')?.textContent.trim()==='Synthetic collection A');
      const views=database().getByLabel('Collection view'); await views.selectOption({label:'Synthetic saved view · table'});
      assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `${theme}/${accent}/${width}/Collections: no global overflow`);
      const host=database().locator('.collection-tabs-host');
      assert(await host.evaluate(el => !!el.querySelector(':scope > .sliding-tabs-indicator') && !el.querySelector('.collection-tabs > .sliding-tabs-indicator')), 'Collections indicator is outside scroller');
      for (const name of ['New collection','New template','New record','New view']) {
        const button=database().getByRole('button',{name,exact:true});
        assert(await button.locator('svg[data-app-icon="plus"][width="16"][height="16"]').count()===1, `Collections ${name}: 16px plus`);
        assert(await button.evaluate(el=>{const r=el.getBoundingClientRect();return r.left>=0&&r.right<=innerWidth;}),`Collections ${name}: accessible horizontally`);
      }
    }
  }
  stage = 'contextual creation and alignment'; await page.setViewportSize({width:1440,height:900}); await page.waitForTimeout(300);
  await go('People'); let peopleRoot=page.locator('.workspace-tab-panel.active .home-people');
  const peopleY=await peopleRoot.locator('.task-view-toolbar').evaluate(el=>el.getBoundingClientRect().top);
  await peopleRoot.getByRole('group',{name:'People views'}).getByRole('button',{name:'Pipeline',exact:true}).click();
  const personGroupNew=peopleRoot.locator('.people-pipeline-column[data-pipeline="contacted"] .task-add-row');
  await personGroupNew.waitFor();
  assert(await personGroupNew.locator('svg[data-app-icon="plus"][width="16"][height="16"]').count()===1,'Person group New page has 16px plus');
  await personGroupNew.click();
  let detail=page.locator('.workspace-tab-panel.peek:not(.peek-hidden):visible').last(); await detail.locator('.title-input').waitFor();
  if (!await detail.locator('.page-details').evaluate(el=>el.open)) await detail.locator('.page-details summary').click();
  assert(await detail.getByLabel('Person pipeline').inputValue()==='contacted','Person group creates draft with contextual pipeline');
  await detail.getByRole('button',{name:'Close peek'}).click(); await page.getByRole('dialog',{name:'You have unsaved changes'}).getByRole('button',{name:'Discard',exact:true}).click();
  await go('Tags'); const tags=page.locator('.workspace-tab-panel.active .home-tags');
  const tagsY=await tags.locator('.task-view-toolbar').evaluate(el=>el.getBoundingClientRect().top);
  assert(Math.abs(peopleY-tagsY)<=1,'People and Tags toolbars share vertical alignment');
  await tags.getByRole('group',{name:'Tags views'}).getByRole('button',{name:'Types',exact:true}).click();
  const tagGroupNew=tags.locator('.tag-group[data-classification="resource"] .task-add-row');
  await tagGroupNew.waitFor();
  assert(await tagGroupNew.locator('svg[data-app-icon="plus"][width="16"][height="16"]').count()===1,'Tag group New page has 16px plus');
  await tagGroupNew.click();
  detail=page.locator('.workspace-tab-panel.peek:not(.peek-hidden):visible').last(); await detail.locator('.title-input').waitFor();
  assert(await detail.getByLabel('Classification').inputValue()==='resource','Tag group creates draft with contextual classification');
  await detail.getByRole('button',{name:'Close peek'}).click(); await page.getByRole('dialog',{name:'You have unsaved changes'}).getByRole('button',{name:'Discard',exact:true}).click();
  stage = 'calendar'; await page.setViewportSize({width:1440,height:900}); await page.waitForTimeout(300); await go('People');
  const people = page.locator('.workspace-tab-panel.active .home-people');
  await people.getByRole('group', {name:'People views'}).getByRole('button', {name:'Birthdays',exact:true}).click();
  await people.locator('.people-calendar').waitFor();
  const month = people.locator('.people-calendar-toolbar h3'); const initial = await month.textContent();
  await people.getByRole('button',{name:'Next birthday month'}).click(); await page.waitForFunction(([el, initial]) => el.textContent !== initial, [await month.elementHandle(),initial]);
  await people.getByRole('button',{name:'Previous birthday month'}).click(); await page.waitForFunction(([el, initial]) => el.textContent === initial, [await month.elementHandle(),initial]);
  await people.getByRole('button',{name:'Today',exact:true}).click(); assert(await month.textContent() === initial, 'Birthday calendar previous/next/Today');
  stage = 'move'; await go('Projects'); const projects = page.locator('.workspace-tab-panel.active .home-projects');
  await projects.getByRole('group',{name:'Projects views'}).getByRole('button',{name:'Board',exact:true}).click();
  const trigger = projects.getByRole('button',{name:'Move Synthetic project',exact:true}); await trigger.focus(); await page.keyboard.press('Enter');
  const dialog = page.getByRole('dialog',{name:'Move Synthetic project',exact:true});
  await page.emulateMedia({contrast:'more',reducedMotion:'reduce'});
  assert(await dialog.evaluate(el => getComputedStyle(el).backdropFilter === 'none'), 'Move high contrast removes blur');
  assert(await dialog.evaluate(el => getComputedStyle(el).backgroundImage === 'none' && getComputedStyle(el).boxShadow === 'none' && getComputedStyle(el,'::before').content === 'none'), 'Move high contrast uses solid material');
  await page.emulateMedia({contrast:'no-preference',reducedMotion:'reduce'});
  // Playwright exposes contrast but not prefers-reduced-transparency in WebKit.
  // Activate only the existing matching CSS rules in place, preserving cascade order;
  // this checks their computed result, not native OS preference delivery.
  const forced = await page.evaluate(() => {
    const saved=[];
    const visit=rules=>{ for(const rule of rules) { if(rule instanceof CSSMediaRule && rule.conditionText.includes('prefers-reduced-transparency')) { saved.push([rule,rule.media.mediaText]); rule.media.mediaText='all'; } else if(rule.cssRules) visit(rule.cssRules); } };
    for(const sheet of document.styleSheets) try {visit(sheet.cssRules);} catch {}
    window.__restoreTransparency=()=>saved.forEach(([rule,value])=>{rule.media.mediaText=value;}); return saved.length;
  });
  assert(forced>0, 'Reduced-transparency fallback rules were exercised');
  assert(await dialog.evaluate(el=>getComputedStyle(el).backdropFilter==='none' && getComputedStyle(el).backgroundImage==='none' && getComputedStyle(el).boxShadow==='none'), 'Reduced-transparency rules make Move solid');
  assert(await dialog.getByRole('button',{name:'Cancel',exact:true}).evaluate(el=>getComputedStyle(el).backgroundImage==='none' && getComputedStyle(el).boxShadow==='none'), 'Reduced-transparency rules make secondary action solid');
  await page.evaluate(()=>window.__restoreTransparency());
  if (page.context().browser().browserType().name() === 'chromium') {
    const cdp=await page.context().newCDPSession(page);
    await cdp.send('Emulation.setEmulatedMedia',{features:[{name:'prefers-reduced-transparency',value:'reduce'}]});
    if (await page.evaluate(()=>matchMedia('(prefers-reduced-transparency: reduce)').matches)) {
      assert(await dialog.evaluate(el=>getComputedStyle(el).backdropFilter==='none' && getComputedStyle(el).backgroundImage==='none'), 'Chromium native emulated reduced transparency activates solid popup');
    } else checks.push('Chromium does not expose native reduced-transparency emulation; CSS rules checked in place');
    await cdp.send('Emulation.setEmulatedMedia',{features:[]}); await cdp.detach();
  }
  await dialog.getByRole('button',{name:'Cancel',exact:true}).click();
  assert(await trigger.evaluate(el => document.activeElement === el), 'Move Cancel restores trigger focus');
  assert((await control('get',{id:project.id})).payload.state === 'doing', 'Move Cancel preserves project');
  await page.keyboard.press('Enter'); await page.keyboard.press('Escape');
  assert(await trigger.evaluate(el => document.activeElement === el), 'Move Escape restores trigger focus');
  await page.keyboard.press('Enter'); await dialog.getByLabel('Estado destino').selectOption('planned'); await dialog.getByRole('button',{name:'Move',exact:true}).click();
  await page.waitForFunction(() => !document.querySelector('.project-move-dialog'));
  assert((await control('get',{id:project.id})).payload.state === 'planned', 'Move persists destination');
  stage = 'peek menus and guard';
  const note = await control('entity_create', {kind:'note',title:'Synthetic detail',document:{schema_version:2,blocks:Array.from({length:24},(_,i)=>({id:`p${i}`,type:'paragraph',content:[{type:'text',text:`Synthetic paragraph ${i}`}]}))}});
  await expand(); await page.locator('.desktop-sidebar .sidebar-search').click();
  await page.getByRole('combobox',{name:'Search workspace'}).fill(note.title);
  await page.getByRole('option').filter({has:page.locator('strong',{hasText:note.title})}).first().click();
  const pane = () => page.locator('.workspace-tab-panel.peek:not(.peek-hidden):visible').last();
  await pane().locator('.title-input').waitFor();
  for (const [name,klass] of [['Side peek','peek-side'],['Center peek','peek-center'],['Full page','peek-full']]) {
    await pane().getByRole('button',{name:'Switch peek mode'}).click();
    await page.getByRole('menuitemradio',{name,exact:true}).click();
    await page.locator(`.${klass} .title-input`).waitFor();
    await pane().locator('.editor').evaluate(el=>{el.scrollTop=250;});
    assert(await pane().locator('.editor').evaluate(el=>el.scrollTop>0), `${name}: content actually scrolled`);
    await pane().getByRole('button',{name:'Page actions',exact:true}).click();
    const menu=pane().getByRole('menu',{name:'Page actions',exact:true}); await menu.waitFor();
    const within=await menu.evaluate(el=>{const r=el.getBoundingClientRect();return r.left>=0&&r.top>=0&&r.right<=innerWidth+1&&r.bottom<=innerHeight+1;});
    assert(within, `${name}: actions menu remains within viewport after scroll`);
    const anchored = await menu.evaluate(el => {
      const menu=el.getBoundingClientRect(), host=el.closest('.page-tools-header').getBoundingClientRect();
      const expected=Math.max(12,Math.min(host.right-menu.width,innerWidth-menu.width-12));
      return Math.abs(menu.left-expected)<2 && (menu.top>=host.bottom+4 || menu.bottom<=host.top-4);
    });
    assert(anchored, `${name}: actions menu stays anchored to trigger group`);
    await page.keyboard.press('Escape');
    assert(await pane().getByRole('button',{name:'Page actions',exact:true}).evaluate(el=>document.activeElement===el), `${name}: Escape restores menu focus`);
  }
  await pane().locator('.title-input').fill('Synthetic detail draft');
  await pane().getByRole('button',{name:'Close peek'}).click();
  let guard=page.getByRole('dialog',{name:'You have unsaved changes'}); await guard.getByRole('button',{name:'Cancel',exact:true}).click();
  assert(await pane().locator('.title-input').inputValue()==='Synthetic detail draft','Guard Cancel keeps draft');
  await pane().getByRole('button',{name:'Close peek'}).click(); guard=page.getByRole('dialog',{name:'You have unsaved changes'});
  await guard.getByRole('button',{name:'Save and continue'}).click();
  await guard.waitFor({state:'hidden'});
  assert((await control('get',{id:note.id})).title === 'Synthetic detail draft','Guard Save persists draft');
  stage='compact relations and fields';
  const longTitle='Synthetic long project ' + 'A'.repeat(120);
  const longProject=await control('entity_create',{kind:'project',title:longTitle,payload:{state:'doing'}});
  let relatedTask=await control('entity_create',{kind:'task',title:'Synthetic relation layout',payload:{state:'todo'}});
  await control('relation_add',{source_id:relatedTask.id,target_id:longProject.id,kind:'project',expected_revision:relatedTask.revision});
  await page.setViewportSize({width:420,height:740}); await page.waitForTimeout(300); await expand();
  await page.locator('.desktop-sidebar .sidebar-search').click();
  await page.getByRole('combobox',{name:'Search workspace'}).fill(relatedTask.title);
  await page.getByRole('option').filter({has:page.locator('strong',{hasText:relatedTask.title})}).first().click();
  await pane().locator('.task-detail-project').waitFor();
  const relation=pane().locator('.task-project-value'), projectButton=relation.locator('.task-detail-project');
  assert(await projectButton.getAttribute('title')===longTitle,'Long project retains its tooltip');
  assert(await projectButton.locator('span').evaluate(el=>getComputedStyle(el).textOverflow==='ellipsis' && el.scrollWidth>el.clientWidth),'Long project name truncates');
  assert(await relation.evaluate(el=>el.getBoundingClientRect().height<=33),'A single project fits one 32px row');
  assert(await relation.evaluate(el=>{const a=el.querySelector('.compact-relation-list').getBoundingClientRect(),b=el.querySelector('.task-project-edit').getBoundingClientRect();return b.left>=a.right-1 && b.top<a.bottom;}),'Relation edit arrow stays outside wrapped list');
  for (const title of ['Synthetic second project','Synthetic third project']) {
    const p=await control('entity_create',{kind:'project',title,payload:{state:'planned'}});
    relatedTask=await control('get',{id:relatedTask.id});
    await control('relation_add',{source_id:relatedTask.id,target_id:p.id,kind:'project',expected_revision:relatedTask.revision});
  }
  await pane().getByRole('button',{name:'Refresh page',exact:true}).click();
  await page.waitForFunction(()=>document.querySelectorAll('.peek:not(.peek-hidden) .task-detail-project').length===3);
  assert(await relation.evaluate(el=>el.getBoundingClientRect().height>33),'Multiple projects wrap inside relation field');
  assert(await pane().getByLabel('Task status').evaluate(el=>el.tagName==='SELECT') && await pane().getByLabel('Due date').evaluate(el=>el.tagName==='BUTTON'),'Status stays native and Due opens the local calendar');
  await pane().getByLabel('Task status').selectOption('doing');
  await pane().getByLabel('Due date').click();
  const dueCalendar=page.locator('.date-picker'); await dueCalendar.waitFor();
  for(let i=0;i<24&&(await dueCalendar.locator('.date-picker-title').textContent()).trim()!=='December 2026';i++) await dueCalendar.getByRole('button',{name:'Next month'}).click();
  await dueCalendar.getByRole('button',{name:'December 25, 2026'}).click();
  await pane().getByRole('button',{name:/^Save/}).click();
  await page.waitForFunction(()=>!document.querySelector('.peek:not(.peek-hidden) .save-status.warning'));
  const savedTask=await control('get',{id:relatedTask.id});
  assert(savedTask.payload.state==='doing' && savedTask.payload.due_date==='2026-12-25','Edited status and Due save through existing workflow');
  await pane().getByRole('button',{name:'Close peek'}).click(); await page.setViewportSize({width:1440,height:900}); await page.waitForTimeout(300);
  const openFixture=async title=>{await expand();await page.locator('.desktop-sidebar .sidebar-search').click();await page.getByRole('combobox',{name:'Search workspace'}).fill(title);await page.getByRole('option').filter({has:page.locator('strong',{hasText:title})}).first().click();await pane().locator('.title-input').waitFor();};
  stage='New icons in detail headings';
  for (const [title,names] of [['Synthetic project',['New task in this project','New note in this project']],['Synthetic tag',['New note in this tag','New clip in this tag']],['Synthetic contact',['Capture a note here','Capture a task here']]]) {
    await openFixture(title);
    for(const name of names){const button=pane().getByRole('button',{name,exact:true});await button.waitFor();assert(await button.locator('svg[data-app-icon="plus"][width="16"][height="16"]').count()===1, `${name}: 16px plus`);}
    await pane().getByRole('button',{name:'Close peek'}).click();
  }
  stage='Goal tag relations';
  const goal=await control('entity_create',{kind:'note',title:'Synthetic relation goal',payload:{inbox:false,goal:{state:'active'}}});
  const longTagTitle='Synthetic long tag '+ 'B'.repeat(120);
  const longTag=await control('entity_create',{kind:'tag',title:longTagTitle,payload:{classification:'topic'}});
  await control('relation_add',{source_id:goal.id,target_id:longTag.id,kind:'tag',expected_revision:goal.revision});
  await page.setViewportSize({width:420,height:740});await page.waitForTimeout(300);await openFixture(goal.title);
  const goalTags=pane().locator('.goal-tag-value'),tagButton=goalTags.locator('.goal-detail-tag');await tagButton.waitFor();
  assert(await tagButton.getAttribute('title')===longTagTitle,'Long Goal tag retains tooltip');
  assert(await tagButton.locator('span').evaluate(el=>getComputedStyle(el).textOverflow==='ellipsis'&&el.scrollWidth>el.clientWidth),'Long Goal tag truncates');
  assert(await goalTags.evaluate(el=>el.getBoundingClientRect().height<=33),'Single Goal tag fits one 32px row');
  assert(await goalTags.evaluate(el=>{const a=el.querySelector('.compact-relation-list').getBoundingClientRect(),b=el.querySelector('.icon-button').getBoundingClientRect();return b.left>=a.right-1&&b.top<a.bottom;}),'Goal tag arrow stays outside relation list');
  const anotherTag=await control('entity_create',{kind:'tag',title:'Synthetic another tag',payload:{classification:'area'}}),freshGoal=await control('get',{id:goal.id});
  const denied=await page.request.post(origin+'/__test__/operate',{data:{workspace:info.path,command:{operation:'relation.add',input:{source_id:goal.id,target_id:anotherTag.id,kind:'tag',expected_revision:freshGoal.revision}}}});
  const denial=await denied.json();assert(denial.ok===false && denial.error?.code==='validation_error','Goal preserves its single direct tag rule');
  assert(await goalTags.locator('.goal-detail-tag').count()===1,'Rejected second Goal tag leaves existing relation intact');
  const entry=pane().getByRole('button',{name:'New entry',exact:true});await entry.waitFor();assert(await entry.locator('svg[data-app-icon="plus"][width="16"][height="16"]').count()===1,'New entry has 16px plus');
  assert(!errors.length, 'No page errors: ' + errors.join('; '));
  await control('finish', {ok:true,checks}); return {ok:true,count:checks.length};
  } catch (error) { throw Error(`${stage}: ${error.stack || error}`); }
}
