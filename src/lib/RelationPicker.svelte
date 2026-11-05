<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { operate } from './api';
  import AppIcon from './AppIcon.svelte';
  import { entityIcon, entityIconColor } from './entity-appearance';
  import type { Contract, Entity, Relation, RelationKind } from './contracts.generated';

  let { workspace, sourceId, kind, anchor, relationships, busy, error, onadd, onremove, onclose }: {
    workspace: string; sourceId: string; kind: RelationKind; anchor: HTMLElement;
    relationships: { relation: Relation; other: Entity }[]; busy: boolean; error: string;
    onadd: (entity: Entity, kind: RelationKind) => Promise<void>;
    onremove: (relation: Relation) => Promise<boolean>; onclose: () => void;
  } = $props();
  const labels = { project: 'projects', tag: 'tags', person: 'people' };
  const kindLabel = $derived(labels[kind]);
  let panel: HTMLDivElement;
  let input: HTMLInputElement;
  let query = $state('');
  let results = $state<Entity[]>([]);
  let cursor = $state<string | null>(null);
  let started = $state(false);
  let loading = $state(false);
  let loadError = $state('');
  let generation = 0;
  let position = $state({ left: 12, top: 12, width: 300, maxHeight: 400 });
  const normalized = (text: string) => text.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLocaleLowerCase().trim();
  const related = $derived(relationships.filter(item => item.other.kind === kind));
  const selectedResults = $derived([...new Map(related.filter(item => !query.trim() || normalized(item.other.title).includes(normalized(query))).map(item => [item.other.id, item.other])).values()]);
  const visible = $derived([...selectedResults, ...results.filter(item => !selectedResults.some(selected => selected.id === item.id))]);
  function linked(entity: Entity) { return related.filter(item => item.other.id === entity.id); }
  function positionPanel() {
    if (!anchor.isConnected) { onclose(); return; }
    const rect = anchor.getBoundingClientRect(), width = Math.min(320, Math.max(180, innerWidth - 24));
    const height = Math.min(panel?.offsetHeight || 300, innerHeight - 24);
    const top = rect.bottom + 6 + height <= innerHeight - 12 ? rect.bottom + 6 : Math.max(12, rect.top - height - 6);
    position = { width, left: Math.max(12, Math.min(rect.left, innerWidth - width - 12)), top, maxHeight: Math.max(120, innerHeight - top - 12) };
  }
  async function load(reset: boolean) {
    const token = ++generation, term = normalized(query);
    let next = reset ? null : cursor;
    const visited = new Set<string>();
    if (!reset && (!started || !next)) return;
    if (reset) { results = []; cursor = null; started = false; }
    loading = true; loadError = '';
    try {
      do {
        if (next && visited.has(next)) throw new Error('Could not continue loading relations.');
        if (next) visited.add(next);
        const page: Contract['entities'] = await operate({ operation: 'entity.list', input: { kind, archived: false, cursor: next, limit: 100 } }, workspace);
        if (token !== generation) return;
        const matches = page.items.filter(entity => entity.id !== sourceId && (!term || normalized(entity.title).includes(term)));
        results = [...results, ...matches.filter(entity => !results.some(existing => existing.id === entity.id))];
        next = page.next_cursor ?? null; cursor = next; started = true;
        if (matches.length || !term || !next) break;
      } while (token === generation);
    } catch (e) {
      if (token === generation) { cursor = next; started = true; loadError = e instanceof Error ? e.message : 'Could not load relations.'; }
    } finally { if (token === generation) loading = false; }
  }
  async function toggle(entity: Entity) {
    if (busy) return;
    const links = linked(entity);
    if (links.length) { for (const link of links) if (!await onremove(link.relation)) break; }
    else if (!entity.archived_at) await onadd(entity, kind);
    await tick();
    if (panel?.isConnected) input?.focus();
  }
  function close() { const target = anchor; onclose(); void tick().then(() => { if (target.isConnected) target.focus(); }); }
  function keys(event: KeyboardEvent) {
    if (event.isComposing) return;
    if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); close(); return; }
    if (!['ArrowDown', 'ArrowUp'].includes(event.key)) return;
    const items = [...panel.querySelectorAll<HTMLButtonElement>('.relation-picker-item:not(:disabled)')];
    if (!items.length) return;
    event.preventDefault();
    const current = items.indexOf(document.activeElement as HTMLButtonElement);
    items[current < 0 ? (event.key === 'ArrowDown' ? 0 : items.length - 1) : (current + (event.key === 'ArrowDown' ? 1 : -1) + items.length) % items.length]?.focus();
  }
  function outside(event: PointerEvent) { if (panel && !panel.contains(event.target as Node) && !anchor.contains(event.target as Node)) onclose(); }
  $effect(() => {
    query; kind; sourceId; workspace;
    const timer = setTimeout(() => void load(true), 120);
    return () => { clearTimeout(timer); generation++; };
  });
  onMount(() => {
    void tick().then(() => { positionPanel(); input?.focus(); });
    const scroll = (event: Event) => { if (!(event.target instanceof Node && panel?.contains(event.target))) positionPanel(); };
    document.addEventListener('scroll', scroll, true);
    return () => { generation++; document.removeEventListener('scroll', scroll, true); };
  });
</script>

<svelte:window onresize={positionPanel} onpointerdown={outside} onkeydown={event => { if (event.key === 'Escape' && !event.defaultPrevented && !event.isComposing) { event.preventDefault(); close(); } }} />
<div bind:this={panel} class="relation-picker" role="dialog" tabindex="-1" aria-label={`Edit ${kindLabel}`} onkeydown={keys} style:left={`${position.left}px`} style:top={`${position.top}px`} style:width={`${position.width}px`} style:max-height={`${position.maxHeight}px`}>
  <div class="relation-picker-search"><AppIcon name="search" size={16} /><input bind:this={input} bind:value={query} aria-label={`Search ${kindLabel}`} placeholder={`Search ${kindLabel}…`} maxlength="256" autocomplete="off" /><button type="button" class="icon-button" aria-label="Close relations" onclick={close}><AppIcon name="close" size={14} /></button></div>
  {#if error}<p class="relation-picker-message error" role="alert">{error}</p>{/if}
  <div class="relation-picker-results" aria-label={`${kindLabel} results`} aria-busy={loading}>
    {#each visible as entity (entity.id)}
      <button type="button" class="relation-picker-item" aria-pressed={linked(entity).length > 0} disabled={busy} onclick={() => void toggle(entity)}>
        <AppIcon name={entityIcon(entity.kind, entity.title, entity.payload)} color={entityIconColor(entity.payload)} size={16} /><span>{entity.title || 'Untitled'}{#if entity.archived_at}<small>Archived</small>{/if}</span>{#if linked(entity).length}<span aria-hidden="true">✓</span>{/if}
      </button>
    {/each}
  </div>
  {#if loading}<p class="relation-picker-message" role="status">Searching…</p>{:else if loadError}<p class="relation-picker-message error" role="alert">{loadError}</p><button type="button" class="relation-picker-more" onclick={() => void load(!cursor)}>Retry</button>{:else if started && !visible.length && !cursor}<p class="relation-picker-message">No {kindLabel} found.</p>{/if}
  {#if !loading && cursor}<button type="button" class="relation-picker-more" onclick={() => void load(false)}>Load more {kindLabel}</button>{/if}
</div>
