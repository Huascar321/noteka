<script lang="ts">
  import type { HomeItem } from './contracts.generated';
  let { item, onopen }: { item: HomeItem; onopen: (id: string) => Promise<void> } = $props();
  const lastNote = $derived(item.summary?.kind === 'tag' ? item.summary.last_note : null);
</script>

<article class="home-item" data-entity-id={item.entity.id}>
  <button class="home-open" onclick={() => void onopen(item.entity.id)} aria-label={`Abrir ${item.entity.title}`}>
    <span class="home-kind" aria-hidden="true">{({ task: '☑', note: '▤', project: '◈', tag: '#', person: '♙' })[item.entity.kind]}</span>
    <span>{item.entity.title}</span>
  </button>
  <div class="home-item-meta">
    {#if item.metadata.state}<span class="home-badge">{item.metadata.state}</span>{/if}
    {#if item.metadata.classification}<span class="home-badge">{item.metadata.classification}</span>{/if}
    {#if item.metadata.due_date}<time datetime={item.metadata.due_date}>Vence {item.metadata.due_date}</time>{/if}
    {#if item.entity.kind === 'note'}
      <span>{item.metadata.updated_days_ago == null ? 'Antigüedad no disponible' : item.metadata.updated_days_ago < 0 ? 'Modificación en fecha futura' : `Actualizada hace ${item.metadata.updated_days_ago} días`}</span>
    {/if}
    {#if item.metadata.issues.length}<span class="home-unavailable">Metadatos no disponibles: {item.metadata.issues.join(', ')}</span>{/if}
    {#if item.summary?.kind === 'project'}
      {#if !item.summary.available}<span class="home-unavailable">Resumen no disponible</span>
      {:else if item.summary.total === 0}<span>Sin tareas</span>
      {:else}<span>{item.summary.pending} pendientes · {item.summary.overdue} vencidas · {item.summary.done}/{item.summary.total} completas</span><span class="home-progress"><progress max="100" value={item.summary.progress_percent ?? 0} aria-label={`Progreso de ${item.entity.title}`}></progress>{item.summary.progress_percent}%</span>{/if}
    {:else if item.summary?.kind === 'tag'}
      {#if !item.summary.available}<span class="home-unavailable">Resumen no disponible</span>
      {:else}<span>{item.summary.note_count} notas</span>{#if lastNote}<button class="home-context" onclick={() => void onopen(lastNote!.id)}>Última nota: {lastNote.title}</button>{/if}{/if}
    {/if}
    {#each item.context as context (context.id)}<button class="home-context" onclick={() => void onopen(context.id)}>{context.title}{context.archived_at ? ' · archivado' : ''}</button>{/each}
    {#if item.context_total > item.context.length}<span>+{item.context_total - item.context.length} referencias al abrir original</span>{/if}
  </div>
</article>
