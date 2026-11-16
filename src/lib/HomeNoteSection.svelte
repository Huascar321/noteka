<script lang="ts">
  import SectionIcon from './SectionIcon.svelte';
  import AppIcon from './AppIcon.svelte';
  import HomeRow from './HomeItem.svelte';
  import type { HomeReadState } from './home-reads';
  import HomeReadStatus from './HomeReadStatus.svelte';
  import type { HomeView } from './contracts.generated';
  import { slidingTabs } from './sliding-tabs';
  let { readState, onview, onnext, onretry, onopen, oncreate, onnavigate, pageMode = false }: {
    pageMode?: boolean;
    readState?: HomeReadState; onview: (view?: HomeView) => Promise<void>; onnext: () => Promise<void>;
    onretry: () => Promise<void>; onopen: (id: string) => Promise<void>;
    oncreate: (seed?: Record<string, unknown>) => Promise<void>; onnavigate: () => Promise<void>;
  } = $props();
  const views: { id: HomeView; label: string; icon: string }[] = [
    { id: 'notes_general', label: 'Notes', icon: 'file-text' },
    { id: 'notes_inbox', label: 'Inbox', icon: 'inbox' },
    { id: 'notes_favorites', label: 'Fav.', icon: 'star' },
    { id: 'notes_clips', label: 'Clips', icon: 'globe' },
    { id: 'notes_voice', label: 'Voice', icon: 'mic' },
    { id: 'notes_diary', label: 'Journal', icon: 'pen' },
    { id: 'notes_meetings', label: 'Meetings', icon: 'users' },
    { id: 'notes_all', label: 'All', icon: 'asterisk' },
    { id: 'notes_unclassified', label: 'Unclassified', icon: 'circle-dot' },
  ];
  const page = $derived(readState?.page);
  function create() {
    const type = ({ notes_clips: 'web_clip', notes_voice: 'voice', notes_diary: 'journal', notes_meetings: 'meeting' } as Record<string, string>)[readState?.view ?? ''];
    return oncreate({ ...(type ? { note_type: type } : {}), ...(readState?.view === 'notes_favorites' ? { favorite: true } : {}) });
  }
</script>

<section class="home-section home-tasks home-notes" class:tasks-page-section={pageMode} data-home-section="note" aria-label="Notes" aria-busy={readState?.busy ?? true}>
  {#if pageMode}
    <header class="tasks-page-heading"><span class="tasks-page-emblem" aria-hidden="true"><SectionIcon section="note" size={68} /></span><h1>Notes</h1></header>
  {:else}<header><h2><span class="task-heading-icon" aria-hidden="true"><SectionIcon section="note" size={31} /></span><button onclick={() => void onnavigate()}>Notes</button></h2></header>{/if}
  <div class="task-view-toolbar">
    <div class="home-tabs task-view-tabs" use:slidingTabs role="group" aria-label="Notes views">
      {#each views as view}<button class:active={readState?.view === view.id} aria-pressed={readState?.view === view.id} title={view.label} onclick={() => void onview(view.id)}><AppIcon name={view.icon} size={16} /><span>{view.label}</span></button>{/each}
    </div>
    <div class="task-view-actions"><HomeReadStatus state={readState} updating />
      <div class="glass-tool-group" role="group" aria-label="Notes tools"><button class="task-icon-action" aria-label="Refresh Notes" title="Refresh Notes" disabled={readState?.busy} onclick={() => void onview()}><AppIcon name="refresh" size={16} /></button>
      </div><button class="task-new primary" onclick={() => void create()}><AppIcon name="plus" size={16} />New</button>
    </div>
  </div>
  {#if readState?.busy && !page}<p role="status" class="task-read-status">Loading Notes…</p>{/if}
  {#if readState?.error}<p class="error" role="alert">Read failed: {readState.error}</p><button onclick={() => void onretry()}>Retry Notes</button>{/if}
  <HomeReadStatus state={readState} />
  {#if page}
    {#if page.unavailable_count}<p class="home-unavailable">{page.unavailable_count} notes with uninterpretable metadata for this view.</p>{/if}
    <div class="home-rows note-rows">{#each page.items as row (row.entity.id)}<HomeRow item={row} {onopen} noteStyle />{/each}{#if !page.items.length}<p class="home-empty">No Notes in this view.</p>{/if}</div>
    <button class="home-new task-add-row" onclick={() => void create()}><AppIcon name="plus" size={16} />New page</button>
    {#if page.next_cursor}<footer class="section-row-footer note-row-footer"><button class="section-row-action note-load-more" disabled={readState?.busy || readState?.stale} onclick={() => void onnext()}><AppIcon name="chevron-down" size={14} />Load more</button></footer>{/if}
  {/if}
</section>
