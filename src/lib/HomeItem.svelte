<script lang="ts">
  import type { HomeItem } from './contracts.generated';
  import AppIcon from './AppIcon.svelte';
  import { visualIconName } from './visual-icon';
  let { item, onopen, taskStyle = false, noteStyle = false }: { item: HomeItem; onopen: (id: string) => Promise<void>; taskStyle?: boolean; noteStyle?: boolean } = $props();
  const contexts = $derived(noteStyle ? [...item.context].sort((a, b) => ({ tag: 0, project: 1, person: 2, note: 3, task: 4 })[a.kind] - ({ tag: 0, project: 1, person: 2, note: 3, task: 4 })[b.kind]) : item.context);
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
  const noteLabels: Record<string, string> = { reference: 'Reference', plan: 'Plan', web_clip: 'Web Clip', voice: 'Voice', journal: 'Journal', meeting: 'Meeting', topic_research: 'Topic Research', idea: 'Idea', lecture: 'Lecture' };
  const noteDays = $derived(item.metadata.source_updated_days_ago ?? item.metadata.updated_days_ago);
  const color = $derived(item.entity.icon_color ?? undefined);
  function dateLabel(value: string): string {
    const date = new Date(`${value}T12:00:00Z`);
    return Number.isFinite(date.getTime()) ? new Intl.DateTimeFormat('en-US', { month: 'long', day: 'numeric', year: 'numeric', timeZone: 'UTC' }).format(date) : value;
  }
</script>

<article class="home-item" class:home-task-row={taskStyle || noteStyle} class:home-note-row={noteStyle} data-entity-id={item.entity.id}>
  <button class="home-open" onclick={() => void onopen(item.entity.id)} aria-label={`Open ${item.entity.title}`}>
    <span class="home-kind" aria-hidden="true">{#if taskStyle && (!item.entity.icon || item.entity.icon === 'task')}<span class="task-status-mark" class:complete={item.metadata.state === 'done'} title={item.metadata.state === 'done' ? "Completed" : "Pending"}>{item.metadata.state === 'done' ? '✓' : '−'}</span>{:else}<AppIcon name={icon} color={color} size={16} />{/if}</span>
    <span title={item.entity.title}>{item.entity.title}</span>
  </button>
  <div class="home-item-meta">
    {#if taskStyle || noteStyle}
      {#each contexts as context (context.id)}<button class="home-context task-project-reference" onclick={() => void onopen(context.id)}><AppIcon name={visualIconName(context.icon, context.kind === 'project' ? 'project' : context.kind === 'person' ? 'person' : 'tag')} color={context.icon_color ?? undefined} size={19} /><span>{context.title}{context.archived_at ? " · archived" : ''}</span></button>{/each}
    {/if}
    {#if item.metadata.state && !taskStyle}<span class="home-badge" class:done={item.metadata.state === 'done'} class:doing={item.metadata.state === 'doing'}>{item.metadata.state}</span>{/if}
    {#if item.metadata.classification}<span class="home-badge">{item.metadata.classification}</span>{/if}
    {#if item.metadata.due_date}<time datetime={item.metadata.due_date}>{taskStyle ? dateLabel(item.metadata.due_date) : `Vence ${item.metadata.due_date}`}</time>{/if}
    {#if noteStyle}
      {#if item.metadata.note_type}<span class="note-type" class:plan={item.metadata.note_type === 'plan'}>{noteLabels[item.metadata.note_type] ?? item.metadata.note_type}</span>{/if}
      {#if noteDays != null}<span class="note-age" title={`${item.metadata.source_updated_at ? "Imported source last modified" : "Last modified locally"}: ${item.metadata.source_updated_at ?? item.entity.updated_at}`} aria-label={`${noteDays < 0 ? "Future modification" : 'Actualizada hace'} ${Math.abs(noteDays)} days`}>{noteDays < 0 ? '→' : ''}{Math.abs(noteDays)}d</span>{/if}
    {:else if item.entity.kind === 'note'}
      <span>{item.metadata.updated_days_ago == null ? "Age unavailable" : item.metadata.updated_days_ago < 0 ? "Modification date is in the future" : `Actualizada hace ${item.metadata.updated_days_ago} days`}</span>
    {/if}
    {#if item.metadata.issues.length}<span class="home-unavailable">Metadata unavailable: {item.metadata.issues.join(', ')}</span>{/if}
    {#if item.summary?.kind === 'project'}
      {#if !item.summary.available}<span class="home-unavailable">Summary unavailable</span>
      {:else if item.summary.total === 0}<span>No tasks</span>
      {:else}<span>{item.summary.pending} pending · {item.summary.overdue} vencidas · {item.summary.done}/{item.summary.total} completas</span><span class="home-progress"><progress max="100" value={item.summary.progress_percent ?? 0} aria-label={`Progress for ${item.entity.title}`}></progress>{item.summary.progress_percent}%</span>{/if}
    {:else if item.summary?.kind === 'tag'}
      {#if !item.summary.available}<span class="home-unavailable">Summary unavailable</span>
      {:else}<span>{item.summary.note_count} notes</span>{#if lastNote}<button class="home-context" onclick={() => void onopen(lastNote!.id)}>Latest note: {lastNote.title}</button>{/if}{/if}
    {/if}
    {#if !taskStyle && !noteStyle}{#each contexts as context (context.id)}<button class="home-context" onclick={() => void onopen(context.id)}><AppIcon name={visualIconName(context.icon, context.kind === 'project' ? 'project' : context.kind === 'person' ? 'person' : 'tag')} color={context.icon_color ?? undefined} size={14} />{context.title}{context.archived_at ? " · archived" : ''}</button>{/each}{/if}
    {#if item.context_total > item.context.length}<span>+{item.context_total - item.context.length} references when opening the original</span>{/if}
  </div>
</article>
