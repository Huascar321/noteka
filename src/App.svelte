<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { isTauri } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { operate, OperationError, setPending, closeConfirmed } from './lib/api';
  import BlockEditor from './lib/BlockEditor.svelte';
  import EntityFields from './lib/EntityFields.svelte';
  import DatabasePanel from './lib/DatabasePanel.svelte';
  import HomePanel from './lib/HomePanel.svelte';
  import AttachmentSearchPanel from './lib/AttachmentSearchPanel.svelte';
  import AttachmentPreparationPanel from './lib/AttachmentPreparationPanel.svelte';
  import AttachmentGenerationPanel from './lib/AttachmentGenerationPanel.svelte';
  import GroupedSearchPanel from './lib/GroupedSearchPanel.svelte';
  import { readTheme, saveTheme, type Theme } from './lib/theme';
  import { pendingJson } from './lib/editor-adapter';
  import type { Entity, EntityKind, WorkspaceInfo, Document, Contract, Relation, ContextLink, WorkflowView, SearchHit } from './lib/contracts.generated';
  type PageEntity = Contract['entities'];
  type PageRelation = Contract['relations'];

  type Section = EntityKind | 'home' | 'archive' | Exclude<WorkflowView, 'context'> | 'areas' | 'resources' | 'search' | 'collections';
  type Draft = { kind: EntityKind; title: string; payload: Record<string, unknown>; document?: Document; context?: ContextLink };
  type Status = 'saved' | 'dirty' | 'saving' | 'error' | 'conflict';
  const sections: { id: Section; label: string; icon: string }[] = [
    { id: 'home', label: 'Home', icon: '⌂' },
    { id: 'collections', label: 'Collections', icon: '▦' },
    { id: 'search', label: 'Search', icon: '⌕' },
    { id: 'inbox', label: 'Inbox', icon: '⇣' }, { id: 'my_day', label: 'My Day', icon: '☀' },
    { id: 'note', label: 'Notes', icon: '▤' }, { id: 'task', label: 'Tasks', icon: '☑' },
    { id: 'project', label: 'Projects', icon: '◈' }, { id: 'tag', label: 'Tags', icon: '#' },
    { id: 'person', label: 'People', icon: '♙' }, { id: 'archive', label: 'Archive', icon: '▣' },
    { id: 'pending', label: 'Pending', icon: '○' }, { id: 'overdue', label: 'Overdue', icon: '!' },
    { id: 'active_projects', label: 'Active Projects', icon: '◈' }, { id: 'areas', label: 'Areas', icon: '#' }, { id: 'resources', label: 'Resources', icon: '#' },
  ];
  let workspace = $state<WorkspaceInfo | null>(null);
  let path = $state('');
  let section = $state<Section>('note');
  let entities = $state<Entity[]>([]);
  let nextCursor = $state<string | null>(null);
  let selected = $state<Entity | null>(null);
  let draft = $state<Draft | null>(null);
  let status = $state<Status>('saved');
  let error = $state('');
  let listError = $state('');
  let loading = $state(false);
  let listGeneration = 0;
  let working = $state(false);
  let remoteRevision = $state<number | null>(null);
  let relationships = $state<{ relation: Relation; other: Entity }[]>([]);
  let guard = $state<(() => Promise<void>) | null>(null);
  let recovery = $state('');
  let titleInput = $state<HTMLInputElement>();
  let editorEpoch = $state(0);
  let editorInitial: Document | null = $state(null);
  let exportPath = $state('');
  let exportMessage = $state('');
  let invalidEditor = $state('');
  let workflowDate = $state('');
  let dashboard = $state<Entity[]>([]);
  let dashboardCursor = $state<string | null>(null);
  let relationKind = $state<'project' | 'tag' | 'person'>('project');
  let relationTarget = $state('');
  let searchQuery = $state('');
  let searchScope = $state<'notes' | 'attachments' | 'combined'>('notes');
  let groupedPanel = $state<{ refreshPanel: () => Promise<void> }>();
  let attachmentPanel = $state<{ refreshPanel: () => Promise<void> }>();
  let searchKind = $state('');
  let searchArchived = $state(false);
  let searchRelated = $state('');
  let searchHits = $state<SearchHit[]>([]);
  let searchMessage = $state('');
  let sourceLocator = $state('');
  let sourceBlock = $state<string | undefined>();
  const createKind = $derived(['note', 'task', 'project', 'tag', 'person'].includes(section) ? section as EntityKind : section === 'areas' || section === 'resources' ? 'tag' : section === 'my_day' || section === 'pending' || section === 'overdue' ? 'task' : section === 'active_projects' ? 'project' : 'note');
  const createLabel: Record<EntityKind, string> = { note: 'Nueva nota', task: 'Nueva tarea', project: 'Nuevo proyecto', tag: 'Nuevo tag', person: 'Nueva persona' };
  let collectionPending = $state(false);
  let homePanel = $state<{ refreshPanel: () => Promise<void> }>();
  let theme = $state<Theme>(readTheme(() => localStorage));
  let themeError = $state('');
  $effect(() => { document.documentElement.dataset.theme = theme; });
  function changeTheme(next: Theme) { theme = next; themeError = saveTheme(next, () => localStorage); }
  let databasePanel = $state<{ savePending: () => Promise<boolean>; discardPending: () => void; refreshPanel: () => Promise<void> }>();
  const dirty = $derived((draft !== null && status !== 'saved') || collectionPending);
  const currentLabel = $derived(sections.find(s => s.id === section)!.label);

  $effect(() => { void setPending(dirty).catch(() => { error = 'No se pudo activar la protección de cierre. No cierres la ventana con cambios pendientes.'; }); });

  function report(e: unknown) { return e instanceof Error ? e.message : 'La operación no pudo completarse.'; }
  async function guarded(action: () => Promise<void>) {
    if (working || status === 'saving') return;
    if (dirty) guard = action; else await action();
  }
  async function resolveGuard(choice: 'save' | 'discard' | 'cancel') {
    if (choice === 'cancel') { guard = null; return; }
    if (choice === 'save' && !(await save())) return;
    const action = guard;
    guard = null;
    if (choice === 'discard') { databasePanel?.discardPending(); draft = null; status = 'saved'; await setPending(false); }
    await action?.();
  }
  function clearSelection() {
    selected = null; draft = null; status = 'saved'; relationships = []; dashboard = []; dashboardCursor = null; remoteRevision = null; error = ''; recovery = ''; exportMessage = ''; sourceLocator = ''; sourceBlock = undefined; invalidEditor = ''; editorInitial = null; editorEpoch++;
  }
  function loadDraft(entity: Entity) {
    const validPayload = entity.payload !== null && typeof entity.payload === 'object' && !Array.isArray(entity.payload);
    draft = validPayload && (entity.kind !== 'note' || entity.document_editable) ? { kind: entity.kind, title: entity.title, payload: JSON.parse(JSON.stringify(entity.payload)) as Record<string, unknown>, ...(entity.kind === 'note' ? { document: JSON.parse(JSON.stringify(entity.document)) as Document } : {}) } : null;
    editorInitial = draft?.document ? JSON.parse(JSON.stringify(draft.document)) as Document : null; editorEpoch++;
  }
  async function selectWorkspace(create: boolean) {
    working = true; clearSelection(); entities = []; workspace = null; listError = '';
    try {
      workspace = await operate<WorkspaceInfo>({ operation: create ? 'workspace.create' : 'workspace.open', input: {} }, path.trim() || undefined);
      path = workspace.path; section = 'home'; await loadList();
    } catch (e) { error = report(e); }
    finally { working = false; }
  }
  async function loadList(cursor?: string) {
    if (!workspace) return;
    const workspacePath = workspace.path;
    const requestedSection = section;
    const generation = ++listGeneration;
    loading = true; listError = '';
    try {
      if (requestedSection === 'collections' || requestedSection === 'home') { entities = []; nextCursor = null; workflowDate = ''; return; }
      if (requestedSection === 'search') {
        if (searchScope !== 'notes') { entities = []; nextCursor = null; workflowDate = ''; return; }
        if (!cursor) searchHits = [];
        const page = searchQuery.trim() ? await operate<Contract['search']>({ operation: 'search.query', input: { query: searchQuery.trim(), kind: searchKind ? searchKind as EntityKind : undefined, archived: searchArchived, related_id: searchRelated.trim() || undefined, limit: 30, cursor } }, workspacePath) : { items: [], next_cursor: null };
        if (generation !== listGeneration || workspace?.path !== workspacePath || section !== requestedSection) return;
        searchHits = cursor ? [...searchHits, ...page.items] : page.items; entities = []; nextCursor = page.next_cursor ?? null; workflowDate = ''; return;
      }
      const workflow = ['inbox', 'my_day', 'pending', 'overdue', 'active_projects'].includes(requestedSection);
      const page = workflow
        ? await operate<Contract['workflow']>({ operation: 'workflow.query', input: { view: requestedSection as WorkflowView, cursor, limit: 50 } }, workspacePath)
        : await operate<PageEntity>({ operation: 'entity.list', input: { archived: requestedSection === 'archive', ...(requestedSection !== 'archive' ? { kind: requestedSection === 'areas' || requestedSection === 'resources' ? 'tag' : requestedSection as EntityKind } : {}), ...(requestedSection === 'areas' || requestedSection === 'resources' ? { classification: requestedSection === 'areas' ? 'area' : 'resource' } : {}), cursor, limit: 50 } }, workspacePath);
      if (generation !== listGeneration || workspace?.path !== workspacePath || section !== requestedSection) return;
      entities = cursor ? [...entities, ...page.items] : page.items; nextCursor = page.next_cursor ?? null;
      workflowDate = 'date' in page ? page.date as string : '';
    } catch (e) { if (generation === listGeneration && workspace?.path === workspacePath && section === requestedSection) listError = report(e); }
    finally { if (generation === listGeneration) loading = false; }
  }
  async function navigate(to: Section) { await guarded(async () => { section = to; clearSelection(); await loadList(); }); }
  function changeSearchScope(scope: 'notes' | 'attachments' | 'combined') {
    listGeneration++; loading = false; listError = ''; nextCursor = null; searchHits = []; searchMessage = '';
    searchScope = scope;
    void loadList();
  }
  async function openSearch(hit: SearchHit) {
    await guarded(async () => {
      if (!workspace) return;
      try {
        const entity = await operate<Entity>({ operation: 'entity.get', input: { id: hit.entity_id } }, workspace.path);
        await openEntity(entity);
        if (selected?.id !== hit.entity_id) return;
        sourceLocator = `Origen: ${hit.entity_id} · bloque ${hit.block_id ?? 'título'} · resultado r${hit.revision}${selected.revision !== hit.revision ? ` · fuente actual r${selected.revision}; buscá otra vez` : ''}`;
        sourceBlock = hit.block_id && selected.revision === hit.revision ? hit.block_id : undefined;
      } catch (e) { error = report(e); }
    });
  }
  async function rebuildSearch() {
    if (!workspace || working) return;
    const workspacePath = workspace.path;
    working = true; searchMessage = '';
    try {
      const index = await operate<Contract['search_index']>({ operation: 'search.rebuild', input: {} }, workspacePath);
      if (workspace?.path !== workspacePath) return;
      searchMessage = `Índice reconstruido: ${index.entities} entidades, ${index.chunks} fuentes. Originales sin cambios.`;
      await loadList();
    } catch (e) { listError = report(e); }
    finally { working = false; }
  }
  async function loadRelations(entity: Entity) {
    if (!workspace) return;
    const workspacePath = workspace.path;
    const all: Relation[] = [];
    let cursor: string | null | undefined;
    do {
      const page = await operate<PageRelation>({ operation: 'relation.list', input: { entity_id: entity.id, cursor, limit: 200 } }, workspacePath);
      all.push(...page.items); cursor = page.next_cursor;
    } while (cursor);
    const resolved = await Promise.all(all.map(async relation => ({ relation, other: await operate<Entity>({ operation: 'entity.get', input: { id: relation.source_id === entity.id ? relation.target_id : relation.source_id } }, workspacePath) })));
    if (workspace?.path === workspacePath && selected?.id === entity.id) relationships = resolved;
  }
  async function loadDashboard(entity: Entity, cursor?: string) {
    if (!workspace || !['project', 'tag', 'person'].includes(entity.kind)) return;
    const workspacePath = workspace.path;
    const page = await operate<Contract['workflow']>({ operation: 'workflow.query', input: { view: 'context', related_id: entity.id, limit: 50, cursor } }, workspacePath);
    if (selected?.id === entity.id && workspace?.path === workspacePath) { dashboard = cursor ? [...dashboard, ...page.items] : page.items; dashboardCursor = page.next_cursor ?? null; }
  }
  async function openEntity(entity: Entity) {
    await guarded(() => openOriginal(entity.id));
  }
  async function openOriginal(id: string) {
      if (!workspace) return;
      working = true;
      try {
        const fresh = await operate<Entity>({ operation: 'entity.get', input: { id } }, workspace.path);
        clearSelection(); selected = fresh;
        loadDraft(fresh);
        await loadRelations(fresh);
        await loadDashboard(fresh);
      } catch (e) { error = report(e); }
      finally { working = false; }
  }
  async function newNote() {
    if (section === 'home') await createHome('note'); else await newEntity('note');
  }
  async function openHome(id: string) {
    await guarded(async () => {
      if (!workspace) return;
      const path = workspace.path;
      working = true;
      try {
        const entity = await operate<Entity>({ operation: 'entity.get', input: { id } }, path);
        if (workspace?.path !== path) return;
        section = entity.kind; await loadList(); await openOriginal(entity.id);
        await tick(); titleInput?.focus();
      } catch (e) { error = report(e); }
      finally { working = false; }
    });
  }
  async function createHome(kind: EntityKind, context?: ContextLink, seed?: Record<string, unknown>) {
    await guarded(async () => {
      if (!workspace) return;
      working = true;
      try { section = kind; await loadList(); }
      finally { working = false; }
      await startDraft(kind, context, seed);
    });
  }
  async function newEntity(kind: EntityKind, context?: ContextLink, seed?: Record<string, unknown>) {
    if (!workspace) return;
    await guarded(() => startDraft(kind, context, seed));
  }
  async function startDraft(kind: EntityKind, context?: ContextLink, seed?: Record<string, unknown>) {
      clearSelection(); draft = { kind, title: '', payload: kind === 'tag' ? { classification: section === 'areas' ? 'area' : section === 'resources' ? 'resource' : 'topic' } : kind === 'task' && section === 'my_day' && workflowDate ? { planned_date: workflowDate } : {}, context, ...(kind === 'note' ? { document: { schema_version: 2, blocks: [{ id: crypto.randomUUID(), type: 'paragraph', content: [] }] } } : {}) }; editorInitial = draft.document ? JSON.parse(JSON.stringify(draft.document)) as Document : null;
      if (seed) draft.payload = { ...draft.payload, ...seed };
      status = 'dirty'; await tick(); titleInput?.focus();
  }
  function edited() { status = remoteRevision ? 'conflict' : 'dirty'; }
  async function save(): Promise<boolean> {
    if (collectionPending) return databasePanel ? await databasePanel.savePending() : false;
    if (!workspace || !draft || status === 'saving') return false;
    if (invalidEditor) { error = 'El editor devolvió un formato inválido. Corregí o copiá el buffer antes de salir; no se guardó una versión anterior.'; status = 'error'; return false; }
    status = 'saving'; error = '';
    try {
      const document = draft.document ? JSON.parse(JSON.stringify(draft.document)) as Document : undefined;
      const payload = JSON.parse(JSON.stringify(draft.payload)) as Record<string, unknown>;
      const payloadChanged = JSON.stringify(payload) !== JSON.stringify(selected?.payload);
      const entity = selected
        ? await operate<Entity>({ operation: 'entity.update', input: { id: selected.id, expected_revision: selected.revision, title: draft.title, document, ...(payloadChanged ? { payload } : {}) } }, workspace.path)
        : await operate<Entity>({ operation: 'entity.create', input: { kind: draft.kind, title: draft.title, document, payload, context: draft.context } }, workspace.path);
      selected = entity; draft = { ...draft, title: entity.title, payload: JSON.parse(JSON.stringify(entity.payload)) as Record<string, unknown>, ...(entity.document ? { document: structuredClone(entity.document) as Document } : {}) };
      status = 'saved'; remoteRevision = null;
      await setPending(false); await loadList();
      try { await loadRelations(entity); await loadDashboard(entity); } catch (e) { error = `Guardado confirmado; no se pudieron actualizar las relaciones: ${report(e)}`; }
      return true;
    } catch (e) {
      if (e instanceof OperationError && e.detail.code === 'revision_conflict') {
        status = 'conflict'; remoteRevision = Number((e.detail.details as { current_revision: number }).current_revision);
      } else status = 'error';
      error = report(e); return false;
    }
  }
  async function refresh() {
    if (!workspace || working || status === 'saving') return;
    const workspacePath = workspace.path;
    const entityId = selected?.id;
    if (section === 'search' && searchScope === 'attachments') await attachmentPanel?.refreshPanel();
    else if (section === 'search' && searchScope === 'combined') await groupedPanel?.refreshPanel();
    else await loadList();
    if (section === 'collections' && databasePanel) await databasePanel.refreshPanel();
    if (section === 'home' && homePanel) await homePanel.refreshPanel();
    if (!selected || selected.id !== entityId || workspace?.path !== workspacePath) return;
    try {
      const fresh = await operate<Entity>({ operation: 'entity.get', input: { id: selected.id } }, workspacePath);
      if (!selected || selected.id !== entityId || workspace?.path !== workspacePath || (status as Status) === 'saving') return;
      if (dirty) {
        if (fresh.revision !== selected.revision) { remoteRevision = fresh.revision; status = 'conflict'; error = 'Hay una revisión externa. Tu contenido pendiente sigue acá.'; }
      } else {
        if (fresh.revision !== selected.revision) { selected = fresh; loadDraft(fresh); }
      }
      await loadRelations(fresh);
      await loadDashboard(fresh);
    } catch (e) { error = report(e); }
  }
  async function reload() {
    const entity = selected;
    if (entity) await guarded(async () => { clearSelection(); await openEntity(entity); });
  }
  async function archive() {
    if (!selected || !workspace) return;
    await guarded(async () => {
      const entity = selected;
      if (!entity) return;
      working = true;
      try {
        await operate<Entity>({ operation: entity.archived_at ? 'entity.restore' : 'entity.archive', input: { id: entity.id, expected_revision: entity.revision } }, workspace!.path);
        clearSelection(); await loadList();
      } catch (e) { error = report(e); }
      finally { working = false; }
    });
  }
  async function copyPending() {
    recovery = invalidEditor || (draft?.document ? pendingJson(draft.title, draft.document) : JSON.stringify(draft ?? selected?.document, null, 2));
    try { await navigator.clipboard.writeText(recovery); } catch { error = 'El contenido está en el campo de recuperación: seleccioná y copiá el texto.'; }
  }
  async function addRelation() {
    if (!workspace || !selected || dirty || !relationTarget.trim()) return;
    working = true;
    try { const r = await operate<Contract['relation_result']>({ operation: 'relation.add', input: { source_id: selected.id, target_id: relationTarget.trim(), kind: relationKind, expected_revision: selected.revision } }, workspace.path); selected = r.entity; relationTarget = ''; await loadRelations(r.entity); await loadDashboard(r.entity); }
    catch (e) { error = report(e); }
    finally { working = false; }
  }
  async function removeRelation(relation: Relation) {
    if (!workspace || !selected || dirty) return;
    working = true;
    try { const source = await operate<Entity>({ operation: 'entity.get', input: { id: relation.source_id } }, workspace.path);
      const r = await operate<Contract['relation_result']>({ operation: 'relation.remove', input: { id: relation.id, expected_revision: source.id === selected.id ? selected.revision : source.revision } }, workspace.path);
      if (selected.id === r.entity.id) selected = r.entity; await loadRelations(selected); await loadDashboard(selected); }
    catch (e) { error = report(e); }
    finally { working = false; }
  }
  function downloadJson() {
    const url = URL.createObjectURL(new Blob([JSON.stringify(selected?.document, null, 2)], { type: 'application/json' }));
    const a = document.createElement('a'); a.href = url; a.download = `noteka-${selected?.id}.json`; a.click(); URL.revokeObjectURL(url);
  }
  async function exportNote(format: 'json' | 'markdown') {
    if (!workspace || !selected || dirty || !exportPath.trim()) return;
    try { const result = await operate<Contract['export']>({ operation: 'document.export', input: { id: selected.id, destination: exportPath.trim(), format } }, workspace.path); exportMessage = `Exportado a ${result.path}. ${result.warnings.join(' ')}`; }
    catch (e) { error = report(e); }
  }
  function keydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 's') { e.preventDefault(); if (dirty) void save(); }
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'n') { e.preventDefault(); void newNote(); }
    if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === 'a') { e.preventDefault(); void archive(); }
    if (e.key === 'Escape' && guard) { e.preventDefault(); guard = null; }
  }
  function beforeUnload(e: BeforeUnloadEvent) { if (dirty) { e.preventDefault(); e.returnValue = ''; } }
  function modal(node: HTMLDialogElement) {
    const previous = document.activeElement as HTMLElement;
    node.showModal(); node.querySelector<HTMLButtonElement>('[data-cancel]')?.focus();
    return { destroy() { previous?.focus(); } };
  }
  onMount(() => {
    let dispose: (() => void) | undefined;
    let disposed = false;
    if (isTauri()) void listen('noteka-close-requested', () => { void guarded(closeConfirmed); }).then(fn => { if (disposed) fn(); else dispose = fn; });
    return () => { disposed = true; dispose?.(); };
  });
</script>

<svelte:window onkeydown={keydown} onbeforeunload={beforeUnload} onfocus={() => void refresh()} />

{#if !workspace}
  <main class="welcome">
    <div class="brandmark">n<span>.</span></div>
    <p class="eyebrow">TU ESPACIO, EN TU EQUIPO</p>
    <h1>Noteka</h1>
    <p>Un lugar para tus notas y lo que viene después.<br />Sin cuentas. Sin modelos. Tus datos, locales.</p>
    <form onsubmit={(e) => { e.preventDefault(); void selectWorkspace(false); }}>
      <label for="workspace-path">Ruta del workspace</label>
      <input id="workspace-path" bind:value={path} placeholder="Dejá vacío para usar el directorio de datos de Noteka" disabled={working} />
      <div class="actions"><button class="primary" type="button" disabled={working} onclick={() => void selectWorkspace(true)}>Crear workspace</button><button disabled={working}>Abrir workspace</button></div>
    </form>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <small>Editor local · Sin servicios externos</small>
  </main>
{:else}
  <div class="shell" class:home-shell={section === 'home'}>
    <aside>
      <div class="workspace-brand"><span class="tiny-mark">n.</span><strong>Noteka</strong><span class="local-dot" title="Almacenamiento local"></span></div>
      <p class="sidebar-caption">WORKSPACE LOCAL</p>
      <nav aria-label="Secciones">
        {#each sections as item}
          <button class:active={section === item.id} onclick={() => void navigate(item.id)} disabled={working || status === 'saving'}><span aria-hidden="true">{item.icon}</span>{item.label}</button>
        {/each}
      </nav>
      <div class="sidebar-bottom"><label>Tema<select aria-label="Tema" value={theme} onchange={(e) => changeTheme(e.currentTarget.value as Theme)}><option value="dark">Oscuro</option><option value="light">Claro</option></select></label>{#if themeError}<p role="status">{themeError}</p>{/if}<small title={workspace.path}>{workspace.path}</small><button onclick={() => void guarded(async () => { clearSelection(); workspace = null; entities = []; })}>Cambiar workspace</button><span>Sin servicios externos</span></div>
    </aside>
    {#if section === 'home'}
      {#key workspace.path}<HomePanel bind:this={homePanel} workspace={workspace.path} {error} onopen={openHome} oncreate={createHome} onnavigate={navigate} />{/key}
    {:else}
    <section class="entity-list" aria-label={currentLabel}>
      <header><h2>{currentLabel}</h2><button title="Actualizar" aria-label="Actualizar" onclick={() => void refresh()} disabled={loading || working}>↻</button></header>
      {#if section !== 'archive' && section !== 'search' && section !== 'collections'}<button class="new-note" onclick={() => void newEntity(createKind)}>+ {createLabel[createKind]} {#if createKind === 'note'}<kbd>⌘N</kbd>{/if}</button>{/if}
      {#if section === 'search'}
        <label>Ámbito de búsqueda<select aria-label="Ámbito de búsqueda" value={searchScope} disabled={working || status === 'saving'} onchange={(e) => changeSearchScope(e.currentTarget.value as 'notes' | 'attachments' | 'combined')}><option value="notes">Notas y entidades</option><option value="attachments">Adjuntos</option><option value="combined">Notas y adjuntos (grupos)</option></select></label>
        {#if searchScope === 'notes'}<form class="search-form" onsubmit={(e) => { e.preventDefault(); void loadList(); }}><label>Texto local<input aria-label="Texto de búsqueda" bind:value={searchQuery} placeholder="1–16 términos" maxlength="256" /></label><label>Tipo<select aria-label="Tipo de búsqueda" bind:value={searchKind}><option value="">Todos</option>{#each ['note','task','project','tag','person'] as kind}<option value={kind}>{kind}</option>{/each}</select></label><label class="checkbox-label"><input type="checkbox" bind:checked={searchArchived} /> Sólo archivadas</label><label>ID contexto opcional<input aria-label="Contexto de búsqueda" bind:value={searchRelated} /></label><button disabled={loading || working || !searchQuery.trim()}>Buscar</button><button type="button" disabled={working || loading} onclick={() => void rebuildSearch()}>Reconstruir índice</button><p class="list-message">Títulos y bloques reconocidos. Para fragmentos seleccionados de archivos, elegí Adjuntos. Sin modelos.</p>{#if searchMessage}<p class="list-message" role="status">{searchMessage}</p>{/if}</form>
        {:else if searchScope === 'attachments'}{#key workspace.path}<AttachmentSearchPanel bind:this={attachmentPanel} workspace={workspace.path} />{/key}{:else}{#key workspace.path}<GroupedSearchPanel bind:this={groupedPanel} workspace={workspace.path} onopen={openSearch} />{/key}{/if}
      {/if}
      {#if workflowDate}<p class="list-message">Día civil {workflowDate} · {workspace.timezone}</p>{/if}
      {#if loading && entities.length === 0}<p class="list-message">Cargando…</p>{/if}
      {#if listError}<p class="error" role="alert">{listError}</p>{/if}
      {#if !loading && !listError && entities.length === 0 && section !== 'search'}<p class="list-message">{section === 'collections' ? 'Colecciones y vistas en el panel principal. Cada registro abre su entidad original.' : section === 'archive' ? 'Nada archivado.' : 'Todavía no hay elementos.'}</p>{/if}
      <div class="list-items">
        {#if section === 'search' && searchScope === 'notes'}{#each searchHits as hit, i (`${hit.entity_id}:${hit.block_id ?? ''}:${i}`)}<button class="search-hit" onclick={() => void openSearch(hit)}><span class="entity-title">{hit.title}</span><small>{hit.block_id ? `Bloque ${hit.block_id}` : 'Título'} · r{hit.revision}</small><span class="search-excerpt">{hit.excerpt}</span></button>{/each}{#if !loading && searchQuery.trim() && !searchHits.length && !listError}<p class="list-message">Sin resultados.</p>{/if}{/if}
        {#each entities as entity (entity.id)}
          <button class:selected={selected?.id === entity.id} onclick={() => void openEntity(entity)} disabled={working || status === 'saving'}><span class="entity-title">{entity.title}</span><small>{entity.kind} · r{entity.revision}</small></button>
        {/each}
      </div>
      {#if nextCursor}<button onclick={() => void loadList(nextCursor!)} disabled={loading}>Cargar más</button>{/if}
    </section>
    <main class="editor">
      <header class="editor-bar"><button onclick={() => void navigate('home')}>Volver a Home</button><span>{currentLabel}{selected ? ` / ${selected.title}` : draft ? ` / ${createLabel[draft.kind]}` : ''}</span>
        <div class="actions">
          {#if draft}<span class:warning={dirty} class="save-status" role="status">{({ saved: 'Guardado', dirty: 'Cambios pendientes', saving: 'Guardando…', error: 'Error · Sin guardar', conflict: 'Conflicto · Sin guardar' })[status]}</span><button class="primary" onclick={() => void save()} disabled={!dirty || status === 'saving'}>{status === 'error' ? 'Reintentar' : 'Guardar'} <kbd>⌘S</kbd></button>{/if}
          {#if selected}<button onclick={() => void archive()} disabled={working || status === 'saving'}>{selected.archived_at ? 'Restaurar' : 'Archivar'}</button>{/if}
        </div>
      </header>
      <div class="page">
        {#if error}<p class="error" role="alert">{error}</p>{/if}
        {#if sourceLocator}<p class="source-locator" role="status">{sourceLocator}</p>{/if}
        {#if status === 'conflict'}<div class="conflict"><p>Revisión local: {selected?.revision}. Revisión actual: {remoteRevision}. No hay merge automático.</p><div class="actions"><button onclick={() => void reload()}>Recargar versión guardada</button><button onclick={() => void copyPending()}>Copiar contenido pendiente</button></div></div>{/if}
        {#if section === 'collections' && !selected && !draft}
          <DatabasePanel bind:this={databasePanel} workspace={workspace.path} onpending={(pending) => collectionPending = pending} onbusy={(busy) => working = busy} onguard={guarded} onopen={openEntity} onerror={(message) => error = message} />
        {:else if draft}
          <p class="eyebrow">{draft.kind}{selected?.archived_at ? ' · ARCHIVADO' : ''}</p>
          <input class="title-input" aria-label={draft.kind === 'note' ? 'Título de la nota' : 'Título de la entidad'} placeholder="Sin título" bind:this={titleInput} bind:value={draft.title} oninput={edited} disabled={status === 'saving' || working} />
          {#if selected}<p class="metadata">Revisión {selected.revision} · {new Date(selected.updated_at).toLocaleString('es', { timeZone: workspace.timezone })} · {workspace.timezone} · Local</p>{/if}
          {#if draft.context}<p class="metadata">Captura vinculada a {draft.context.kind}: {draft.context.target_id}</p>{/if}
          <EntityFields kind={draft.kind} payload={draft.payload} disabled={status === 'saving' || working} onchange={edited} />
          {#key editorEpoch}{#if editorInitial}<BlockEditor initial={editorInitial} workspace={workspace.path} focusBlock={sourceBlock} disabled={status === 'saving' || working} onopen={openEntity} onchange={(document) => { if (draft) { invalidEditor = ''; draft.document = document; edited(); } }} onerror={(message) => { error = message; }} oninvalid={(raw, message) => { invalidEditor = raw; error = message; edited(); }} />{/if}{/key}
        {:else if selected}
          <p class="eyebrow">{selected.kind}{selected.archived_at ? ' · ARCHIVADO' : ''}</p><h1>{selected.title}</h1><p class="metadata">Revisión {selected.revision} · Local</p>
          {#if selected.kind === 'note' && !selected.document_editable}
            <div class="conflict"><h3>Formato no editable</h3><p>Esta versión no reconoce el documento. No vamos a convertirlo ni descartar campos.</p><button onclick={() => { recovery = JSON.stringify(selected?.document, null, 2); }}>Ver JSON original</button> <button onclick={downloadJson}>Descargar JSON</button></div>
          {:else}<pre class="properties">{JSON.stringify(selected.payload, null, 2)}</pre>{/if}
        {:else}
          <div class="editor-empty"><span aria-hidden="true">▤</span><h1>Espacio para pensar.</h1><p>Abrí un elemento de la lista{section === 'note' ? ' o creá tu primera nota.' : '.'}</p>{#if section === 'note'}<button onclick={() => void newNote()}>Crear una nota</button>{/if}</div>
        {/if}
        {#if selected && ['project', 'tag', 'person'].includes(selected.kind)}<section class="dashboard"><h3>Entidades relacionadas</h3><div class="actions"><button disabled={dirty || working} onclick={() => void newEntity('note', { target_id: selected!.id, kind: selected!.kind as 'project' | 'tag' | 'person' })}>Capturar nota acá</button><button disabled={dirty || working} onclick={() => void newEntity('task', { target_id: selected!.id, kind: selected!.kind as 'project' | 'tag' | 'person' })}>Capturar tarea acá</button></div>{#each dashboard as item (item.id)}<button class="dashboard-item" onclick={() => void openEntity(item)}>{item.title} · {item.kind} {String((item.payload as Record<string, unknown>)?.state ?? '')}</button>{/each}{#if dashboardCursor}<button onclick={() => void loadDashboard(selected!, dashboardCursor!)}>Más relacionadas</button>{/if}</section>{/if}
        {#if selected?.document && !dirty}<div class="export-tools"><label>Directorio nuevo para exportar<input aria-label="Destino de exportación" bind:value={exportPath} placeholder="/ruta/a/exportacion" /></label><button disabled={!exportPath.trim()} onclick={() => void exportNote('json')}>Exportar JSON + originales</button><button disabled={!selected.document_editable || !exportPath.trim()} onclick={() => void exportNote('markdown')}>Exportar Markdown</button>{#if exportMessage}<p role="status">{exportMessage}</p>{/if}</div>{/if}
        {#if selected}<section class="relations"><h3>Relaciones explícitas</h3>{#each relationships as item}<div class="relation-row"><button onclick={() => void openEntity(item.other)}>{item.relation.kind} → {item.other.title}{item.other.archived_at ? ' · Archivado' : ''}</button><button aria-label={`Quitar relación con ${item.other.title}`} disabled={dirty || working} onclick={() => void removeRelation(item.relation)}>Quitar</button></div>{/each}<label>Tipo<select aria-label="Tipo de relación" bind:value={relationKind}><option value="project">Proyecto</option><option value="tag">Tag</option><option value="person">Persona</option></select></label><label>ID destino local<input aria-label="ID de relación" bind:value={relationTarget} placeholder="ID de proyecto, tag o persona" /></label><button disabled={dirty || working || !relationTarget.trim()} onclick={() => void addRelation()}>Relacionar</button><p class="metadata">{selected.id} · Guardá antes de cambiar relaciones.</p></section>{/if}
        {#if (draft?.kind ?? selected?.kind) === 'note'}{#key `${workspace.path}:${selected?.id ?? 'draft'}:${editorEpoch}`}<AttachmentPreparationPanel workspace={workspace.path} document={draft?.document} /><AttachmentGenerationPanel workspace={workspace.path} />{/key}{/if}
        {#if recovery}<label for="recovery">Contenido recuperable</label><textarea id="recovery" class="recovery" readonly value={recovery}></textarea>{/if}
      </div>
    </main>
    {/if}
  </div>
{/if}

{#if guard}
    <dialog use:modal aria-labelledby="pending-title" class="modal" oncancel={(e) => { e.preventDefault(); guard = null; }}>
      <h2 id="pending-title">Hay cambios pendientes</h2><p>Elegí qué hacer antes de salir del buffer actual.</p>
      {#if error}<p class="error" role="alert">{error}</p>{/if}
       <div class="actions"><button class="primary" onclick={() => void resolveGuard('save')} disabled={working || status === 'saving'}>Guardar y continuar</button><button onclick={() => void resolveGuard('discard')} disabled={working || status === 'saving'}>Descartar</button><button data-cancel disabled={working || status === 'saving'} onclick={() => void resolveGuard('cancel')}>Cancelar</button></div>
    </dialog>
{/if}
