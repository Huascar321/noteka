<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import type { HomeItem } from './contracts.generated';
  let { item, onopen }: { item: HomeItem; onopen: (id: string) => Promise<void> } = $props();
  const summary = $derived(item.summary?.kind === 'project' ? item.summary : undefined);
  const progress = $derived(summary?.progress_tenths);
  const references = $derived(item.context.filter(ref => ref.kind === 'tag' || ref.kind === 'note'));
  function percentage(value: number) { return `${value % 10 === 0 ? value / 10 : (value / 10).toFixed(1)}%`; }
</script>

<div class="person-project-row" data-project-id={item.entity.id}>
  <button class="person-project-open" onclick={() => void onopen(item.entity.id)}><AppIcon name={item.entity.icon ?? 'project'} color={item.entity.icon_color ?? undefined} size={19} /><span>{item.entity.title}</span></button>
  <div class="person-project-meta">{#each references as ref (ref.id)}<button class="person-project-reference" onclick={() => void onopen(ref.id)}><AppIcon name={ref.icon ?? (ref.kind === 'tag' ? 'tag' : 'trophy')} color={ref.icon_color ?? undefined} size={14} /><span>{ref.title}</span></button>{/each}
    {#if summary?.available && progress != null}<span class="person-project-progress"><span>{percentage(progress)}</span><progress max="1000" value={progress} aria-label={`Progress for ${item.entity.title}`}>{percentage(progress)}</progress></span>
    {:else if summary?.available}<span class="metadata">No tasks</span>{:else}<span class="home-unavailable">Summary unavailable</span>{/if}
  </div>
</div>
