<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import { calendarDays, shiftDay, shiftMonth, weekDays } from './project-task-calendar';
  import type { HomeItem } from './contracts.generated';
  let { items, date, mode, partial = false, busy = false, onopen, oncreate, ondate }: {
    items: HomeItem[]; date: string; mode: 'week' | 'month'; partial?: boolean; busy?: boolean;
    onopen: (id: string) => Promise<void>; oncreate: (date: string) => void; ondate: (date?: string) => Promise<void>;
  } = $props();
  const days = $derived(mode === 'week' ? weekDays(date) : calendarDays(date.slice(0, 7)));
  const label = $derived.by(() => {
    const format = (d: string, options: Intl.DateTimeFormatOptions) => new Intl.DateTimeFormat('en-US', { ...options, timeZone:'UTC' }).format(new Date(`${d}T12:00:00Z`));
    if (mode === 'month') return format(date, { month:'long', year:'numeric' });
    const first = days.find(Boolean), last = [...days].reverse().find(Boolean);
    return first && last ? `${format(first, { month:'short', day:'numeric', year:'numeric' })} – ${format(last, { month:'short', day:'numeric', year:'numeric' })}` : '';
  });
  const prev = $derived(mode === 'week' ? shiftDay(date, -7) : (shiftMonth(date.slice(0, 7), -1) ? `${shiftMonth(date.slice(0, 7), -1)}-01` : null));
  const next = $derived(mode === 'week' ? shiftDay(date, 7) : (shiftMonth(date.slice(0, 7), 1) ? `${shiftMonth(date.slice(0, 7), 1)}-01` : null));
  const ordered = $derived([...items].sort((a,b) => a.entity.title.localeCompare(b.entity.title) || a.entity.id.localeCompare(b.entity.id)));
  const undated = $derived(ordered.filter(item => !item.metadata.due_date && !item.metadata.issues.some(issue => issue === 'due_date' || issue === 'payload')));
  const invalid = $derived(ordered.filter(item => item.metadata.issues.some(issue => issue === 'due_date' || issue === 'payload')));
</script>
<div class="task-calendar" aria-busy={busy}>
  <div class="people-calendar-toolbar"><h3>{label}</h3><div class="glass-tool-group birthday-tools" role="group" aria-label="Task calendar navigation">
    <button aria-label={`Previous task ${mode}`} disabled={busy || !prev} onclick={() => { if (prev) void ondate(prev); }}><AppIcon name="arrow-left" size={16} /></button>
    <button disabled={busy} onclick={() => void ondate()}>Today</button>
    <button aria-label={`Next task ${mode}`} disabled={busy || !next} onclick={() => { if (next) void ondate(next); }}><AppIcon name="arrow-right" size={16} /></button>
  </div></div>
  {#if partial}<p class="project-context-message">Calendar shows loaded tasks; load more to include remaining tasks.</p>{/if}
  <!-- svelte-ignore a11y_no_noninteractive_tabindex (The scrollable calendar needs keyboard access.) -->
  <div class="people-calendar" role="region" aria-label={`Tasks in ${label}`} tabindex="0"><div class="people-calendar-grid">
    {#each ['Mon','Tue','Wed','Thu','Fri','Sat','Sun'] as weekday}<div class="people-weekday">{weekday}</div>{/each}
    {#each days as day, index (index)}<div class="people-calendar-day" class:people-calendar-outside={!day}>
      {#if day}<div class="project-calendar-day-heading"><time datetime={day}>{Number(day.slice(8))}</time><button class="project-calendar-create" aria-label={`New task due ${day}`} title={`New task due ${day}`} disabled={busy} onclick={() => oncreate(day)}><AppIcon name="plus" size={16} /></button></div>
        {#each ordered.filter(item => item.metadata.due_date === day) as item (item.entity.id)}<button class="people-birthday project-calendar-task" onclick={() => void onopen(item.entity.id)} title={item.entity.title}><span>{item.entity.title}</span>{#if item.metadata.state === 'done'}<small>Done</small>{:else if !item.metadata.state}<small>Unknown</small>{/if}</button>{/each}
      {/if}
    </div>{/each}
  </div></div>
  <details class="project-calendar-undated"><summary>No date ({undated.length}){partial ? ' · loaded' : ''}</summary>{#each undated as item (item.entity.id)}<button class="project-calendar-unplaced" onclick={() => void onopen(item.entity.id)}>{item.entity.title}{item.metadata.state === 'done' ? ' · Done' : ''}</button>{/each}{#if !undated.length}<p>No loaded tasks without a due date.</p>{/if}</details>
  {#if invalid.length}<details class="project-calendar-undated"><summary>Date unavailable ({invalid.length})</summary><p>These tasks have an unrecognized date; their original data is preserved.</p>{#each invalid as item (item.entity.id)}<button class="project-calendar-unplaced" onclick={() => void onopen(item.entity.id)}>{item.entity.title}</button>{/each}</details>{/if}
</div>
