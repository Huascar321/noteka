<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { operate } from './api';
  import HomePersonSection from './HomePersonSection.svelte';
  import { HomeReader, type HomeReadState } from './home-reads';
  import { ReadQueue } from './embedded-reads';
  import type { HomePage, HomeView, EntityKind, ContextLink } from './contracts.generated';

  let { workspace, active = true, blocked = false, error = '', onopen, oncreate }: {
    workspace: string; active?: boolean; blocked?: boolean; error?: string;
    onopen: (id: string) => Promise<void>;
    oncreate: (kind: EntityKind, context?: ContextLink, seed?: Record<string, unknown>) => Promise<void>;
  } = $props();
  let readState = $state<HomeReadState>();
  let root: HTMLElement;
  const queue = new ReadQueue(2);
  let reader: HomeReader | undefined;
  onMount(() => {
    reader = new HomeReader('people_contacts', queue, (query, signal) => operate<HomePage>({ operation: 'home.query', input: query }, workspace, { signal }), state => readState = state, 10, true);
    void reader.load();
    window.addEventListener('keydown', refreshKeydown, true);
    return () => { reader?.dispose(); window.removeEventListener('keydown', refreshKeydown, true); };
  });
  async function load(view?: HomeView, next = false, retry = false, date?: string) {
    if (!reader) return;
    const focused = document.activeElement instanceof HTMLButtonElement ? document.activeElement : null;
    const label = focused?.textContent;
    const page = reader.state.page;
    if (retry) await reader.retry();
    else await reader.load(view, next ? page?.next_cursor ?? undefined : undefined, next ? page?.date : date);
    await tick();
    if (focused && document.activeElement === document.body) {
      const replacement = [...(root?.querySelectorAll<HTMLButtonElement>('button') ?? [])].find(button => button.textContent === label && !button.disabled)
        ?? root?.querySelector<HTMLButtonElement>('.home-tabs [aria-pressed="true"]');
      if (focused.isConnected && !focused.disabled) focused.focus(); else replacement?.focus();
    }
  }
  export async function refreshPanel() { await load(); }
  function refreshKeydown(event: KeyboardEvent) {
    if (!active || event.defaultPrevented) return;
    const commandR = (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'r';
    const f5 = event.key === 'F5' && !event.metaKey && !event.ctrlKey;
    if (event.altKey || event.shiftKey || (!commandR && !f5)) return;
    event.preventDefault();
    if (blocked || event.isComposing || event.repeat || document.querySelector('dialog[open]')) return;
    void refreshPanel();
  }
</script>

<main class="tasks-page people-page" aria-label="People page" aria-keyshortcuts="Meta+R Control+R F5" bind:this={root}>
  <div class="tasks-page-content people-page-content">
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <HomePersonSection pageMode {readState} onview={(view) => load(view)} onmonth={(date) => load('people_birthdays', false, false, date)} onnext={() => load(undefined, true)} onretry={() => load(undefined, false, true)} {onopen} oncreate={(seed) => oncreate('person', undefined, seed)} onnavigate={refreshPanel} />
  </div>
</main>
