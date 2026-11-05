<script lang="ts">
  import { noteTypeLabels as types } from './note-types';
  import AppIcon from './AppIcon.svelte';
  import ProjectTaskRow from './ProjectTaskRow.svelte';
  import ProjectTaskBoard from './ProjectTaskBoard.svelte';
  import ProjectTaskCalendar from './ProjectTaskCalendar.svelte';
  import { entityIcon, entityIconColor } from './entity-appearance';
  import { slidingTabs } from './sliding-tabs';
  import { updateTaskState, taskStateOf, type TaskState } from './task-board';
  import type { Entity } from './contracts.generated';

  let { items, workspace, timezone, hasMore = false, busy = false, stale = false, notice = '', error = '', createDisabled = false, onopen, oncreate, onchanged, onmore, onretry }: {
    items: Entity[]; workspace: string; timezone: string; hasMore?: boolean; busy?: boolean; stale?: boolean; notice?: string; error?: string; createDisabled?: boolean;
    onopen: (entity: Entity) => Promise<void>; oncreate: (kind: 'task' | 'note', seed?: Record<string, unknown>) => void;
    onchanged: () => Promise<void>; onmore: () => void; onretry: () => void;
  } = $props();
  const tasks = $derived(items.filter(item => item.kind === 'task'));
  const notes = $derived(items.filter(item => item.kind === 'note' && !(item.payload as Record<string, unknown>)?.goal && !(item.payload as Record<string, unknown>)?.milestone));
  const taskGroups = $derived([
    { key: 'todo', label: 'To Do', items: tasks.filter(item => taskStateOf(item) === 'todo') },
    { key: 'doing', label: 'Doing', items: tasks.filter(item => taskStateOf(item) === 'doing') },
    { key: 'done', label: 'Done', items: tasks.filter(item => taskStateOf(item) === 'done') },
    { key: 'unknown', label: 'Status unavailable', items: tasks.filter(item => taskStateOf(item) === null) },
  ]);
  let taskView = $state<'list' | 'board' | 'calendar'>('list');
  let movingId = $state('');
  let taskError = $state('');

  async function applyTaskState(task: Entity, state: TaskState) {
    if (!workspace || movingId) return;
    movingId = task.id; taskError = '';
    try {
      if (await updateTaskState(workspace, task, state)) await onchanged();
    } catch (e) {
      taskError = e instanceof Error ? e.message : 'Could not update the task status.';
      try { await onchanged(); } catch {}
    } finally { movingId = ''; }
  }
  function createTask(state?: TaskState) { oncreate('task', state ? { state } : undefined); }
</script>

<div class="project-contents" aria-busy={busy}>
  {#if busy}<p role="status" class="project-context-message">Loading relations…</p>{/if}
  {#if error}<p role="alert" class="error">{error}</p><button onclick={onretry}>Retry relations</button>{/if}
  {#if stale}<p role="status" class="project-context-message">{busy ? 'Updating; showing previous results.' : 'Showing previous results; they may be out of date.'}</p>{/if}
  {#if notice && !busy}<p role="status" class="project-context-message">{notice}</p>{/if}
  <section aria-label="Project Tasks" class="project-content-section">
    <header><h2>Tasks</h2><button class="primary glass-action" aria-label="New task in this project" disabled={createDisabled} onclick={() => createTask()}><AppIcon name="plus" size={16} />New</button></header>
    <div class="project-task-views" use:slidingTabs role="group" aria-label="Project task views">
      <button class:active={taskView === 'list'} aria-pressed={taskView === 'list'} onclick={() => taskView = 'list'}><AppIcon name="list" size={15} />List</button>
      <button class:active={taskView === 'board'} aria-pressed={taskView === 'board'} onclick={() => taskView = 'board'}><AppIcon name="columns" size={15} />Board</button>
      <button class:active={taskView === 'calendar'} aria-pressed={taskView === 'calendar'} onclick={() => taskView = 'calendar'}><AppIcon name="calendar" size={15} />Calendar</button>
    </div>
    {#if taskView === 'list'}
      {#if taskError}<p class="error" role="alert">{taskError}</p>{/if}
      {#each taskGroups as group (group.key)}{#if group.items.length}<details class="project-task-group" open={group.key !== 'done'}><summary><span class="project-task-state" data-state={group.key}>{group.label}</span><span>{group.items.length}</span></summary>{#each group.items as item (item.id)}<ProjectTaskRow task={item} disabled={busy || createDisabled || movingId !== ''} onopen={onopen} ontoggle={applyTaskState} />{/each}</details>{/if}{/each}
      {#if !busy && !error && !tasks.length}<p class="project-context-message">{hasMore ? "No Tasks in the loaded relations. You can load more." : "No related Tasks."}</p>{/if}
    {:else if taskView === 'board'}
      <ProjectTaskBoard tasks={tasks} movingId={movingId} error={taskError} disabled={busy || createDisabled} {onopen} onmove={applyTaskState} oncreate={(state) => createTask(state)} />
      {#if !busy && !error && !tasks.length && !hasMore}<p class="project-context-message">No related Tasks.</p>{/if}
    {:else}
      <ProjectTaskCalendar {tasks} {timezone} partial={hasMore} disabled={busy || createDisabled} {onopen} oncreate={(date) => oncreate('task', { due_date: date })} />
    {/if}
    <button class="project-add-row" disabled={createDisabled} onclick={() => createTask()}><AppIcon name="plus" size={16} />New task</button>
  </section>
  <section aria-label="Project Notes" class="project-content-section">
    <header><h2>Notes</h2><button class="primary glass-action" aria-label="New note in this project" disabled={createDisabled} onclick={() => oncreate('note')}><AppIcon name="plus" size={16} />New</button></header>
    <div class="project-content-view"><span><AppIcon name="file-text" size={15} />Project</span></div>
    {#each notes as item (item.id)}{@const type = (item.payload as Record<string,unknown>)?.note_type}<button class="project-content-row" onclick={() => void onopen(item)}><AppIcon name={entityIcon(item.kind,item.title,item.payload)} color={entityIconColor(item.payload)} size={17} /><span>{item.title}</span>{#if typeof type === 'string'}<small class="property-chip">{types[type] ?? type}</small>{/if}</button>{/each}
    {#if !busy && !error && !notes.length}<p class="project-context-message">{hasMore ? "No Notes in the loaded relations. You can load more." : "No related Notes."}</p>{/if}
    <button class="project-add-row" disabled={createDisabled} onclick={() => oncreate('note')}><AppIcon name="plus" size={16} />New note</button>
  </section>
  {#if hasMore}<div class="project-context-footer"><span>Sections show loaded relations.</span><button disabled={busy || stale} onclick={onmore}><AppIcon name="arrow-down" size={15} />Load more relations</button></div>{/if}
</div>
