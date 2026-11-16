<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { operate } from './api';
  import HomeNoteSection from './HomeNoteSection.svelte';
  import HomeTagSection from './HomeTagSection.svelte';
  import { HomeReader, type HomeReadState } from './home-reads';
  import { ReadQueue } from './embedded-reads';
  import type { HomePage, HomeView, EntityKind, ContextLink } from './contracts.generated';

  let { workspace, active = true, blocked = false, error = '', onopen, oncreate }: {
    workspace: string; active?: boolean; blocked?: boolean; error?: string;
    onopen: (id: string) => Promise<void>;
    oncreate: (kind: EntityKind, context?: ContextLink, seed?: Record<string, unknown>) => Promise<void>;
  } = $props();
  let noteState = $state<HomeReadState>();
  let tagState = $state<HomeReadState>();
  let root: HTMLElement;
  let tagsAnchor: HTMLDivElement;
  const queue = new ReadQueue(2);
  let notes: HomeReader | undefined;
  let tags: HomeReader | undefined;

  onMount(() => {
    notes = new HomeReader('notes_general', queue, (query, signal) => operate<HomePage>({ operation: 'home.query', input: query }, workspace, { signal }), state => noteState = state, 10, true);
    tags = new HomeReader('tags_recent', queue, (query, signal) => operate<HomePage>({ operation: 'home.query', input: query }, workspace, { signal }), state => tagState = state, 10, true);
    void notes.load(); void tags.load();
    window.addEventListener('keydown', refreshKeydown, true);
    return () => { notes?.dispose(); tags?.dispose(); window.removeEventListener('keydown', refreshKeydown, true); };
  });

  async function load(reader: HomeReader | undefined, kind: 'note' | 'tag', view?: HomeView, next = false, retry = false) {
    if (!reader) return;
    const focused = document.activeElement instanceof HTMLButtonElement ? document.activeElement : null;
    const label = focused?.textContent;
    if (retry) await reader.retry();
    else await reader.load(view, next ? reader.state.page?.next_cursor ?? undefined : undefined);
    await tick();
    if (focused && document.activeElement === document.body) {
      const section = root?.querySelector(`[data-home-section="${kind}"]`);
      const replacement = [...(section?.querySelectorAll<HTMLButtonElement>('button') ?? [])].find(button => button.textContent === label && !button.disabled)
        ?? section?.querySelector<HTMLButtonElement>('.home-tabs [aria-pressed="true"]');
      if (focused.isConnected && !focused.disabled) focused.focus(); else replacement?.focus();
    }
  }
  export async function refreshPanel() { await Promise.all([load(notes, 'note'), load(tags, 'tag')]); }
  function refreshKeydown(event: KeyboardEvent) {
    if (!active || event.defaultPrevented) return;
    const commandR = (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'r';
    const f5 = event.key === 'F5' && !event.metaKey && !event.ctrlKey;
    if (event.altKey || event.shiftKey || (!commandR && !f5)) return;
    event.preventDefault();
    if (blocked || event.isComposing || event.repeat || document.querySelector('dialog[open]')) return;
    void refreshPanel();
  }
  function scrollToTags() {
    tagsAnchor?.scrollIntoView({ block: 'start', behavior: 'instant' });
    tagsAnchor?.querySelector<HTMLButtonElement>('h2 button')?.focus({ preventScroll: true });
  }
</script>

<main class="tasks-page notes-page" aria-label="Notes page" aria-keyshortcuts="Meta+R Control+R F5" bind:this={root}>
  <div class="tasks-page-content notes-page-content">
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <HomeNoteSection pageMode readState={noteState} onview={(view) => load(notes, 'note', view)} onnext={() => load(notes, 'note', undefined, true)} onretry={() => load(notes, 'note', undefined, false, true)} {onopen} oncreate={(seed) => oncreate('note', undefined, seed)} onnavigate={async () => { await load(notes, 'note'); }} />
    <div class="notes-page-tags-anchor" bind:this={tagsAnchor}>
      <HomeTagSection pageMode readState={tagState} onview={(view) => load(tags, 'tag', view)} onnext={() => load(tags, 'tag', undefined, true)} onretry={() => load(tags, 'tag', undefined, false, true)} {onopen} oncreate={(seed) => oncreate('tag', undefined, seed)} onnavigate={async () => { scrollToTags(); }} />
    </div>
  </div>
</main>
