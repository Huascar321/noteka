<script lang="ts">
  import QuickNavMenu from './QuickNavMenu.svelte';
  import { liquidSurface, liquidTransition } from './liquid-overlay';
  import { onDestroy } from 'svelte';
  import SectionIcon from './SectionIcon.svelte';
  import AppIcon from './AppIcon.svelte';
  import HomeProjectCard from './HomeProjectCard.svelte';
  import TaskPageProjectRow from './TaskPageProjectRow.svelte';
  import ProjectDeadlineCalendar from './ProjectDeadlineCalendar.svelte';
  import { homeGroupLabel, type HomeReadState } from './home-reads';
  import HomeReadStatus from './HomeReadStatus.svelte';
  import type { HomeView, HomeGroup, ProjectState } from './contracts.generated';
  import { boardIsComplete, moveProject, type ProjectPlacement } from './project-board';
  import { slidingTabs } from './sliding-tabs';
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
  const displayedViews = $derived(pageMode ? [...views, { id: 'projects_calendar' as HomeView, label: 'Deadlines', icon: 'calendar' }, { id: 'projects_completed' as HomeView, label: 'Completed', icon: 'circle-check' }, ...(standalone ? [{ id: 'projects_on_hold' as HomeView, label: 'On Hold', icon: 'pause' }] : [])] : views);
  const columns = $derived(definitions.filter(c => readState?.view === 'projects_active' ? c.state === 'doing' || c.state === 'ongoing' : readState?.view === 'projects_planned' ? c.state === 'planned' : readState?.view === 'projects_completed' ? c.state === 'done' : readState?.view === 'projects_on_hold' ? c.state === 'on_hold' : true));
  const relationView = $derived(readState?.view === 'projects_by_tag' || readState?.view === 'projects_by_people');
  let root: HTMLElement;
  let board = $state<HTMLElement>();
  let alive = true;
  let moving = $state(false);
  let draggedId = $state<string | null>(null);
  let destination = $state<{ state: ProjectState; beforeId: string | null } | null>(null);
  let pointer: { id: number; projectId: string; x: number; y: number; started: boolean } | null = null;
  let ghost = $state<{ x: number; y: number; title: string } | null>(null);
  let suppressClick = false;
  let dialogId = $state<string | null>(null);
  let moveAnchor = $state<HTMLElement | null>(null);
  let dialogState = $state<ProjectState>('planned');
  let beforeId = $state<string | null>(null);
  let moveError = $state('');
  let message = $state('');
  const titleId = $props.id();
  const available = $derived(active && boardIsComplete(page) && !readState?.busy && !readState?.stale);
  const dialogProject = $derived(page?.items.find(row => row.entity.id === dialogId));
  const dialogPeers = $derived(page?.items.filter(row => row.metadata.state === dialogState && row.entity.id !== dialogId) ?? []);
  onDestroy(() => { alive = false; });
  $effect(() => { if (!active || readState?.view !== 'projects_board') { endDrag(); dialogId = null; } });
  function modal(node: HTMLDialogElement) {
    const previous = document.activeElement as HTMLElement | null;
    node.showModal(); node.querySelector<HTMLElement>('[data-cancel]')?.focus();
    return { destroy() { if (previous?.isConnected) previous.focus(); else root?.querySelector<HTMLButtonElement>('h2 button, .home-tabs [aria-pressed="true"]')?.focus(); } };
  }
  function endDrag() { pointer = null; draggedId = null; destination = null; ghost = null; }
  function startPointer(id: string, event: PointerEvent) {
    if (!available || moving || pointer || event.button !== 0 || event.pointerType === 'touch') return;
    pointer = { id: event.pointerId, projectId: id, x: event.clientX, y: event.clientY, started: false };
  }
  function movePointer(event: PointerEvent) {
    if (!pointer || event.pointerId !== pointer.id) return;
    if (!available || moving || event.buttons !== 1) { endDrag(); return; }
    if (!pointer.started && Math.hypot(event.clientX - pointer.x, event.clientY - pointer.y) < 6) return;
    if (!pointer.started) { pointer.started = true; draggedId = pointer.projectId; moveError = ''; message = ''; }
    event.preventDefault();
    const title = page?.items.find(row => row.entity.id === pointer?.projectId)?.entity.title ?? 'Project';
    ghost = { x: event.clientX, y: event.clientY, title };
    const target = document.elementFromPoint(event.clientX, event.clientY);
    const column = target?.closest<HTMLElement>('.project-column[data-project-state]');
    if (!column || !board?.contains(column)) { destination = null; return; }
    const state = column.dataset.projectState as ProjectState;
    const card = target?.closest<HTMLElement>('.project-card[data-project-id]');
    destination = { state, beforeId: card && column.contains(card) ? card.dataset.projectId ?? null : null };
  }
  function finishPointer(event: PointerEvent) {
    if (!pointer) { if (suppressClick) window.setTimeout(() => { suppressClick = false; }, 0); return; }
    if (event.pointerId !== pointer.id) return;
    const id = pointer.projectId, started = pointer.started, target = destination;
    endDrag();
    if (!started) return;
    suppressClick = true;
    window.setTimeout(() => { suppressClick = false; }, 0);
    if (target && available && !moving) void savePlacement({ id, ...target });
  }
  function cancelClick(event: MouseEvent) { if (suppressClick) { event.preventDefault(); event.stopPropagation(); suppressClick = false; } }
  function openMove(id: string) {
    if (!available || moving || !page) return;
    moveAnchor = root?.querySelector<HTMLElement>(`[data-project-id="${id}"] .project-move-trigger`) ?? null;
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

<svelte:window onpointermove={movePointer} onpointerup={finishPointer} onpointercancel={endDrag} onblur={endDrag} onclickcapture={cancelClick} onkeydown={(event) => { if (event.key === 'Escape') { if (pointer?.started) suppressClick = true; endDrag(); } }} />

<section class="home-section home-tasks home-projects" class:task-page-projects={pageMode} class:tasks-page-section={standalone} data-home-section="project" aria-label="Projects" aria-busy={readState?.busy ?? true} bind:this={root}>
  {#if standalone}
    <header class="tasks-page-heading"><span class="tasks-page-emblem" aria-hidden="true"><SectionIcon section="project" size={68} /></span><h1>Projects</h1></header>
    <nav class="tasks-page-nav" aria-label="Projects quick navigation"><QuickNavMenu>{#each displayedViews as view}<button disabled={moving} onclick={() => void onview(view.id)}>{view.label}</button>{/each}</QuickNavMenu><span aria-hidden="true">|</span><button aria-pressed={readState?.view === 'projects_by_tag'} disabled={moving} onclick={() => void onview('projects_by_tag')}>By Tag</button><span aria-hidden="true">|</span><button aria-pressed={readState?.view === 'projects_by_people'} disabled={moving} onclick={() => void onview('projects_by_people')}>By People</button></nav>
  {:else}<header><h2><span class="task-heading-icon project-heading-icon" aria-hidden="true"><SectionIcon section="project" size={31} /></span><button onclick={() => void onnavigate()}>Projects</button></h2></header>{/if}
  <div class="task-view-toolbar">
    <div class="home-tabs task-view-tabs" use:slidingTabs role="group" aria-label="Projects views">{#each displayedViews as view}<button class:active={readState?.view === view.id} aria-pressed={readState?.view === view.id} disabled={moving} onclick={() => void onview(view.id)}><AppIcon name={view.icon} size={16} /><span>{view.label}</span></button>{/each}</div>
    <div class="task-view-actions"><HomeReadStatus state={readState} updating />
      <div class="glass-tool-group" role="group" aria-label="Projects tools"><button class="task-icon-action" aria-label="Refresh Projects" title="Refresh Projects" disabled={readState?.busy || moving} onclick={() => void onview()}><AppIcon name="refresh" size={16} /></button>
      </div><button class="task-new primary" disabled={moving} onclick={() => void create()}><AppIcon name="plus" size={16} />New</button>
    </div>
  </div>
  {#if readState?.busy && !page}<p role="status" class="task-read-status">Loading Projects…</p>{/if}
  {#if readState?.error}<p class="error" role="alert">Read failed: {readState.error}</p><button onclick={() => void onretry()}>Retry Projects</button>{/if}
  <HomeReadStatus state={readState} />
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
            <div class="group-open-shell">
              <details open class="project-relation-group" data-related-id={group.related_entity?.id}>
                <summary>{#if group.related_entity}<span class="project-relation-title"><AppIcon name={group.related_entity.icon ?? (group.related_entity.kind === 'tag' ? 'tag' : 'person')} color={group.related_entity.icon_color ?? undefined} size={20} /><span>{homeGroupLabel(group, readState?.view)}{group.related_entity.archived_at ? " · archived" : ''}</span></span>{:else}<span>{homeGroupLabel(group, readState?.view)}</span>{/if}<span class="task-group-count" title="Projects on this page">{group.entity_ids.length}</span></summary>
                {#each group.entity_ids as id (id)}{@const row = page.items.find(item => item.entity.id === id)}{#if row}<TaskPageProjectRow item={row} {onopen} showState />{/if}{/each}
                <button class="home-new task-add-row" aria-label={`New page in ${homeGroupLabel(group, readState?.view)}`} onclick={() => void (ongroupcreate ? ongroupcreate(group) : create())}><AppIcon name="plus" size={16} />New page</button>
              </details>
              {#if group.related_entity}<button class="group-open" aria-label={`Open ${homeGroupLabel(group, readState?.view)}`} title={`Open ${homeGroupLabel(group, readState?.view)}`} onclick={() => { if (group.related_entity) void onopen(group.related_entity.id); }}><AppIcon name="arrow-up-right" size={15} /></button>{/if}
            </div>
          {/each}
          {#if !page.items.length}<p class="home-empty">No projects in this view.</p>{/if}
        {:else if readState?.view === 'projects_calendar'}
          <ProjectDeadlineCalendar items={page.items} date={page.date} partial={!!page.next_cursor} disabled={!!readState?.busy || moving} {onopen} oncreate={(date) => void oncreate({ state:'planned', end_date:date })} />
        {:else}
          {#each columns as column (column.state)}
            {@const rows = page.items.filter(row => row.metadata.state === column.state)}
            <details open class="task-page-project-group" data-project-state={column.state}>
              <summary><span class="project-state-pill"><span class="project-state-dot" aria-hidden="true"></span>{column.label}</span><span class="task-group-count" title="Projects on this page">{rows.length}</span></summary>
              {#each rows as row (row.entity.id)}<TaskPageProjectRow item={row} {onopen} />{/each}
              {#if !rows.length}<p class="home-empty">No projects in this view.</p>{/if}
              <button class="home-new task-add-row" onclick={() => void create(column.state)}><AppIcon name="plus" size={16} />New page</button>
            </details>
          {/each}
          {#each page.items.filter(row => !definitions.some(c => c.state === row.metadata.state)) as row (row.entity.id)}<TaskPageProjectRow item={row} {onopen} />{/each}
        {/if}
      </div>
    {:else}<div bind:this={board} class="project-board" class:project-is-dragging={draggedId !== null} role="region" aria-label="Projects board" aria-busy={moving}>
      {#each columns as column (column.state)}
      <section class="project-column" class:project-column-destination={destination?.state === column.state} data-project-state={column.state} aria-label={column.label}>
        <h3><span class="project-state-pill"><span class="project-state-dot" aria-hidden="true"></span>{column.label}</span></h3>
        <div class="project-column-cards">{#each page.items.filter(row => row.metadata.state === column.state) as row (row.entity.id)}<HomeProjectCard item={row} {onopen} movable={available} {moving} dragging={draggedId === row.entity.id} dropBefore={destination?.beforeId === row.entity.id} onmove={() => openMove(row.entity.id)} onpointerstart={(event) => startPointer(row.entity.id, event)} onclickcapture={cancelClick} />{/each}</div>
        <button class="project-new-card" disabled={moving} onclick={() => void create(column.state)} aria-label={`New page in ${column.label}`}><AppIcon name="plus" size={16} />New page</button>
      </section>{/each}
      {#if page.items.some(row => !definitions.some(c => c.state === row.metadata.state))}<section class="project-column" aria-label="Status unavailable"><h3>Unavailable</h3>{#each page.items.filter(row => !definitions.some(c => c.state === row.metadata.state)) as row (row.entity.id)}<HomeProjectCard item={row} {onopen} />{/each}</section>{/if}
    </div>{/if}
    {#if ghost}<div class="project-drag-ghost" aria-hidden="true" style:left={`${ghost.x + 12}px`} style:top={`${ghost.y + 12}px`}>{ghost.title}</div>{/if}
    {#if page.next_cursor}<footer class="section-row-footer"><button class="section-row-action" disabled={readState?.busy || readState?.stale || moving} onclick={() => void onnext()}><AppIcon name="arrow-down" size={15} />Load more</button></footer>{/if}
  {/if}
  {#if dialogId && dialogProject}
    <dialog use:modal class="modal project-move-dialog glass-popup" use:liquidSurface={{ anchor: () => moveAnchor }} in:liquidTransition|global out:liquidTransition|global aria-labelledby={titleId} oncancel={(event) => { event.preventDefault(); if (!moving) dialogId = null; }}>
      <form onsubmit={(event) => { event.preventDefault(); if (dialogId) void savePlacement({ id: dialogId, state: dialogState, beforeId }); }}>
        <h3 id={titleId}>Move {dialogProject.entity.title}</h3>
        <label>Status<select aria-label="Estado destino" value={dialogState} disabled={moving} onchange={(event) => { dialogState = event.currentTarget.value as ProjectState; beforeId = null; }}>{#each definitions as column}<option value={column.state}>{column.label}</option>{/each}</select></label>
        <label>Position<select aria-label="Destination position" value={beforeId ?? ''} disabled={moving} onchange={(event) => beforeId = event.currentTarget.value || null}>{#each dialogPeers as peer}<option value={peer.entity.id}>Before {peer.entity.title}</option>{/each}<option value="">Al final</option></select></label>
        {#if moveError}<p class="error" role="alert">{moveError}</p>{/if}
        <div class="actions"><button class="glass-secondary" type="button" data-cancel disabled={moving} onclick={() => dialogId = null}>Cancel</button><button class="primary glass-action" disabled={moving}>{moving ? "Saving…" : "Move"}</button></div>
      </form>
    </dialog>
  {/if}
</section>
