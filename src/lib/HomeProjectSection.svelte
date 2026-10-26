<script lang="ts">
  import { onDestroy } from 'svelte';
  import SectionIcon from './SectionIcon.svelte';
  import AppIcon from './AppIcon.svelte';
  import HomeProjectCard from './HomeProjectCard.svelte';
  import TaskPageProjectRow from './TaskPageProjectRow.svelte';
  import { homeGroupLabel, type HomeReadState } from './home-reads';
  import type { HomeView, HomeGroup, ProjectState } from './contracts.generated';
  import { boardIsComplete, moveProject, type ProjectPlacement } from './project-board';
  let { workspace, active = true, readState, onview, onnext, onretry, onopen, oncreate, onnavigate, onmoved, pageMode = false, standalone = false, ongroupcreate }: {
    pageMode?: boolean; standalone?: boolean; ongroupcreate?: (group: HomeGroup) => Promise<void>;
    workspace: string; active?: boolean; onmoved: () => Promise<void>;
    readState?: HomeReadState; onview: (view?: HomeView) => Promise<void>; onnext: () => Promise<void>;
    onretry: () => Promise<void>; onopen: (id: string) => Promise<void>;
    oncreate: (seed?: Record<string, unknown>) => Promise<void>; onnavigate: () => Promise<void>;
  } = $props();
  const views: { id: HomeView; label: string; icon: string }[] = [
    { id: 'projects_active', label: 'Active', icon: 'forward' },
    { id: 'projects_planned', label: 'Planned', icon: 'briefcase' },
    { id: 'projects_board', label: 'Board', icon: 'columns' },
  ];
  const definitions: { state: ProjectState; label: string }[] = [
    { state: 'planned', label: 'Planned' }, { state: 'doing', label: 'Doing' },
    { state: 'ongoing', label: 'Ongoing' }, { state: 'done', label: 'Done' }, { state: 'on_hold', label: 'On Hold' },
  ];
  const page = $derived(readState?.page);
  const displayedViews = $derived(pageMode ? [...views, { id: 'projects_deadlines' as HomeView, label: 'Deadlines', icon: 'calendar' }, { id: 'projects_completed' as HomeView, label: 'Completed', icon: 'circle-check' }, ...(standalone ? [{ id: 'projects_on_hold' as HomeView, label: 'On Hold', icon: 'pause' }] : [])] : views);
  const columns = $derived(definitions.filter(c => readState?.view === 'projects_active' ? c.state === 'doing' || c.state === 'ongoing' : readState?.view === 'projects_planned' ? c.state === 'planned' : readState?.view === 'projects_completed' ? c.state === 'done' : readState?.view === 'projects_on_hold' ? c.state === 'on_hold' : true));
  const relationView = $derived(readState?.view === 'projects_by_tag' || readState?.view === 'projects_by_people');
  let optionsOpen = $state(false);
  let root: HTMLElement;
  let alive = true;
  let moving = $state(false);
  let draggedId = $state<string | null>(null);
  let destination = $state<{ state: ProjectState; beforeId: string | null } | null>(null);
  let dialogId = $state<string | null>(null);
  let dialogState = $state<ProjectState>('planned');
  let beforeId = $state<string | null>(null);
  let moveError = $state('');
  let message = $state('');
  const titleId = $props.id();
  const available = $derived(active && boardIsComplete(page) && !readState?.busy);
  const dialogProject = $derived(page?.items.find(row => row.entity.id === dialogId));
  const dialogPeers = $derived(page?.items.filter(row => row.metadata.state === dialogState && row.entity.id !== dialogId) ?? []);
  onDestroy(() => { alive = false; });
  $effect(() => { if (!active || readState?.view !== 'projects_board') { draggedId = null; destination = null; dialogId = null; } });
  function modal(node: HTMLDialogElement) {
    const previous = document.activeElement as HTMLElement | null;
    node.showModal(); node.querySelector<HTMLElement>('[data-cancel]')?.focus();
    return { destroy() { if (previous?.isConnected) previous.focus(); else root?.querySelector<HTMLButtonElement>('h2 button, .home-tabs [aria-pressed="true"]')?.focus(); } };
  }
  function endDrag() { draggedId = null; destination = null; }
  function startDrag(id: string, event: DragEvent) {
    if (!available || moving || (event.target instanceof Element && event.target.closest('.project-card-context'))) { event.preventDefault(); return; }
    draggedId = id; moveError = ''; message = '';
    if (event.dataTransfer) { event.dataTransfer.effectAllowed = 'move'; event.dataTransfer.setData('application/x-noteka-project', id); }
  }
  function dragOver(state: ProjectState, before: string | null, event: DragEvent) {
    if (!draggedId || !available || moving) return;
    event.preventDefault(); event.stopPropagation();
    if (event.dataTransfer) event.dataTransfer.dropEffect = 'move';
    destination = { state, beforeId: before };
  }
  async function drop(state: ProjectState, before: string | null, event: DragEvent) {
    if (!draggedId || !available || moving) return;
    event.preventDefault(); event.stopPropagation();
    const id = draggedId; endDrag(); await savePlacement({ id, state, beforeId: before });
  }
  function openMove(id: string) {
    if (!available || moving || !page) return;
    const row = page.items.find(item => item.entity.id === id); if (!row) return;
    dialogState = row.metadata.state as ProjectState;
    const peers = page.items.filter(item => item.metadata.state === dialogState);
    beforeId = peers[peers.findIndex(item => item.entity.id === id) + 1]?.entity.id ?? null;
    dialogId = id; moveError = '';
  }
  async function savePlacement(placement: ProjectPlacement) {
    if (!page || !available || moving) return;
    const snapshot = page; moving = true; moveError = ''; message = '';
    try {
      const saved = await moveProject(workspace, snapshot, placement, () => alive && active && readState?.view === 'projects_board');
      if (!alive || !active) return;
      dialogId = null;
      await onmoved();
      if (alive) message = saved ? `Position saved in ${definitions.find(column => column.state === placement.state)?.label ?? placement.state}.` : "The position did not change.";
    } catch (error) {
      if (alive) {
        moveError = `${error instanceof Error ? error.message : String(error)} Saving was not retried. Review the board before moving again.`;
        await onmoved();
      }
    } finally { if (alive) moving = false; }
  }
  function create(state: ProjectState = 'planned') { return oncreate({ state }); }
</script>

<svelte:window onkeydown={(event) => { if (event.key === 'Escape' && root?.contains(document.activeElement)) optionsOpen = false; }} />

<section class="home-section home-tasks home-projects" class:task-page-projects={pageMode} class:tasks-page-section={standalone} data-home-section="project" aria-label="Projects" aria-busy={readState?.busy ?? true} bind:this={root}>
  {#if standalone}
    <header class="tasks-page-heading"><span class="tasks-page-emblem" aria-hidden="true"><SectionIcon section="project" size={68} /></span><h1>Projects</h1></header>
    <nav class="tasks-page-nav" aria-label="Projects quick navigation"><details><summary>Nav</summary>{#each displayedViews as view}<button disabled={moving} onclick={() => void onview(view.id)}>{view.label}</button>{/each}</details><span aria-hidden="true">|</span><button aria-pressed={readState?.view === 'projects_by_tag'} disabled={moving} onclick={() => void onview('projects_by_tag')}>By Tag</button><span aria-hidden="true">|</span><button aria-pressed={readState?.view === 'projects_by_people'} disabled={moving} onclick={() => void onview('projects_by_people')}>By People</button></nav>
  {:else}<header><h2><span class="task-heading-icon project-heading-icon" aria-hidden="true"><SectionIcon section="project" size={31} /></span><button onclick={() => void onnavigate()}>Projects</button></h2></header>{/if}
  {#if !pageMode || standalone}<div class="task-capture"><button disabled={moving} onclick={() => void create()} aria-label="Create in Projects"><SectionIcon section="project" size={16} />New Project</button></div>{/if}
  <div class="task-view-toolbar">
    <div class="home-tabs task-view-tabs" role="group" aria-label="Projects views">{#each displayedViews as view}<button class:active={readState?.view === view.id} aria-pressed={readState?.view === view.id} disabled={moving} onclick={() => void onview(view.id)}><AppIcon name={view.icon} size={16} /><span>{view.label}</span></button>{/each}</div>
    <div class="task-view-actions">
      <button class="task-icon-action" aria-label="Refresh Projects" title="Refresh Projects" disabled={readState?.busy || moving} onclick={() => void onview()}><AppIcon name="refresh" size={16} /></button>
      {#if !standalone}<button class="task-icon-action" aria-label="Open Projects list" title="Open Projects list" onclick={() => void onnavigate()}><AppIcon name="expand" size={16} /></button>{/if}
      <button class="task-icon-action" aria-label="Projects view options" aria-expanded={optionsOpen} title="View options" onclick={() => optionsOpen = !optionsOpen}><AppIcon name="sliders" size={16} /></button>
      <button class="task-new primary" disabled={moving} onclick={() => void create()}>New</button>
    </div>
  </div>
  {#if optionsOpen}<div class="task-view-options"><p>Active: Doing and Ongoing. Planned: planned projects. Board: all five statuses. Projects archived locally or in their imported source are excluded; general search keeps source-archived imports accessible.</p>{#if pageMode}<p>Deadlines: incomplete projects with an end date, sorted by that date. Completed: Done status. Dates are not derived from tasks.</p>{/if}{#if standalone}<p>On Hold: paused projects. By Tag and By People group all five statuses by local links; a project may appear in several groups. Groups and counts cover loaded projects. New page in a relation group prepares Planned with that link.</p>{/if}<p>Progress uses related local tasks: completed / total, rounded to one decimal place. Overdue: pending tasks due before the workspace calendar day. Goal references open their imported pages.</p>{#if page}<p class="home-coverage">Loaded: {page.items.length}. In this view: {page.matching_count}.</p>{/if}</div>{/if}
  {#if readState?.busy}<p role="status" class="task-read-status">Loading Projects…</p>{/if}
  {#if readState?.error}<p class="error" role="alert">Read failed: {readState.error}</p><button onclick={() => void onretry()}>Retry Projects</button>{/if}
  {#if page}
    {#if page.unavailable_count}<p class="home-unavailable">{page.unavailable_count} projects with uninterpretable metadata for this view.</p>{/if}
    {#if readState?.view === 'projects_board' && !boardIsComplete(page)}<p class="home-unavailable">To move cards, the board must show all its projects (up to 100) with available metadata.</p>{/if}
    {#if moving}<p class="project-move-status" role="status">Saving position…</p>{/if}
    {#if message}<p class="project-move-status" role="status">{message}</p>{/if}
    {#if moveError && !dialogId}<p class="error" role="alert">{moveError}</p>{/if}
    {#if pageMode && readState?.view !== 'projects_board'}
      <div class="task-page-project-list">
        {#if relationView}
          <div class="project-relation-heading"><AppIcon name={readState?.view === 'projects_by_tag' ? 'tag' : 'person'} size={17} /><h2>{readState?.view === 'projects_by_tag' ? 'By Tag' : 'By People'}</h2></div>
          {#each page.groups as group (group.key)}
            <details open class="project-relation-group" data-related-id={group.related_entity?.id}>
              <summary>{#if group.related_entity}<button class="project-relation-title" onclick={(event) => { event.stopPropagation(); if (group.related_entity) void onopen(group.related_entity.id); }}><AppIcon name={group.related_entity.icon ?? (group.related_entity.kind === 'tag' ? 'tag' : 'person')} color={group.related_entity.icon_color ?? undefined} size={20} /><span>{homeGroupLabel(group, readState?.view)}{group.related_entity.archived_at ? " · archived" : ''}</span></button>{:else}<span>{homeGroupLabel(group, readState?.view)}</span>{/if}<span class="task-group-count" title="Projects on this page">{group.entity_ids.length}</span></summary>
              {#each group.entity_ids as id (id)}{@const row = page.items.find(item => item.entity.id === id)}{#if row}<TaskPageProjectRow item={row} {onopen} showState />{/if}{/each}
              <button class="home-new task-add-row" aria-label={`New page in ${homeGroupLabel(group, readState?.view)}`} onclick={() => void (ongroupcreate ? ongroupcreate(group) : create())}><AppIcon name="plus" size={15} />New page</button>
            </details>
          {/each}
          {#if !page.items.length}<p class="home-empty">No projects in this view.</p>{/if}
        {:else if readState?.view === 'projects_deadlines'}
          {#each page.items as row (row.entity.id)}<TaskPageProjectRow item={row} {onopen} showDeadline />{/each}
          {#if !page.items.length}<p class="home-empty">No pending projects with an end date.</p>{/if}
          <button class="home-new task-add-row" onclick={() => void create()}><AppIcon name="plus" size={15} />New page</button>
        {:else}
          {#each columns as column (column.state)}
            {@const rows = page.items.filter(row => row.metadata.state === column.state)}
            <details open class="task-page-project-group" data-project-state={column.state}>
              <summary><span class="project-state-pill"><span class="project-state-dot" aria-hidden="true"></span>{column.label}</span><span class="task-group-count" title="Projects on this page">{rows.length}</span></summary>
              {#each rows as row (row.entity.id)}<TaskPageProjectRow item={row} {onopen} />{/each}
              {#if !rows.length}<p class="home-empty">No projects in this view.</p>{/if}
              <button class="home-new task-add-row" onclick={() => void create(column.state)}><AppIcon name="plus" size={15} />New page</button>
            </details>
          {/each}
          {#each page.items.filter(row => !definitions.some(c => c.state === row.metadata.state)) as row (row.entity.id)}<TaskPageProjectRow item={row} {onopen} />{/each}
        {/if}
      </div>
    {:else}<div class="project-board" class:project-is-dragging={draggedId !== null} role="region" aria-label="Projects board" aria-busy={moving}>
      {#each columns as column (column.state)}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions (Native drop region has an equivalent keyboard Move dialog.) -->
      <section class="project-column" class:project-column-destination={destination?.state === column.state && destination.beforeId === null} data-project-state={column.state} aria-label={column.label} ondragover={(event) => dragOver(column.state, null, event)} ondrop={(event) => void drop(column.state, null, event)}>
        <h3><span class="project-state-pill"><span class="project-state-dot" aria-hidden="true"></span>{column.label}</span></h3>
        <div class="project-column-cards">{#each page.items.filter(row => row.metadata.state === column.state) as row (row.entity.id)}<HomeProjectCard item={row} {onopen} movable={available} {moving} dragging={draggedId === row.entity.id} dropBefore={destination?.beforeId === row.entity.id} onmove={() => openMove(row.entity.id)} onstartdrag={(event) => startDrag(row.entity.id, event)} onenddrag={endDrag} onover={(event) => dragOver(column.state, row.entity.id, event)} ondropcard={(event) => void drop(column.state, row.entity.id, event)} />{/each}</div>
        <button class="project-new-card" disabled={moving} onclick={() => void create(column.state)} aria-label={`New page in ${column.label}`}><AppIcon name="plus" size={16} />New page</button>
      </section>{/each}
      {#if page.items.some(row => !definitions.some(c => c.state === row.metadata.state))}<section class="project-column" aria-label="Status unavailable"><h3>Unavailable</h3>{#each page.items.filter(row => !definitions.some(c => c.state === row.metadata.state)) as row (row.entity.id)}<HomeProjectCard item={row} {onopen} />{/each}</section>{/if}
    </div>{/if}
    {#if page.next_cursor}<footer class="section-row-footer"><button class="section-row-action" disabled={readState?.busy || moving} onclick={() => void onnext()}><AppIcon name="arrow-down" size={15} />Load more</button></footer>{/if}
  {/if}
  {#if dialogId && dialogProject}
    <dialog use:modal class="modal project-move-dialog" aria-labelledby={titleId} oncancel={(event) => { event.preventDefault(); if (!moving) dialogId = null; }}>
      <form onsubmit={(event) => { event.preventDefault(); if (dialogId) void savePlacement({ id: dialogId, state: dialogState, beforeId }); }}>
        <h3 id={titleId}>Move {dialogProject.entity.title}</h3>
        <label>Status<select aria-label="Estado destino" value={dialogState} disabled={moving} onchange={(event) => { dialogState = event.currentTarget.value as ProjectState; beforeId = null; }}>{#each definitions as column}<option value={column.state}>{column.label}</option>{/each}</select></label>
        <label>Position<select aria-label="Destination position" value={beforeId ?? ''} disabled={moving} onchange={(event) => beforeId = event.currentTarget.value || null}>{#each dialogPeers as peer}<option value={peer.entity.id}>Before {peer.entity.title}</option>{/each}<option value="">Al final</option></select></label>
        {#if moveError}<p class="error" role="alert">{moveError}</p>{/if}
        <div class="actions"><button type="button" data-cancel disabled={moving} onclick={() => dialogId = null}>Cancel</button><button class="primary" disabled={moving}>{moving ? "Saving…" : "Move"}</button></div>
      </form>
    </dialog>
  {/if}
</section>
