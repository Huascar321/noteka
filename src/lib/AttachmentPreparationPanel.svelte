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
  <summary>Prepare local attachments</summary>
  <p>Choose an existing managed original. Each step is explicit: enqueue → resume if needed → run → query → select generation. Search only searches.</p>
  <p>Leaving this panel does not cancel an active operation. There is no polling. Query again to see changes made by other clients.</p>
  {#if refs.length}
    <label>References in this note<select class="glass-field" aria-label="Reference to prepare" value="" onchange={e => controller.select(e.currentTarget.value)}>
      <option value="" disabled>Choose a reference</option>
      {#each refs as ref (ref.id)}<option value={ref.id}>{ref.id} · Block label: {ref.label}</option>{/each}
    </select></label>
    <p>Labels belong to the document, may be unsaved, and are not file metadata or evidence of exclusive ownership.</p>
  {/if}
  <label>Managed ID to prepare<input class="glass-field" aria-label="Managed ID to prepare" value={view.attachmentId} oninput={e => controller.select(e.currentTarget.value)} /></label>
  <div class="actions preparation-actions">
    <button class="glass-secondary" disabled={view.busy || !view.attachmentId} onclick={() => void controller.list()}>Query jobs</button>
    <label>Preparation extractor<select class="glass-field" aria-label="Preparation extractor" bind:value={extractor} disabled={view.busy}>
      <option value="utf8_text">utf8_text · .txt/.md literal</option><option value="pdfium_text">pdfium_text · PDF universal</option><option value="pdf_text">pdf_text · PDF textual acotado</option><option value="xlsx_cells">xlsx_cells · XLSX Transitional acotado</option><option value="image_ocr">image_ocr · PNG/JPEG inferido local</option><option value="pdfkit_text">pdfkit_text · PDFKit seleccionable local</option><option value="paddle_ocr">paddle_ocr · PDF OCR universal</option><option value="pdf_ocr">pdf_ocr · PDF inferido local</option>
    </select></label>
    <button class="glass-secondary" disabled={view.busy || !view.attachmentId || view.needsConsult} onclick={() => void controller.enqueue(extractor)}>Enqueue extraction</button>
  </div>
  {#if view.busy}<p role="status">Local operation in progress… Active calls retain their slot until they finish.</p>{/if}
  {#if view.error}<div class="preparation-error error" role="alert"><strong>{view.error.code}</strong><p>{view.error.message}</p>{#if view.error.details}<pre>{json(view.error.details)}</pre>{/if}{#if view.error.uncertain}<p>The state may have changed. Query before choosing another action; the write is not repeated.</p>{/if}</div>{/if}
  {#if view.receipt}<div class="preparation-receipt" role="status"><strong>Result of {view.receipt.operation}</strong><pre>{json(view.receipt.data)}</pre><p>This operation does not run the next step. Query again for current revisions.</p></div>{/if}
  {#if view.jobs}
    <p class="preparation-page">Page {view.pageNumber} · {view.jobs.items.length} jobs for this attachment. Snapshot per call.</p>
    {#if !view.jobs.items.length}<p>No jobs for this ID. This does not confirm that the original exists.</p>{/if}
    <div class="preparation-jobs">{#each view.jobs.items as job (job.id)}
      <button class="preparation-job" disabled={view.busy} aria-pressed={view.jobId === job.id} onclick={() => controller.chooseJob(job.id)}>{job.id} · {job.extractor} v{job.extractor_version} · {job.state} · r{job.revision} · intentos {job.attempts}{!job.supported ? ' · incompatible' : ''}{job.result_from_previous_attempt ? " · previous result" : ''}</button>
    {/each}</div>
    {#if view.jobs.next_cursor}<button class="glass-secondary" disabled={view.busy} onclick={() => void controller.list(true)}>Next jobs page</button>{/if}
  {/if}
  {#if view.jobId}<p>Selected job: <code>{view.jobId}</code></p><button class="glass-secondary" disabled={view.busy} onclick={() => void controller.inspect()}>Query selected job</button>{/if}
  {#if view.job && view.control && view.index}
    {@const job = view.job}
    <section class="preparation-inspection" aria-label="Procesamiento consultado">
      <h3>Procesamiento consultado</h3>
      <p>Consecutive queries are independent and may differ. Writes verify revisions and generation in Rust.</p>
      <p>Status <strong data-preparation="state">{job.state}</strong> · revision <span data-preparation="revision">{job.revision}</span> · intentos {job.attempts} · soporte {job.supported ? 'compatible' : 'incompatible'} {job.unsupported_reason ?? ''}</p>
      <p>Ownership activo: {String(job.ownership_active)} · recuperable: {String(job.recoverable)} · result from an earlier attempt: {String(job.result_from_previous_attempt)}</p>
      {#if job.error}<div role="alert"><strong>Job error: {job.error.code}</strong><p>{job.error.message}</p><pre>{json(job.error.details)}</pre></div>{/if}
      <details><summary>Job provenance and configuration</summary><pre>{json(job)}</pre></details>
      <p class="preparation-control">Cola global {view.control.paused ? 'pausada' : 'habilitada'} · revision {view.control.revision}. Resuming does not run jobs. Running by ID may also recover expired attempts in the workspace.</p>
      <div class="actions">
        <button class="glass-secondary" disabled={view.busy} onclick={() => void controller.setPaused()}>{view.control.paused ? "Resume global queue" : "Pause global queue"}</button>
        <button class="glass-secondary" disabled={view.busy || !job.supported || job.state !== 'queued'} onclick={() => void controller.jobAction('run')}>Run selected job</button>
        <button class="glass-secondary" disabled={view.busy || !['queued', 'running', 'cancelled'].includes(job.state)} onclick={() => void controller.jobAction('cancel')}>Cancel selected job</button>
        <button class="glass-secondary" disabled={view.busy || !job.supported || ['queued', 'running'].includes(job.state)} onclick={() => void controller.jobAction('retry')}>Reencolar trabajo elegido</button>
      </div>
      <h3>Queried generation and index</h3>
      {#if job.generation}<p>Generation <code data-preparation="generation">{job.generation.id}</code> · intento {job.generation.attempt} · completa {String(job.generation.complete)} · {job.generation.fragment_count} internal fragments. For XLSX/OCR these are internal JSON chunks, not cells or regions.</p>{:else}<p>No published generation.</p>{/if}
      <p>Index {view.index.profile} · version {view.index.version} · revision <span data-preparation="index-revision">{view.index.revision}</span>. Coverage is filtered by job; get does not verify originals.</p>
      <pre class="preparation-index">{json(view.index)}</pre>
      <button class="glass-secondary" disabled={!canPut} onclick={() => void controller.selectGeneration()}>Select generation for search</button>
      <p>Selecting does not extract or search. A compatible generation with no text can be selected with zero fragments. Replacing a generation invalidates its previous selection.</p>
    </section>
  {/if}
  <details class="preparation-limits"><summary>Local limits and scope</summary>
    <p>Source up to 1 MiB. Strict UTF-8 without NUL: literal .txt/.md. PDF 1.0–1.7: 1–100 pages, Latin Type1 Standard/WinAnsi; no OCR, visual order, geometry or structured tables. XLSX Transitional: up to 32 sheets/10,000 cells; separate formulas and caches, no recalculation or inferred headers. Local macOS OCR: single-image PNG/JPEG, up to 4,096 pixels per axis/4 Mp, 512 regions and 1 MiB representation. Inferred text may be missing or incorrect; visual order and tables are not guaranteed. The core validates extension, MIME, content and configuration.</p>
    <p>Up to 1,000 queued/running jobs, 10,000 total jobs and 32 MiB retained output. Generations up to 1 MiB/512 fragments of up to 4,096 bytes. Index up to 1,024 selections/16,384 fragments/8 MiB of text. Responses up to 2 MiB; no automatic purging or truncation.</p>
    <p>Cooperative execution budget: 5 s, lease: 15 s, contention: 2,000 ms; no real-time guarantee or RSS limit. Disposable PDF/XLSX helpers request 128 MiB heap, 4 s CPU and a 5 s watchdog. OCR uses system Vision r3/fast/es-ES/en-US, 4 s CPU, a 5 s watchdog and observed RSS up to 128 MiB sampled every 5 ms; this is not a hard RSS/GPU limit. Execution without an engine fails explicitly. No models are installed or remote providers used. This panel does not repair or reset the index.</p>
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
  .preparation-actions { align-items: end; }
  .preparation-actions label { margin-block: 0; }
  .preparation-actions > button { margin-bottom: 2px; }
</style>
