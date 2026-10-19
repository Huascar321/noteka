<script lang="ts">
  import { onMount } from 'svelte';
  import { operate } from './api';
  import { AttachmentGenerationReader } from './attachment-generation-reader';
  import type { IndexedExtractor } from './contracts.generated';
  let { workspace }: { workspace: string } = $props();
  let id = $state(''), kind = $state<IndexedExtractor>('utf8_text');
  const controller = new AttachmentGenerationReader(command => operate(command, workspace), next => { view = next; });
  let view = $state(controller.state);
  const json = (value: unknown) => JSON.stringify(value, null, 2);
  onMount(() => () => controller.dispose());
</script>

<details class="generation-panel">
  <summary>Leer generación local</summary>
  <p>Usá el ID de trabajo mostrado en preparación o consultado por CLI/MCP. Lee el derivado confirmado completo por páginas, aunque no esté seleccionado para Search. No muestra un render del original ni procesa archivos.</p>
  <form onsubmit={e => { e.preventDefault(); void controller.read({ id, kind }); }}>
    <label>ID de trabajo para lectura<input aria-label="ID de trabajo para lectura" bind:value={id} oninput={() => controller.edit()} /></label>
    <label>Tipo de lectura<select aria-label="Tipo de lectura" bind:value={kind} onchange={() => controller.edit()}>
      <option value="utf8_text">utf8_text · bytes y líneas originales</option><option value="pdf_text">pdf_text · texto derivado por página</option><option value="xlsx_cells">xlsx_cells · celdas tipadas</option><option value="image_ocr">image_ocr · regiones de texto inferido</option>
    </select></label>
    <button disabled={view.busy || !id.trim()}>Leer desde el inicio</button>
  </form>
  {#if view.busy}<p role="status">Leyendo generación local…</p>{/if}
  {#if view.error}<div class="generation-error error" role="alert"><strong>{view.error.code}</strong><p>{view.error.message}</p>{#if view.error.details}<pre>{json(view.error.details)}</pre>{/if}<p>La página anterior se retiró. Corregí o volvé a leer explícitamente; no se repara ni procesa.</p></div>{/if}
  {#if view.page}
    {@const page = view.page}
    <section class="generation-result" aria-label="Generación leída">
      <p>Página <span data-reader="page">{view.pageNumber}</span> · {page.data.items.length} items. Una página reemplaza a la anterior.</p>
      <p>Trabajo <code>{page.data.job_id}</code> · generación <code data-reader="generation">{page.data.generation.id}</code> · extractor {page.data.generation.extractor} v{page.data.generation.extractor_version}</p>
      <p>Generación completa: {String(page.data.generation.complete)} · página cubre generación: <strong data-reader="covers">{String(page.data.covers_generation)}</strong> · resultado de intento anterior: {String(page.data.result_from_previous_attempt)}</p>
      <details><summary>Procedencia de la generación</summary><pre class="generation-source">{json(page.data.generation)}</pre></details>
      {#if page.kind === 'pdf_text'}
        <p>PDF: {page.data.manifest.page_count} páginas físicas · {page.data.manifest.extracted_bytes} bytes de texto derivado · scope {page.data.manifest.scope}. Límites: {page.data.manifest.limitations.join(', ')}. No es orden visual, geometría, tablas ni OCR.</p>
        <details><summary>Manifiesto PDF completo</summary><pre class="generation-manifest">{json(page.data.manifest)}</pre></details>
        {#each page.data.items as item (item.ordinal)}<article class="generation-item" data-ordinal={item.ordinal}><p>Fragmento {item.ordinal} · página {item.locator.page_number} · bytes derivados [{item.locator.text_byte_start}, {item.locator.text_byte_end})</p><pre class="generation-text">{item.text}</pre><details><summary>Localizador PDF</summary><pre class="generation-locator">{json(item.locator)}</pre></details></article>{/each}
      {:else if page.kind === 'xlsx_cells'}
        <p>XLSX: {page.data.manifest.cell_count} celdas · sistema 1904: {String(page.data.manifest.date_1904)} · parser {page.data.manifest.parser}. El manifiesto incluye todas las hojas, rangos, merges y estilos, incluso fuera de esta página.</p>
        <p>Sin recálculo ni formato visual. Números léxicos y seriales se conservan; date_iso es la cadena declarada. Una caché puede faltar o estar obsoleta. No se expanden grillas/merges ni se infieren encabezados.</p>
        <details><summary>Manifiesto XLSX completo</summary><pre class="generation-manifest">{json(page.data.manifest)}</pre></details>
        {#each page.data.items as item (item.ordinal)}
          {@const sheet = page.data.manifest.sheets.find(sheet => sheet.id === item.locator.sheet_id)}
          <article class="generation-item" data-ordinal={item.ordinal}>
            <h4>Hoja {item.locator.sheet_id}: {sheet?.name ?? 'Sin descriptor'} · {item.locator.reference}</h4><p>Visibilidad: {sheet?.visibility ?? 'Sin descriptor'} · ordinal {item.ordinal} · fila {item.cell.row} / columna {item.cell.column} · índice de estilo {item.cell.style}</p>
            <p>Valor guardado (caché): {item.cell.cached == null ? 'ausente' : item.cell.cached.kind}</p><pre class="generation-cached">{json(item.cell.cached ?? null)}</pre>
            <p>Fórmula literal y metadatos: {item.cell.formula == null ? 'ausente' : item.cell.formula.kind}</p><pre class="generation-formula">{json(item.cell.formula ?? null)}</pre>
            <details><summary>Celda y localizador completos</summary><pre class="generation-cell">{json(item.cell)}</pre><pre class="generation-locator">{json(item.locator)}</pre></details>
          </article>
        {/each}
      {:else if page.kind === 'image_ocr'}
        <p>OCR inferido: {page.data.manifest.region_count} regiones · perfil {page.data.manifest.profile}. Completa indica que se procesó la fuente; no garantiza transcripción completa ni correcta.</p>
        <p>Sin orden de lectura garantizado, tablas reconstruidas ni descripción visual. Confianza no es probabilidad de exactitud. Bbox está normalizada respecto de la imagen orientada, con origen inferior izquierdo; números decimales conservados como cadenas.</p>
        <details><summary>Manifiesto OCR completo</summary><pre class="generation-manifest">{json(page.data.manifest)}</pre></details>
        {#each page.data.items as item (item.ordinal)}<article class="generation-item" data-ordinal={item.ordinal}><h4>Región OCR inferida {item.ordinal}</h4><pre class="generation-text">{item.text}</pre><details><summary>Geometría y confianza OCR</summary><pre class="generation-locator">{json({ bbox: item.bbox, confidence: item.confidence, coordinate_space: page.data.manifest.coordinate_space })}</pre></details></article>{/each}
      {:else}
        <p>UTF-8 literal: offsets en bytes [inicio, fin) y líneas del original. No se interpreta Markdown, HTML, comandos ni enlaces.</p>
        {#each page.data.items as item (item.ordinal)}<article class="generation-item" data-ordinal={item.ordinal}><p>Fragmento {item.ordinal} · bytes originales [{item.byte_start}, {item.byte_end}) · líneas {item.line_start}–{item.line_end}</p><pre class="generation-text">{item.text}</pre><details><summary>Localizador UTF-8</summary><pre class="generation-locator">{json({ byte_start: item.byte_start, byte_end: item.byte_end, line_start: item.line_start, line_end: item.line_end })}</pre></details></article>{/each}
      {/if}
      {#if !page.data.items.length}<p class="generation-empty">Generación sin items para este lector. No prueba que el original esté visualmente vacío ni que todo su texto sea reconocible.</p>{/if}
      {#if page.data.next_cursor}<button disabled={view.busy} onclick={() => void controller.next()}>Siguiente página de lectura</button>{/if}
    </section>
  {:else if !view.busy && !view.error}<p class="generation-initial">Todavía no se solicitó lectura. Abrir este panel no consulta ni procesa.</p>{/if}
  <p>Lecturas revalidan original y derivado en Rust; un error no hace reparación ni fallback. Cambiar criterios descarta la página; salir descarta respuestas tardías, sin cancelar IPC ya iniciado. No hay polling ni contenido persistido en el navegador.</p>
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
