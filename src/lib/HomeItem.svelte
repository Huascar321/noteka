<script lang="ts">
  import type { HomeItem } from './contracts.generated';
  import AppIcon from './AppIcon.svelte';
  import { visualIconName } from './visual-icon';
  import { noteTypeLabels as noteLabels } from './note-types';
  let { item, onopen, taskStyle = false, noteStyle = false, ontoggle, toggleDisabled = false, omitContextId }: {
    item: HomeItem; onopen: (id: string, hint?: Pick<HomeItem['entity'], 'kind' | 'title'>) => Promise<void>; taskStyle?: boolean; noteStyle?: boolean;
    ontoggle?: (item: HomeItem) => Promise<void>; toggleDisabled?: boolean; omitContextId?: string;
  } = $props();
  let toggling = $state(false);
  const done = $derived(item.metadata.state === 'done');
  const defaultTaskIcon = $derived(taskStyle && (!item.entity.icon || item.entity.icon === 'task'));
  async function toggle() {
    if (!ontoggle || toggling || toggleDisabled) return;
    toggling = true;
    try { await ontoggle(item); } finally { toggling = false; }
  }
  const contexts = $derived(noteStyle ? [...item.context].sort((a, b) => ({ tag: 0, project: 1, person: 2, note: 3, task: 4 })[a.kind] - ({ tag: 0, project: 1, person: 2, note: 3, task: 4 })[b.kind]) : item.context);
  const visibleContexts = $derived(omitContextId ? contexts.filter(context => context.id !== omitContextId) : contexts);
  const ageLabel = (days: number) => days < 0 ? `Modified in the future (${Math.abs(days)} days)` : `Updated ${days} day${days === 1 ? '' : 's'} ago`;
  const lastNote = $derived(item.summary?.kind === 'tag' ? item.summary.last_note : null);
  function fallbackIcon(entity: HomeItem['entity']) {
    const title = entity.title.trim().toLowerCase();
    if (title === 'home') return 'home';
    if (title === 'notes') return 'notebook';
    if (title === 'tasks') return 'task';
    if (title === 'projects') return 'project';
    return ({ task: 'task', note: 'file-text', project: 'project', tag: 'tag', person: 'person' })[entity.kind] as never;
  }
  const icon = $derived(visualIconName(item.entity.icon, fallbackIcon(item.entity)));
  const noteDays = $derived(item.metadata.source_updated_days_ago ?? item.metadata.updated_days_ago);
  const color = $derived(item.entity.icon_color ?? undefined);
  function dateLabel(value: string): string {
    const date = new Date(`${value}T12:00:00Z`);
    return Number.isFinite(date.getTime()) ? new Intl.DateTimeFormat('en-US', { month: 'long', day: 'numeric', year: 'numeric', timeZone: 'UTC' }).format(date) : value;
  }
</script>

<article class="home-item" class:home-task-row={taskStyle || noteStyle} class:home-note-row={noteStyle} data-entity-id={item.entity.id}>
  {#if taskStyle}
    <div class="home-task-main">
      <button type="button" class="task-status-toggle" role="checkbox" aria-checked={done} aria-label={`${done ? 'Mark pending' : 'Mark complete'}: ${item.entity.title}`} title={done ? "Completed · mark pending" : "Pending · mark complete"} disabled={toggling || toggleDisabled} onclick={() => void toggle()}>
        <span class="task-status-mark" class:complete={done} aria-hidden="true">{done ? '✓' : ''}</span>
      </button>
      <button class="home-open" onclick={() => void onopen(item.entity.id, item.entity)} aria-label={`Open ${item.entity.title}`}>
        {#if !defaultTaskIcon}<span class="home-kind" aria-hidden="true"><AppIcon name={icon} color={color} size={16} /></span>{/if}
        <span title={item.entity.title}>{item.entity.title}</span>
      </button>
    </div>
  {:else}
    <button class="home-open" onclick={() => void onopen(item.entity.id, item.entity)} aria-label={`Open ${item.entity.title}`}>
      <span class="home-kind" aria-hidden="true"><AppIcon name={icon} color={color} size={16} /></span>
      <span title={item.entity.title}>{item.entity.title}</span>
    </button>
  {/if}
  <div class="home-item-meta">
    {#if taskStyle || noteStyle}
      {#each visibleContexts as context (context.id)}<button class="home-context task-project-reference" onclick={() => void onopen(context.id)}><AppIcon name={visualIconName(context.icon, context.kind === 'project' ? 'project' : context.kind === 'person' ? 'person' : 'tag')} color={context.icon_color ?? undefined} size={19} /><span>{context.title}{context.archived_at ? " · archived" : ''}</span></button>{/each}
    {/if}
    {#if item.metadata.state && (!taskStyle || item.metadata.state === 'doing')}<span class="home-badge" class:done={item.metadata.state === 'done'} class:doing={item.metadata.state === 'doing'}>{item.metadata.state === 'doing' ? 'Doing' : item.metadata.state}</span>{/if}
    {#if item.metadata.classification}<span class="home-badge">{item.metadata.classification}</span>{/if}
    {#if item.metadata.due_date}<time datetime={item.metadata.due_date}>{taskStyle ? dateLabel(item.metadata.due_date) : `Due ${item.metadata.due_date}`}</time>{/if}
    {#if noteStyle}
      {#if item.metadata.note_type}<span class="note-type" class:plan={item.metadata.note_type === 'plan'}>{noteLabels[item.metadata.note_type] ?? item.metadata.note_type}</span>{/if}
      {#if noteDays != null}<span class="note-age" title={`${item.metadata.source_updated_at ? "Imported source last modified" : "Last modified locally"}: ${item.metadata.source_updated_at ?? item.entity.updated_at}`} aria-label={ageLabel(noteDays)}>{noteDays < 0 ? '→' : ''}{Math.abs(noteDays)}d</span>{/if}
    {:else if item.entity.kind === 'note'}
      <span>{item.metadata.updated_days_ago == null ? "Age unavailable" : item.metadata.updated_days_ago < 0 ? "Modification date is in the future" : `Updated ${item.metadata.updated_days_ago} day${item.metadata.updated_days_ago === 1 ? '' : 's'} ago`}</span>
    {/if}
    {#if item.metadata.issues.length}<span class="home-unavailable">Metadata unavailable: {item.metadata.issues.join(', ')}</span>{/if}
    {#if item.summary?.kind === 'project'}
      {#if !item.summary.available}<span class="home-unavailable">Summary unavailable</span>
      {:else if item.summary.total === 0}<span>No tasks</span>
      {:else}<span>{item.summary.pending} pending · {item.summary.overdue} overdue · {item.summary.done}/{item.summary.total} complete</span><span class="home-progress"><progress max="100" value={item.summary.progress_percent ?? 0} aria-label={`Progress for ${item.entity.title}`}></progress>{item.summary.progress_percent}%</span>{/if}
    {:else if item.summary?.kind === 'tag'}
      {#if !item.summary.available}<span class="home-unavailable">Summary unavailable</span>
      {:else}<span>{item.summary.note_count} notes</span>{#if lastNote}<button class="home-context" onclick={() => void onopen(lastNote!.id)}>Latest note: {lastNote.title}</button>{/if}{/if}
    {/if}
    {#if !taskStyle && !noteStyle}{#each visibleContexts as context (context.id)}<button class="home-context" onclick={() => void onopen(context.id)}><AppIcon name={visualIconName(context.icon, context.kind === 'project' ? 'project' : context.kind === 'person' ? 'person' : 'tag')} color={context.icon_color ?? undefined} size={14} />{context.title}{context.archived_at ? " · archived" : ''}</button>{/each}{/if}
    {#if !omitContextId && item.context_total > item.context.length}<span>+{item.context_total - item.context.length} references when opening the original</span>{/if}
  </div>
</article>
