<script lang="ts">
  import SectionIcon from './SectionIcon.svelte';
  import AppIcon from './AppIcon.svelte';
  import HomeGoalRow from './HomeGoalRow.svelte';
  import { homeGroupLabel, type HomeReadState } from './home-reads';
  import HomeReadStatus from './HomeReadStatus.svelte';
  import type { HomeView, HomeGroup, ContextLink, GoalFields } from './contracts.generated';
  import { slidingTabs } from './sliding-tabs';
  type GoalState = GoalFields['state'];
  let { readState, pageMode = false, onview, onnext, onretry, onopen, oncreate, onnavigate, onsection = async () => {} }: {
    readState?: HomeReadState; pageMode?: boolean; onview: (view?: HomeView) => Promise<void>; onnext: () => Promise<void>;
    onretry: () => Promise<void>; onopen: (id: string) => Promise<void>;
    oncreate: (context?: ContextLink, seed?: Record<string, unknown>) => Promise<void>; onnavigate: () => Promise<void>;
    onsection?: (kind: 'task' | 'note' | 'project' | 'tag') => Promise<void>;
  } = $props();
  const views: { id: HomeView; label: string; icon: string }[] = [
    { id: 'goals_activity', label: 'By Activity', icon: 'arrow-up' }, { id: 'goals_deadline', label: 'By Deadline', icon: 'calendar' },
    { id: 'goals_by_tag', label: 'By Tag', icon: 'tag' }, { id: 'goals_achieved', label: 'Achieved', icon: 'trophy' },
  ];
  const states = [{ key: 'active', label: 'Active' }, { key: 'dream', label: 'Dream' }, { key: 'achieved', label: 'Achieved' }] as const;
  const page = $derived(readState?.page);
  let optionsOpen = $state(false);
  let root: HTMLElement;
  function create(state: GoalState = 'dream', group?: HomeGroup) {
    const tag = group?.related_entity;
    return oncreate(tag?.kind === 'tag' ? { kind: 'tag', target_id: tag.id } : undefined, { inbox: false, icon: 'trophy', goal: { state } });
  }
</script>

<svelte:window onkeydown={(event) => { if (event.key === 'Escape' && root?.contains(document.activeElement)) optionsOpen = false; }} />

<section class="home-section home-tasks home-goals" class:tasks-page-section={pageMode} data-home-section="goals" aria-label="Goals" aria-busy={readState?.busy ?? true} bind:this={root}>
  {#if pageMode}
    <header class="tasks-page-heading"><span class="tasks-page-emblem" aria-hidden="true"><SectionIcon section="goals" size={68} /></span><h1>Goals</h1></header>
    <nav class="tasks-page-nav" aria-label="Goals quick navigation"><details><summary>Nav</summary>{#each views as view}<button onclick={() => void onview(view.id)}>{view.label}</button>{/each}</details>{#each [['task','Tasks'],['note','Notes'],['project','Projects'],['tag','Tags']] as [kind,label]}<span aria-hidden="true">|</span><button onclick={() => void onsection(kind as 'task' | 'note' | 'project' | 'tag')}>{label}</button>{/each}</nav>
  {:else}<header><h2><span class="task-heading-icon" aria-hidden="true"><SectionIcon section="goals" size={31} /></span><button onclick={() => void onnavigate()}>Goals</button></h2></header>{/if}
  <div class="task-view-toolbar goal-view-toolbar">
    <div class="home-tabs task-view-tabs" use:slidingTabs role="group" aria-label="Goals views">{#each views as view}<button class:active={readState?.view === view.id} aria-pressed={readState?.view === view.id} onclick={() => void onview(view.id)}><AppIcon name={view.icon} size={16} /><span>{view.label}</span></button>{/each}</div>
    <div class="task-view-actions"><HomeReadStatus state={readState} updating /><button class="task-icon-action" aria-label="Refresh Goals" title="Refresh Goals" disabled={readState?.busy} onclick={() => void onview()}><AppIcon name="refresh" size={16} /></button><button class="task-icon-action" aria-label="Goals view options" aria-expanded={optionsOpen} title="View options" onclick={() => optionsOpen = !optionsOpen}><AppIcon name="sliders" size={16} /></button><button class="task-new primary" onclick={() => void create()}>New</button></div>
  </div>
  {#if optionsOpen}<div class="task-view-options"><p>By Activity uses verified source activity and subsequent local edits. By Deadline sorts by Target Deadline, keeping Latest Activity visible. By Tag shows Active Goals by tag; Achieved uses status and date achieved. Groups and counts cover the loaded page.</p><p>Progress: completed milestones / total active milestones, to one decimal place. No milestones does not imply 0%; Goal status is edited separately. Imported activity is a snapshot without Notion synchronization.</p>{#if page}<p class="home-coverage">Loaded: {page.items.length}. In this view: {page.matching_count}.</p>{/if}</div>{/if}
  {#if readState?.busy && !page}<p role="status" class="task-read-status">Loading Goals…</p>{/if}
  {#if readState?.error}<p class="error" role="alert">Read failed: {readState.error}</p><button onclick={() => void onretry()}>Retry Goals</button>{/if}
  <HomeReadStatus state={readState} />
  {#if page}
    {#if page.unavailable_count}<p class="home-unavailable">{page.unavailable_count} goals with uninterpretable metadata.</p>{/if}
    {#if readState?.view === 'goals_by_tag'}
      <div class="goal-tag-groups">{#each page.groups as group (group.key)}<details open class="goal-tag-group"><summary>{#if group.related_entity}<button class="goal-group-tag" onclick={(event) => { event.preventDefault(); void onopen(group.related_entity!.id); }}><AppIcon name={group.related_entity.icon ?? 'tag'} color={group.related_entity.icon_color ?? undefined} size={18} />{homeGroupLabel(group, readState?.view)}</button>{:else}<span>{homeGroupLabel(group, readState?.view)}</span>{/if}<span class="goal-group-count">{group.entity_ids.length}</span></summary><div class="goal-board">{#each states as state}<section class="goal-board-column" aria-label={`${homeGroupLabel(group, readState?.view)}: ${state.label}`}><h3><span class="goal-state-pill" data-goal-state={state.key}><span></span>{state.label}</span></h3>{#each group.entity_ids as id (id)}{@const row = page.items.find(item => item.entity.id === id && item.metadata.goal?.state === state.key)}{#if row}<HomeGoalRow item={row} timezone={page.timezone} {onopen} card />{/if}{/each}<button class="home-new task-add-row" aria-label={`New page in ${homeGroupLabel(group, readState?.view)}: ${state.label}`} onclick={() => void create(state.key, group)}><AppIcon name="plus" size={16} />New page</button></section>{/each}</div></details>{/each}{#if !page.items.length}<p class="home-empty">No Active Goals in this view.</p><button class="home-new task-add-row" onclick={() => void create('active')}><AppIcon name="plus" size={16} />New page</button>{/if}</div>
    {:else}
      <div class="goal-groups">{#each page.groups as group (group.key)}<details open class="goal-group" class:goal-empty-group={!group.entity_ids.length} data-goal-state={group.key}><summary><span class="goal-state-pill" data-goal-state={group.key}><span></span>{homeGroupLabel(group, readState?.view)}</span><span class="goal-group-count">{group.entity_ids.length}</span></summary><div class="goal-rows">{#each group.entity_ids as id (id)}{@const row = page.items.find(item => item.entity.id === id)}{#if row}<HomeGoalRow item={row} timezone={page.timezone} {onopen} achieved={readState?.view === 'goals_achieved'} />{/if}{/each}</div><button class="home-new task-add-row" aria-label={`New page in ${homeGroupLabel(group, readState?.view)}`} onclick={() => void create(group.key as GoalState)}><AppIcon name="plus" size={16} />New page</button></details>{/each}</div>
    {/if}
    {#if page.next_cursor}<footer class="section-row-footer"><button class="section-row-action" disabled={readState?.busy || readState?.stale} onclick={() => void onnext()}><AppIcon name="arrow-down" size={15} />Load more</button></footer>{/if}
  {/if}
</section>
