<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import type { HomeItem } from './contracts.generated';
  let { item, onopen, showDeadline = false, showState = false }: { item: HomeItem; onopen: (id: string) => Promise<void>; showDeadline?: boolean; showState?: boolean } = $props();
  const stateLabels: Record<string, string> = { planned: 'Planned', doing: 'Doing', ongoing: 'Ongoing', done: 'Done', on_hold: 'On Hold' };
  const summary = $derived(item.summary?.kind === 'project' ? item.summary : undefined);
  const progress = $derived(summary?.progress_tenths);
  const references = $derived(item.context.filter(ref => ref.kind === 'tag' || ref.kind === 'note').sort((a,b) => Number(a.kind !== 'tag') - Number(b.kind !== 'tag')));
  function percentage(value: number) { return `${value % 10 === 0 ? value / 10 : (value / 10).toFixed(1)}%`; }
  function dateLabel(value: string) { return new Intl.DateTimeFormat('en-US', { month: 'long', day: 'numeric', year: 'numeric', timeZone: 'UTC' }).format(new Date(`${value}T12:00:00Z`)); }
</script>

<article class="task-page-project-row" data-project-id={item.entity.id}>
  <button class="task-page-project-open" onclick={() => void onopen(item.entity.id)}><AppIcon name={item.entity.icon ?? 'project'} color={item.entity.icon_color ?? undefined} size={19} /><span>{item.entity.title}</span></button>
  <div class="task-page-project-meta">
    {#if showState && item.metadata.state}<span class="project-row-state" data-project-state={item.metadata.state}>{stateLabels[item.metadata.state] ?? item.metadata.state}</span>{/if}
    {#each references as ref (ref.id)}<button class="task-page-project-reference" onclick={() => void onopen(ref.id)} title={ref.title}><AppIcon name={ref.icon ?? (ref.kind === 'tag' ? 'tag' : 'trophy')} color={ref.icon_color ?? undefined} size={18} /><span>{ref.title}</span></button>{/each}
    {#if showDeadline && item.metadata.project_end_date}<time datetime={item.metadata.project_end_date}>{dateLabel(item.metadata.project_end_date)}</time>{/if}
    {#if summary?.available && progress != null}
      <span class="task-page-project-status">{#if summary.pending === 0}<span class="project-all-done">All Tasks Done</span>{:else}<span class="project-active-count">{summary.pending} Active</span>{#if (summary.overdue ?? 0) > 0}<span class="project-status-separator" aria-hidden="true">-</span><span class="project-overdue-count">{summary.overdue} Overdue</span>{/if}{/if}</span>
      <span class="task-page-project-progress"><span>{percentage(progress)}</span><progress max="1000" value={progress} aria-label={`Progress for ${item.entity.title}`}>{percentage(progress)}</progress></span>
    {:else if summary?.available}<span class="metadata">No tasks</span>{:else}<span class="home-unavailable">Summary unavailable</span>{/if}
    {#if item.metadata.issues.length}<span class="home-unavailable">Metadata unavailable: {item.metadata.issues.join(', ')}</span>{/if}
  </div>
</article>
