<script lang="ts">
  import { onDestroy, untrack } from 'svelte';
  import { operate } from './api';
  import { AttachmentReader, type AttachmentCriteria, type AttachmentReadState } from './attachment-search';
  import type { AttachmentTextLocator, Contract, IndexedExtractor } from './contracts.generated';
  import { RefreshCoordinator } from './refresh-coordinator';

  let { workspace }: { workspace: string } = $props();
  let query = $state('');
  let extractor = $state<IndexedExtractor | ''>('');
  let jobId = $state('');
  let attachmentId = $state('');
  let view = $state<AttachmentReadState>({ page: null, pageNumber: 0, busy: false, error: null });
  const reader = new AttachmentReader((input, signal) => operate<Contract['attachment_search']>({ operation: 'extraction.search', input }, workspace, { signal }), next => { view = next; });
  const criteria = (): AttachmentCriteria => ({ query, extractor, job_id: jobId, attachment_id: attachmentId });
  const refresh = new RefreshCoordinator(() => !view.busy, () => reader.search(criteria()), () => {});
  const reset = () => { refresh.cancelPending(); reader.edit(); };
  export function refreshPanel() { return refresh.request(); }
  $effect(() => { if (!view.busy) untrack(() => refresh.notifyReady()); });
  onDestroy(() => { refresh.dispose(); reader.dispose(); });

  function locatorTitle(locator: AttachmentTextLocator) {
    if (locator.kind === 'utf8_bytes') return `UTF-8 · lines ${locator.line_start}–${locator.line_end}`;
    if (locator.kind === 'pdf_page_text') return `PDF · page ${locator.page_number}`;
    if (locator.kind === 'pdfkit_page_text') return `PDFKit · page ${locator.page_number}`;
    if (locator.kind === 'pdf_ocr_region_text') return `Inferred PDF OCR · page ${locator.page_number} · region ${locator.region_ordinal}`;
    if (locator.kind === 'image_ocr_region_text') return `Inferred OCR · region ${locator.region_ordinal}`;
    return `XLSX · hoja ID ${locator.sheet_id} · ${locator.reference} · ${locator.field}`;
  }
</script>

<div class="attachment-search" aria-label="Attachment search">
  <form class="search-form" onsubmit={(event) => { event.preventDefault(); void reader.search(criteria()); }}>
    <label>Attachment text<input aria-label="Attachment text" bind:value={query} oninput={reset} placeholder="1–16 terms · up to 256 UTF-8 bytes" /></label>
    <label>Extractor<select aria-label="Attachment extractor" bind:value={extractor} onchange={reset}>
      <option value="">All extractors</option><option value="utf8_text">UTF-8 text</option><option value="pdfium_text">PDF universal (PDFium)</option><option value="pdf_text">PDF textual</option><option value="xlsx_cells">Celdas XLSX</option><option value="image_ocr">Image OCR (inferred)</option><option value="pdfkit_text">PDFKit (texto seleccionable)</option><option value="paddle_ocr">PDF OCR universal (PaddleOCR)</option><option value="pdf_ocr">PDF OCR (inferido)</option>
    </select></label>
    <label>Optional job ID<input aria-label="Attachment job" bind:value={jobId} oninput={reset} /></label>
    <label>Optional attachment ID<input aria-label="Attachment ID" bind:value={attachmentId} oninput={reset} /></label>
    <button disabled={view.busy || !query.trim()}>Search attachments</button>
  </form>
  <p class="attachment-help">Only explicitly selected generations. Importing or extracting does not index them. Select a generation in “Prepare local attachments” in a note or through CLI/MCP; searching does not extract files or change indexes.</p>
  <p class="attachment-help">Literal terms within a fragment, not substrings. No entity, file or context filters, or ranking combined with notes.</p>
  {#if view.busy}
    <p role="status" class="attachment-status">Searching attachments…</p>
  {:else if view.error}
    <div class="error attachment-error" role="alert">
      <p>{view.error.code}{view.error.reason ? ` · ${view.error.reason}` : ''}</p><p>{view.error.message}</p>
      <p>There is no current page. Searching again starts at the beginning; it does not repair or index.</p>
      <button disabled={!query.trim()} onclick={() => void refreshPanel()}>Search again</button>
    </div>
  {:else if view.page}
    <section class="attachment-coverage" aria-label="Attachment coverage">
      <h3>Cobertura filtrada</h3>
      <dl>
        <dt>Generaciones seleccionadas</dt><dd data-coverage="indexed_generations">{view.page.coverage.indexed_generations}</dd>
        <dt>Compatible but not indexed</dt><dd data-coverage="unindexed_supported_generations">{view.page.coverage.unindexed_supported_generations}</dd>
        <dt>Incompatible</dt><dd data-coverage="unsupported_generations">{view.page.coverage.unsupported_generations}</dd>
        <dt>Jobs without a generation</dt><dd data-coverage="jobs_without_generation">{view.page.coverage.jobs_without_generation}</dd>
        <dt>Fragmentos seleccionados</dt><dd data-coverage="fragments">{view.page.coverage.fragments}</dd>
        <dt>Selected text bytes</dt><dd data-coverage="text_bytes">{view.page.coverage.text_bytes}</dd>
        <dt>Profile</dt><dd data-page="profile">{view.page.profile}</dd>
        <dt>Index version / revision</dt><dd>{view.page.version} / <span data-page="revision">{view.page.revision}</span></dd>
        <dt>Scope</dt><dd>{view.page.coverage.scope}</dd>
        <dt>Index integrity</dt><dd>{view.page.coverage.index_validation}</dd>
        <dt>Original file verification</dt><dd>{view.page.coverage.source_validation}</dd>
      </dl>
      <p>Canonical index and postings verified. Originals are checked at query time for page/lookahead candidates only, not every file or its later validity.</p>
      <p>Zero matches do not prove absence across the database. A selection with no text does not establish OCR coverage or a visually empty workbook; fragment count is not total matches. Compatibility does not guarantee fitting within quotas.</p>
    </section>
    <p class="attachment-status" role="status">Page {view.pageNumber} · {view.page.items.length} fragments on this page</p>
    {#if !view.page.items.length}<p class="attachment-empty">No matches in the selected and filtered set.</p>{/if}
    {#each view.page.items as hit (`${hit.source.job_id}:${hit.source.generation_id}:${hit.ordinal}`)}
      <article class="attachment-hit" data-job={hit.source.job_id} data-generation={hit.source.generation_id} data-ordinal={hit.ordinal}>
        <h3>{locatorTitle(hit.locator)}</h3>
        {#if hit.result_from_previous_attempt}<p class="attachment-previous">Result from an earlier attempt.</p>{/if}
        <pre class="attachment-text">{hit.text}</pre>
        <details>
          <summary>Fragment provenance {hit.ordinal}</summary>
          <p>Evidence verified at query time. Inspecting this detail does not reopen or revalidate the file.</p>
          <dl>
            <dt>Job</dt><dd data-source="job_id">{hit.source.job_id}</dd>
            <dt>Attachment</dt><dd data-source="attachment_id">{hit.source.attachment_id}</dd>
            <dt>Generation</dt><dd data-source="generation_id">{hit.source.generation_id}</dd>
            <dt>Original SHA-256</dt><dd data-source="source_sha256">{hit.source.source_sha256}</dd>
            <dt>Extractor</dt><dd data-source="extractor">{hit.source.extractor}</dd>
            <dt>Extractor version</dt><dd data-source="extractor_version">{hit.source.extractor_version}</dd>
            <dt>Configuration hash</dt><dd data-source="config_hash">{hit.source.config_hash}</dd>
            <dt>Fragment ordinal (0-based)</dt><dd data-source="ordinal">{hit.ordinal}</dd>
            <dt>Locator</dt><dd data-locator="kind">{hit.locator.kind}</dd>
            {#if hit.locator.kind === 'utf8_bytes'}
              <dt>Original bytes [start, end)</dt><dd><span data-locator="byte_start">{hit.locator.byte_start}</span>–<span data-locator="byte_end">{hit.locator.byte_end}</span></dd>
              <dt>Original lines</dt><dd><span data-locator="line_start">{hit.locator.line_start}</span>–<span data-locator="line_end">{hit.locator.line_end}</span></dd>
            {:else if hit.locator.kind === 'pdf_page_text'}
              <dt>Page (1-based)</dt><dd data-locator="page_number">{hit.locator.page_number}</dd>
              <dt>Derived text bytes [start, end)</dt><dd><span data-locator="text_byte_start">{hit.locator.text_byte_start}</span>–<span data-locator="text_byte_end">{hit.locator.text_byte_end}</span></dd>
              <dt>Alcance PDF</dt><dd data-locator="scope">{hit.locator.scope}</dd>
              <dt>Limitaciones PDF</dt><dd data-locator="limitations">{hit.locator.limitations.join(', ')}</dd>
            {:else if hit.locator.kind === 'pdfkit_page_text'}
              <dt>PDFKit projection</dt><dd data-locator="projection">{hit.locator.projection}</dd>
              <dt>Perfil PDFKit</dt><dd data-locator="profile">{hit.locator.profile}</dd>
              <dt>Physical page (1-based)</dt><dd data-locator="page_number">{hit.locator.page_number}</dd>
              <dt>Derived page bytes</dt><dd data-locator="page_bytes">{hit.locator.page_bytes}</dd>
              <dt>Derived text bytes [start, end)</dt><dd><span data-locator="text_byte_start">{hit.locator.text_byte_start}</span>–<span data-locator="text_byte_end">{hit.locator.text_byte_end}</span></dd>
              <dt>Engine and system</dt><dd data-locator="engine">{JSON.stringify(hit.locator.engine)}</dd>
              <dt>Limitaciones PDFKit</dt><dd data-locator="limitations">{hit.locator.limitations.join(', ')}</dd>
            {:else if hit.locator.kind === 'pdf_ocr_region_text'}
              <dt>PDF OCR projection</dt><dd data-locator="projection">{hit.locator.projection}</dd>
              <dt>Physical page (1-based)</dt><dd data-locator="page_number">{hit.locator.page_number}</dd>
              <dt>Local / global region (0-based)</dt><dd><span data-locator="region_ordinal">{hit.locator.region_ordinal}</span> / {hit.locator.global_region_ordinal}</dd>
              <dt>Profile</dt><dd data-locator="profile">{hit.locator.profile}</dd>
              <dt>Raster derivado completo</dt><dd data-locator="raster">{JSON.stringify(hit.locator.raster)}</dd>
              <dt>Engine and system</dt><dd data-locator="engine">{JSON.stringify(hit.locator.engine)}</dd>
              <dt>Normalized raster bounding box</dt><dd data-locator="bbox">{JSON.stringify(hit.locator.bbox)}</dd>
              <dt>Engine confidence</dt><dd data-locator="confidence">{hit.locator.confidence}</dd>
              <dt>Coordinate space</dt><dd data-locator="coordinate_space">{hit.locator.coordinate_space}</dd>
              <dt>Region bytes</dt><dd data-locator="region_bytes">{hit.locator.region_bytes}</dd>
              <dt>Inferred text bytes [start, end)</dt><dd><span data-locator="text_byte_start">{hit.locator.text_byte_start}</span>–<span data-locator="text_byte_end">{hit.locator.text_byte_end}</span></dd>
            {:else if hit.locator.kind === 'image_ocr_region_text'}
              <dt>Inferred OCR projection</dt><dd data-locator="projection">{hit.locator.projection}</dd>
              <dt>Region (0-based)</dt><dd data-locator="region_ordinal">{hit.locator.region_ordinal}</dd>
              <dt>Normalized geometry of the oriented image</dt><dd data-locator="bbox">{JSON.stringify(hit.locator.bbox)}</dd>
              <dt>Engine confidence</dt><dd data-locator="confidence">{hit.locator.confidence}</dd>
              <dt>Coordinate space</dt><dd data-locator="coordinate_space">{hit.locator.coordinate_space}</dd>
              <dt>Region bytes</dt><dd data-locator="region_bytes">{hit.locator.region_bytes}</dd>
              <dt>Inferred text bytes [start, end)</dt><dd><span data-locator="text_byte_start">{hit.locator.text_byte_start}</span>–<span data-locator="text_byte_end">{hit.locator.text_byte_end}</span></dd>
            {:else}
              <dt>Projection</dt><dd data-locator="projection">{hit.locator.projection}</dd>
              <dt>Sheet ID</dt><dd data-locator="sheet_id">{hit.locator.sheet_id}</dd>
              <dt>A1 reference</dt><dd data-locator="reference">{hit.locator.reference}</dd>
              <dt>Global cell ordinal (0-based)</dt><dd data-locator="cell_ordinal">{hit.locator.cell_ordinal}</dd>
              <dt>Field</dt><dd data-locator="field">{hit.locator.field}</dd>
              <dt>Total field bytes</dt><dd data-locator="field_bytes">{hit.locator.field_bytes}</dd>
              <dt>Field UTF-8 bytes [start, end)</dt><dd><span data-locator="text_byte_start">{hit.locator.text_byte_start}</span>–<span data-locator="text_byte_end">{hit.locator.text_byte_end}</span></dd>
            {/if}
          </dl>
          {#if hit.locator.kind === 'utf8_bytes'}<p>Original ranges, exclusive end; UTF-8 bytes, not character indexes.</p>
          {:else if hit.locator.kind === 'pdf_page_text'}<p>Text derived from page objects: no OCR, visual order, geometry or structured tables. These are not PDF offsets or coordinates.</p>
          {:else if hit.locator.kind === 'pdfkit_page_text'}<p>PDFKit: bytes of text derived from the system parser, not original offsets. OCR, visual order, geometry and tables are not guaranteed; results depend on the OS/build shown.</p>
            {#if hit.locator.text_byte_start !== 0 || hit.locator.text_byte_end !== hit.locator.page_bytes}<p class="attachment-partial">Page excerpt: not its complete text.</p>{/if}
          {:else if hit.locator.kind === 'pdf_ocr_region_text'}<p>Inferred PDF OCR may be incorrect or incomplete; visual order and tables are not guaranteed. Bounding boxes use the derived raster, not original PDF coordinates; confidence is not a probability. Full profile, engine, OS/build and raster details are in the locator.</p>
          {:else if hit.locator.kind === 'image_ocr_region_text'}<p>Inferred text may be incorrect or incomplete. Region/bytes refer to derived text, not original offsets. Confidence is not a probability of accuracy; visual order and tables are not guaranteed. Full manifest/engine details are available in the explicit reader.</p>
          {:else}<p>Formula and cache are separate fields. Formulas are not recalculated or executed; cached values may be outdated. The range refers to the decoded field, not ZIP/JSON.</p>
            {#if hit.locator.text_byte_start !== 0 || hit.locator.text_byte_end !== hit.locator.field_bytes}<p class="attachment-partial">Field excerpt: not the complete field.</p>{/if}
          {/if}
        </details>
      </article>
    {/each}
    <div class="attachment-pages">
      {#if view.page.next_cursor}<button onclick={() => void reader.next()}>Next attachment page</button>{/if}
      <button onclick={() => void refreshPanel()}>Back to the first attachment page</button>
    </div>
  {:else}<p class="attachment-status" role="status">Enter text and search attachments. The index has not been queried yet.</p>{/if}
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
