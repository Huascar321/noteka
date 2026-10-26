<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import type { HomeItem } from './contracts.generated';
  let { items, onopen }: { items: HomeItem[]; onopen: (id: string) => Promise<void> } = $props();
  const labels: Record<string, string> = { area: 'Area', resource: 'Resource', entity: 'Entity', topic: 'Topic' };
</script>

<div class="tag-table-scroll" role="region" aria-label="Tags table, horizontal scrolling">
  <table class="tag-table" aria-label="Tags">
    <thead><tr><th scope="col">Tag</th><th scope="col">Last Note</th><th scope="col">Type</th></tr></thead>
    <tbody>
      {#each items as item (item.entity.id)}
        {@const summary = item.summary?.kind === 'tag' ? item.summary : undefined}
        {@const latest = summary?.last_note}
        <tr data-entity-id={item.entity.id}>
          <td><button class="tag-row-open" onclick={() => void onopen(item.entity.id)} title={item.entity.title} aria-label={`Open ${item.entity.title}`}><AppIcon name={item.entity.icon ?? 'tag'} color={item.entity.icon_color ?? undefined} size={19} /><span>{item.entity.title}</span></button>{#if item.metadata.issues.length}<p class="home-unavailable">Metadata unavailable: {item.metadata.issues.join(', ')}</p>{/if}</td>
          <td>{#if latest}<button class="tag-latest-note" onclick={() => void onopen(latest!.id)} title={latest.title} aria-label={`Open latest note: ${latest.title}`}><AppIcon name={latest.icon ?? 'file-text'} color={latest.icon_color ?? undefined} size={19} /><span>{latest.title}</span></button>{:else if !summary?.available}<span class="home-unavailable">Summary unavailable</span>{:else}<span class="tag-table-empty" aria-label="No latest note">—</span>{/if}</td>
          <td>{#if item.metadata.classification}<span class="tag-type-pill" data-classification={item.metadata.classification}><span class="tag-type-dot" aria-hidden="true"></span>{labels[item.metadata.classification] ?? item.metadata.classification}</span>{/if}</td>
        </tr>
      {/each}
      {#if !items.length}<tr><td colspan="3" class="home-empty">No Tags in this view.</td></tr>{/if}
    </tbody>
  </table>
</div>
