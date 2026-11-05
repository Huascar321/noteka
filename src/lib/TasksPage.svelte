<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { operate } from './api';
  import HomeTaskSection from './HomeTaskSection.svelte';
  import HomeProjectSection from './HomeProjectSection.svelte';
  import { HomeReader, type HomeReadState } from './home-reads';
  import { ReadQueue } from './embedded-reads';
  import { captureRefreshFocus, restoreRefreshFocus } from './refresh-focus';
  import type { HomePage, HomeView, HomeGroup, EntityKind, ContextLink } from './contracts.generated';
  let { workspace, active = true, blocked = false, error = '', onopen, oncreate, onhome, onprojects }: {
    workspace: string; active?: boolean; blocked?: boolean; error?: string;
    onopen: (id: string) => Promise<void>;
    oncreate: (kind: EntityKind, context?: ContextLink, seed?: Record<string, unknown>) => Promise<void>;
    onhome: () => Promise<void>; onprojects: () => Promise<void>;
  } = $props();
  let taskState = $state<HomeReadState>();
  let projectState = $state<HomeReadState>();
  let root: HTMLElement;
  const queue = new ReadQueue(2);
  let tasks: HomeReader | undefined;
  let projects: HomeReader | undefined;
  onMount(() => {
    tasks = new HomeReader('tasks_projects_in_progress', queue, (query, signal) => operate<HomePage>({ operation: 'home.query', input: query }, workspace, { signal }), state => taskState = state, 10, true);
    projects = new HomeReader('projects_active', queue, (query, signal) => operate<HomePage>({ operation: 'home.query', input: query }, workspace, { signal }), state => projectState = state, 100, true);
    void tasks.load(); void projects.load();
    window.addEventListener('keydown', refreshKeydown, true);
    return () => { tasks?.dispose(); projects?.dispose(); window.removeEventListener('keydown', refreshKeydown, true); };
  });
  function load(reader: HomeReader | undefined, view?: HomeView, next = false) { return reader?.load(view, next ? reader.state.page?.next_cursor ?? undefined : undefined) ?? Promise.resolve(); }
  export async function refreshPanel() {
    const focused = captureRefreshFocus(root);
    await Promise.all([load(tasks), load(projects)]);
    await tick(); restoreRefreshFocus(focused);
  }
  function refreshKeydown(event: KeyboardEvent) {
    if (!active || event.defaultPrevented) return;
    const commandR = (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'r';
    const f5 = event.key === 'F5' && !event.metaKey && !event.ctrlKey;
    if (event.altKey || event.shiftKey || (!commandR && !f5)) return;
    event.preventDefault();
    if (blocked || event.isComposing || event.repeat || document.querySelector('dialog[open]')) return;
    void refreshPanel();
  }
  function createIn(group: HomeGroup) { return oncreate('task', group.project_id ? { kind: 'project', target_id: group.project_id } : undefined); }
</script>

<main class="tasks-page" aria-label="Tasks page" aria-keyshortcuts="Meta+R Control+R F5" bind:this={root}>
  <div class="tasks-page-content">
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <HomeTaskSection pageMode {workspace} onchanged={refreshPanel} readState={taskState} onview={(view) => load(tasks, view)} onnext={() => load(tasks, undefined, true)} onretry={() => tasks?.retry() ?? Promise.resolve()} {onopen} oncreate={() => oncreate('task')} ongroupcreate={createIn} onnavigate={onhome} />
    <HomeProjectSection pageMode {workspace} {active} readState={projectState} onview={(view) => load(projects, view)} onnext={() => load(projects, undefined, true)} onretry={() => projects?.retry() ?? Promise.resolve()} {onopen} oncreate={(seed) => oncreate('project', undefined, seed)} onnavigate={onprojects} onmoved={refreshPanel} />
  </div>
</main>
