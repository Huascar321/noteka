<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { operate, OperationError } from './api';
  import { ReadQueue, ReadLifetime } from './embedded-reads';
  import CollectionPresentation from './CollectionPresentation.svelte';
  import type { DatabasePage, Entity } from './contracts.generated';
  let { workspace, collectionId, viewId, queue, lifetime, onopen }: {
    workspace: string; collectionId: string; viewId: string; queue: ReadQueue; lifetime: ReadLifetime; onopen: (entity: Entity) => Promise<void>;
  } = $props();
  let element: HTMLElement;
  let page = $state<DatabasePage | null>(null), offset = $state<string | undefined>();
  let busy = $state(false), error = $state(''), attempted = false;
  async function load(cursor?: string) {
    if (busy) return;
    const focused = document.activeElement instanceof HTMLButtonElement && element?.contains(document.activeElement) ? document.activeElement : null;
    const { token, signal } = lifetime.begin(); busy = true; error = ''; attempted = true;
    try {
      const result = await queue.run(() => operate<DatabasePage>({ operation: 'database.query', input: { collection_id: collectionId, view_id: viewId, limit: 50, cursor } }, workspace), signal);
      if (!lifetime.current(token)) return;
      page = result; offset = cursor;
    } catch (e) {
      if (!lifetime.current(token)) return;
      page = null; offset = undefined;
      error = e instanceof OperationError ? `${e.detail.code}: ${e.message}` : String(e);
    } finally {
      if (lifetime.current(token)) {
        busy = false;
        await tick();
        // Disabled retira el foco del botón nativo mientras consulta. Restaurar
        // sólo ese foco perdido, no robarlo de otro control/navegación.
        if (lifetime.current(token) && focused?.isConnected && !focused.disabled && document.activeElement === document.body) focused.focus();
      }
    }
  }
  onMount(() => {
    const observer = new IntersectionObserver(entries => { if (!attempted && entries.some(e => e.isIntersecting)) void load(); });
    observer.observe(element);
    return () => { observer.disconnect(); lifetime.dispose(); };
  });
</script>

<section bind:this={element} class="embedded-collection" aria-label="Vista local incrustada" data-view-id={viewId} aria-busy={busy}>
  <h3>Vista local incrustada</h3><p class="metadata">Colección {collectionId} · vista {viewId} · lectura del original, no snapshot guardado.</p>
  <button disabled={busy} onclick={() => void load()}>{busy ? 'Consultando vista…' : page ? 'Actualizar vista incrustada' : 'Cargar vista incrustada'}</button>
  {#if error}<p class="error" role="alert">{error}</p>{/if}
  {#if page}<CollectionPresentation {page} {busy} {onopen} />{#if page.records.next_cursor}<button disabled={busy} onclick={() => void load(page?.records.next_cursor ?? undefined)}>Página siguiente incrustada</button>{/if}{#if offset}<button disabled={busy} onclick={() => void load()}>Inicio de vista incrustada</button>{/if}{/if}
</section>

<style>.embedded-collection { border: 1px solid var(--border); padding: 12px; margin: 12px 0; border-radius: 8px; overflow-wrap: anywhere; } h3 { margin: 0; }</style>
