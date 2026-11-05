<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { operate } from './api';
  import AsciiShark from './AsciiShark.svelte';
  import SectionIcon from './SectionIcon.svelte';
  import ScrollOutline from './ScrollOutline.svelte';
  import HomeTaskSection from './HomeTaskSection.svelte';
  import HomeNoteSection from './HomeNoteSection.svelte';
  import HomeProjectSection from './HomeProjectSection.svelte';
  import HomeTagSection from './HomeTagSection.svelte';
  import HomeGoalSection from './HomeGoalSection.svelte';
  import HomePersonSection from './HomePersonSection.svelte';
  import HomeWeather from './HomeWeather.svelte';
  import { HomeReader, type HomeReadState } from './home-reads';
  import { ReadQueue } from './embedded-reads';
  import { captureRefreshFocus, restoreRefreshFocus } from './refresh-focus';
  import { readCover, saveCover, type CoverMode } from './home-cover';
  import { slidingTabs } from './sliding-tabs';
  import type { HomeGroup, HomeView, EntityKind, ContextLink, HomePage } from './contracts.generated';
  let { workspace, active = true, visible = active, animateAlongsideSidePeek = false, blocked = false, searchOpen = false, error = '', onopen, oncreate, onnavigate }: {
    workspace: string; active?: boolean; visible?: boolean; animateAlongsideSidePeek?: boolean; blocked?: boolean; searchOpen?: boolean; error?: string; onopen: (id: string) => Promise<void>;
    oncreate: (kind: EntityKind, context?: ContextLink, seed?: Record<string, unknown>) => Promise<void>;
    onnavigate: (kind: EntityKind | 'goals') => Promise<void>;
  } = $props();
  const sections: { id: EntityKind; title: string; icon: string; views: [HomeView, string][]; pending: string; hint: string }[] = [
    { id: 'task', title: 'Tasks', icon: 'task', views: [['tasks_projects_in_progress','Today'],['tasks_inbox','Inbox'],['tasks_week','Week'],['tasks_month','Month'],['tasks_scheduled','Scheduled'],['tasks_no_due','No Due'],['tasks_active_projects','All Active Projects'],['tasks_all','All'],['tasks_done','Done']], pending: "Recurring and Active Content.", hint: "The initial Today view groups pending tasks by Doing/Ongoing projects. All Active Projects also includes Planned. My Day uses planning dates. Creating a task does not assign dates." },
    { id: 'note', title: 'Notes', icon: 'file-text', views: [['notes_library','Notes'],['notes_inbox','Inbox']], pending: '', hint: "Notes separates the library and categories; All keeps every page accessible." },
    { id: 'project', title: 'Projects', icon: 'project', views: [['projects_active','Active'],['projects_planned','Planned'],['projects_board','Board']], pending: "Parity with unreviewed Notion formulas/fields.", hint: "Active: Doing/Ongoing; sidebar Active Projects also includes Planned. Progress = completed / total related active tasks; no tasks does not mean 0%." },
    { id: 'tag', title: 'Tags', icon: 'tag', views: [['tags_recent','Recent'],['tags_alphabetical','A–Z'],['tags_types','Types']], pending: "Fav. and Entity type from the reference; Topic preserves its classification.", hint: "Recent sorts by tag modification, not the latest note. A–Z: Unicode order without local collation; summaries include all active notes." },
    { id: 'person', title: 'People', icon: 'person', views: [['people_contacts','Contacts'],['people_new','New'],['people_updated','Updated']], pending: '', hint: "Local contacts and views with verified metadata and photos." },
  ];
  let states = $state<Record<string, HomeReadState>>({});
  let scrollRoot = $state<HTMLElement>();
  const readers = new Map<string, HomeReader>();
  const queue = new ReadQueue(2);
  let cover = $state<CoverMode>(readCover(() => localStorage));
  let coverError = $state('');
  function changeCover(mode: CoverMode) {
    cover = mode; coverError = saveCover(mode, () => localStorage);
    window.dispatchEvent(new Event('noteka-cover-change'));
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
  onMount(() => {
    const syncCover = () => { if (!coverError) cover = readCover(() => localStorage); };
    window.addEventListener('noteka-cover-change', syncCover);
    window.addEventListener('keydown', refreshKeydown, true);
    for (const section of sections) {
      const reader = new HomeReader(section.id === 'project' ? 'projects_board' : section.id === 'tag' ? 'tags_types' : section.views[0][0], queue, (query, signal) => operate<HomePage>({ operation: 'home.query', input: query }, workspace, { signal }), state => { states = { ...states, [section.id]: state }; }, section.id === 'note' || section.id === 'person' ? 10 : section.id === 'project' ? 100 : 50, true, section.id === 'person' ? { people_new: 100, people_updated: 100, people_check_ins: 100, people_birthdays: 100, people_pipeline: 100, people_active_projects: 100 } : {});
      readers.set(section.id, reader); void reader.load();
    }
    const goals = new HomeReader('goals_activity', queue, (query, signal) => operate<HomePage>({ operation: 'home.query', input: query }, workspace, { signal }), state => { states = { ...states, goals: state }; }, 10, true);
    readers.set('goals', goals); void goals.load();
    return () => { window.removeEventListener('noteka-cover-change', syncCover); window.removeEventListener('keydown', refreshKeydown, true); for (const reader of readers.values()) reader.dispose(); readers.clear(); };
  });
  export async function refreshPanel() {
    const focused = captureRefreshFocus(scrollRoot);
    await Promise.all([...readers.values()].map(r => r.load()));
    await tick();
    restoreRefreshFocus(focused);
  }
  async function load(id: string, view?: HomeView, next = false, retry = false, date?: string) {
    const reader = readers.get(id); if (!reader) return;
    const focused = document.activeElement instanceof HTMLButtonElement ? document.activeElement : null;
    const focusLabel = focused?.textContent;
    const page = reader.state.page;
    if (retry) await reader.retry(); else await reader.load(view, next ? page?.next_cursor ?? undefined : undefined, next ? page?.date : date);
    await tick();
    if (focused && document.activeElement === document.body) {
      if (focused.isConnected && !focused.disabled) focused.focus({ preventScroll: true });
      else {
        const section = document.querySelector(`[data-home-section="${id}"]`);
        const replacement = [...(section?.querySelectorAll<HTMLButtonElement>('button') ?? [])].find(b => b.textContent === focusLabel && !b.disabled)
          ?? section?.querySelector<HTMLButtonElement>('.home-tabs [aria-pressed="true"]');
        replacement?.focus({ preventScroll: true });
      }
    }
  }
  function createIn(kind: EntityKind, group: HomeGroup) {
    return oncreate(kind, group.project_id ? { kind: 'project', target_id: group.project_id } : undefined,
      group.project_state ? { state: group.project_state } : group.classification ? { classification: group.classification } : undefined);
  }
</script>

<main bind:this={scrollRoot} class="home has-scroll-outline" aria-label="Home" aria-keyshortcuts="Meta+R Control+R F5">
  <div class="home-cover" data-cover={cover}>
    {#if cover === 'ascii'}<AsciiShark active={visible && (!blocked || searchOpen) && (active || animateAlongsideSidePeek)} />{:else}<img class="home-cover-photo" src="/images/home-cover.png" alt="" />{/if}
    <div class="home-cover-controls" use:slidingTabs role="group" aria-label="Home cover"><button aria-pressed={cover === 'ascii'} onclick={() => changeCover('ascii')}>ASCII</button><button aria-pressed={cover === 'photo'} onclick={() => changeCover('photo')}>Photo</button></div>
  </div>
  <div class="home-content">
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    {#if coverError}<p role="status">{coverError}</p>{/if}
    <div class="home-heading"><span aria-hidden="true" class="home-house"><SectionIcon section="home" size={62} /></span><h1>Home</h1></div>
    <HomeWeather {visible} />
    {#each sections as section}
      {@const state = states[section.id]}
      {#if section.id === 'task'}
        <HomeTaskSection {workspace} onchanged={refreshPanel} readState={state} onview={(view) => load('task', view)} onnext={() => load('task', undefined, true)} onretry={() => load('task', undefined, false, true)} {onopen} oncreate={() => oncreate('task')} ongroupcreate={(group) => createIn('task', group)} onnavigate={() => onnavigate('task')} />
      {:else if section.id === 'note'}
        <HomeNoteSection readState={state} onview={(view) => load('note', view)} onnext={() => load('note', undefined, true)} onretry={() => load('note', undefined, false, true)} {onopen} oncreate={(seed) => oncreate('note', undefined, seed)} onnavigate={() => onnavigate('note')} />
      {:else if section.id === 'project'}
        <HomeProjectSection {workspace} {active} onmoved={refreshPanel} readState={state} onview={(view) => load('project', view)} onnext={() => load('project', undefined, true)} onretry={() => load('project', undefined, false, true)} {onopen} oncreate={(seed) => oncreate('project', undefined, seed)} onnavigate={() => onnavigate('project')} />
      {:else if section.id === 'tag'}
        <HomeTagSection readState={state} onview={(view) => load('tag', view)} onnext={() => load('tag', undefined, true)} onretry={() => load('tag', undefined, false, true)} {onopen} oncreate={(seed) => oncreate('tag', undefined, seed)} onnavigate={() => onnavigate('tag')} />
      {:else if section.id === 'person'}
        <HomePersonSection readState={state} onview={(view) => load('person', view)} onmonth={(date) => load('person', 'people_birthdays', false, false, date)} onnext={() => load('person', undefined, true)} onretry={() => load('person', undefined, false, true)} {onopen} oncreate={(seed) => oncreate('person', undefined, seed)} onnavigate={() => onnavigate('person')} />
      {/if}
      {#if section.id === 'tag'}<HomeGoalSection readState={states.goals} onview={(view) => load('goals', view)} onnext={() => load('goals', undefined, true)} onretry={() => load('goals', undefined, false, true)} {onopen} oncreate={(context,seed) => oncreate('note',context,seed)} onnavigate={() => onnavigate('goals')} />{/if}
    {/each}
  </div>
</main>
<ScrollOutline root={scrollRoot} mode="home" active={visible} disabled={blocked} />
