<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import type { HomeItem } from './contracts.generated';
  let { item, onopen, movable = false, moving = false, dragging = false, dropBefore = false, onmove,
    onstartdrag, onenddrag, onover, ondropcard }: {
    item: HomeItem; onopen: (id: string) => Promise<void>; movable?: boolean; moving?: boolean;
    dragging?: boolean; dropBefore?: boolean; onmove?: () => void;
    onstartdrag?: (event: DragEvent) => void; onenddrag?: () => void;
    onover?: (event: DragEvent) => void; ondropcard?: (event: DragEvent) => void;
  } = $props();
  const summary = $derived(item.summary?.kind === 'project' ? item.summary : undefined);
  const progress = $derived(summary?.progress_tenths);
  const context = $derived(item.context.filter(ref => ref.kind === 'tag' || ref.kind === 'note').sort((a, b) => Number(a.kind !== 'tag') - Number(b.kind !== 'tag')));
  function percentage(tenths: number) { return tenths % 10 === 0 ? `${tenths / 10}%` : `${(tenths / 10).toFixed(1)}%`; }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions (Native drag/drop has a keyboard Move button.) -->
<article class="project-card" class:project-dragging={dragging} class:project-drop-before={dropBefore} data-project-id={item.entity.id}
  draggable={movable && !moving} ondragstart={onstartdrag} ondragend={onenddrag} ondragover={onover} ondrop={ondropcard}>
  {#if movable}<button class="project-move-trigger" aria-label={`Move ${item.entity.title}`} title="Move project" disabled={moving} onclick={() => onmove?.()}><AppIcon name="grip" size={15} /></button>{/if}
  <button class="project-card-open" disabled={moving} onclick={() => void onopen(item.entity.id)} aria-label={`Open ${item.entity.title}`}>
    <span class="project-card-title"><AppIcon name={item.entity.icon ?? 'project'} color={item.entity.icon_color ?? undefined} size={19} /><span>{item.entity.title}</span></span>
    {#if summary?.available && progress != null}
      <span class="project-card-progress"><span>{percentage(progress)}</span><progress max="1000" value={progress} aria-label={`Progress for ${item.entity.title}`}>{percentage(progress)}</progress></span>
      <span class="project-card-status">
        {#if summary.pending === 0}<span class="project-all-done">All Tasks Done</span>
        {:else}<span class="project-active-count">{summary.pending} Active</span>{#if (summary.overdue ?? 0) > 0}<span class="project-status-separator" aria-hidden="true">-</span><span class="project-overdue-count">{summary.overdue} Overdue</span>{/if}{/if}
      </span>
    {:else if summary?.available}<span class="project-card-empty">No tasks</span>
    {:else}<span class="home-unavailable">Summary unavailable</span>{/if}
  </button>
  {#if context.length}<div class="project-card-context">{#each context as ref (ref.id)}<button class:archived={ref.archived_at} disabled={moving} onclick={() => void onopen(ref.id)} title={ref.title}><AppIcon name={ref.icon ?? (ref.kind === 'note' ? 'trophy' : ref.kind)} color={ref.icon_color ?? undefined} size={14} /><span>{ref.title}</span></button>{/each}{#if item.context_total > context.length}<span class="metadata">+{item.context_total - context.length} referencias</span>{/if}</div>{/if}
  {#if item.metadata.issues.length}<p class="home-unavailable">Metadata unavailable: {item.metadata.issues.join(', ')}</p>{/if}
</article>
