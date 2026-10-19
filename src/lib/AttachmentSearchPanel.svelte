<script lang="ts">
  import { onDestroy } from 'svelte';
  import { operate } from './api';
  import { AttachmentReader, type AttachmentCriteria, type AttachmentReadState } from './attachment-search';
  import type { AttachmentTextLocator, Contract, IndexedExtractor } from './contracts.generated';

  let { workspace }: { workspace: string } = $props();
  let query = $state('');
  let extractor = $state<IndexedExtractor | ''>('');
  let jobId = $state('');
  let attachmentId = $state('');
  let view = $state<AttachmentReadState>({ page: null, pageNumber: 0, busy: false, error: null });
  const reader = new AttachmentReader(input => operate<Contract['attachment_search']>({ operation: 'extraction.search', input }, workspace), next => { view = next; });
  const criteria = (): AttachmentCriteria => ({ query, extractor, job_id: jobId, attachment_id: attachmentId });
  const reset = () => reader.edit();
  export function refreshPanel() { return reader.search(criteria()); }
  onDestroy(() => reader.dispose());

  function locatorTitle(locator: AttachmentTextLocator) {
    if (locator.kind === 'utf8_bytes') return `UTF-8 · líneas ${locator.line_start}–${locator.line_end}`;
    if (locator.kind === 'pdf_page_text') return `PDF · página ${locator.page_number}`;
    if (locator.kind === 'image_ocr_region_text') return `OCR inferido · región ${locator.region_ordinal}`;
    return `XLSX · hoja ID ${locator.sheet_id} · ${locator.reference} · ${locator.field}`;
  }
</script>

<div class="attachment-search" aria-label="Búsqueda de adjuntos">
  <form class="search-form" onsubmit={(event) => { event.preventDefault(); void refreshPanel(); }}>
    <label>Texto de adjuntos<input aria-label="Texto de adjuntos" bind:value={query} oninput={reset} placeholder="1–16 términos · hasta 256 bytes UTF-8" /></label>
    <label>Extractor<select aria-label="Extractor de adjuntos" bind:value={extractor} onchange={reset}>
      <option value="">Todos los extractores</option><option value="utf8_text">Texto UTF-8</option><option value="pdf_text">PDF textual</option><option value="xlsx_cells">Celdas XLSX</option><option value="image_ocr">OCR de imágenes (inferido)</option>
    </select></label>
    <label>ID de trabajo opcional<input aria-label="Trabajo de adjuntos" bind:value={jobId} oninput={reset} /></label>
    <label>ID de adjunto opcional<input aria-label="ID de adjunto" bind:value={attachmentId} oninput={reset} /></label>
    <button disabled={view.busy || !query.trim()}>Buscar adjuntos</button>
  </form>
  <p class="attachment-help">Sólo generaciones seleccionadas explícitamente. Importar o extraer no indexa. Prepará la selección desde «Preparar adjuntos locales» en una nota o por CLI/MCP; buscar no ejecuta extracción ni modifica índices.</p>
  <p class="attachment-help">Términos literales dentro de un fragmento, no subcadenas. Sin filtros de entidad, archivo o contexto ni ranking combinado con notas.</p>
  {#if view.busy}
    <p role="status" class="attachment-status">Consultando adjuntos…</p>
  {:else if view.error}
    <div class="error attachment-error" role="alert">
      <p>{view.error.code}{view.error.reason ? ` · ${view.error.reason}` : ''}</p><p>{view.error.message}</p>
      <p>No hay una página vigente. Buscar de nuevo inicia desde el principio; no repara ni indexa.</p>
      <button disabled={!query.trim()} onclick={() => void refreshPanel()}>Buscar de nuevo</button>
    </div>
  {:else if view.page}
    <section class="attachment-coverage" aria-label="Cobertura de adjuntos">
      <h3>Cobertura filtrada</h3>
      <dl>
        <dt>Generaciones seleccionadas</dt><dd data-coverage="indexed_generations">{view.page.coverage.indexed_generations}</dd>
        <dt>Compatibles sin indexar</dt><dd data-coverage="unindexed_supported_generations">{view.page.coverage.unindexed_supported_generations}</dd>
        <dt>Incompatibles</dt><dd data-coverage="unsupported_generations">{view.page.coverage.unsupported_generations}</dd>
        <dt>Trabajos sin generación</dt><dd data-coverage="jobs_without_generation">{view.page.coverage.jobs_without_generation}</dd>
        <dt>Fragmentos seleccionados</dt><dd data-coverage="fragments">{view.page.coverage.fragments}</dd>
        <dt>Bytes de texto seleccionados</dt><dd data-coverage="text_bytes">{view.page.coverage.text_bytes}</dd>
        <dt>Perfil</dt><dd data-page="profile">{view.page.profile}</dd>
        <dt>Versión / revisión del índice</dt><dd>{view.page.version} / <span data-page="revision">{view.page.revision}</span></dd>
        <dt>Alcance</dt><dd>{view.page.coverage.scope}</dd>
        <dt>Integridad del índice</dt><dd>{view.page.coverage.index_validation}</dd>
        <dt>Verificación de originales</dt><dd>{view.page.coverage.source_validation}</dd>
      </dl>
      <p>Índice canónico y postings verificados. Originales comprobados al consultar sólo para candidatos de página/lookahead, no todos los archivos ni su vigencia posterior.</p>
      <p>Cero coincidencias no prueba ausencia en toda la base. Selección sin texto no acredita OCR ni libro visualmente vacío; fragmentos no es el total de coincidencias. Compatible no garantiza caber en cuotas.</p>
    </section>
    <p class="attachment-status" role="status">Página {view.pageNumber} · {view.page.items.length} fragmentos en esta página</p>
    {#if !view.page.items.length}<p class="attachment-empty">Sin coincidencias en el conjunto seleccionado y filtrado.</p>{/if}
    {#each view.page.items as hit (`${hit.source.job_id}:${hit.source.generation_id}:${hit.ordinal}`)}
      <article class="attachment-hit" data-job={hit.source.job_id} data-generation={hit.source.generation_id} data-ordinal={hit.ordinal}>
        <h3>{locatorTitle(hit.locator)}</h3>
        {#if hit.result_from_previous_attempt}<p class="attachment-previous">Resultado de un intento anterior.</p>{/if}
        <pre class="attachment-text">{hit.text}</pre>
        <details>
          <summary>Procedencia del fragmento {hit.ordinal}</summary>
          <p>Evidencia verificada al consultar. Inspeccionar este detalle no reabre ni revalida el archivo.</p>
          <dl>
            <dt>Trabajo</dt><dd data-source="job_id">{hit.source.job_id}</dd>
            <dt>Adjunto</dt><dd data-source="attachment_id">{hit.source.attachment_id}</dd>
            <dt>Generación</dt><dd data-source="generation_id">{hit.source.generation_id}</dd>
            <dt>SHA-256 del original</dt><dd data-source="source_sha256">{hit.source.source_sha256}</dd>
            <dt>Extractor</dt><dd data-source="extractor">{hit.source.extractor}</dd>
            <dt>Versión del extractor</dt><dd data-source="extractor_version">{hit.source.extractor_version}</dd>
            <dt>Hash de configuración</dt><dd data-source="config_hash">{hit.source.config_hash}</dd>
            <dt>Ordinal de fragmento (0-based)</dt><dd data-source="ordinal">{hit.ordinal}</dd>
            <dt>Localizador</dt><dd data-locator="kind">{hit.locator.kind}</dd>
            {#if hit.locator.kind === 'utf8_bytes'}
              <dt>Bytes del original [inicio, fin)</dt><dd><span data-locator="byte_start">{hit.locator.byte_start}</span>–<span data-locator="byte_end">{hit.locator.byte_end}</span></dd>
              <dt>Líneas del original</dt><dd><span data-locator="line_start">{hit.locator.line_start}</span>–<span data-locator="line_end">{hit.locator.line_end}</span></dd>
            {:else if hit.locator.kind === 'pdf_page_text'}
              <dt>Página (1-based)</dt><dd data-locator="page_number">{hit.locator.page_number}</dd>
              <dt>Bytes del texto derivado [inicio, fin)</dt><dd><span data-locator="text_byte_start">{hit.locator.text_byte_start}</span>–<span data-locator="text_byte_end">{hit.locator.text_byte_end}</span></dd>
              <dt>Alcance PDF</dt><dd data-locator="scope">{hit.locator.scope}</dd>
              <dt>Limitaciones PDF</dt><dd data-locator="limitations">{hit.locator.limitations.join(', ')}</dd>
            {:else if hit.locator.kind === 'image_ocr_region_text'}
              <dt>Proyección OCR inferida</dt><dd data-locator="projection">{hit.locator.projection}</dd>
              <dt>Región (0-based)</dt><dd data-locator="region_ordinal">{hit.locator.region_ordinal}</dd>
              <dt>Geometría normalizada de imagen orientada</dt><dd data-locator="bbox">{JSON.stringify(hit.locator.bbox)}</dd>
              <dt>Confianza del motor</dt><dd data-locator="confidence">{hit.locator.confidence}</dd>
              <dt>Espacio de coordenadas</dt><dd data-locator="coordinate_space">{hit.locator.coordinate_space}</dd>
              <dt>Bytes de región</dt><dd data-locator="region_bytes">{hit.locator.region_bytes}</dd>
              <dt>Bytes de texto inferido [inicio, fin)</dt><dd><span data-locator="text_byte_start">{hit.locator.text_byte_start}</span>–<span data-locator="text_byte_end">{hit.locator.text_byte_end}</span></dd>
            {:else}
              <dt>Proyección</dt><dd data-locator="projection">{hit.locator.projection}</dd>
              <dt>Hoja ID</dt><dd data-locator="sheet_id">{hit.locator.sheet_id}</dd>
              <dt>Referencia A1</dt><dd data-locator="reference">{hit.locator.reference}</dd>
              <dt>Ordinal global de celda (0-based)</dt><dd data-locator="cell_ordinal">{hit.locator.cell_ordinal}</dd>
              <dt>Campo</dt><dd data-locator="field">{hit.locator.field}</dd>
              <dt>Bytes totales del campo</dt><dd data-locator="field_bytes">{hit.locator.field_bytes}</dd>
              <dt>Bytes UTF-8 del campo [inicio, fin)</dt><dd><span data-locator="text_byte_start">{hit.locator.text_byte_start}</span>–<span data-locator="text_byte_end">{hit.locator.text_byte_end}</span></dd>
            {/if}
          </dl>
          {#if hit.locator.kind === 'utf8_bytes'}<p>Rangos del original, final exclusivo; bytes UTF-8, no índices de caracteres.</p>
          {:else if hit.locator.kind === 'pdf_page_text'}<p>Texto derivado de objetos de página: sin OCR, orden visual, geometría ni tablas estructuradas. No son offsets del PDF ni coordenadas.</p>
          {:else if hit.locator.kind === 'image_ocr_region_text'}<p>Texto inferido, potencialmente incorrecto o incompleto. Región/bytes del derivado, no offsets originales. Confianza no es probabilidad de exactitud; sin orden visual ni tablas garantizados. Manifest/motor completos disponibles en el lector explícito.</p>
          {:else}<p>Fórmula y caché son campos separados. No se recalcula ni ejecuta la fórmula; la caché puede estar obsoleta. El rango es del campo decodificado, no del ZIP/JSON.</p>
            {#if hit.locator.text_byte_start !== 0 || hit.locator.text_byte_end !== hit.locator.field_bytes}<p class="attachment-partial">Porción del campo: no es el campo completo.</p>{/if}
          {/if}
        </details>
      </article>
    {/each}
    <div class="attachment-pages">
      {#if view.page.next_cursor}<button onclick={() => void reader.next()}>Siguiente página de adjuntos</button>{/if}
      <button onclick={() => void refreshPanel()}>Volver al inicio de adjuntos</button>
    </div>
  {:else}<p class="attachment-status" role="status">Ingresá texto y buscá adjuntos. Todavía no se consultó el índice.</p>{/if}
</div>

<style>
  .attachment-search { min-width: 0; font-size: 12px; }
  .attachment-search button, .attachment-search input, .attachment-search select { max-width: 100%; }
  .attachment-help, .attachment-coverage, .attachment-status { color: var(--muted); }
  .attachment-help { font-size: 11px; }
  .attachment-hit, .attachment-coverage { border-top: 1px solid var(--border); padding-top: 12px; margin-top: 16px; min-width: 0; }
  .attachment-hit h3, .attachment-coverage, dd, summary, .attachment-error { overflow-wrap: anywhere; }
  .attachment-text { white-space: pre-wrap; overflow-wrap: anywhere; font: inherit; line-height: 1.6; color: var(--text); }
  dl { margin: 12px 0; }
  dt { font-weight: 600; margin-top: 8px; }
  dd { margin: 3px 0 8px; white-space: pre-wrap; }
  summary { cursor: pointer; padding: 8px 3px; color: var(--link); }
  .attachment-previous, .attachment-partial { color: var(--warning); }
  .attachment-pages { display: grid; gap: 8px; margin-top: 20px; }
</style>
