<script lang="ts">
  import QuickNavMenu from './QuickNavMenu.svelte';
  import SectionIcon from './SectionIcon.svelte';
  import AppIcon from './AppIcon.svelte';
  import HomePersonCard from './HomePersonCard.svelte';
  import HomePersonProjectRow from './HomePersonProjectRow.svelte';
  import { homeGroupLabel, type HomeReadState } from './home-reads';
  import HomeReadStatus from './HomeReadStatus.svelte';
  import type { HomeView } from './contracts.generated';
  import { slidingTabs } from './sliding-tabs';
  let { readState, onview, onmonth, onnext, onretry, onopen, oncreate, onnavigate, pageMode = false }: {
    pageMode?: boolean;
    readState?: HomeReadState; onview: (view?: HomeView) => Promise<void>; onmonth: (date: string) => Promise<void>;
    onnext: () => Promise<void>; onretry: () => Promise<void>; onopen: (id: string) => Promise<void>;
    oncreate: (seed?: Record<string, unknown>) => Promise<void>; onnavigate: () => Promise<void>;
  } = $props();
  const views: { id: HomeView; label: string; icon: string }[] = [
    { id: 'people_contacts', label: 'Contacts', icon: 'contact' }, { id: 'people_new', label: 'New', icon: 'user-plus' },
    { id: 'people_updated', label: 'Updated', icon: 'arrow-up' }, { id: 'people_check_ins', label: 'Check-Ins', icon: 'phone-call' },
    { id: 'people_birthdays', label: 'Birthdays', icon: 'calendar' }, { id: 'people_pipeline', label: 'Pipeline', icon: 'forward' },
    { id: 'people_active_projects', label: 'Active Projects', icon: 'project' },
  ];
  const page = $derived(readState?.page);
  const month = $derived(page ? new Date(`${page.date}T00:00:00Z`) : null);
  const monthLabel = $derived(month ? new Intl.DateTimeFormat("en-US", { month: 'long', year: 'numeric', timeZone: 'UTC' }).format(month) : '');
  const calendarDays = $derived.by(() => {
    if (!month) return [];
    const first = new Date(month); first.setUTCDate(1);
    const start = (first.getUTCDay() + 6) % 7;
    const last = new Date(first); last.setUTCMonth(last.getUTCMonth() + 1); last.setUTCDate(0);
    return Array.from({ length: Math.ceil((start + last.getUTCDate()) / 7) * 7 }, (_, index) => {
      const day = index - start + 1;
      return day > 0 && day <= last.getUTCDate() ? `${page!.date.slice(0, 7)}-${String(day).padStart(2, '0')}` : null;
    });
  });
  function navigateMonth(delta: number) {
    if (!month) return;
    const date = new Date(month); date.setUTCDate(1); date.setUTCMonth(date.getUTCMonth() + delta);
    if (date.getUTCFullYear() >= 1 && date.getUTCFullYear() <= 9999) void onmonth(date.toISOString().slice(0, 10));
  }
</script>

<section class="home-section home-tasks home-people" class:tasks-page-section={pageMode} data-home-section="person" aria-label="People" aria-busy={readState?.busy ?? true}>
  {#if pageMode}
    <header class="tasks-page-heading"><span class="tasks-page-emblem" aria-hidden="true"><SectionIcon section="person" size={68} /></span><h1>People</h1></header>
    <nav class="tasks-page-nav" aria-label="People quick navigation"><QuickNavMenu>{#each views as view}<button onclick={() => void onview(view.id)}>{view.label}</button>{/each}</QuickNavMenu><span aria-hidden="true">|</span><button onclick={() => void onview('people_contacts')}>Contacts</button><span aria-hidden="true">|</span><button onclick={() => void onview('people_pipeline')}>Pipeline</button><span aria-hidden="true">|</span><button aria-label="Cal: birthday calendar" title="Birthday calendar" onclick={() => void onview('people_birthdays')}>Cal</button></nav>
  {:else}<header><h2><span class="task-heading-icon" aria-hidden="true"><SectionIcon section="person" size={31} /></span><button onclick={() => void onnavigate()}>People</button></h2></header>{/if}
  <div class="task-view-toolbar people-view-toolbar">
    <div class="home-tabs task-view-tabs" use:slidingTabs role="group" aria-label="People views">{#each views as view}<button class:active={readState?.view === view.id} aria-pressed={readState?.view === view.id} onclick={() => void onview(view.id)}><AppIcon name={view.icon} size={16} /><span>{view.label}</span></button>{/each}</div>
    <div class="task-view-actions"><HomeReadStatus state={readState} updating />
      <div class="glass-tool-group" role="group" aria-label="People tools"><button class="task-icon-action" aria-label="Refresh People" title="Refresh People" disabled={readState?.busy} onclick={() => void onview()}><AppIcon name="refresh" size={16} /></button>
      </div><button class="task-new primary" onclick={() => void oncreate()}><AppIcon name="plus" size={16} />New</button>
    </div>
  </div>
  {#if readState?.busy && !page}<p role="status" class="task-read-status">Loading People…</p>{/if}
  {#if readState?.error}<p class="error" role="alert">Read failed: {readState.error}</p><button onclick={() => void onretry()}>Retry People</button>{/if}
  <HomeReadStatus state={readState} />
  {#if page}
    {#if page.unavailable_count}<p class="home-unavailable">{page.unavailable_count} records with uninterpretable metadata for this view.</p>{/if}
    {#if readState?.view === 'people_contacts'}
      <div class="people-gallery">{#each page.items as item (item.entity.id)}<HomePersonCard {item} {onopen} />{/each}<button class="person-new-card" onclick={() => void oncreate()}><AppIcon name="plus" size={16} />New page</button></div>
      {#if !page.items.length}<p class="home-empty">No contacts. Create a person to get started.</p>{/if}
    {:else if readState?.view === 'people_pipeline'}
      <!-- svelte-ignore a11y_no_noninteractive_tabindex (Scrollable region needs keyboard focus.) -->
      <div class="people-pipeline" role="region" aria-label="Contact pipeline" tabindex="0">{#each page.groups as group (group.key)}<section class="people-pipeline-column" aria-label={homeGroupLabel(group, readState?.view)} data-pipeline={group.key}>
        <h3><span class="person-pipeline-pill" data-pipeline={group.key}>{homeGroupLabel(group, readState?.view)}</span><span class="project-column-count">{group.entity_ids.length}</span></h3>
        {#each group.entity_ids as id (id)}{@const item = page.items.find(row => row.entity.id === id)}{#if item}<HomePersonCard {item} {onopen} layout="pipeline" />{/if}{/each}
        <button class="home-new task-add-row" onclick={() => void oncreate(group.key === 'none' ? {} : { pipeline_status: group.key })}><AppIcon name="plus" size={16} />New page</button>
      </section>{/each}</div>
    {:else if readState?.view === 'people_birthdays'}
      <div class="people-calendar-toolbar"><h3>{monthLabel}</h3><div class="glass-tool-group birthday-tools" role="group" aria-label="Birthday calendar navigation"><button aria-label="Previous birthday month" disabled={readState?.busy || page.date.startsWith('0001-01')} onclick={() => navigateMonth(-1)}><AppIcon name="arrow-left" size={16} /></button><button onclick={() => void onview('people_birthdays')}>Today</button><button aria-label="Next birthday month" disabled={readState?.busy || page.date.startsWith('9999-12')} onclick={() => navigateMonth(1)}><AppIcon name="arrow-right" size={16} /></button></div></div>
      <!-- svelte-ignore a11y_no_noninteractive_tabindex (Scrollable calendar needs keyboard focus.) -->
      <div class="people-calendar" role="region" aria-label={`Birthdays in ${monthLabel}`} tabindex="0"><div class="people-calendar-grid">{#each ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] as day}<div class="people-weekday">{day}</div>{/each}{#each calendarDays as date, index (index)}<div class="people-calendar-day" class:people-calendar-outside={!date}>{#if date}<time datetime={date}>{Number(date.slice(8))}</time>{#each page.items.filter(item => item.metadata.birthday_occurrence === date) as item (item.entity.id)}<button class="people-birthday" onclick={() => void onopen(item.entity.id)}><AppIcon name={item.entity.icon ?? 'circle-user'} color={item.entity.icon_color ?? 'var(--section-icon)'} size={16} /><span>{item.entity.title}</span></button>{/each}{/if}</div>{/each}</div></div>
    {:else if readState?.view === 'people_active_projects'}
      <div class="people-project-groups">{#each page.groups as group (group.key)}<div class="group-open-shell"><details open class="task-group people-project-group"><summary><span class="task-group-title"><SectionIcon section="person" size={18} />{homeGroupLabel(group, readState?.view)}</span><span class="project-column-count">{group.entity_ids.length}</span></summary><div class="people-project-list">{#each group.entity_ids as id (id)}{@const item = page.items.find(row => row.entity.id === id)}{#if item}<HomePersonProjectRow {item} {onopen} />{/if}{/each}</div></details>{#if group.key}<button class="group-open" aria-label={`Open ${homeGroupLabel(group, readState?.view)}`} title={`Open ${homeGroupLabel(group, readState?.view)}`} onclick={() => void onopen(group.key.slice(7))}><AppIcon name="arrow-up-right" size={15} /></button>{/if}</div>{/each}{#if !page.items.length}<p class="home-empty">No active projects linked to contacts.</p>{/if}</div>
    {:else}
      <div class="people-list">{#each page.items as item (item.entity.id)}<HomePersonCard {item} {onopen} layout="list" checkIn={readState?.view === 'people_check_ins'} />{/each}{#if !page.items.length}<p class="home-empty">No contacts in this view.</p>{/if}</div>
    {/if}
    {#if page.next_cursor}<footer class="section-row-footer people-row-footer"><button class="section-row-action people-load-more" disabled={readState?.busy || readState?.stale} onclick={() => void onnext()}><AppIcon name="arrow-down" size={16} />Load more</button></footer>{/if}
  {/if}
</section>
