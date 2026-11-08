<script lang="ts">
  import QuickNavMenu from './QuickNavMenu.svelte';
  import { liquidSurface, liquidTransition } from './liquid-overlay';
  import SectionIcon from './SectionIcon.svelte';
  import AppIcon from './AppIcon.svelte';
  import HomeTagRow from './HomeTagRow.svelte';
  import HomeTagTable from './HomeTagTable.svelte';
  import { homeGroupLabel, type HomeReadState } from './home-reads';
  import HomeReadStatus from './HomeReadStatus.svelte';
  import type { HomeView, Classification } from './contracts.generated';
  import { slidingTabs } from './sliding-tabs';
  let { readState, onview, onnext, onretry, onopen, oncreate, onnavigate, pageMode = false, standalone = false, onsection = async () => {} }: {
    pageMode?: boolean; standalone?: boolean; onsection?: (kind: 'task' | 'note' | 'project') => Promise<void>;
    readState?: HomeReadState; onview: (view?: HomeView) => Promise<void>; onnext: () => Promise<void>;
    onretry: () => Promise<void>; onopen: (id: string) => Promise<void>;
    oncreate: (seed?: Record<string, unknown>) => Promise<void>; onnavigate: () => Promise<void>;
  } = $props();
  const views: { id: HomeView; label: string; icon: string }[] = [
    { id: 'tags_recent', label: 'Recent', icon: 'clock' }, { id: 'tags_favorites', label: 'Fav.', icon: 'star' },
    { id: 'tags_alphabetical', label: 'A-Z', icon: 'list' }, { id: 'tags_types', label: 'Types', icon: 'shapes' },
  ];
  const page = $derived(readState?.page);
  let optionsOpen = $state(false);
  let tableMode = $state(false);
  let root: HTMLElement;
  function selectView(view: HomeView) { tableMode = false; return onview(view); }
  function selectTable() { tableMode = true; return onview('tags_alphabetical'); }
  function create(classification: Classification = 'resource') { return oncreate({ classification, ...(readState?.view === 'tags_favorites' ? { favorite: true } : {}) }); }
</script>

<svelte:window onkeydown={(event) => { if (event.key === 'Escape' && root?.contains(document.activeElement)) optionsOpen = false; }} />

<section class="home-section home-tasks home-tags" class:notes-page-tags={pageMode && !standalone} class:tasks-page-section={standalone} data-home-section="tag" aria-label="Tags" aria-busy={readState?.busy ?? true} bind:this={root}>
  {#if standalone}
    <header class="tasks-page-heading"><span class="tasks-page-emblem" aria-hidden="true"><SectionIcon section="tag" size={68} /></span><h1>Tags</h1></header>
    <nav class="tasks-page-nav" aria-label="Tags quick navigation"><QuickNavMenu>{#each views as view}<button onclick={() => void selectView(view.id)}>{view.label}</button>{/each}<button onclick={() => void selectTable()}>Table</button></QuickNavMenu><span aria-hidden="true">|</span><button onclick={() => void onsection('task')}>Tasks</button><span aria-hidden="true">|</span><button onclick={() => void onsection('note')}>Notes</button><span aria-hidden="true">|</span><button onclick={() => void onsection('project')}>Projects</button></nav>
  {:else}<header><h2><span class="task-heading-icon" aria-hidden="true"><SectionIcon section="tag" size={31} /></span><button onclick={() => void onnavigate()}>Tags</button></h2></header>{/if}
  <div class="task-view-toolbar tag-view-toolbar">
    <div class="home-tabs task-view-tabs" use:slidingTabs role="group" aria-label="Tags views">{#each views as view}<button class:active={!tableMode && readState?.view === view.id} aria-pressed={!tableMode && readState?.view === view.id} onclick={() => void selectView(view.id)}><AppIcon name={view.icon} size={16} /><span>{view.label}</span></button>{/each}{#if standalone}<button class:active={tableMode} aria-pressed={tableMode} onclick={() => void selectTable()}><AppIcon name="columns" size={16} /><span>Table</span></button>{/if}</div>
    <div class="task-view-actions"><HomeReadStatus state={readState} updating />
      <div class="glass-tool-group" role="group" aria-label="Tags tools"><button class="task-icon-action" aria-label="Refresh Tags" title="Refresh Tags" disabled={readState?.busy} onclick={() => void onview()}><AppIcon name="refresh" size={16} /></button>
      {#if !standalone}<button class="task-icon-action" aria-label="Open Tags list" title="Open Tags list" onclick={() => void onnavigate()}><AppIcon name="expand" size={16} /></button>{/if}
      <button class="task-icon-action" aria-label="Tags view options" aria-expanded={optionsOpen} title="View options" onclick={() => optionsOpen = !optionsOpen}><AppIcon name="sliders" size={16} /></button>
      </div><button class="task-new primary" onclick={() => void create()}><AppIcon name="plus" size={16} />New</button>
    </div>
  </div>
  {#if optionsOpen}<div class="task-view-options" use:liquidSurface={{ anchor: () => root?.querySelector('[aria-label$="view options"]') }} in:liquidTransition|global out:liquidTransition|global><p>Types groups Area, Resource and Entity; Topic keeps its own type. Fav. uses the explicit flag. Views exclude source-archived tags; original entities are preserved.</p><p>Counts and latest notes use local relations and exclude notes archived locally or at the source. The latest note uses its available original date. Recent combines imported activity with related note dates; the source is a snapshot without continuous synchronization.</p>{#if standalone}<p>Table shows Tag, Last Note and Type in local A–Z order with the same pagination.</p>{/if}{#if page}<p class="home-coverage">Loaded: {page.items.length}. In this view: {page.matching_count}.</p>{/if}</div>{/if}
  {#if readState?.busy && !page}<p role="status" class="task-read-status">Loading Tags…</p>{/if}
  {#if readState?.error}<p class="error" role="alert">Read failed: {readState.error}</p><button onclick={() => void onretry()}>Retry Tags</button>{/if}
  <HomeReadStatus state={readState} />
  {#if page}
    {#if page.unavailable_count}<p class="home-unavailable">{page.unavailable_count} tags with uninterpretable metadata for this view.</p>{/if}
    {#if standalone && tableMode}<HomeTagTable items={page.items} {onopen} />
    {:else if readState?.view === 'tags_types'}<div class="tag-groups">{#each page.groups as group (group.key)}<details open class="tag-group" data-classification={group.classification}>
      <summary><span class="tag-type-pill" data-classification={group.classification}><span class="tag-type-dot" aria-hidden="true"></span>{homeGroupLabel(group, readState?.view)}</span><span class="tag-group-count" title="Tags on this page">{group.entity_ids.length}</span></summary>
      {#each group.entity_ids as id (id)}{@const row = page.items.find(r => r.entity.id === id)}{#if row}<HomeTagRow item={row} {onopen} showCount={!pageMode} />{/if}{/each}
      {#if group.classification && group.entity_ids.length}<button class="home-new task-add-row tag-add-row" aria-label={`New page in ${homeGroupLabel(group, readState?.view)}`} onclick={() => void create(group.classification!)}><AppIcon name="plus" size={16} />New page</button>{/if}
    </details>{/each}</div>
    {:else}<div class="tag-rows">{#each page.items as row (row.entity.id)}<HomeTagRow item={row} {onopen} showType showCount={!pageMode} />{/each}{#if !page.items.length}<p class="home-empty">No Tags in this view.</p>{/if}</div>{/if}
    {#if pageMode}<button class="home-new task-add-row" onclick={() => void create()}><AppIcon name="plus" size={16} />New page</button>{:else}<footer class="task-coverage-footer tag-coverage-footer"><button class="tag-footer-new" onclick={() => void create()}><AppIcon name="plus" size={16} />New page</button></footer>{/if}
    {#if page.next_cursor}<footer class="section-row-footer"><button class="section-row-action" disabled={readState?.busy || readState?.stale} onclick={() => void onnext()}><AppIcon name="arrow-down" size={15} />Load more</button></footer>{/if}
  {/if}
</section>
