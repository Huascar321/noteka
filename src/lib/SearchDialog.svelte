<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { operate } from './api';
  import AppIcon from './AppIcon.svelte';
  import { ContextSearch, resultTarget, documentPreview, type ContextSearchState, type ContextResult } from './context-search';
  import { entityIcon, entityIconColor } from './entity-appearance';
  import type { NavigationTarget } from './navigation';
  import type { EntityKind } from './contracts.generated';
  let { workspace, mode, onclose, onchoose }: {
    workspace: string; mode: 'current' | 'new'; onclose: () => void;
    onchoose: (target: NavigationTarget, newTab: boolean) => Promise<void>;
  } = $props();
  let dialog: HTMLDialogElement;
  let input: HTMLInputElement;
  let query = $state('');
  let kind = $state<EntityKind | ''>('');
  let archived = $state(false);
  let related = $state('');
  let filters = $state(false);
  let reader = $state<ContextSearch>();
  let searchState = $state<ContextSearchState>({ results: [], selected: 0, preview: null, busy: true, previewBusy: false, error: '', previewError: '', cursor: null });
  const selected = $derived(searchState.results[searchState.selected]);
  const previewText = $derived(documentPreview(searchState.preview?.document));
  const payload = $derived((searchState.preview?.payload && typeof searchState.preview.payload === 'object' ? searchState.preview.payload : {}) as Record<string, unknown>);
  function resultIcon(result: ContextResult) {
    return result.type === 'section' ? result.section.icon : entityIcon(result.kind, result.title, result.entity?.payload);
  }
  function choose(result = selected, newTab = mode === 'new') {
    if (result) void onchoose(resultTarget(result), newTab);
  }
  function keydown(event: KeyboardEvent) {
    if (event.isComposing || event.target instanceof HTMLSelectElement || (event.target instanceof HTMLInputElement && event.target !== input)) return;
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault(); reader?.move(event.key === 'ArrowDown' ? 1 : -1);
      void tick().then(() => document.getElementById(`search-result-${searchState.selected}`)?.scrollIntoView({ block: 'nearest' }));
    } else if (event.key === 'Enter') {
      event.preventDefault(); choose(selected, mode === 'new' || event.metaKey || event.ctrlKey);
    }
  }
  $effect(() => {
    const criteria = { query, kind, archived, related }; const current = reader;
    if (!current) return;
    current.invalidate();
    const timer = setTimeout(() => { void current.search(criteria); }, query.trim() ? 160 : 0);
    return () => clearTimeout(timer);
  });
  onMount(() => {
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    reader = new ContextSearch(command => operate(command, workspace), value => { searchState = value; });
    dialog.showModal(); input.focus();
    return () => { reader?.dispose(); if (previous?.isConnected) previous.focus(); };
  });
</script>

<dialog bind:this={dialog} class="context-search glass-popup" aria-labelledby="context-search-title" oncancel={(event) => { event.preventDefault(); onclose(); }} onclick={(event) => { if (event.target === dialog) onclose(); }} onkeydown={keydown}>
  <h2 class="sr-only" id="context-search-title">{mode === 'new' ? "Open in a new tab" : "Search workspace"}</h2>
  <div class="context-search-input"><AppIcon name="search" size={16} /><input bind:this={input} bind:value={query} aria-label="Search workspace" role="combobox" aria-expanded="true" aria-controls="context-search-results" aria-activedescendant={selected ? `search-result-${searchState.selected}` : undefined} autocomplete="off" placeholder="Search" maxlength="256" /><button class="icon-button" aria-label="Close search" title="Close (Esc)" onclick={onclose}><AppIcon name="close" size={14} /></button></div>
  <div class="context-search-filters"><button class:active={filters} aria-expanded={filters} onclick={() => { filters = !filters; }}>Filters{kind || archived || related ? ' · activos' : ''}</button><span>{mode === 'new' ? "Open in new tab" : "Local search"}</span></div>
  {#if filters}<div class="context-filter-fields"><label>Type<select aria-label="Context search type" bind:value={kind}><option value="">All</option><option value="note">Notes</option><option value="task">Tasks</option><option value="project">Projects</option><option value="tag">Tags</option><option value="person">People</option></select></label><label>Local context<input aria-label="Context search scope" bind:value={related} placeholder="Project, tag or person ID" /></label><label class="checkbox-label"><input type="checkbox" bind:checked={archived} /> Archived only</label></div>{/if}
  <div class="context-search-body">
    <section class="context-results" aria-label="Search results">
      <p class="context-results-label">{query ? "Results" : "Go to a view or page"}</p>
      {#if searchState.error}<p role="alert" class="error">{searchState.error}</p>{/if}
      <div role="listbox" id="context-search-results" aria-label="Results" aria-busy={searchState.busy}>
        {#each searchState.results as result, index (result.key)}<button role="option" aria-selected={searchState.selected === index} id={`search-result-${index}`} class:active={searchState.selected === index} class="context-result" onfocus={() => { if (searchState.selected !== index) void reader?.select(index); }} onclick={(event) => choose(result, mode === 'new' || event.metaKey || event.ctrlKey)} onpointermove={() => { if (searchState.selected !== index) void reader?.select(index); }}><span class="context-result-icon"><AppIcon name={resultIcon(result)} size={18} /></span><span><strong>{result.type === 'section' ? result.section.label : result.title}</strong><small>{result.type === 'section' ? result.section.description : `${result.kind}${result.hit?.block_id ? ' · contenido' : ''}`}</small></span><span class="result-open" aria-hidden="true"><AppIcon name="arrow-up-right" size={16} /></span></button>{/each}
      </div>
      {#if searchState.busy}<p class="context-message" role="status">Searching…</p>{:else if !searchState.results.length && !searchState.error}<p class="context-message">No results. Try another term or filter.</p>{/if}
      {#if searchState.cursor}<button class="context-more" disabled={searchState.busy} onclick={() => void reader?.more()}>Load more results</button>{/if}
    </section>
    <section class="context-preview" aria-label="Preview">
      {#if selected?.type === 'section'}<div class="preview-path">Workspace / {selected.section.label}</div><AppIcon name={selected.section.icon} size={46} /><h2>{selected.section.label}</h2><p>{selected.section.description}</p><button class="preview-open" onclick={() => choose()}>Open {mode === 'new' ? "in a new tab" : 'vista'} <AppIcon name="arrow-up-right" size={16} /></button>
      {:else if searchState.previewBusy}<p class="context-message" role="status">Cargando vista previa…</p>
      {:else if searchState.previewError}<p role="alert" class="error">{searchState.previewError}</p>
      {:else if searchState.preview}{@const entity = searchState.preview}<div class="preview-path">Workspace / {entity.kind}{entity.archived_at ? ' / Archive' : ''}</div><AppIcon name={entityIcon(entity.kind, entity.title, entity.payload)} color={entityIconColor(entity.payload)} size={46} /><h2>{entity.title}</h2><dl><dt>Type</dt><dd>{entity.kind}</dd><dt>Updated</dt><dd>{new Date(entity.updated_at).toLocaleDateString('en-US')}</dd>{#each ['state','classification','due_date','planned_date'] as key}{#if typeof payload[key] === 'string'}<dt>{({ state: "Status", classification: "Tag type", due_date: "Due date", planned_date: "Planned" })[key]}</dt><dd>{String(payload[key])}</dd>{/if}{/each}</dl>{#if previewText}<div class="preview-document">{previewText}</div>{:else if selected?.type === 'entity' && selected.hit?.excerpt}<p class="preview-document">{selected.hit.excerpt}</p>{/if}<button class="preview-open" onclick={() => choose()}>Open page <AppIcon name="arrow-up-right" size={16} /></button>
      {:else}<p class="context-message">Select a page to see its content.</p>{/if}
    </section>
  </div>
  <footer class="context-search-footer"><span><kbd>↑ ↓</kbd> recorrer <kbd>↵</kbd> abrir <kbd>⌘/Ctrl ↵</kbd> new tab</span><button onclick={() => { void onchoose({ type: 'section', section: 'search' }, mode === 'new'); }}>Advanced search <AppIcon name="arrow-up-right" size={14} /></button></footer>
</dialog>
