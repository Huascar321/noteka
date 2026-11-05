<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { operate } from './api';
  import { GroupedTextSearch } from './grouped-text-search';
  import { RefreshCoordinator } from './refresh-coordinator';
  import type { SearchHit } from './contracts.generated';
  let { workspace, onopen }: { workspace: string; onopen: (hit: SearchHit) => Promise<void> } = $props();
  let query = $state('');
  const controller = new GroupedTextSearch((command, signal) => operate(command, workspace, { signal }), next => { view = next; });
  let view = $state(controller.state);
  const json = (value: unknown) => JSON.stringify(value, null, 2);
  const refresh = new RefreshCoordinator(() => !view.notes.busy && !view.attachments.busy, () => controller.search(query), () => {});
  export function refreshPanel() { return refresh.request(); }
  $effect(() => { if (!view.notes.busy && !view.attachments.busy) untrack(() => refresh.notifyReady()); });
  onMount(() => () => { refresh.dispose(); controller.dispose(); });
</script>

<div class="grouped-search" aria-label="Consulta conjunta local">
  <form onsubmit={event => { event.preventDefault(); void controller.search(query); }}>
    <label>Combined text<input class="glass-field" aria-label="Combined text" bind:value={query} oninput={() => { refresh.cancelPending(); controller.edit(); }} placeholder="1–16 terms · up to 256 UTF-8 bytes" /></label>
    <button class="glass-secondary" disabled={!query.trim() || view.notes.busy || view.attachments.busy}>Search notes and attachments</button>
  </form>
  <p>Active entities and selected generations. Two independent queries; pages are not a shared snapshot and scores/cursors are not merged. For file, context or extractor filters, use the individual scopes.</p>
  <section class="grouped-notes" aria-label="Active entity results">
    <h3>Notes and active entities</h3>
    {#if view.notes.busy}<p role="status">Searching entities…</p>
    {:else if view.notes.error}<div role="alert" class="error"><strong>{view.notes.error.code}</strong><p>{view.notes.error.message}</p><pre>{json(view.notes.error.details)}</pre><button class="glass-secondary" onclick={() => void controller.startNotes()}>Retry entities</button></div>
    {:else if view.notes.page}
      <p>Page {view.notes.pageNumber} · {view.notes.page.items.length} entity hits.</p>
      {#each view.notes.page.items as hit, i (`${hit.entity_id}:${hit.block_id ?? ''}:${i}`)}<article class="grouped-note-hit"><button class="glass-secondary" onclick={() => void onopen(hit)}>{hit.title}</button><p>{hit.kind} · entidad {hit.entity_id} · revision {hit.revision} · {hit.block_id ? `bloque ${hit.block_id}` : "title"}</p><pre class="grouped-note-excerpt">{hit.excerpt}</pre><details><summary>Entity source</summary><pre>{json(hit)}</pre></details></article>{/each}
      {#if !view.notes.page.items.length}<p>No matches among active entities.</p>{/if}
      {#if view.notes.page.next_cursor}<button class="glass-secondary" onclick={() => void controller.nextNotes()}>Next entity page</button>{/if}<button class="glass-secondary" onclick={() => void controller.startNotes()}>First entity page</button>
    {:else}<p>Entities have not been queried yet.</p>{/if}
  </section>
  <section class="grouped-attachments" aria-label="Selected attachment results">
    <h3>Selected attachments</h3>
    {#if view.attachments.busy}<p role="status">Searching attachments…</p>
    {:else if view.attachments.error}<div role="alert" class="error"><strong>{view.attachments.error.code} · {view.attachments.error.reason ?? ''}</strong><p>{view.attachments.error.message}</p><button class="glass-secondary" onclick={() => void controller.startAttachments()}>Retry attachments</button></div>
    {:else if view.attachments.page}
      <p>Page {view.attachments.pageNumber} · {view.attachments.page.items.length} fragments on this page.</p>
      <details><summary>Attachment coverage</summary><pre class="grouped-coverage">{json({ ...view.attachments.page.coverage, profile: view.attachments.page.profile, version: view.attachments.page.version, revision: view.attachments.page.revision })}</pre></details>
      <p>Index and postings verified; originals are checked only for page/lookahead candidates at query time. Coverage is not a total hit count or proof of later freshness or universal absence. Importing/extracting does not select generations.</p>
      {#each view.attachments.page.items as hit (`${hit.source.job_id}:${hit.source.generation_id}:${hit.ordinal}`)}<article class="grouped-attachment-hit"><h4>{hit.locator.kind} · fragmento {hit.ordinal}</h4>{#if hit.result_from_previous_attempt}<p>Result from an earlier attempt.</p>{/if}<pre class="grouped-attachment-text">{hit.text}</pre><details><summary>Provenance and locator</summary><pre class="grouped-attachment-source">{json(hit.source)}</pre><pre class="grouped-attachment-locator">{json(hit.locator)}</pre></details><p>{hit.locator.kind === 'utf8_bytes' ? "Original bytes/lines; exclusive end, not JavaScript indexes." : hit.locator.kind === 'pdf_page_text' ? "Derived page/bytes; no OCR, visual order, geometry or structured tables." : hit.locator.kind === 'pdfkit_page_text' ? "PDFKit: page/range of derived text; full profile and engine/OS details in the locator. OCR, visual order, geometry and tables are not guaranteed." : hit.locator.kind === 'pdf_ocr_region_text' ? "Inferred PDF OCR: page/region of the derived raster; visual order and tables are not guaranteed. Normalized bounding box, bottom-left origin; confidence is not a probability. Full engine/OS and raster hash details in the locator." : hit.locator.kind === 'image_ocr_region_text' ? "Inferred OCR may be missing or incorrect; visual order and tables are not guaranteed. Bounding box of the oriented image, bottom-left origin; confidence is not a probability. Bytes refer to derived text, not the original." : "Derived XLSX field: separate formula and cache, no recalculation or ZIP offsets. The range may cover only part of the field."}</p></article>{/each}
      {#if !view.attachments.page.items.length}<p>No matches in selected generations. This does not prove the original is visually empty or OCR is available.</p>{/if}
      {#if view.attachments.page.next_cursor}<button class="glass-secondary" onclick={() => void controller.nextAttachments()}>Next combined attachment page</button>{/if}<button class="glass-secondary" onclick={() => void controller.startAttachments()}>First combined attachment page</button>
    {:else}<p>Attachments have not been queried yet.</p>{/if}
  </section>
  <p>Errors and retries are independent per group. Searching and inspecting do not import, enqueue, run, index, reset or repair. Content is presented as data, without remote services.</p>
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
