<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { operate } from './api';
  import HomeGoalSection from './HomeGoalSection.svelte';
  import MilestonesPanel from './MilestonesPanel.svelte';
  import { HomeReader, type HomeReadState } from './home-reads';
  import { ReadQueue } from './embedded-reads';
  import type { HomePage, HomeView, EntityKind, ContextLink } from './contracts.generated';
  let { workspace, active = true, blocked = false, error = '', onopen, oncreate, onsection }: {
    workspace: string; active?: boolean; blocked?: boolean; error?: string;
    onopen: (id: string) => Promise<void>;
    oncreate: (kind: EntityKind, context?: ContextLink, seed?: Record<string, unknown>) => Promise<void>;
    onsection: (kind: 'task' | 'note' | 'project' | 'tag') => Promise<void>;
  } = $props();
  let readState = $state<HomeReadState>();
  let milestones = $state<{ refreshPanel: () => Promise<void> }>();
  let root: HTMLElement;
  const queue = new ReadQueue(2);
  let reader: HomeReader | undefined;
  onMount(() => {
    reader = new HomeReader('goals_activity', queue, (query, signal) => operate<HomePage>({ operation: 'home.query', input: query }, workspace, { signal }), state => readState = state, 10, true);
    void reader.load(); window.addEventListener('keydown', refreshKeydown, true);
    return () => { reader?.dispose(); window.removeEventListener('keydown', refreshKeydown, true); };
  });
  async function load(view?: HomeView, next = false, retry = false) {
    if (!reader) return;
    const focused = document.activeElement instanceof HTMLButtonElement ? document.activeElement : null;
    if (retry) await reader.retry(); else await reader.load(view, next ? reader.state.page?.next_cursor ?? undefined : undefined);
    await tick();
    if (focused && document.activeElement === document.body) { if (focused.isConnected && !focused.disabled) focused.focus(); else root?.querySelector<HTMLButtonElement>('[aria-pressed="true"]')?.focus(); }
  }
  export async function refreshPanel() { await Promise.all([load(), milestones?.refreshPanel()]); }
  function refreshKeydown(event: KeyboardEvent) {
    if (!active || event.defaultPrevented) return;
    const commandR = (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'r';
    const f5 = event.key === 'F5' && !event.metaKey && !event.ctrlKey;
    if (event.altKey || event.shiftKey || (!commandR && !f5)) return;
    event.preventDefault();
    if (blocked || event.isComposing || event.repeat || document.querySelector('dialog[open]')) return;
    void refreshPanel();
  }
  function createMilestone(goalId?: string) { return oncreate('note', undefined, { inbox: false, icon: 'target', milestone: { ...(goalId ? { goal_id: goalId } : {}) } }); }
</script>

<main class="tasks-page goals-page" aria-label="Goals page" aria-keyshortcuts="Meta+R Control+R F5" bind:this={root}>
  <div class="tasks-page-content goals-page-content">
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <HomeGoalSection pageMode {readState} onview={(view) => load(view)} onnext={() => load(undefined, true)} onretry={() => load(undefined, false, true)} {onopen} oncreate={(context,seed) => oncreate('note',context,seed)} onnavigate={refreshPanel} {onsection} />
    <MilestonesPanel bind:this={milestones} {workspace} {queue} {onopen} oncreate={createMilestone} />
  </div>
</main>
