<script lang="ts">
  import SectionIcon from './SectionIcon.svelte';
  import AppIcon from './AppIcon.svelte';
  import HomeRow from './HomeItem.svelte';
  import { visualIconName } from './visual-icon';
  import { homeGroupLabel, type HomeReadState } from './home-reads';
  import type { HomeView, HomeGroup, EntityRef } from './contracts.generated';

  let { readState, onview, onnext, onretry, onopen, oncreate, ongroupcreate, onnavigate, pageMode = false }: {
    pageMode?: boolean;
    readState?: HomeReadState; onview: (view?: HomeView) => Promise<void>; onnext: () => Promise<void>;
    onretry: () => Promise<void>; onopen: (id: string) => Promise<void>; oncreate: () => Promise<void>;
    ongroupcreate: (group: HomeGroup) => Promise<void>; onnavigate: () => Promise<void>;
  } = $props();
  const views: { id: HomeView; label: string; icon: string }[] = [
    { id: 'tasks_today', label: 'Today', icon: 'sun' },
    { id: 'tasks_inbox', label: 'Inbox', icon: 'inbox' },
    { id: 'tasks_week', label: 'Week', icon: 'calendar-week' },
    { id: 'tasks_month', label: 'Month', icon: 'calendar' },
    { id: 'tasks_scheduled', label: 'Scheduled', icon: 'calendar-clock' },
    { id: 'tasks_no_due', label: 'No Due', icon: 'circle-dot' },
    { id: 'tasks_active_projects', label: 'Active Projects', icon: 'hammer' },
    { id: 'tasks_projects_in_progress', label: 'Active Projects (In Progress)', icon: 'hammer' },
    { id: 'tasks_all', label: 'All', icon: 'list-checks' },
  ];
  const page = $derived(readState?.page);
  const displayedViews = $derived(pageMode ? [
    ...views.slice(0, 6),
    { id: 'tasks_projects_in_progress' as HomeView, label: 'Active Projects', icon: 'hammer' },
    { id: 'tasks_all' as HomeView, label: 'All', icon: 'list-checks' },
    { id: 'tasks_done' as HomeView, label: 'Done', icon: 'circle-check' },
  ] : views);
  let optionsOpen = $state(false);
  let moreOpen = $state(false);
  let root: HTMLElement;
  let moreButton = $state<HTMLButtonElement>();
  function closeMore() { moreOpen = false; moreButton?.focus(); }
  function projectRef(group: HomeGroup): EntityRef | undefined {
    return page?.items.flatMap(row => row.context).find(ref => ref.id === group.project_id);
  }
</script>

<svelte:window onkeydown={(event) => { if (event.key === 'Escape' && root?.contains(document.activeElement)) { if (moreOpen) { event.preventDefault(); closeMore(); } else if (optionsOpen) optionsOpen = false; } }} onclick={(event) => { if (event.target instanceof Node && root && !root.contains(event.target)) moreOpen = false; }} />

<section class="home-section home-tasks" class:tasks-page-section={pageMode} data-home-section="task" aria-label="Tasks" aria-busy={readState?.busy ?? true} bind:this={root}>
  {#if pageMode}
    <header class="tasks-page-heading"><span class="tasks-page-emblem" aria-hidden="true"><SectionIcon section="task" size={68} /></span><h1>Tasks</h1></header>
    <nav class="tasks-page-nav" aria-label="Tasks quick navigation"><details><summary>Nav</summary><button onclick={() => void onview('tasks_all')}>All tasks</button><button onclick={() => void onview('tasks_done')}>Done</button></details>{#each displayedViews.slice(0, 4).sort((a,b) => Number(a.id !== 'tasks_inbox') - Number(b.id !== 'tasks_inbox')) as view}<span aria-hidden="true">|</span><button onclick={() => void onview(view.id)}>{view.label}</button>{/each}</nav>
  {:else}<header><h2><span class="task-heading-icon" aria-hidden="true"><SectionIcon section="task" size={29} /></span><button onclick={() => void onnavigate()}>Tasks</button></h2></header>{/if}
  <div class="task-capture"><button onclick={() => void oncreate()} aria-label="Create in Tasks"><SectionIcon section="task" size={15} />New Task</button></div>
  <div class="task-view-toolbar">
    <div class="home-tabs task-view-tabs" role="group" aria-label="Tasks views">
      {#each displayedViews as view}
        {#if pageMode && view.id === 'tasks_projects_in_progress'}<button disabled class="task-pending-view" title="Recurring: coming later"><AppIcon name="refresh" size={16} /><span>Recurring</span></button>{/if}
        {#if pageMode && view.id === 'tasks_all'}<button disabled class="task-pending-view" title="Active Content: coming later"><AppIcon name="file-text" size={16} /><span>Active Content</span></button>{/if}
        <button class:active={readState?.view === view.id} aria-pressed={readState?.view === view.id} title={view.label} onclick={() => void onview(view.id)}><AppIcon name={view.icon} size={16} /><span>{view.label}</span></button>
      {/each}
      {#if !pageMode}<button class="task-more-views" aria-label="More Tasks views" aria-expanded={moreOpen} onclick={() => moreOpen = !moreOpen} bind:this={moreButton}>1 more…</button>{/if}
    </div>
    <div class="task-view-actions">
      <button class="task-icon-action" aria-label="Refresh Tasks" title="Refresh Tasks" disabled={readState?.busy} onclick={() => void onview()}><AppIcon name="refresh" size={16} /></button>
      {#if !pageMode}<button class="task-icon-action" aria-label="Open Tasks list" title="Open Tasks list" onclick={() => void onnavigate()}><AppIcon name="expand" size={16} /></button>{/if}
      <button class="task-icon-action" aria-label="Tasks view options" aria-expanded={optionsOpen} title="View options" onclick={() => optionsOpen = !optionsOpen}><AppIcon name="sliders" size={16} /></button>
      <button class="task-new primary" onclick={() => void oncreate()}>New</button>
    </div>
  </div>
  {#if moreOpen}<div class="task-more-menu"><button aria-pressed={readState?.view === 'tasks_done'} onclick={() => { closeMore(); void onview('tasks_done'); }}><AppIcon name="circle-check" size={15} />Done</button><p>Recurring and Active Content are not yet implemented.</p><button onclick={closeMore}>Close</button></div>{/if}
  {#if optionsOpen}<div class="task-view-options"><p>Today: pending tasks due through the current calendar day. Week/Month: ISO week and calendar month of the due date. Scheduled: assigned due date; No Due: no due date.</p><p>Active Projects: Planned/Doing/Ongoing; In Progress: Doing/Ongoing. Only pending tasks in unarchived projects. Creating a task does not assign dates.</p><p>Groups cover this page; a task may appear in several projects. Recurring and Active Content require features that are not yet implemented.</p>{#if page}<p class="home-coverage">Loaded: {page.items.length}. In this view: {page.matching_count}.</p>{/if}</div>{/if}
  {#if readState?.busy}<p role="status" class="task-read-status">Loading Tasks…</p>{/if}
  {#if readState?.error}<p class="error" role="alert">Read failed: {readState.error}</p><button onclick={() => void onretry()}>Retry Tasks</button>{/if}
  {#if page}
    {#if page.unavailable_count}<p class="home-unavailable">{page.unavailable_count} tasks with uninterpretable metadata for this view.</p>{/if}
    {#if page.groups.length}
      <div class="home-groups task-groups">
        {#each page.groups as group (group.key)}
          {@const project = projectRef(group)}
          <details open class="home-group task-group" data-group-key={group.key}>
            <summary><span class="task-group-label">{#if project}<AppIcon name={visualIconName(project.icon, 'project')} color={project.icon_color ?? undefined} size={20} />{/if}{#if group.project_id}<button class="task-group-title" onclick={(event) => { event.stopPropagation(); void onopen(group.project_id!); }}>{homeGroupLabel(group, readState?.view)}{group.project_archived ? " · archived" : ''}</button>{:else}<span class="task-group-title">{homeGroupLabel(group, readState?.view)}</span>{/if}<span class="task-group-count" title="Occurrences on this page">{group.entity_ids.length}</span></span></summary>
            {#each group.entity_ids as id (id)}{@const row = page.items.find(item => item.entity.id === id)}{#if row}<HomeRow item={row} {onopen} taskStyle />{/if}{/each}
            {#if !group.entity_ids.length}<p class="home-empty">No tasks on this page.</p>{/if}
            {#if group.key !== 'unavailable'}<button class="home-new task-add-row" onclick={() => void ongroupcreate(group)}><AppIcon name="plus" size={15} />New task</button>{/if}
          </details>
        {/each}
      </div>
    {:else}
      <div class="home-rows task-rows">{#each page.items as row (row.entity.id)}<HomeRow item={row} {onopen} taskStyle />{/each}{#if !page.items.length}<p class="home-empty">No Tasks in this view.</p>{/if}</div>
      <button class="home-new task-add-row" onclick={() => void oncreate()}><AppIcon name="plus" size={15} />New task</button>
    {/if}
    {#if page.next_cursor}<footer class="section-row-footer"><button class="section-row-action" disabled={readState?.busy} onclick={() => void onnext()}><AppIcon name="arrow-down" size={15} />Load more</button></footer>{/if}
  {/if}
</section>
