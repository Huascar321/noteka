<script lang="ts">
  import { untrack } from 'svelte';
  import AppIcon from './AppIcon.svelte';
  import { calendarDays, shiftMonth, validCivilDate } from './project-task-calendar';
  import type { HomeItem } from './contracts.generated';
  let { items, date, partial = false, disabled = false, onopen, oncreate }: {
    items: HomeItem[]; date: string; partial?: boolean; disabled?: boolean;
    onopen: (id: string) => Promise<void>; oncreate: (date: string) => void;
  } = $props();
  let month = $state(untrack(() => date.slice(0, 7)));
  const days = $derived(calendarDays(month));
  const label = $derived(new Intl.DateTimeFormat('en-US', { month:'long', year:'numeric', timeZone:'UTC' }).format(new Date(`${month}-15T12:00:00Z`)));
  const ordered = $derived([...items].sort((a,b) => a.entity.title.localeCompare(b.entity.title) || a.entity.id.localeCompare(b.entity.id)));
  const undated = $derived(ordered.filter(item => !item.metadata.project_end_date && !item.metadata.issues.includes('end_date')));
  const invalid = $derived(ordered.filter(item => item.metadata.issues.includes('end_date') || (item.metadata.project_end_date && !validCivilDate(item.metadata.project_end_date))));
  function move(delta: number) { const next = shiftMonth(month, delta); if (next) month = next; }
</script>
<div class="project-deadline-calendar">
  <div class="people-calendar-toolbar"><h3>{label}</h3><div class="glass-tool-group birthday-tools" role="group" aria-label="Project deadlines calendar navigation">
    <button aria-label="Previous deadline month" disabled={!shiftMonth(month, -1)} onclick={() => move(-1)}><AppIcon name="arrow-left" size={16} /></button>
    <button onclick={() => month = date.slice(0, 7)}>Today</button>
    <button aria-label="Next deadline month" disabled={!shiftMonth(month, 1)} onclick={() => move(1)}><AppIcon name="arrow-right" size={16} /></button>
  </div></div>
  {#if partial}<p class="project-context-message">Calendar shows loaded projects; load more to include remaining projects.</p>{/if}
  <!-- svelte-ignore a11y_no_noninteractive_tabindex (The scrollable calendar needs keyboard access.) -->
  <div class="people-calendar" role="region" aria-label={`Project deadlines in ${label}`} tabindex="0"><div class="people-calendar-grid">
    {#each ['Mon','Tue','Wed','Thu','Fri','Sat','Sun'] as weekday}<div class="people-weekday">{weekday}</div>{/each}
    {#each days as day, index (index)}<div class="people-calendar-day" class:people-calendar-outside={!day}>
      {#if day}<div class="project-calendar-day-heading"><time datetime={day}>{Number(day.slice(8))}</time><button class="project-calendar-create" aria-label={`New project due ${day}`} title={`New project due ${day}`} {disabled} onclick={() => oncreate(day)}><AppIcon name="plus" size={16} /></button></div>
        {#each ordered.filter(item => item.metadata.project_end_date === day) as item (item.entity.id)}<button class="people-birthday project-calendar-task" onclick={() => void onopen(item.entity.id)} title={item.entity.title}><span>{item.entity.title}</span>{#if item.metadata.state === 'done'}<small>Done</small>{/if}</button>{/each}
      {/if}
    </div>{/each}
  </div></div>
  <details class="project-calendar-undated"><summary>No date ({undated.length}){partial ? ' · loaded' : ''}</summary>{#each undated as item (item.entity.id)}<button class="project-calendar-unplaced" onclick={() => void onopen(item.entity.id)}>{item.entity.title}</button>{/each}{#if !undated.length}<p>No loaded projects without a deadline.</p>{/if}</details>
  {#if invalid.length}<details class="project-calendar-undated"><summary>Date unavailable ({invalid.length})</summary><p>These projects have an unrecognized deadline; their original data is preserved.</p>{#each invalid as item (item.entity.id)}<button class="project-calendar-unplaced" onclick={() => void onopen(item.entity.id)}>{item.entity.title}</button>{/each}</details>{/if}
</div>
