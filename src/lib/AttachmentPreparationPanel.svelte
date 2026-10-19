<script lang="ts">
  import { onMount } from 'svelte';
  import { operate } from './api';
  import { AttachmentPreparation, attachmentReferences } from './attachment-preparation';
  import type { Document, IndexedExtractor } from './contracts.generated';
  let { workspace, document }: { workspace: string; document: Document | null | undefined } = $props();
  const controller = new AttachmentPreparation(command => operate(command, workspace), next => { view = next; });
  let view = $state(controller.state);
  let extractor = $state<IndexedExtractor>('utf8_text');
  const refs = $derived(attachmentReferences(document));
  const json = (value: unknown) => JSON.stringify(value, null, 2);
  const canPut = $derived(view.job && view.index && !view.busy && controller.canSelectGeneration);
  onMount(() => () => controller.dispose());
</script>

<details class="preparation-panel">
  <summary>Preparar adjuntos locales</summary>
  <p>Elegí un original ya administrado. Cada paso es explícito: encolar → reanudar si hace falta → ejecutar → consultar → seleccionar generación. Search sólo busca.</p>
  <p>Salir de este panel no cancela una operación iniciada. No hay polling. Consultá nuevamente para conocer cambios de otros clientes.</p>
  {#if refs.length}
    <label>Referencias de esta nota<select aria-label="Referencia para preparar" value="" onchange={e => controller.select(e.currentTarget.value)}>
      <option value="" disabled>Elegí una referencia</option>
      {#each refs as ref (ref.id)}<option value={ref.id}>{ref.id} · Rótulo de bloque: {ref.label}</option>{/each}
    </select></label>
    <p>Los rótulos pertenecen al documento, pueden estar pendientes de guardar y no son metadatos del archivo ni indican un dueño exclusivo.</p>
  {/if}
  <label>ID administrado para preparar<input aria-label="ID administrado para preparar" value={view.attachmentId} oninput={e => controller.select(e.currentTarget.value)} /></label>
  <div class="actions">
    <button disabled={view.busy || !view.attachmentId} onclick={() => void controller.list()}>Consultar trabajos</button>
    <label>Extractor de preparación<select aria-label="Extractor de preparación" bind:value={extractor} disabled={view.busy}>
      <option value="utf8_text">utf8_text · .txt/.md literal</option><option value="pdf_text">pdf_text · PDF textual acotado</option><option value="xlsx_cells">xlsx_cells · XLSX Transitional acotado</option><option value="image_ocr">image_ocr · PNG/JPEG inferido local</option>
    </select></label>
    <button disabled={view.busy || !view.attachmentId || view.needsConsult} onclick={() => void controller.enqueue(extractor)}>Encolar extracción</button>
  </div>
  {#if view.busy}<p role="status">Operación local en curso… Las llamadas iniciadas conservan su cupo hasta terminar.</p>{/if}
  {#if view.error}<div class="preparation-error error" role="alert"><strong>{view.error.code}</strong><p>{view.error.message}</p>{#if view.error.details}<pre>{json(view.error.details)}</pre>{/if}{#if view.error.uncertain}<p>El estado puede haber cambiado. Consultá antes de decidir otra acción; no se repite la escritura.</p>{/if}</div>{/if}
  {#if view.receipt}<div class="preparation-receipt" role="status"><strong>Resultado de {view.receipt.operation}</strong><pre>{json(view.receipt.data)}</pre><p>Esta operación no ejecuta el paso siguiente. Consultá nuevamente para obtener las revisiones actuales.</p></div>{/if}
  {#if view.jobs}
    <p class="preparation-page">Página {view.pageNumber} · {view.jobs.items.length} trabajos de este adjunto. Snapshot por llamada.</p>
    {#if !view.jobs.items.length}<p>Sin trabajos para este ID. Esto no comprueba que exista el original.</p>{/if}
    <div class="preparation-jobs">{#each view.jobs.items as job (job.id)}
      <button class="preparation-job" disabled={view.busy} aria-pressed={view.jobId === job.id} onclick={() => controller.chooseJob(job.id)}>{job.id} · {job.extractor} v{job.extractor_version} · {job.state} · r{job.revision} · intentos {job.attempts}{!job.supported ? ' · incompatible' : ''}{job.result_from_previous_attempt ? ' · resultado anterior' : ''}</button>
    {/each}</div>
    {#if view.jobs.next_cursor}<button disabled={view.busy} onclick={() => void controller.list(true)}>Siguiente página de trabajos</button>{/if}
  {/if}
  {#if view.jobId}<p>Trabajo elegido: <code>{view.jobId}</code></p><button disabled={view.busy} onclick={() => void controller.inspect()}>Consultar trabajo elegido</button>{/if}
  {#if view.job && view.control && view.index}
    {@const job = view.job}
    <section class="preparation-inspection" aria-label="Procesamiento consultado">
      <h3>Procesamiento consultado</h3>
      <p>Consultas consecutivas independientes; pueden cambiar entre sí. Las escrituras verifican revisiones y generación en Rust.</p>
      <p>Estado <strong data-preparation="state">{job.state}</strong> · revisión <span data-preparation="revision">{job.revision}</span> · intentos {job.attempts} · soporte {job.supported ? 'compatible' : 'incompatible'} {job.unsupported_reason ?? ''}</p>
      <p>Ownership activo: {String(job.ownership_active)} · recuperable: {String(job.recoverable)} · resultado de intento anterior: {String(job.result_from_previous_attempt)}</p>
      {#if job.error}<div role="alert"><strong>Error del trabajo: {job.error.code}</strong><p>{job.error.message}</p><pre>{json(job.error.details)}</pre></div>{/if}
      <details><summary>Procedencia y configuración del trabajo</summary><pre>{json(job)}</pre></details>
      <p class="preparation-control">Cola global {view.control.paused ? 'pausada' : 'habilitada'} · revisión {view.control.revision}. Reanudar no ejecuta. Ejecutar por ID también puede recuperar intentos vencidos del workspace.</p>
      <div class="actions">
        <button disabled={view.busy} onclick={() => void controller.setPaused()}>{view.control.paused ? 'Reanudar cola global' : 'Pausar cola global'}</button>
        <button disabled={view.busy || !job.supported || job.state !== 'queued'} onclick={() => void controller.jobAction('run')}>Ejecutar trabajo elegido</button>
        <button disabled={view.busy || !['queued', 'running', 'cancelled'].includes(job.state)} onclick={() => void controller.jobAction('cancel')}>Cancelar trabajo elegido</button>
        <button disabled={view.busy || !job.supported || ['queued', 'running'].includes(job.state)} onclick={() => void controller.jobAction('retry')}>Reencolar trabajo elegido</button>
      </div>
      <h3>Generación e índice consultados</h3>
      {#if job.generation}<p>Generación <code data-preparation="generation">{job.generation.id}</code> · intento {job.generation.attempt} · completa {String(job.generation.complete)} · {job.generation.fragment_count} fragmentos internos. En XLSX/OCR son chunks JSON internos, no celdas ni regiones.</p>{:else}<p>Sin generación publicada.</p>{/if}
      <p>Índice {view.index.profile} · versión {view.index.version} · revisión <span data-preparation="index-revision">{view.index.revision}</span>. Cobertura filtrada por trabajo; get no verifica originales.</p>
      <pre class="preparation-index">{json(view.index)}</pre>
      <button disabled={!canPut} onclick={() => void controller.selectGeneration()}>Seleccionar generación para búsqueda</button>
      <p>Seleccionar no extrae ni busca. Una generación compatible sin texto puede seleccionarse con cero fragmentos. Reemplazar una generación invalida su selección anterior.</p>
    </section>
  {/if}
  <details class="preparation-limits"><summary>Límites y alcance local</summary>
    <p>Fuente hasta 1 MiB. UTF-8 estricto sin NUL: .txt/.md literal. PDF 1.0–1.7: 1–100 páginas, Type1 latino Standard/WinAnsi; sin OCR, orden visual, geometría ni tablas estructuradas. XLSX Transitional: hasta 32 hojas/10.000 celdas; fórmulas y cachés separadas, sin recálculo ni encabezados inferidos. OCR local macOS: PNG/JPEG de una imagen, hasta 4.096 por eje/4 Mp, 512 regiones y 1 MiB de representación. Texto inferido: puede faltar o estar equivocado, sin orden visual ni tablas garantizados. El núcleo valida extensión, MIME, contenido y configuración.</p>
    <p>Máximo 1.000 trabajos queued/running, 10.000 trabajos totales y 32 MiB derivados retenidos. Generación hasta 1 MiB/512 fragmentos de hasta 4.096 bytes. Índice hasta 1.024 selecciones/16.384 fragmentos/8 MiB de texto. Respuesta hasta 2 MiB; sin purga o truncamiento automático.</p>
    <p>Presupuesto cooperativo de ejecución 5 s, lease 15 s y contención 2.000 ms; no garantía de tiempo real ni límite RSS. Helpers PDF/XLSX descartables con heap solicitado 128 MiB, CPU 4 s y watchdog 5 s. OCR usa Vision r3/fast/es-ES/en-US disponible en el sistema, CPU 4 s, watchdog 5 s y RSS observado hasta 128 MiB con muestreo de 5 ms; no es límite duro RSS/GPU. Una ejecución sin motor falla explícitamente. No se instalan modelos ni se usan proveedores remotos. No se repara ni reinicia el índice desde este panel.</p>
  </details>
</details>

<style>
  .preparation-panel { margin-block: 2rem; padding: 1rem; border: 1px solid var(--border); border-radius: 8px; }
  summary { cursor: pointer; font-weight: 600; }
  p { line-height: 1.5; }
  pre, code, .preparation-job { white-space: pre-wrap; overflow-wrap: anywhere; word-break: break-word; }
  pre { max-height: 24rem; overflow: auto; padding: .75rem; background: var(--surface); }
  .preparation-jobs { display: grid; gap: .4rem; margin-block: .6rem; }
  .preparation-job { text-align: left; }
  .preparation-receipt, .preparation-inspection { margin-block: 1rem; padding: .75rem; border: 1px solid var(--border); }
  label { display: grid; gap: .35rem; margin-block: .6rem; }
  input, select { min-width: 0; max-width: 100%; }
</style>
