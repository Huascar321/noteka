<script lang="ts">
  import { onMount } from 'svelte';
  import { operate } from './api';
  import type { PdfStructureResult } from './contracts.generated';
  import { AttachmentGenerationReader, type GenerationKind } from './attachment-generation-reader';
  let { workspace }: { workspace: string } = $props();
  let id = $state(''), kind = $state<GenerationKind>('utf8_text');
  let pdfPage = $state('1'), edges = $state('0,0.5,1'), yMin = $state('0'), yMax = $state('1'), tableHeader = $state('');
  let sheetId = $state('1'), range = $state('A1:D100'), headerRow = $state('');
  const controller = new AttachmentGenerationReader(command => operate(command, workspace), next => { view = next; });
  let view = $state(controller.state);
  let structure = $state<PdfStructureResult | null>(null);
  let structureError = $state<string | null>(null);
  const json = (value: unknown) => JSON.stringify(value, null, 2);
  async function loadStructure() {
    structureError = null; structure = null;
    try { structure = await operate({ operation: 'extraction.pdf_structure', input: { id } }, workspace) as PdfStructureResult; }
    catch (error) { structureError = error instanceof Error ? error.message : "Could not read the structure."; }
  }
  onMount(() => () => controller.dispose());
</script>

<details class="generation-panel">
  <summary>Read local generation</summary>
  <p>Use the job ID shown in preparation or obtained through CLI/MCP. Read the confirmed output by page, even if it is not selected for Search; xlsx_range reads only the chosen region. This does not render the original or process files.</p>
  <form onsubmit={e => { e.preventDefault(); void controller.read(kind === 'xlsx_range' ? { id, kind, sheet_id: Number(sheetId), range, ...(headerRow === '' ? {} : { header_row: Number(headerRow) }) } : kind === 'pdf_ocr_table' ? { id, kind, page_number: Number(pdfPage), x_edges: edges.split(',').map(s => s.trim()), y_min: yMin, y_max: yMax, ...(tableHeader === '' ? {} : { header_row: Number(tableHeader) }) } : { id, kind }); }}>
    <label>Job ID to read<input aria-label="Job ID to read" bind:value={id} oninput={() => controller.edit()} /></label>
    <label>Reader type<select aria-label="Reader type" bind:value={kind} onchange={() => controller.edit()}>
      <option value="utf8_text">utf8_text · original bytes and lines</option><option value="pdfium_text">pdfium_text · universal PDF by page</option><option value="pdf_text">pdf_text · derived text by page</option><option value="xlsx_cells">xlsx_cells · typed cells</option><option value="xlsx_range">xlsx_range · selected sheet, range and headers</option><option value="image_ocr">image_ocr · inferred text regions</option><option value="pdfkit_text">pdfkit_text · PDFKit pages</option><option value="paddle_ocr">paddle_ocr · universal text by page</option><option value="pdf_ocr">pdf_ocr · inferred regions by page</option><option value="pdf_ocr_table">pdf_ocr_table · table with selected columns</option>
    </select></label>
    {#if kind === 'xlsx_range'}
      <label>XLSX sheet ID<input aria-label="XLSX sheet ID" inputmode="numeric" bind:value={sheetId} oninput={() => controller.edit()} /></label>
      <label>XLSX A1 range<input aria-label="XLSX A1 range" bind:value={range} oninput={() => controller.edit()} /></label>
      <label>XLSX header row (optional)<input aria-label="XLSX header row" inputmode="numeric" bind:value={headerRow} oninput={() => controller.edit()} /></label>
      <p>Choose the ID from the cell manifest. Use a canonical range such as A1:D100, up to 64 columns; headers must be explicitly selected within the range. Sparse grids are preserved; merges are not expanded and titles are not inferred.</p>
    {/if}
    {#if kind === 'pdf_ocr_table'}
      <label>Physical PDF page<input aria-label="Physical PDF page" inputmode="numeric" bind:value={pdfPage} oninput={() => controller.edit()} /></label>
      <label>PDF column boundaries<input aria-label="PDF column boundaries" bind:value={edges} oninput={() => controller.edit()} /></label>
      <label>PDF minimum Y<input aria-label="PDF minimum Y" bind:value={yMin} oninput={() => controller.edit()} /></label>
      <label>PDF maximum Y<input aria-label="PDF maximum Y" bind:value={yMax} oninput={() => controller.edit()} /></label>
      <label>PDF header row (optional)<input aria-label="PDF header row" inputmode="numeric" bind:value={tableHeader} oninput={() => controller.edit()} /></label>
      <p>Use a prepared pdf_ocr generation. Separate boundaries with commas, such as 0,0.5,1; coordinates range from 0 to 1 relative to the raster, with the origin at the bottom left. Columns are selected; rows are inferred. This does not detect generic tables or split regions spanning columns.</p>
    {/if}
    <button disabled={view.busy || !id.trim()}>Read from the beginning</button>
  </form>
  {#if view.busy}<p role="status">Reading local generation…</p>{/if}
  {#if view.error}<div class="generation-error error" role="alert"><strong>{view.error.code}</strong><p>{view.error.message}</p>{#if view.error.details}<pre>{json(view.error.details)}</pre>{/if}{#if view.error.details && typeof view.error.details === 'object' && 'reason' in view.error.details && view.error.details.reason === 'response_size_limit'}<p>The response exceeds 2 MiB. For XLSX, choose a smaller range or omit the header row; content is never truncated.</p>{/if}<p>The previous page was cleared. Correct the criteria or explicitly read again; this does not repair or process files.</p></div>{/if}
  {#if view.page}
    {@const page = view.page}
    <section class="generation-result" aria-label="Generation read">
      <p>Page <span data-reader="page">{view.pageNumber}</span> · {page.data.items.length} items. Each page replaces the previous one.</p>
      <p>Job <code>{page.data.job_id}</code> · generation <code data-reader="generation">{page.data.generation.id}</code> · extractor {page.data.generation.extractor} v{page.data.generation.extractor_version}</p>
      <p>Complete generation: {String(page.data.generation.complete)} · page covers generation: <strong data-reader="covers">{String(page.data.covers_generation)}</strong> · result from an earlier attempt: {String(page.data.result_from_previous_attempt)}</p>
      <details><summary>Generation provenance</summary><pre class="generation-source">{json(page.data.generation)}</pre></details>
      {#if page.kind === 'pdf_text' || page.kind === 'pdfium_text' || page.kind === 'paddle_ocr'}
        {#if page.kind === 'paddle_ocr'}
          <p>Universal PaddleOCR: derived text by page. Table structure is optional and separate from the text.</p>
          <button disabled={!id} onclick={() => void loadStructure()}>Read table structure</button>
          {#if structureError}<p role="alert">{structureError}</p>{/if}
          {#if structure}<details open><summary>Estructura Paddle ({structure.manifest.table_count} tables)</summary><pre class="generation-manifest">{json(structure.structure)}</pre></details>{/if}
        {/if}
        <p>PDF ({page.data.manifest.parser_name} {page.data.manifest.parser_version}): {page.data.manifest.page_count} physical pages · {page.data.manifest.extracted_bytes} derived text bytes · scope {page.data.manifest.scope}. Limits: {page.data.manifest.limitations.join(', ')}. This does not provide visual order, geometry, tables or OCR.</p>
        <details><summary>Manifiesto PDF completo</summary><pre class="generation-manifest">{json(page.data.manifest)}</pre></details>
        {#each page.data.items as item (item.ordinal)}<article class="generation-item" data-ordinal={item.ordinal}><p>Fragment {item.ordinal} · page {item.locator.page_number} · bytes derivados [{item.locator.text_byte_start}, {item.locator.text_byte_end})</p><pre class="generation-text">{item.text}</pre><details><summary>Localizador PDF</summary><pre class="generation-locator">{json(item.locator)}</pre></details></article>{/each}
      {:else if page.kind === 'xlsx_cells' || page.kind === 'xlsx_range'}
        {#if page.kind === 'xlsx_range'}
          <p data-reader="range-coverage">Sheet {page.data.sheet.id}: {page.data.sheet.name} · range {page.data.range} · {page.data.matching_cell_count} matching cells · page covers range: {String(page.data.covers_range)} · selected header row: {page.data.header_row ?? 'ninguna'}. Range coverage does not imply coverage of the entire workbook.</p>
          <details><summary>Selected headers by column</summary><pre class="generation-headers">{json(page.data.headers)}</pre></details>
        {/if}
        <p>XLSX: {page.data.manifest.cell_count} cells · 1904 system: {String(page.data.manifest.date_1904)} · parser {page.data.manifest.parser}. The manifest includes all sheets, ranges, merges and styles, including those outside this page.</p>
        <p>No recalculation or visual formatting. Numeric lexemes and serials are preserved; date_iso is the declared string. A cache may be missing or outdated. Grids and merges are not expanded; headers are not inferred.</p>
        <details><summary>Manifiesto XLSX completo</summary><pre class="generation-manifest">{json(page.data.manifest)}</pre></details>
        {#each page.data.items as item (item.ordinal)}
          {@const sheet = page.data.manifest.sheets.find(sheet => sheet.id === item.locator.sheet_id)}
          <article class="generation-item" data-ordinal={item.ordinal}>
            <h4>Sheet {item.locator.sheet_id}: {sheet?.name ?? "No descriptor"} · {item.locator.reference}</h4><p>Visibility: {sheet?.visibility ?? "No descriptor"} · ordinal {item.ordinal} · fila {item.cell.row} / columna {item.cell.column} · style index {item.cell.style}</p>
            <p>Stored value (cache): {item.cell.cached == null ? 'ausente' : item.cell.cached.kind}</p><pre class="generation-cached">{json(item.cell.cached ?? null)}</pre>
            <p>Literal formula and metadata: {item.cell.formula == null ? 'ausente' : item.cell.formula.kind}</p><pre class="generation-formula">{json(item.cell.formula ?? null)}</pre>
            <details><summary>Complete cell and locator</summary><pre class="generation-cell">{json(item.cell)}</pre><pre class="generation-locator">{json(item.locator)}</pre></details>
          </article>
        {/each}
      {:else if page.kind === 'pdfkit_text'}
        <p>PDFKit: {page.data.manifest.page_count} physical pages · {page.data.manifest.text_bytes} derived text bytes · profile {page.data.manifest.profile}. Selectable text from the system parser; no_text does not prove visual absence.</p>
        <p>OCR, visual order, geometry and tables are not guaranteed. API text and actual engine provenance are preserved.</p>
        <details><summary>Manifiesto PDFKit completo</summary><pre class="generation-manifest">{json(page.data.manifest)}</pre></details>
        {#each page.data.items as item (item.number)}<article class="generation-item" data-ordinal={item.number - 1} data-physical-page={item.number}><h4>Physical page {item.number} · {item.status}</h4><pre class="generation-text">{item.text}</pre><details><summary>PDFKit page locator</summary><pre class="generation-locator">{json({ page_number: item.number, status: item.status })}</pre></details></article>{/each}
      {:else if page.kind === 'pdf_ocr_table'}
        <p data-reader="table-coverage">Physical page table {page.data.page.raster.page_number} · {page.data.row_count} filas / {page.data.column_count} columns · {page.data.cell_count} cells · page covers table: {String(page.data.covers_table)} · selected header row: {page.data.header_row ?? 'ninguna'}. This projection never covers the full generation.</p>
        <p>Profile {page.data.profile}. Outside regions are preserved as references to the pdf_ocr reader. Headers are not removed or corrected. OCR text is inferred and may contain errors.</p>
        <details><summary>Complete PDF table manifest and criteria</summary><pre class="generation-manifest">{json({ manifest: page.data.manifest, page: page.data.page, profile: page.data.profile, x_edges: page.data.x_edges, y_min: page.data.y_min, y_max: page.data.y_max, header_row: page.data.header_row, limitations: page.data.limitations })}</pre></details>
        <details><summary>References to outside regions</summary><pre class="generation-outside">{json(page.data.outside)}</pre></details>
        {#each page.data.items as item (item.ordinal)}
          <article class="generation-item" data-ordinal={item.ordinal} data-row={item.row} data-column={item.column}>
            <h4>Row {item.row} · columna {item.column} · cell {item.ordinal}</h4>
            {#if !item.regions.length}<p class="generation-cell-empty">No OCR regions assigned to this cell</p>{/if}
            {#each item.regions as region (region.ordinal)}<p>Physical page {region.page_number} · region {region.region_ordinal} · ordinal global {region.ordinal}</p><pre class="generation-text">{region.text}</pre><details><summary>Cell region provenance</summary><pre class="generation-locator">{json(region)}</pre></details>{/each}
          </article>
        {/each}
      {:else if page.kind === 'pdf_ocr'}
        <p>PDF OCR inferido: {page.data.manifest.page_count} physical pages · {page.data.manifest.region_count} regiones · perfil {page.data.manifest.profile}. Complete does not guarantee a complete or correct transcription.</p>
        <p>Visual order and tables are not guaranteed. Bounding boxes are normalized to the page raster, with the origin at the bottom left; confidence is not a probability. Text orientation is not detected automatically. The manifest includes pages with no regions and the hash, dimensions and rotation of each temporary derived PNG; the original PDF is preserved.</p>
        <details><summary>Manifiesto PDF OCR completo</summary><pre class="generation-manifest">{json(page.data.manifest)}</pre></details>
        {#each page.data.items as item (item.ordinal)}<article class="generation-item" data-ordinal={item.ordinal} data-physical-page={item.page_number}><h4>Physical page {item.page_number} · inferred OCR region {item.region_ordinal} · ordinal global {item.ordinal}</h4><pre class="generation-text">{item.text}</pre><details><summary>PDF OCR geometry and confidence</summary><pre class="generation-locator">{json({ page_number: item.page_number, region_ordinal: item.region_ordinal, bbox: item.bbox, confidence: item.confidence, coordinate_space: 'rendered_page_normalized_bottom_left' })}</pre></details></article>{/each}
      {:else if page.kind === 'image_ocr'}
        <p>OCR inferido: {page.data.manifest.region_count} regiones · perfil {page.data.manifest.profile}. Complete means the source was processed; it does not guarantee a complete or correct transcription.</p>
        <p>Reading order, reconstructed tables and visual descriptions are not guaranteed. Confidence is not a probability of accuracy. Bounding boxes are normalized to the oriented image, with the origin at the bottom left; decimals are preserved as strings.</p>
        <details><summary>Manifiesto OCR completo</summary><pre class="generation-manifest">{json(page.data.manifest)}</pre></details>
        {#each page.data.items as item (item.ordinal)}<article class="generation-item" data-ordinal={item.ordinal}><h4>Inferred OCR region {item.ordinal}</h4><pre class="generation-text">{item.text}</pre><details><summary>OCR geometry and confidence</summary><pre class="generation-locator">{json({ bbox: item.bbox, confidence: item.confidence, coordinate_space: page.data.manifest.coordinate_space })}</pre></details></article>{/each}
      {:else}
        <p>Literal UTF-8: byte offsets [start, end) and original lines. Markdown, HTML, commands and links are not interpreted.</p>
        {#each page.data.items as item (item.ordinal)}<article class="generation-item" data-ordinal={item.ordinal}><p>Fragment {item.ordinal} · original bytes [{item.byte_start}, {item.byte_end}) · lines {item.line_start}–{item.line_end}</p><pre class="generation-text">{item.text}</pre><details><summary>Localizador UTF-8</summary><pre class="generation-locator">{json({ byte_start: item.byte_start, byte_end: item.byte_end, line_start: item.line_start, line_end: item.line_end })}</pre></details></article>{/each}
      {/if}
      {#if !page.data.items.length}<p class="generation-empty">{page.kind === 'xlsx_range' ? "No stored cells in this range" : "No items for this reader in this generation"}. This does not prove the original is visually empty or all its text is recognizable.</p>{/if}
      {#if page.data.next_cursor}<button disabled={view.busy} onclick={() => void controller.next()}>Next reader page</button>{/if}
    </section>
  {:else if !view.busy && !view.error}<p class="generation-initial">No reading requested yet. Opening this panel does not query or process files.</p>{/if}
  <p>Reads revalidate original and derived files in Rust; errors do not trigger repairs or fallback. Changing criteria clears the page; leaving discards late responses without cancelling active IPC. No polling or browser content persistence.</p>
</details>

<style>
  .generation-panel { margin-block: 2rem; padding: 1rem; border: 1px solid var(--border); border-radius: 8px; min-width: 0; }
  summary { cursor: pointer; font-weight: 600; }
  label { display: grid; gap: .35rem; margin-block: .6rem; }
  input, select { min-width: 0; max-width: 100%; }
  p { line-height: 1.5; }
  pre, code, h4 { white-space: pre-wrap; overflow-wrap: anywhere; word-break: break-word; }
  pre { max-height: 24rem; overflow: auto; padding: .75rem; background: var(--surface); }
  .generation-item { padding-block: .8rem; border-top: 1px solid var(--border); }
</style>
