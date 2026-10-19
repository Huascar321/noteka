// Se ejecuta en WKWebView nativo; CLI real, SQLite real y sin red. No sustituye el recorrido anterior.
window.notekaHomeSmoke = async (h, stage) => {
  const { control, assert, wait, click, fill, change, delay, allEntities, openListed, info } = h;
  const panel = kind => document.querySelector(`[data-home-section="${kind}"]`);
  const rowIds = kind => [...new Set([...panel(kind).querySelectorAll('[data-entity-id]')].map(r => r.dataset.entityId))];
  const ready = kind => panel(kind)?.querySelector('.home-coverage');
  const settled = () => ['task','note','project','tag','person'].every(ready);
  const home = async () => { await click('Home'); await wait(settled, 'cinco lecturas Home'); };
  const within = async (scope, label) => {
    const find = () => [...scope.querySelectorAll('button')].find(b => b.textContent.trim() === label || b.getAttribute('aria-label') === label);
    await wait(() => find() && !find().disabled, 'Home control ' + label); find().click(); await delay();
  };
  const view = async (kind, label) => { await within(panel(kind).querySelector('.home-tabs'), label); await wait(() => ready(kind) && [...panel(kind).querySelectorAll('.home-tabs button')].some(b => b.textContent === label && b.getAttribute('aria-pressed') === 'true'), 'vista ' + label); };
  const create = (kind, title, payload = {}) => control('entity_create', { kind, title, payload });
  const link = async (source, target, kind) => control('relation_add', { source_id: source.id, target_id: target.id, kind, expected_revision: (await control('get', { id: source.id })).revision });
  const openRow = async (kind, id) => { await wait(() => panel(kind)?.querySelector(`[data-entity-id="${id}"] .home-open`), 'fila Home'); panel(kind).querySelector(`[data-entity-id="${id}"] .home-open`).click(); await wait(() => !document.querySelector('.home') && document.querySelector('.title-input')?.disabled === false, 'original desde Home cargado'); };
  const saved = () => document.querySelector('.save-status')?.textContent === 'Guardado';
  const layouts = async () => {
    for (const [width,height] of [[1920,845],[1280,800],[900,800]]) {
      await control('viewport',{width,height});await delay();
      // macOS puede incluir el título en el frame: corregir con dimensiones reales del WebView.
      if(innerWidth!==width||innerHeight!==height) {await control('viewport',{width:width+(width-innerWidth),height:height+(height-innerHeight)});await delay();}
      await wait(()=>innerWidth===width&&innerHeight===height,`viewport ${width}×${height} (actual ${innerWidth}×${innerHeight}; screen ${screen.availWidth}×${screen.availHeight})`);
      await control('focus_host');await wait(()=>document.hasFocus(),'foco nativo tras resize');
      await wait(settled,'lecturas Home tras activación de resize');
      assert(document.documentElement.scrollWidth<=innerWidth && document.querySelector('.home').clientWidth>0,`Tema ${info.theme}: ${width}×${height} sin overflow de shell`);
      let el;
      for(let attempt=0;attempt<4;attempt++) {
        if(!document.hasFocus()) {await control('focus_host');await wait(()=>document.hasFocus(),'activación nativa antes de foco de tarjeta');}
        await wait(settled,'datos vigentes antes de foco de tarjeta');
        el=panel('person').querySelector('.home-open');el.scrollIntoView({block:'center'});el.focus();await delay();
        if(document.hasFocus() && document.activeElement===el && el.matches(':focus')) break;
      }
      assert(document.activeElement===el && el.matches(':focus') && getComputedStyle(el).outlineStyle!=='none',`Tema ${info.theme}: foco/nombre de tarjeta visible a ${width}`);
      const rect=el.getBoundingClientRect();assert(rect.left>=0&&rect.right<=innerWidth&&el.getAttribute('aria-label'),`Tema ${info.theme}: título largo/control visible a ${width}`);
      el.blur();document.querySelector('.home').scrollTop=0;await delay();
      await wait(()=>document.querySelector('.home-cover').getBoundingClientRect().top>=0,'portada visible antes de captura');
      const capture=await control('capture',{name:`home-${info.theme}-${width}`});assert(capture.includes('.png'),`Captura sintética nativa ${info.theme} ${width}`);
    }
    const css=getComputedStyle(document.documentElement);
    const luminance=hex=>{let color=hex.trim().slice(1);if(color.length===3)color=[...color].map(c=>c+c).join('');if(!/^[a-f0-9]{6}$/i.test(color))throw new Error('Token de contraste no soportado: '+hex);const c=color.match(/../g).map(v=>parseInt(v,16)/255).map(v=>v<=.04045?v/12.92:((v+.055)/1.055)**2.4);return .2126*c[0]+.7152*c[1]+.0722*c[2];};
    const contrast=(a,b)=>{const x=luminance(css.getPropertyValue(a)),y=luminance(css.getPropertyValue(b));return (Math.max(x,y)+.05)/(Math.min(x,y)+.05);};
    for(const [fg,bg,min] of [['--text','--bg',4.5],['--muted','--surface',4.5],['--section-text','--section-bg',4.5],['--error','--error-bg',4.5],['--warning','--warning-bg',4.5],['--focus','--control',3],['--border','--control',3]]) assert(contrast(fg,bg)>=min,`Tema ${info.theme}: contraste ${fg}/${bg} >=${min}:1`);
    await view('project','Board');panel('project').scrollIntoView({block:'start'});await delay();await control('capture',{name:`home-${info.theme}-board`});
  };
  if(stage==='layout') {await wait(settled,'Home layouts');await change('[aria-label="Tema"]',info.theme);await layouts();return;}
  if (stage === 'initial') {
    await wait(settled, 'Home vacío'); await change('[aria-label="Tema"]', info.theme);
    assert(document.querySelector('.shell.home-shell') && !document.querySelector('.entity-list'), 'Home inicial usa ancho restante sin lista lateral vacía');
    assert([...document.querySelectorAll('.home-section')].map(s => s.getAttribute('aria-label')).join(',') === 'Tasks,Notes,Projects,Tags,Goals,People', 'Orden de las seis secciones y portada local');
    assert(document.querySelector('.home-weather').textContent.includes('no se realizan consultas remotas') && document.querySelector('[aria-label="Goals"]').textContent.includes('pendiente'), 'Clima/Goals pendientes explícitos, no datos simulados');
    assert(['task','note','project','tag','person'].every(k => rowIds(k).length === 0) && !document.querySelector('.home img'), 'Vacío offline sin registros ni previews inventados');
    return;
  }
  if (stage === 'reopen') {
    await wait(settled, 'Home reabierto'); assert(document.documentElement.dataset.theme === info.theme, 'Reapertura/restauración conserva tema local fuera del workspace');
    const contact = (await allEntities()).find(e => e.title === 'Home persona original');
    assert(!!panel('person').querySelector(`[data-entity-id="${contact.id}"]`) && !panel('person').querySelector('img'), 'Home reabierto consulta misma persona con galería textual offline');
    await view('task', 'All');
    const task = (await allEntities()).find(e => e.title === 'Home tarea original');
    while(!panel('task').querySelector(`[data-entity-id="${task.id}"]`)) {
      const coverage=panel('task').querySelector('.home-coverage').textContent;
      await within(panel('task'),'Siguiente Tasks');await wait(()=>ready('task')&&panel('task').querySelector('.home-coverage').textContent!==coverage,'página de tarea reabierta');
    }
    assert(panel('task').querySelectorAll(`[data-entity-id="${task.id}"]`).length === 2, 'Reapertura/restauración conserva dos referencias de tarea original');
    for (const v of ['tasks_today','tasks_inbox','tasks_all','tasks_done','notes_all','notes_inbox','projects_active','projects_planned','projects_board','tags_recent','tags_alphabetical','tags_types','people_contacts','people_new','people_updated']) assert((await control('home', { view: v })).view === v, 'Home CLI reabierto/restaurado: ' + v);
    return;
  }
  // Guardias de entidades y colecciones al dirigirse específicamente a Home.
  await click('Notes'); await openListed('Nota integrada');
  fill('[aria-label="Título de la nota"]', 'Buffer antes de Home'); await delay(); await click('Volver a Home'); await click('Cancelar');
  assert(document.querySelector('.title-input').value === 'Buffer antes de Home' && !document.querySelector('.home'), 'Cancelar llegada a Home mantiene buffer y editor');
  const before = (await allEntities()).find(e => e.title === 'Nota integrada');
  await control('failure_on'); await click('Volver a Home'); await click('Guardar y continuar');
  await wait(() => document.querySelector('.save-status')?.textContent.includes('Error'), 'fallo de guardia Home');
  assert(!document.querySelector('.home') && document.querySelector('.title-input').value === 'Buffer antes de Home', 'Fallo de guardar hacia Home no monta sustituto ni pierde buffer');
  await control('failure_off'); await click('Cancelar'); await control('cli_update', { id: before.id, expected_revision: before.revision, title: before.title });
  await click('Volver a Home'); await click('Guardar y continuar'); await wait(() => document.querySelector('.save-status')?.textContent.includes('Conflicto'), 'conflicto Home');
  assert(!document.querySelector('.home') && document.querySelector('.title-input').value === 'Buffer antes de Home' && (await control('get', { id: before.id })).revision === before.revision + 1, 'Conflicto hacia Home conserva revisión capturada sin sobrescribir CLI');
  await click('Cancelar'); await click('Volver a Home'); await click('Descartar'); await wait(settled, 'Home después de descartar');
  await click('Collections'); await wait(() => document.querySelector('.database-panel'), 'colecciones Home'); await click('Colección sintética'); await click('Definir propiedad');
  fill('[aria-label="JSON del buffer de colección"]', '{"name":"Pendiente Home","type":"text"}'); await delay(); await click('Volver a Home'); await click('Cancelar');
  assert(document.querySelector('[aria-label="JSON del buffer de colección"]').value.includes('Pendiente Home'), 'Cancelar Home desde colección conserva JSON y revisión');
  await click('Volver a Home'); await click('Descartar'); await wait(settled, 'Home tras colección');
  const day = (await control('home', { view: 'tasks_today' })).date;
  const project = await create('project', 'Home proyecto Doing', { state: 'doing' });
  const other = await create('project', 'Home proyecto Planned');
  const empty = await create('project', 'Home proyecto sin tareas', { state: 'ongoing' });
  const task = await create('task', 'Home tarea original', { due_date: day, inbox: true });
  const planned = await create('task', 'Home planificada sin vencimiento', { planned_date: day });
  const tag = await create('tag', 'Home tag original', { classification: 'topic' });
  const person = await create('person', 'Home persona original');
  const note = await create('note', 'Home nota original');
  await link(task, project, 'project'); await link(task, other, 'project'); await link(note, tag, 'tag'); await link(note, person, 'person'); await link(person, tag, 'tag');
  await click('Actualizar Home'); await wait(settled, 'Home CLI');
  assert(rowIds('task').includes(task.id) && !rowIds('task').includes(planned.id) && (await control('workflow', { view: 'my_day' })).items.some(e => e.id === planned.id), 'Today y My Day distintos: vencimiento versus planificación');
  assert(panel('project').querySelector(`[data-entity-id="${empty.id}"]`).textContent.includes('Sin tareas') && panel('project').querySelector(`[data-entity-id="${project.id}"]`).textContent.includes('0%'), 'Home diferencia sin tareas de progreso cero real');
  await view('task','All'); assert(panel('task').querySelectorAll(`[data-entity-id="${task.id}"]`).length === 2 && panel('task').textContent.includes('elementos únicos'), 'Mismo ID en dos grupos, cantidades de apariciones explícitas');
  const group = panel('task').querySelector(`[data-group-key="project:${project.id}"]`); group.open = false;
  assert((await control('get', { id: task.id })).title === task.title, 'Plegar grupo no escribe ni copia tarea'); group.open = true;
  const count = await control('entity_count'); await within(group, '+ New page en Home proyecto Doing'); await wait(() => document.querySelector('.title-input'), 'captura Home en proyecto');
  assert(document.querySelector('.metadata').textContent.includes(project.id), 'Captura desde grupo conserva ID contextual');
  await click('Volver a Home'); await click('Descartar'); await wait(settled, 'descarte captura Home'); assert(await control('entity_count') === count, 'Descartar captura Home antes de guardar no escribe');
  await view('task','All'); await within(panel('task').querySelector(`[data-group-key="project:${project.id}"]`), '+ New page en Home proyecto Doing');
  fill('[aria-label="Título de la entidad"]','Home creada contextual'); await click('Guardar'); await wait(saved,'captura contextual Home guardada');
  const captured = (await allEntities()).find(e => e.title === 'Home creada contextual');
  await wait(()=>document.querySelector('.relations')?.textContent.includes(project.title),'lectura de relación tras captura confirmada');
  assert(document.querySelector('.relations').textContent.includes(project.title) && captured.payload.due_date == null && captured.payload.planned_date == null, 'Captura Home única con relación atómica y sin fechas ocultas');
  await click('Volver a Home'); await wait(settled, 'retorno Home'); await within(panel('task'), 'Crear en Tasks');
  assert(document.activeElement.getAttribute('aria-label') === 'Título de la entidad' && !document.querySelector('[aria-label="Vencimiento"]').value && !document.querySelector('[aria-label="Fecha planificada"]').value, 'Today captura con foco real y fechas vacías');
  fill('[aria-label="Título de la entidad"]','Home commit lectura fallida'); await control('home_read', { view: 'tasks_today', failure: true });
  await click('Volver a Home'); await click('Guardar y continuar'); await wait(() => panel('task')?.querySelector('.error') && !document.querySelector('dialog'), 'commit y fallo Home');
  assert((await allEntities()).filter(e => e.title === 'Home commit lectura fallida').length === 1 && !panel('task').querySelector('[data-entity-id]') && !document.querySelector('.title-input'), 'Commit antes del fallo de lectura limpia buffer y no ofrece repetir creación');
  assert(ready('note') && ready('project'), 'Fallo de Tasks no borra secciones independientes'); await control('home_read', {}); await within(panel('task'),'Reintentar Tasks'); await wait(() => ready('task'),'reintento Home');
  await view('project','Board'); assert(panel('project').querySelectorAll('.home-group').length === 5, 'Board Home declara cinco estados incluidos vacíos');
  await within(panel('project').querySelector('[data-group-key="on_hold"]'),'+ New page en On Hold');
  assert(document.querySelector('[aria-label="Estado de proyecto"]').value === 'on_hold', 'Captura Board aplica sólo estado de columna');
  fill('[aria-label="Título de la entidad"]','Home creado On Hold'); await click('Guardar'); await wait(saved,'estado Home guardado');
  assert((await allEntities()).find(e => e.title === 'Home creado On Hold').payload.state === 'on_hold','Estado Home confirmado consultable CLI');
  await click('Volver a Home'); await wait(settled,'Home tag'); await view('tag','Types');
  await within(panel('tag').querySelector('[data-group-key="resource"]'),'+ New page en Resource');
  assert(document.querySelector('[aria-label="Clasificación"]').value === 'resource','Captura Types conserva clasificación tipada');
  await click('Volver a Home'); await click('Descartar'); await wait(settled,'Home persona'); await openRow('person',person.id);
  await wait(()=>document.querySelector('.relations')?.textContent.includes(tag.title),'relación de persona original cargada');
  assert(document.querySelector('.title-input').value === person.title && document.querySelector('.relations').textContent.includes(tag.title),'Tarjeta abre persona original con relación real, no atributos ficticios');
  fill('[aria-label="Título de la entidad"]','Buffer tema Home'); const revision=(await control('get',{id:person.id})).revision;
  await change('[aria-label="Tema"]',info.theme==='dark'?'light':'dark');
  assert(document.querySelector('.title-input').value==='Buffer tema Home' && (await control('get',{id:person.id})).revision===revision && !saved(),'Cambiar tema no escribe ni descarta buffer');
  await change('[aria-label="Tema"]',info.theme); await click('Volver a Home'); await click('Descartar'); await wait(settled,'Home antes de raw');
  await control('entity_raw',{id:captured.id,payload:{state:'future'}}); await click('Actualizar Home'); await wait(settled,'resumen futuro');
  assert(panel('project').querySelector(`[data-entity-id="${project.id}"]`).textContent.includes('Resumen no disponible'),'Metadato futuro no se presenta como progreso válido');
  await control('entity_raw',{id:captured.id,payload:captured.payload}); await control('entity_archive',{id:note.id,expected_revision:(await control('get',{id:note.id})).revision});
  await click('Actualizar Home'); await wait(settled,'nota archivada Home'); assert(!rowIds('note').includes(note.id) && panel('tag').querySelector(`[data-entity-id="${tag.id}"]`).textContent.includes('0 notas'),'Archivo de fuente cambia resumen sin cambiar tag');
  await control('entity_restore',{id:note.id,expected_revision:(await control('get',{id:note.id})).revision});
  await control('cli_update',{id:note.id,expected_revision:(await control('get',{id:note.id})).revision,title:note.title});
  window.dispatchEvent(new Event('focus')); await wait(() => panel('tag')?.querySelector(`[data-entity-id="${tag.id}"]`)?.textContent.includes('1 notas'),'activación actualiza Home');
  assert((await control('get',{id:tag.id})).revision===1,'Refresh de activación no incrementa revisión del tag');
  await control('home_read',{view:'tasks_today',delay:700}); await within(panel('task'),'Actualizar Tasks'); await view('task','All'); await delay(); await delay();
  assert(panel('task').querySelector('[aria-pressed="true"]').textContent==='All' && panel('task').querySelector('.home-group'),'Respuesta retrasada Today no publica sobre All'); await control('home_read',{});
  await control('home_seed',{date:day}); await click('Actualizar Home'); await wait(settled,'corpus Home 51');
  for (const [kind,label,v] of [['task','Today','tasks_today'],['task','Inbox','tasks_inbox'],['task','All','tasks_all'],['task','Done','tasks_done'],['note','Notes','notes_all'],['note','Inbox','notes_inbox'],['project','Active','projects_active'],['project','Planned','projects_planned'],['project','Board','projects_board'],['tag','Recent','tags_recent'],['tag','A–Z','tags_alphabetical'],['tag','Types','tags_types'],['person','Contacts','people_contacts'],['person','New','people_new'],['person','Updated','people_updated']]) {
    if(label==='Done') panel('task').querySelector('.home-tabs details').open=true;
    await view(kind,label);const expected=await control('home',{view:v});const first=rowIds(kind);
    assert(first.length===50 && first.every(id=>expected.items.some(i=>i.entity.id===id)),`${v}: primera página UI/CLI acotada y mismos IDs`);
    await within(panel(kind),'Siguiente '+panel(kind).getAttribute('aria-label'));await wait(()=>ready(kind)&&panel(kind).querySelector('.home-coverage').textContent.includes('desde 51'),'segunda página '+v);
    const second=rowIds(kind);assert(second.length>0 && second.every(id=>!first.includes(id)),`${v}: segunda página reemplazada, sin duplicar ni acumular`);
    await within(panel(kind),'Inicio '+panel(kind).getAttribute('aria-label'));await wait(()=>ready(kind)&&rowIds(kind).length===50,'inicio '+v);
    assert(rowIds(kind).every((id,i)=>id===first[i]),`${v}: inicio conserva orden e identidad`);
  }
  assert(!document.querySelector('.home img') && !document.querySelector('.home iframe') && (await control('workflow',{view:'active_projects',limit:200})).items.some(e=>e.id===other.id),'Galería sin fotos/requests remotos y workflow Active Projects previo preservado');
  await layouts();
  // Desmontaje/workspace: no datos anteriores ni respuesta retrasada.
  await control('home_read',{view:'notes_all',delay:700});await within(panel('note').querySelector('.home-tabs'),'Notes');await click('Cambiar workspace');await wait(()=>document.querySelector('#workspace-path'),'sin workspace');
  await delay();await delay();assert(!document.querySelector('.home')&&!document.querySelector('[data-entity-id]'),'Consulta descartada al cambiar workspace no publica datos viejos');
  await control('home_read',{});fill('#workspace-path',info.path);await click('Abrir workspace');await wait(settled,'volver workspace');
  await click('Notes');await openListed('Nota integrada');
};
