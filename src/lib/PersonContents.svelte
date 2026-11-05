<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import { entityIcon, entityIconColor } from './entity-appearance';
  import type { Entity } from './contracts.generated';
  let { items, hasMore = false, busy = false, stale = false, notice = '', error = '', createDisabled = false, onopen, oncreate, onmore, onretry }: {
    items: Entity[]; hasMore?: boolean; busy?: boolean; stale?: boolean; notice?: string; error?: string; createDisabled?: boolean;
    onopen: (entity: Entity) => Promise<void>; oncreate: (kind: 'note' | 'task') => void;
    onmore: () => void; onretry: () => void;
  } = $props();
  const sections = $derived([
    { key:'note' as const, label:'Notes', items:items.filter(item => item.kind === 'note' && !(item.payload as Record<string,unknown>)?.goal && !(item.payload as Record<string,unknown>)?.milestone) },
    { key:'project' as const, label:'Projects', items:items.filter(item => item.kind === 'project') },
    { key:'task' as const, label:'Tasks', items:items.filter(item => item.kind === 'task') },
  ]);
  const states: Record<string,string> = { planned:'Planned',doing:'Doing',ongoing:'Ongoing',on_hold:'On Hold',done:'Done',todo:'To Do' };
</script>

<div class="person-contents" aria-busy={busy}>
  {#if busy}<p class="person-context-message" role="status">Loading relations…</p>{/if}
  {#if stale}<p class="person-context-message" role="status">{busy ? 'Updating; showing previous results.' : 'Showing previous results; they may be out of date.'}</p>{/if}
  {#if notice && !busy}<p class="person-context-message" role="status">{notice}</p>{/if}
  {#if error}<p class="error" role="alert">{error}</p><button onclick={onretry}>Retry relations</button>{/if}
  {#each sections as section (section.key)}
    <section class="person-content-section" aria-label={`${section.label} for this person`}>
      <header><h2>{section.label}</h2>{#if section.key !== 'project'}<button class="primary glass-action" aria-label={section.key === 'note' ? "Capture a note here" : "Capture a task here"} disabled={createDisabled} onclick={() => oncreate(section.key)}><AppIcon name="plus" size={16} />New</button>{/if}</header>
      <div class="person-content-view"><span><AppIcon name="list" size={15} />List</span></div>
      {#each section.items as item (item.id)}{@const payload = item.payload as Record<string,unknown>}<button class="person-content-row" onclick={() => void onopen(item)}><AppIcon name={entityIcon(item.kind,item.title,item.payload)} color={entityIconColor(item.payload)} size={17} /><span>{item.title}</span>{#if typeof payload?.state === 'string'}<small class="property-chip">{states[payload.state] ?? payload.state}</small>{/if}</button>{/each}
      {#if !busy && !error && !section.items.length}<p class="person-context-message">{hasMore ? `No ${section.label} in the loaded relations. You can load more.` : `No related ${section.label}.`}</p>{/if}
      {#if section.key !== 'project'}<button class="person-add-row" disabled={createDisabled} onclick={() => oncreate(section.key)}><AppIcon name="plus" size={16} />{section.key === 'note' ? 'New note' : 'New task'}</button>{/if}
    </section>
  {/each}
  {#if hasMore}<div class="person-context-footer"><span>Sections show loaded relations.</span><button disabled={busy || stale} onclick={onmore}><AppIcon name="arrow-down" size={15} />More related items</button></div>{/if}
</div>
