<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
  import AppIcon from './AppIcon.svelte';
  import { operate } from './api';
  import { HomeReader, type HomeReadState } from './home-reads';
  import { ReadQueue } from './embedded-reads';
  import { goalDate } from './goal-dates';
  import type { HomePage } from './contracts.generated';
  let { workspace, goalId, revision = 0, queue, onopen, oncreate }: {
    workspace: string; goalId?: string; revision?: number; queue?: ReadQueue;
    onopen: (id: string) => Promise<void>; oncreate: (goalId?: string) => Promise<void>;
  } = $props();
  let readState = $state<HomeReadState>();
  let reader: HomeReader | undefined;
  let seenRevision = untrack(() => revision);
  let optionsOpen = $state(false);
  let root: HTMLElement;
  const page = $derived(readState?.page);
  onMount(() => {
    reader = new HomeReader(goalId ? 'milestones_all' : 'milestones_completed', queue ?? new ReadQueue(2), query => operate<HomePage>({ operation: 'home.query', input: { ...query, goal_id: goalId } }, workspace), state => readState = state, 10, true);
    void reader.load();
    return () => reader?.dispose();
  });
  $effect(() => { const value = revision; if (value !== seenRevision) { seenRevision = value; void reader?.load(); } });
  async function load(next = false, retry = false) {
    if (!reader) return;
    const previous = document.activeElement instanceof HTMLButtonElement ? document.activeElement : null;
    if (retry) await reader.retry(); else await reader.load(undefined, next ? reader.state.page?.next_cursor ?? undefined : undefined);
    await tick();
    if (previous && document.activeElement === document.body) { if (previous.isConnected && !previous.disabled) previous.focus(); else root?.querySelector<HTMLButtonElement>('[aria-pressed="true"]')?.focus(); }
  }
  export async function refreshPanel() { await load(); }
</script>

<section class="milestones-section" aria-label={goalId ? "Goal milestones" : 'Milestones'} aria-busy={readState?.busy ?? true} bind:this={root}>
  <header><h2>Milestones</h2></header>
  <div class="task-view-toolbar"><div class="home-tabs task-view-tabs" role="group" aria-label="Milestone views"><button class="active" aria-pressed="true" onclick={() => void load()}><AppIcon name={goalId ? 'collections' : 'list'} size={16} />{goalId ? 'Table' : 'List'}</button></div><div class="task-view-actions"><button class="task-icon-action" aria-label="Refresh Milestones" title="Refresh Milestones" disabled={readState?.busy} onclick={() => void load()}><AppIcon name="refresh" size={16} /></button><button class="task-icon-action" aria-label="Milestone view options" aria-expanded={optionsOpen} onclick={() => optionsOpen = !optionsOpen}><AppIcon name="sliders" size={16} /></button><button class="task-new primary" onclick={() => void oncreate(goalId)}>New</button></div></div>
  {#if optionsOpen}<div class="task-view-options"><p>{goalId ? "All active milestones for this Goal, with their deadline or completion date." : "List shows milestones with Date Completed, sorted by that date. Pending milestones are available inside their Goal or through All in Notes."}</p><p>Complete a milestone by saving its completion date. Goal progress is calculated from its milestones and does not change Goal status automatically.</p>{#if page}<p class="home-coverage">Loaded: {page.items.length}. In this view: {page.matching_count}.</p>{/if}</div>{/if}
  {#if readState?.busy}<p role="status" class="task-read-status">Loading Milestones…</p>{/if}
  {#if readState?.error}<p class="error" role="alert">Read failed: {readState.error}</p><button onclick={() => void load(false, true)}>Retry Milestones</button>{/if}
  {#if page}
    {#if page.unavailable_count}<p class="home-unavailable">{page.unavailable_count} milestones with uninterpretable metadata.</p>{/if}
    {#if goalId}<div class="goal-milestone-table"><table aria-label="Goal milestones"><thead><tr><th scope="col">Name</th><th scope="col">Target Deadline</th><th scope="col">Date Completed</th></tr></thead><tbody>{#each page.items as item (item.entity.id)}<tr data-entity-id={item.entity.id}><th scope="row"><button class="goal-milestone-open" aria-label={`Open ${item.entity.title}`} onclick={() => void onopen(item.entity.id)}><AppIcon name={item.entity.icon ?? 'target'} color={item.entity.icon_color ?? undefined} size={18} /><span>{item.entity.title}</span></button>{#if item.metadata.issues.length}<p class="home-unavailable">Metadata unavailable: {item.metadata.issues.join(', ')}</p>{/if}</th><td>{#if item.metadata.milestone?.target_deadline}<time datetime={item.metadata.milestone.target_deadline}>{goalDate(item.metadata.milestone.target_deadline,page.timezone,true)}</time>{:else}<span class="goal-property-empty">Empty</span>{/if}</td><td>{#if item.metadata.milestone?.completed_on}<span class="goal-milestone-completed"><AppIcon name="circle-check" size={16} /><time datetime={item.metadata.milestone.completed_on}>{goalDate(item.metadata.milestone.completed_on,page.timezone,true)}</time></span>{:else}<span class="goal-property-empty">Empty</span>{/if}</td></tr>{/each}</tbody></table>{#if !page.items.length && !page.unavailable_count}<p class="home-empty">No Milestones in this Goal.</p>{/if}</div>{:else}
    <div class="milestone-rows">{#each page.items as item (item.entity.id)}
      {@const milestone = item.metadata.milestone}
      {@const goal = item.context.find(ref => ref.kind === 'note')}
      {@const date = milestone?.completed_on ?? milestone?.target_deadline}
      <article class="goal-row milestone-row" data-entity-id={item.entity.id}><button class="goal-row-open" title={item.entity.title} aria-label={`Open ${item.entity.title}`} onclick={() => void onopen(item.entity.id)}><AppIcon name={item.entity.icon ?? 'target'} color={item.entity.icon_color ?? undefined} size={18} /><span>{item.entity.title}</span></button><div class="goal-row-meta">{#if goal && !goalId}<button class="goal-row-tag" title={goal.title} onclick={() => void onopen(goal!.id)}><AppIcon name={goal.icon ?? 'trophy'} color={goal.icon_color ?? undefined} size={18} /><span>{goal.title}</span></button>{/if}{#if milestone?.completed_on}<span class="milestone-done" aria-label="Completed"><AppIcon name="circle-check" size={16} /></span>{/if}{#if date}<time datetime={date} title={milestone?.completed_on ? 'Date Completed' : 'Target Deadline'}>{goalDate(date, page.timezone, true)}</time>{/if}</div>{#if item.metadata.issues.length}<p class="home-unavailable">Metadata unavailable: {item.metadata.issues.join(', ')}</p>{/if}</article>
    {/each}{#if !page.items.length && !page.unavailable_count}<p class="home-empty">{goalId ? "No Milestones in this Goal." : "No completed Milestones."}</p>{/if}</div>{/if}
    <button class="home-new task-add-row" onclick={() => void oncreate(goalId)}><AppIcon name="plus" size={16} />New page</button>
    {#if page.next_cursor}<footer class="section-row-footer"><button class="section-row-action" disabled={readState?.busy} onclick={() => void load(true)}><AppIcon name="arrow-down" size={15} />Load more</button></footer>{/if}
  {/if}
</section>
