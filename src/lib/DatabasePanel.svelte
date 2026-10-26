<script lang="ts">
  import { onMount } from 'svelte';
  import { operate, OperationError } from './api';
  import CollectionPresentation from './CollectionPresentation.svelte';
  import type { Collection, CollectionBundle, CollectionView, Contract, Entity, Expression, PropertySpec, RecordData, ViewConfig, ViewLayout, DatabasePage, ViewGroup, Template, TemplateDefinition } from './contracts.generated';
  type Values = Record<string, unknown>;
  let { workspace, onpending, onbusy, onguard, onopen, onerror }: {
    workspace: string; onpending: (dirty: boolean) => void; onbusy: (busy: boolean) => void;
    onguard: (action: () => Promise<void>) => Promise<void>; onopen: (entity: Entity) => Promise<void>;
    onerror: (message: string) => void;
  } = $props();
  type Mode = 'collection' | 'property' | 'formula' | 'record' | 'attach' | 'values' | 'view' | 'template' | 'instance';
  let collections = $state<Collection[]>([]), collectionCursor = $state<string | null>(null);
  let current = $state<CollectionBundle | null>(null);
  let records = $state<RecordData[]>([]), recordCursor = $state<string | null>(null);
  let recordOffset = $state<string | undefined>();
  let viewId = $state(''), layout = $state<ViewLayout>('table'), bufferLayout = $state<ViewLayout>('table');
  let temporaryLayout = $state<ViewLayout | null>(null), pageConfig = $state<ViewConfig>({}), groups = $state<ViewGroup[]>([]);
  let matchingCount = $state(0), resultsReady = $state(false);
  let mode = $state<Mode | null>(null), title = $state(''), jsonInput = $state('{}');
  let entityId = $state(''), expectedRevision = $state(1), editingView: CollectionView | null = $state(null);
  let templates = $state<Template[]>([]), templateCursor = $state<string | null>(null);
  let editingTemplate = $state<Template | null>(null), expectedCollectionRevision = $state(1);
  let error = $state(''), message = $state(''), recovery = $state(''), busy = $state(false);
  function clearResults() { records = []; recordCursor = null; recordOffset = undefined; groups = []; resultsReady = false; matchingCount = 0; }
  function working(value: boolean) { busy = value; onbusy(value); }
  function report(e: unknown) { return e instanceof OperationError ? `${e.detail.code}: ${e.message}` : e instanceof Error ? e.message : String(e); }
  $effect(() => onerror(error));
  export function discardPending() { mode = null; error = ''; recovery = ''; onpending(false); }
  function copyBuffer() { recovery = JSON.stringify({ mode, collection_id: current?.collection.id, title, json: jsonInput, entity_id: entityId, expected_revision: mode === 'view' ? editingView?.revision : expectedRevision, layout: mode === 'view' ? bufferLayout : undefined, view_id: editingView?.id, template_id: editingTemplate?.id, expected_collection_revision: expectedCollectionRevision }, null, 2); }
  async function listTemplates(c: string, cursor?: string) {
    const page = await operate<Contract['templates']>({ operation: 'template.list', input: { collection_id: c, cursor, limit: 50 } }, workspace);
    templates = cursor ? [...templates, ...page.items] : page.items; templateCursor = page.next_cursor ?? null;
  }
  async function moreTemplates() { if (!current || busy) return; working(true); try { await listTemplates(current.collection.id, templateCursor ?? undefined); } catch (e) { error = report(e); } finally { working(false); } }
  async function beginTemplate(t?: Template, instantiate = false) {
    if (!current || (t && !t.editable)) return;
    await onguard(async () => {
      mode = instantiate ? 'instance' : 'template'; editingTemplate = t ?? null;
      expectedRevision = t?.revision ?? 1; expectedCollectionRevision = current!.collection.revision;
      title = instantiate ? '' : t?.name ?? "New template"; error = ''; recovery = '';
      jsonInput = JSON.stringify(instantiate ? {} : t?.definition ?? { schema_version: 1, title: "New record", document: { schema_version: 2, blocks: [{ id: crypto.randomUUID(), type: 'paragraph', content: [] }] }, values: {} }, null, 2);
      onpending(true);
    });
  }
  async function list(cursor?: string) {
    const page = await operate<Contract['collections']>({ operation: 'collection.list', input: { cursor, limit: 50 } }, workspace);
    collections = cursor ? [...collections, ...page.items] : page.items; collectionCursor = page.next_cursor ?? null;
  }
  async function query(cursor?: string, bundle = current, selectedId = viewId, temporary = temporaryLayout) {
    if (!bundle) return;
    const config = temporary ? { ...(bundle.views.find(v => v.id === selectedId)?.config ?? {}), group_by: undefined, calendar: undefined } : undefined;
    const page = await operate<DatabasePage>({ operation: 'database.query', input: { collection_id: bundle.collection.id, view_id: !temporary && selectedId ? selectedId : undefined, layout: temporary ?? undefined, config, limit: 50, cursor } }, workspace);
    current = { ...bundle, properties: page.properties }; pageConfig = page.config; layout = page.layout; groups = page.groups; matchingCount = page.matching_count;
    records = page.records.items; recordCursor = page.records.next_cursor ?? null; recordOffset = cursor; resultsReady = true;
  }
  export async function refreshPanel() {
    if (busy) return;
    working(true); error = '';
    try { await list(); if (current) { const bundle = await operate<CollectionBundle>({ operation: 'collection.get', input: { id: current.collection.id } }, workspace); await query(undefined, bundle); await listTemplates(bundle.collection.id); } }
    catch (e) { clearResults(); error = report(e); }
    finally { working(false); }
  }
  async function choose(c: Collection) {
    await onguard(async () => {
      working(true); error = ''; message = '';
      try { templates = []; templateCursor = null; const bundle = await operate<CollectionBundle>({ operation: 'collection.get', input: { id: c.id } }, workspace); await query(undefined, bundle, '', null); viewId = ''; temporaryLayout = null; await listTemplates(c.id); }
      catch (e) { clearResults(); error = report(e); }
      finally { working(false); }
    });
  }
  async function begin(next: Mode, record?: RecordData, updateView = false, calculationId?: string) {
    await onguard(async () => {
      mode = next; editingTemplate = null; title = ''; error = ''; recovery = ''; entityId = record?.entity.id ?? ''; expectedRevision = record?.entity.revision ?? current?.collection.revision ?? 1;
      editingView = updateView ? current?.views.find(v => v.id === viewId) ?? null : null;
      if (next === 'property') jsonInput = JSON.stringify({ name: "New property", type: 'text' }, null, 2);
      else if (next === 'formula') {
        title = current?.properties.find(p => p.id === calculationId)?.spec.name ?? "New calculation";
        jsonInput = JSON.stringify({ property_id: calculationId, expression: calculationId ? current?.calculations[calculationId] : { op: 'number', value: 1 } }, null, 2);
      }
      else if (next === 'view') { title = editingView?.name ?? "New view"; bufferLayout = editingView?.layout ?? (resultsReady ? layout : 'table'); jsonInput = JSON.stringify(editingView?.config ?? (resultsReady ? pageConfig : { columns: current?.properties.map(p => p.id) ?? [], filters: [], sorts: [] }), null, 2); }
      else if (next === 'values') {
        const writable = new Set(current?.properties.filter(p => !['title','created_at','updated_at','formula','rollup'].includes(p.spec.type)).map(p => p.id));
        jsonInput = JSON.stringify(Object.fromEntries(Object.entries(record!.values).filter(([key]) => writable.has(key))), null, 2);
      } else jsonInput = '{}';
      onpending(true);
    });
  }
  export async function savePending(): Promise<boolean> {
    if (!mode || busy) return !mode;
    const operation = mode, target = current?.collection.id;
    working(true); error = ''; message = '';
    let committed = false;
    try {
      if (operation === 'collection') {
        current = await operate<CollectionBundle>({ operation: 'collection.create', input: { name: title } }, workspace); viewId = ''; temporaryLayout = null;
      } else {
        if (!target) throw new Error("Select a collection.");
        const value = JSON.parse(jsonInput);
        if (operation === 'property') await operate({ operation: 'property.define', input: { collection_id: target, expected_revision: expectedRevision, spec: value as PropertySpec } }, workspace);
        if (operation === 'formula') await operate<CollectionBundle>({ operation: 'formula.set', input: { collection_id: target, expected_revision: expectedRevision, name: title, property_id: value.property_id, expression: value.expression as Expression } }, workspace);
        if (operation === 'record') await operate({ operation: 'record.create', input: { collection_id: target, title, values: value as Values, document: { schema_version: 2, blocks: [{ id: crypto.randomUUID(), type: 'paragraph', content: [] }] } } }, workspace);
        if (operation === 'attach' || operation === 'values') await operate({ operation: operation === 'attach' ? 'record.attach' : 'record.update', input: { collection_id: target, entity_id: entityId.trim(), expected_revision: expectedRevision, values: value as Values } }, workspace);
        if (operation === 'view') {
          const v = await operate<CollectionView>({ operation: 'view.save', input: { collection_id: target, name: title, layout: bufferLayout, config: value as ViewConfig, id: editingView?.id, expected_revision: editingView?.revision } }, workspace); viewId = v.id; temporaryLayout = null;
        }
        if (operation === 'template') await operate<Template>({ operation: 'template.save', input: { collection_id: target, expected_collection_revision: expectedCollectionRevision, id: editingTemplate?.id, expected_revision: editingTemplate ? expectedRevision : undefined, name: title, definition: value as TemplateDefinition } }, workspace);
        if (operation === 'instance') {
          if (!editingTemplate) throw new Error("Choose a confirmed template.");
          await operate<RecordData>({ operation: 'template.instantiate', input: { id: editingTemplate.id, expected_template_revision: expectedRevision, expected_collection_revision: expectedCollectionRevision, ...(title ? { title } : {}), values: value as Values } }, workspace);
        }
      }
      committed = true; discardPending(); message = "Saved to the local core.";
      await list(); const bundle = await operate<CollectionBundle>({ operation: 'collection.get', input: { id: current!.collection.id } }, workspace); await query(undefined, bundle); await listTemplates(bundle.collection.id); return true;
    } catch (e) { if (committed) clearResults(); error = committed ? `Save confirmed; refresh failed: ${report(e)}` : report(e); return committed; }
    finally { working(false); }
  }
  async function selectView() {
    if (!current) return;
    temporaryLayout = null;
    working(true); error = ''; try { await query(); } catch (e) { clearResults(); error = report(e); } finally { working(false); }
  }
  async function selectLayout(e: Event) {
    const next = (e.target as HTMLSelectElement).value as ViewLayout;
    if (!['table','list','gallery'].includes(next)) return;
    working(true); error = '';
    try { await query(undefined, current, viewId, next); temporaryLayout = next; }
    catch (e) { clearResults(); error = report(e); }
    finally { working(false); }
  }
  async function nextPage(kind: 'collections' | 'records') {
    if (busy) return;
    working(true); error = '';
    try { if (kind === 'collections') await list(collectionCursor ?? undefined); else await query(recordCursor ?? undefined); }
    catch (e) { clearResults(); error = report(e); }
    finally { working(false); }
  }
  onMount(() => { void refreshPanel(); return () => onpending(false); });
</script>

<section class="database-panel" aria-label="Local collections">
  <div class="actions"><button disabled={busy} onclick={() => void begin('collection')}>New collection</button><button disabled={busy} onclick={() => void refreshPanel()}>Refresh collections</button></div>
  <div class="collection-tabs">{#each collections as c (c.id)}<button disabled={busy} onclick={() => void choose(c)} class:active={current?.collection.id === c.id}>{c.name}</button>{/each}{#if collectionCursor}<button disabled={busy} onclick={() => void nextPage('collections')}>More collections</button>{/if}</div>
  {#if error}<p class="error" role="alert">{error}</p>{/if}{#if message}<p role="status">{message}</p>{/if}
  {#if current}
    <h1>{current.collection.name}</h1><p class="metadata">{current.collection.id} · esquema r{current.collection.revision} · Records = shared entities</p>
    <section aria-label="Collection templates"><h2>Templates</h2><button disabled={busy} onclick={() => void beginTemplate()}>New template</button>{#each templates as t (t.id)}<div class="template-item"><span>{t.name} · r{t.revision}</span><button disabled={busy || !t.editable} onclick={() => void beginTemplate(t)}>Edit template {t.name}</button><button disabled={busy || !t.editable} onclick={() => void beginTemplate(t, true)}>Create from template {t.name}</button><button onclick={() => recovery = JSON.stringify(t, null, 2)}>Recover template {t.name}</button>{#if !t.editable}<p class="error">Future format: definition can be recovered but not edited.</p>{/if}</div>{/each}{#if templateCursor}<button disabled={busy} onclick={() => void moreTemplates()}>More templates</button>{/if}</section>
    <details class="property-schema"><summary>Schema and property IDs</summary>{#each current.properties as p (p.id)}<p><strong>{p.spec.name}</strong> · {p.spec.type} · <code>{p.id}</code>{#if p.spec.currency} · {p.spec.currency}{/if}{#if p.spec.calculation_type} · {p.spec.calculation_type.kind} {p.spec.calculation_type.currency ?? ''} <button disabled={busy} onclick={() => void begin('formula', undefined, false, p.id)}>Edit calculation {p.spec.name}</button>{/if}</p>{/each}</details>
    <div class="actions"><button disabled={busy} onclick={() => void begin('property')}>Define property</button><button disabled={busy} onclick={() => void begin('formula')}>Define calculation</button><button disabled={busy} onclick={() => void begin('record')}>New record</button><button disabled={busy} onclick={() => void begin('attach')}>Add entity by ID</button><button disabled={busy} onclick={() => void begin('view')}>New view</button>{#if viewId}<button disabled={busy} onclick={() => void begin('view', undefined, true)}>Edit saved view</button>{/if}</div>
    <label>Saved view<select aria-label="Collection view" disabled={busy || mode !== null} bind:value={viewId} onchange={() => void selectView()}><option value="">All active records</option>{#each current.views as v (v.id)}<option value={v.id}>{v.name} · {v.layout}</option>{/each}</select></label>
    {#if !mode}<label>Temporary layout<select aria-label="Collection layout" disabled={busy} value={layout} onchange={(e) => void selectLayout(e)}><option value="table">Table</option><option value="list">List</option><option value="gallery">Gallery</option><option value="board" disabled>Saved board</option><option value="calendar" disabled>Saved calendar</option></select></label>{/if}
  {/if}
  {#if mode}<form class="collection-buffer" onsubmit={(e) => { e.preventDefault(); void savePending(); }}>
    <h3>Draft: {mode}</h3>
    {#if ['collection','record','view','formula','template','instance'].includes(mode)}<label>Name / title{mode === 'instance' ? " (optional; leave blank for the default)" : ''}<input aria-label="Collection draft name" bind:value={title} disabled={busy} /></label>{/if}
    {#if mode === 'template' || mode === 'instance'}<p class="metadata">Template {editingTemplate?.id ?? "new"} · captured revision {editingTemplate ? expectedRevision : "new"} · captured schema r{expectedCollectionRevision}. Refresh does not rebase the draft. {mode === 'template' ? "Definition: schema_version 1, title, document and values; static defaults." : 'Override parcial: omitir conserva default; null limpia; bloques/entidad nuevos, referencias compartidas.'}</p>{/if}
    {#if mode === 'attach'}<label>Original entity ID<input aria-label="Entity to add" bind:value={entityId} disabled={busy} /></label>{/if}
    {#if mode === 'attach' || mode === 'values'}<label>Expected revision<input aria-label="Record revision" type="number" min="1" bind:value={expectedRevision} disabled={busy} /></label>{/if}
    {#if mode === 'view'}<label>Layout<select aria-label="Draft layout" bind:value={bufferLayout} disabled={busy}><option value="table">Table</option><option value="list">List</option><option value="board">Board</option><option value="calendar">Calendar</option><option value="gallery">Gallery</option></select></label><p class="metadata">Board: group_by = select/checkbox ID. Calendar: calendar = {JSON.stringify({ property_id: 'ID date/date_range', month: 'YYYY-MM' })}. Compound filters: filter_tree with condition/all/any/not. Text gallery; no preview downloads.</p>{/if}
    {#if mode !== 'collection'}<label>{mode === 'property' ? "Property definition JSON" : mode === 'view' ? "View configuration JSON" : mode === 'formula' ? "AST JSON: expression and optional property_id" : "Value patch JSON by property ID"}<textarea aria-label="Collection draft JSON" bind:value={jsonInput} disabled={busy} spellcheck="false"></textarea></label><p class="metadata">Decimal and monetary values are strings. Null clears a value; omission preserves it. Titles, timestamps and calculations are derived. No code or SQL is executed.</p>{/if}
    <div class="actions"><button disabled={busy}>{busy ? "Saving…" : "Save collection"}</button><button type="button" disabled={busy} onclick={discardPending}>Discard form</button><button type="button" onclick={copyBuffer}>Recover draft JSON</button></div>
  </form>{/if}
  {#if recovery}<label>JSON recuperable<textarea readonly aria-label="Collection recovery" value={recovery}></textarea></label>{/if}
  {#if current && resultsReady}
    <CollectionPresentation page={{ properties: current.properties, config: pageConfig, records: { items: records, next_cursor: recordCursor }, layout, matching_count: matchingCount, groups }} {busy} {onopen} onedit={(record) => begin('values', record)} />
    {#if recordCursor}<button disabled={busy} onclick={() => void nextPage('records')}>Next page</button>{/if}{#if recordOffset}<button disabled={busy} onclick={() => void refreshPanel()}>Back to the beginning</button>{/if}
  {/if}
</section>
