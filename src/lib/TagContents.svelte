<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import { entityIcon, entityIconColor } from './entity-appearance';
  import type { Entity } from './contracts.generated';
  let { items, hasMore = false, busy = false, error = '', createDisabled = false, onopen, oncreate, onmore, onretry }: {
    items: Entity[]; hasMore?: boolean; busy?: boolean; error?: string; createDisabled?: boolean;
    onopen: (entity: Entity) => Promise<void>; oncreate: (kind: 'note' | 'clip' | 'task') => void;
    onmore: () => void; onretry: () => void;
  } = $props();
  const noteItems = $derived(items.filter(item => item.kind === 'note' && !(item.payload as Record<string,unknown>)?.goal && !(item.payload as Record<string,unknown>)?.milestone));
  const sections = $derived([
    { key: 'note' as const, label:'Notes', items:noteItems.filter(item => (item.payload as Record<string,unknown>)?.note_type !== 'web_clip') },
    { key: 'clip' as const, label:'Clips', items:noteItems.filter(item => (item.payload as Record<string,unknown>)?.note_type === 'web_clip') },
  ]);
  const tasks = $derived(items.filter(item => item.kind === 'task'));
  const types: Record<string,string> = { reference:'Reference', plan:'Plan', web_clip:'Web Clip', voice:'Voice', journal:'Journal', meeting:'Meeting', topic_research:'Topic Research', idea:'Idea', lecture:'Lecture' };
</script>

<div class="tag-contents" aria-busy={busy}>
  {#if busy}<p role="status" class="tag-context-message">Loading relations…</p>{/if}
  {#if error}<p role="alert" class="error">{error}</p><button onclick={onretry}>Retry relations</button>{/if}
  {#each sections as section (section.key)}
    <section aria-label={`${section.label} for this tag`} class="tag-content-section" data-tag-notes={section.key === 'note' ? '' : undefined}>
      <header><h2 tabindex="-1">{section.label}</h2><button class="primary" aria-label={section.key === 'note' ? "New note in this tag" : "New clip in this tag"} disabled={createDisabled} onclick={() => oncreate(section.key)}><AppIcon name="plus" size={15} />New</button></header>
      <div class="tag-content-view"><span><AppIcon name={section.key === 'note' ? 'file-text' : 'globe'} size={15} />List</span></div>
      {#each section.items as item (item.id)}{@const payload = item.payload as Record<string,unknown>}<button class="tag-content-row" onclick={() => void onopen(item)}><AppIcon name={entityIcon(item.kind,item.title,item.payload)} color={entityIconColor(item.payload)} size={17} /><span>{item.title}</span>{#if payload?.favorite === true}<span class="tag-note-favorite" role="img" aria-label="Favorite note"><AppIcon name="star" size={14} /></span>{/if}{#if typeof payload?.note_type === 'string'}<small class="property-chip">{types[payload.note_type] ?? payload.note_type}</small>{/if}</button>{/each}
      {#if !busy && !error && !section.items.length}<p class="tag-context-message">{hasMore ? `No ${section.label} in the loaded relations. You can load more.` : `No related ${section.label}.`}</p>{/if}
      <button class="tag-add-row" disabled={createDisabled} onclick={() => oncreate(section.key)}><AppIcon name="plus" size={15} />{section.key === 'note' ? 'New note' : 'New clip'}</button>
    </section>
  {/each}
  <details class="tag-task-context" open={tasks.length > 0}>
    <summary>Tasks <span>{tasks.length}</span></summary>
    {#each tasks as item (item.id)}<button class="tag-content-row" onclick={() => void onopen(item)}><AppIcon name={entityIcon(item.kind,item.title,item.payload)} color={entityIconColor(item.payload)} size={17} /><span>{item.title}</span></button>{/each}
    {#if !busy && !error && !tasks.length}<p class="tag-context-message">{hasMore ? "No Tasks in the loaded relations." : "No related Tasks."}</p>{/if}
    <button class="tag-add-row" disabled={createDisabled} onclick={() => oncreate('task')}><AppIcon name="plus" size={15} />Capture a task here</button>
  </details>
  {#if hasMore}<div class="tag-context-footer"><span>Sections show loaded relations.</span><button disabled={busy} onclick={onmore}><AppIcon name="arrow-down" size={15} />Load more relations</button></div>{/if}
</div>
