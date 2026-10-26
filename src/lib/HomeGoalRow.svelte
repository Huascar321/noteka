<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import { goalDate } from './goal-dates';
  import type { HomeItem } from './contracts.generated';
  let { item, timezone, achieved = false, card = false, onopen }: {
    item: HomeItem; timezone: string; achieved?: boolean; card?: boolean; onopen: (id: string) => Promise<void>;
  } = $props();
  const goal = $derived(item.metadata.goal);
  const tag = $derived(item.context.find(ref => ref.kind === 'tag'));
  const summary = $derived(item.summary?.kind === 'goal' ? item.summary : undefined);
  const progress = $derived(summary?.progress_tenths);
  const date = $derived(achieved ? goal?.achieved_on : card ? goal?.target_deadline : item.metadata.goal_activity_at);
</script>

<article class="goal-row" class:goal-card={card} data-entity-id={item.entity.id}>
  <button class="goal-row-open" title={item.entity.title} aria-label={`Open ${item.entity.title}`} onclick={() => void onopen(item.entity.id)}><AppIcon name={item.entity.icon ?? 'trophy'} color={item.entity.icon_color ?? undefined} size={19} /><span>{item.entity.title}</span></button>
  <div class="goal-row-meta">
    {#if tag && !card}<button class="goal-row-tag" title={tag.title} onclick={() => void onopen(tag!.id)}><AppIcon name={tag.icon ?? 'tag'} color={tag.icon_color ?? undefined} size={19} /><span>{tag.title}</span>{#if tag.archived_at}<span class="sr-only">Archived</span>{/if}</button>{/if}
    {#if summary?.available}
      {#if progress !== null && progress !== undefined}<span class="goal-row-progress"><span>{progress % 10 ? (progress / 10).toFixed(1) : progress / 10}%</span><progress max="1000" value={progress} aria-label={`Progress for ${item.entity.title}: ${progress / 10}%`}></progress></span>{:else}<span class="goal-no-progress">No milestones</span>{/if}
    {:else}<span class="home-unavailable">Progress unavailable</span>{/if}
    {#if date}<time datetime={date} title={card ? 'Target Deadline' : achieved ? 'Achieved' : 'Latest Activity'}>{goalDate(date, timezone, achieved || card)}</time>{/if}
  </div>
  {#if item.metadata.issues.length}<p class="home-unavailable">Metadata unavailable: {item.metadata.issues.join(', ')}</p>{/if}
</article>
