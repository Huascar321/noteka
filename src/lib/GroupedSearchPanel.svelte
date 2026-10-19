<script lang="ts">
  import { onMount } from 'svelte';
  import { operate } from './api';
  import { GroupedTextSearch } from './grouped-text-search';
  import type { SearchHit } from './contracts.generated';
  let { workspace, onopen }: { workspace: string; onopen: (hit: SearchHit) => Promise<void> } = $props();
  let query = $state('');
  const controller = new GroupedTextSearch(command => operate(command, workspace), next => { view = next; });
  let view = $state(controller.state);
  const json = (value: unknown) => JSON.stringify(value, null, 2);
  export function refreshPanel() { return controller.search(query); }
  onMount(() => () => controller.dispose());
</script>

<div class="grouped-search" aria-label="Consulta conjunta local">
  <form onsubmit={event => { event.preventDefault(); void refreshPanel(); }}>
    <label>Texto conjunto<input aria-label="Texto conjunto" bind:value={query} oninput={() => controller.edit()} placeholder="1–16 términos · hasta 256 bytes UTF-8" /></label>
    <button disabled={!query.trim() || view.notes.busy || view.attachments.busy}>Buscar notas y adjuntos</button>
  </form>
  <p>Entidades activas y generaciones seleccionadas. Dos consultas independientes; las páginas no son un snapshot conjunto y sus scores/cursores no se fusionan. Para filtros de archivo/contexto o extractor, usá los ámbitos individuales.</p>
  <section class="grouped-notes" aria-label="Resultados de entidades activas">
    <h3>Notas y entidades activas</h3>
    {#if view.notes.busy}<p role="status">Consultando entidades…</p>
    {:else if view.notes.error}<div role="alert" class="error"><strong>{view.notes.error.code}</strong><p>{view.notes.error.message}</p><pre>{json(view.notes.error.details)}</pre><button onclick={() => void controller.startNotes()}>Reintentar entidades</button></div>
    {:else if view.notes.page}
      <p>Página {view.notes.pageNumber} · {view.notes.page.items.length} hits de entidades.</p>
      {#each view.notes.page.items as hit, i (`${hit.entity_id}:${hit.block_id ?? ''}:${i}`)}<article class="grouped-note-hit"><button onclick={() => void onopen(hit)}>{hit.title}</button><p>{hit.kind} · entidad {hit.entity_id} · revisión {hit.revision} · {hit.block_id ? `bloque ${hit.block_id}` : 'título'}</p><pre class="grouped-note-excerpt">{hit.excerpt}</pre><details><summary>Fuente de entidad</summary><pre>{json(hit)}</pre></details></article>{/each}
      {#if !view.notes.page.items.length}<p>Sin coincidencias entre entidades activas.</p>{/if}
      {#if view.notes.page.next_cursor}<button onclick={() => void controller.nextNotes()}>Siguiente página de entidades</button>{/if}<button onclick={() => void controller.startNotes()}>Inicio de entidades</button>
    {:else}<p>Todavía no se consultaron entidades.</p>{/if}
  </section>
  <section class="grouped-attachments" aria-label="Resultados de adjuntos seleccionados">
    <h3>Adjuntos seleccionados</h3>
    {#if view.attachments.busy}<p role="status">Consultando adjuntos…</p>
    {:else if view.attachments.error}<div role="alert" class="error"><strong>{view.attachments.error.code} · {view.attachments.error.reason ?? ''}</strong><p>{view.attachments.error.message}</p><button onclick={() => void controller.startAttachments()}>Reintentar adjuntos</button></div>
    {:else if view.attachments.page}
      <p>Página {view.attachments.pageNumber} · {view.attachments.page.items.length} fragmentos en esta página.</p>
      <details><summary>Cobertura de adjuntos</summary><pre class="grouped-coverage">{json({ ...view.attachments.page.coverage, profile: view.attachments.page.profile, version: view.attachments.page.version, revision: view.attachments.page.revision })}</pre></details>
      <p>Índice y postings verificados; originales sólo de candidatos de página/lookahead al consultar. La cobertura no es un total de hits ni prueba de frescura posterior o ausencia universal. Importar/extraer no selecciona generaciones.</p>
      {#each view.attachments.page.items as hit (`${hit.source.job_id}:${hit.source.generation_id}:${hit.ordinal}`)}<article class="grouped-attachment-hit"><h4>{hit.locator.kind} · fragmento {hit.ordinal}</h4>{#if hit.result_from_previous_attempt}<p>Resultado de un intento anterior.</p>{/if}<pre class="grouped-attachment-text">{hit.text}</pre><details><summary>Procedencia y localizador</summary><pre class="grouped-attachment-source">{json(hit.source)}</pre><pre class="grouped-attachment-locator">{json(hit.locator)}</pre></details><p>{hit.locator.kind === 'utf8_bytes' ? 'Bytes/líneas del original; final exclusivo, no índices JavaScript.' : hit.locator.kind === 'pdf_page_text' ? 'Página/bytes derivados; sin OCR, orden visual, geometría ni tablas estructuradas.' : hit.locator.kind === 'image_ocr_region_text' ? 'OCR inferido: puede faltar o ser incorrecto; sin orden visual ni tablas garantizados. Bbox de imagen orientada, origen inferior izquierdo; confianza no es probabilidad. Bytes del derivado, no del original.' : 'Campo XLSX derivado: fórmula y caché separadas, sin recálculo ni offsets del ZIP. El rango puede ser sólo una porción del campo.'}</p></article>{/each}
      {#if !view.attachments.page.items.length}<p>Sin coincidencias en generaciones seleccionadas. No prueba que el original esté visualmente vacío ni que haya OCR.</p>{/if}
      {#if view.attachments.page.next_cursor}<button onclick={() => void controller.nextAttachments()}>Siguiente página de adjuntos conjuntos</button>{/if}<button onclick={() => void controller.startAttachments()}>Inicio de adjuntos conjuntos</button>
    {:else}<p>Todavía no se consultaron adjuntos.</p>{/if}
  </section>
  <p>Errores y reintentos son por grupo. Buscar e inspeccionar no importa, encola, ejecuta, indexa, resetea ni repara. El contenido se presenta como datos, sin servicios remotos.</p>
</div>

<style>
  .grouped-search { min-width: 0; font-size: 12px; }
  label { display: grid; gap: .35rem; }
  input, button { max-width: 100%; }
  section, article { min-width: 0; border-top: 1px solid var(--border); padding-top: .7rem; margin-top: 1rem; }
  pre, p, h4, summary { white-space: pre-wrap; overflow-wrap: anywhere; word-break: break-word; }
  pre { max-height: 24rem; overflow: auto; font: inherit; line-height: 1.6; }
  summary { cursor: pointer; }
</style>
