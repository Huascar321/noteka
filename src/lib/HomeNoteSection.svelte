<script lang="ts">
  import SectionIcon from './SectionIcon.svelte';
  import AppIcon from './AppIcon.svelte';
  import HomeRow from './HomeItem.svelte';
  import type { HomeReadState } from './home-reads';
  import type { HomeView } from './contracts.generated';
  let { readState, onview, onnext, onretry, onopen, oncreate, onnavigate, pageMode = false, ontags = () => {} }: {
    pageMode?: boolean; ontags?: () => void;
    readState?: HomeReadState; onview: (view?: HomeView) => Promise<void>; onnext: () => Promise<void>;
    onretry: () => Promise<void>; onopen: (id: string) => Promise<void>;
    oncreate: (seed?: Record<string, unknown>) => Promise<void>; onnavigate: () => Promise<void>;
  } = $props();
  const views: { id: HomeView; label: string; icon: string }[] = [
    { id: 'notes_library', label: 'Notes', icon: 'file-text' },
    { id: 'notes_inbox', label: 'Inbox', icon: 'inbox' },
    { id: 'notes_favorites', label: 'Fav.', icon: 'star' },
    { id: 'notes_clips', label: 'Clips', icon: 'globe' },
    { id: 'notes_voice', label: 'Voice', icon: 'mic' },
    { id: 'notes_journal', label: 'Journal', icon: 'pen' },
    { id: 'notes_meetings', label: 'Meetings', icon: 'users' },
    { id: 'notes_all', label: 'All', icon: 'asterisk' },
  ];
  const page = $derived(readState?.page);
  let optionsOpen = $state(false);
  let root: HTMLElement;
  function create() {
    const type = ({ notes_clips: 'web_clip', notes_voice: 'voice', notes_journal: 'journal', notes_meetings: 'meeting' } as Record<string, string>)[readState?.view ?? ''];
    return oncreate({ inbox: readState?.view === 'notes_inbox', ...(type ? { note_type: type } : {}), ...(readState?.view === 'notes_favorites' ? { favorite: true } : {}) });
  }
</script>

<svelte:window onkeydown={(event) => { if (event.key === 'Escape' && root?.contains(document.activeElement)) optionsOpen = false; }} />

<section class="home-section home-tasks home-notes" class:tasks-page-section={pageMode} data-home-section="note" aria-label="Notes" aria-busy={readState?.busy ?? true} bind:this={root}>
  {#if pageMode}
    <header class="tasks-page-heading"><span class="tasks-page-emblem" aria-hidden="true"><SectionIcon section="note" size={68} /></span><h1>Notes</h1></header>
    <nav class="tasks-page-nav" aria-label="Notes quick navigation"><details><summary>Nav</summary>{#each views.filter(view => !['notes_inbox', 'notes_favorites', 'notes_all'].includes(view.id)) as view}<button onclick={() => void onview(view.id)}>{view.label}</button>{/each}</details><span aria-hidden="true">|</span><button onclick={() => void onview('notes_inbox')}>Inbox</button><span aria-hidden="true">|</span><button onclick={() => void onview('notes_favorites')}>Fav</button><span aria-hidden="true">|</span><button onclick={() => void onview('notes_all')}>All</button><span aria-hidden="true">|</span><button onclick={ontags}>Tags</button></nav>
  {:else}<header><h2><span class="task-heading-icon" aria-hidden="true"><SectionIcon section="note" size={31} /></span><button onclick={() => void onnavigate()}>Notes</button></h2></header>{/if}
  <div class="task-capture note-capture"><button onclick={() => void create()} aria-label="Create in Notes"><SectionIcon section="note" size={16} />New Note</button></div>
  <div class="task-view-toolbar">
    <div class="home-tabs task-view-tabs" role="group" aria-label="Notes views">
      {#each views as view}<button class:active={readState?.view === view.id} aria-pressed={readState?.view === view.id} title={view.label} onclick={() => void onview(view.id)}><AppIcon name={view.icon} size={16} /><span>{view.label}</span></button>{/each}
    </div>
    <div class="task-view-actions">
      <button class="task-icon-action" aria-label="Refresh Notes" title="Refresh Notes" disabled={readState?.busy} onclick={() => void onview()}><AppIcon name="refresh" size={16} /></button>
      {#if !pageMode}<button class="task-icon-action" aria-label="Open Notes list" title="Open Notes list" onclick={() => void onnavigate()}><AppIcon name="expand" size={16} /></button>{/if}
      <button class="task-icon-action" aria-label="Notes view options" aria-expanded={optionsOpen} title="View options" onclick={() => optionsOpen = !optionsOpen}><AppIcon name="sliders" size={16} /></button>
      <button class="task-new primary" onclick={() => void create()}>New</button>
    </div>
  </div>
  {#if optionsOpen}<div class="task-view-options"><p>Notes: local notes and general notes from the Notes database, excluding those archived at the source. Clips, Voice, Journal and Meetings use their explicit type. Fav. uses the favorite flag; Inbox preserves its local flag. All includes every active local page.</p><p>Imported age uses the last recorded source modification; local notes use their local modification. Voice does not record or transcribe audio.</p>{#if page}<p class="home-coverage">Loaded: {page.items.length}. In this view: {page.matching_count}.</p>{/if}</div>{/if}
  {#if readState?.busy}<p role="status" class="task-read-status">Loading Notes…</p>{/if}
  {#if readState?.error}<p class="error" role="alert">Read failed: {readState.error}</p><button onclick={() => void onretry()}>Retry Notes</button>{/if}
  {#if page}
    {#if page.unavailable_count}<p class="home-unavailable">{page.unavailable_count} notes with uninterpretable metadata for this view.</p>{/if}
    <div class="home-rows note-rows">{#each page.items as row (row.entity.id)}<HomeRow item={row} {onopen} noteStyle />{/each}{#if !page.items.length}<p class="home-empty">No Notes in this view.</p>{/if}</div>
    <button class="home-new task-add-row" onclick={() => void create()}><AppIcon name="plus" size={16} />New page</button>
    {#if page.next_cursor}<footer class="section-row-footer note-row-footer"><button class="section-row-action note-load-more" disabled={readState?.busy} onclick={() => void onnext()}><AppIcon name="chevron-down" size={14} />Load more</button></footer>{/if}
  {/if}
</section>
