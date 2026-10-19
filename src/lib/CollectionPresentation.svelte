<script lang="ts">
  import type { DatabasePage, Entity, RecordData } from './contracts.generated';
  let { page, busy = false, onopen, onedit }: { page: DatabasePage; busy?: boolean; onopen: (entity: Entity) => Promise<void>; onedit?: (record: RecordData) => Promise<void> } = $props();
  const columns = $derived(page.config.columns?.length ? page.config.columns.flatMap(id => page.properties.filter(p => p.id === id)) : page.properties);
  const records = $derived(page.records.items);
  const recordsById = $derived(new Map(records.map(r => [r.entity.id, r])));
  function cell(r: RecordData, id: string) { const value = r.values[id]; return value == null ? '—' : typeof value === 'string' ? value : JSON.stringify(value); }
</script>

<div class="collection-presentation" data-layout={page.layout}>
  <p class="metadata" aria-label="Cobertura de página">Página actual: {records.length} de {page.matching_count} registros coincidentes · Grupos/días cubren sólo esta página.</p>
  {#if page.layout === 'table'}<div class="collection-table"><table><thead><tr>{#each columns as p (p.id)}<th>{p.spec.name}</th>{/each}<th>Original</th></tr></thead><tbody>{#each records as r (r.entity.id)}<tr data-entity-id={r.entity.id}>{#each columns as p (p.id)}<td>{@render value(r, p.id)}</td>{/each}<td>{@render actions(r)}</td></tr>{/each}</tbody></table></div>
  {:else if page.layout === 'list'}<div class="collection-records">{#each records as r (r.entity.id)}<article data-entity-id={r.entity.id}>{@render actions(r)}<p>{r.entity.id} · r{r.entity.revision}</p><dl>{#each columns as p (p.id)}<dt>{p.spec.name}</dt><dd>{@render value(r, p.id)}</dd>{/each}</dl></article>{/each}</div>
  {:else if page.layout === 'gallery'}<div class="collection-gallery" aria-label="Galería de registros">{#each records as r (r.entity.id)}{@render card(r)}{/each}</div>
  {:else}<div class:collection-board={page.layout === 'board'} class:collection-calendar={page.layout === 'calendar'} aria-label={page.layout === 'board' ? 'Tablero de registros' : 'Calendario civil'}>{#each page.groups as group, index (index)}<section class="collection-group" data-group-key={JSON.stringify(group.key)}><h3>{group.label} <span class="metadata">{group.entity_ids.length}</span></h3>{#each group.entity_ids as id (id)}{@const r = recordsById.get(id)}{#if r}{@render card(r)}{/if}{/each}{#if !group.entity_ids.length}<p class="metadata">Sin registros en esta página.</p>{/if}</section>{/each}</div>{/if}
  {#if !records.length}<p class="metadata">Sin registros para esta vista.</p>{/if}
</div>

{#snippet value(r: RecordData, id: string)}{#if r.calculation_errors[id]}<span class="error" role="alert">{r.calculation_errors[id].code}: {r.calculation_errors[id].message}</span>{:else}{cell(r, id)}{/if}{/snippet}
{#snippet actions(r: RecordData)}<button disabled={busy} onclick={() => void onopen(r.entity)}>Abrir {r.entity.title}</button>{#if onedit}<button disabled={busy} onclick={() => void onedit?.(r)}>Editar valores {r.entity.title}</button>{/if}{/snippet}
{#snippet card(r: RecordData)}<article class="collection-card" data-entity-id={r.entity.id}>{@render actions(r)}<dl>{#each columns as p (p.id)}<dt>{p.spec.name}</dt><dd>{@render value(r, p.id)}</dd>{/each}</dl></article>{/snippet}

<style>
  .collection-table { overflow-x: auto; } table { width: 100%; border-collapse: collapse; } th, td { padding: 8px; border-bottom: 1px solid var(--border); text-align: left; }
  .collection-gallery { display: grid; grid-template-columns: repeat(auto-fit, minmax(230px, 1fr)); gap: 12px; }
  .collection-board { display: flex; gap: 12px; overflow-x: auto; align-items: flex-start; }
  .collection-board .collection-group { flex: 0 0 270px; }
  .collection-calendar { display: grid; grid-template-columns: repeat(auto-fit, minmax(190px, 1fr)); gap: 8px; }
  .collection-group { border: 1px solid var(--border); border-radius: 8px; padding: 10px; min-width: 0; }
  .collection-group h3 { font-size: 14px; margin: 0 0 10px; }
  .collection-card { border: 1px solid var(--border); border-radius: 8px; padding: 12px; margin-bottom: 8px; background: var(--surface); overflow-wrap: anywhere; }
  dl { font-size: 13px; margin: 12px 0; } dt { color: var(--muted); } dd { margin: 0 0 8px; }
</style>
