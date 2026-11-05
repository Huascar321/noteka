<script lang="ts">
  import { untrack } from 'svelte';
  import AppIcon from './AppIcon.svelte';
  import { calendarDays, civilToday, shiftMonth, taskDue } from './project-task-calendar';
  import { taskStateOf } from './task-board';
  import type { Entity } from './contracts.generated';
  let { tasks, timezone, partial = false, disabled = false, onopen, oncreate }: {
    tasks: Entity[]; timezone: string; partial?: boolean; disabled?: boolean;
    onopen: (task: Entity) => Promise<void>; oncreate: (date: string) => void;
  } = $props();
  let month = $state(untrack(() => civilToday(timezone).slice(0, 7)));
  const days = $derived(calendarDays(month));
  const label = $derived(new Intl.DateTimeFormat('en-US', { month:'long', year:'numeric', timeZone:'UTC' }).format(new Date(`${month}-15T12:00:00Z`)));
  const ordered = $derived([...tasks].sort((a,b) => a.title.localeCompare(b.title) || a.id.localeCompare(b.id)));
  const dated = $derived(ordered.map(task => ({task, due:taskDue(task)})));
  const undated = $derived(dated.filter(item => item.due.kind === 'none').map(item => item.task));
  const invalid = $derived(dated.filter(item => item.due.kind === 'invalid').map(item => item.task));
  function move(delta: number) { const next = shiftMonth(month, delta); if (next) month = next; }
</script>

<div class="project-task-calendar">
  <div class="people-calendar-toolbar"><h3>{label}</h3><div class="glass-tool-group birthday-tools" role="group" aria-label="Project task calendar navigation">
    <button aria-label="Previous task month" disabled={!shiftMonth(month, -1)} onclick={() => move(-1)}><AppIcon name="arrow-left" size={16} /></button>
    <button onclick={() => month = civilToday(timezone).slice(0, 7)}>Today</button>
    <button aria-label="Next task month" disabled={!shiftMonth(month, 1)} onclick={() => move(1)}><AppIcon name="arrow-right" size={16} /></button>
  </div></div>
  {#if partial}<p class="project-context-message">Calendar shows loaded relations; load more to include remaining tasks.</p>{/if}
  <!-- svelte-ignore a11y_no_noninteractive_tabindex (The scrollable calendar needs keyboard access.) -->
  <div class="people-calendar" role="region" aria-label={`Project tasks in ${label}`} tabindex="0"><div class="people-calendar-grid">
    {#each ['Mon','Tue','Wed','Thu','Fri','Sat','Sun'] as weekday}<div class="people-weekday">{weekday}</div>{/each}
    {#each days as date, index (index)}<div class="people-calendar-day" class:people-calendar-outside={!date}>
      {#if date}<div class="project-calendar-day-heading"><time datetime={date}>{Number(date.slice(8))}</time><button class="project-calendar-create" aria-label={`New task due ${date}`} title={`New task due ${date}`} {disabled} onclick={() => oncreate(date)}><AppIcon name="plus" size={16} /></button></div>
        {#each dated.filter(item => item.due.kind === 'date' && item.due.value === date) as item (item.task.id)}<button class="people-birthday project-calendar-task" onclick={() => void onopen(item.task)} title={item.task.title}><span>{item.task.title}</span>{#if taskStateOf(item.task) === 'done'}<small>Done</small>{/if}</button>{/each}
      {/if}
    </div>{/each}
  </div></div>
  <details class="project-calendar-undated"><summary>No date ({undated.length}){partial ? ' · loaded' : ''}</summary>{#each undated as task (task.id)}<button class="project-calendar-unplaced" onclick={() => void onopen(task)}>{task.title}</button>{/each}{#if !undated.length}<p>No loaded tasks without a due date.</p>{/if}</details>
  {#if invalid.length}<details class="project-calendar-undated"><summary>Date unavailable ({invalid.length})</summary><p>These tasks have an unrecognized date; their original data is preserved.</p>{#each invalid as task (task.id)}<button class="project-calendar-unplaced" onclick={() => void onopen(task)}>{task.title}</button>{/each}</details>{/if}
</div>
