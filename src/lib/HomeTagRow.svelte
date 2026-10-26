<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import type { HomeItem } from './contracts.generated';
  let { item, onopen, showType = false, showCount = true }: { item: HomeItem; onopen: (id: string) => Promise<void>; showType?: boolean; showCount?: boolean } = $props();
  const summary = $derived(item.summary?.kind === 'tag' ? item.summary : undefined);
  const latest = $derived(summary?.last_note);
  const labels: Record<string, string> = { area: 'Area', resource: 'Resource', entity: 'Entity', topic: 'Topic' };
</script>

<article class="home-tag-row" data-entity-id={item.entity.id}>
  <button class="tag-row-open" onclick={() => void onopen(item.entity.id)} aria-label={`Open ${item.entity.title}`}><AppIcon name={item.entity.icon ?? 'tag'} color={item.entity.icon_color ?? undefined} size={19} /><span>{item.entity.title}</span></button>
  <div class="tag-row-meta">
    {#if latest}<button class="tag-latest-note" onclick={() => void onopen(latest!.id)} title={latest.title} aria-label={`Open latest note: ${latest.title}`}><AppIcon name={latest.icon ?? 'file-text'} color={latest.icon_color ?? undefined} size={19} /><span>{latest.title}</span></button>{/if}
    {#if showType && item.metadata.classification}<span class="tag-type-pill" data-classification={item.metadata.classification}>{labels[item.metadata.classification] ?? item.metadata.classification}</span>{/if}
    {#if summary?.available}{#if showCount}<span class="tag-note-count" aria-label={`${summary.note_count} related notes`}>{summary.note_count} {summary.note_count === 1 ? 'note' : 'notes'}</span>{/if}{:else}<span class="home-unavailable">Summary unavailable</span>{/if}
  </div>
  {#if item.metadata.issues.length}<p class="home-unavailable">Metadata unavailable: {item.metadata.issues.join(', ')}</p>{/if}
</article>
