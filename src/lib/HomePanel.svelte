<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { operate } from './api';
  import HomeRow from './HomeItem.svelte';
  import { HomeReader, type HomeReadState } from './home-reads';
  import { ReadQueue } from './embedded-reads';
  import type { HomeGroup, HomeView, EntityKind, ContextLink, HomePage } from './contracts.generated';
  let { workspace, error = '', onopen, oncreate, onnavigate }: {
    workspace: string; error?: string; onopen: (id: string) => Promise<void>;
    oncreate: (kind: EntityKind, context?: ContextLink, seed?: Record<string, unknown>) => Promise<void>;
    onnavigate: (kind: EntityKind) => Promise<void>;
  } = $props();
  const sections: { id: EntityKind; title: string; icon: string; views: [HomeView, string][]; pending: string; hint: string }[] = [
    { id: 'task', title: 'Tasks', icon: '☑', views: [['tasks_today','Today'],['tasks_inbox','Inbox'],['tasks_all','All'],['tasks_done','Done']], pending: 'Week, Month, Scheduled, No Due, Recurring, Active Projects, Active Projects (In Progress), Active Content.', hint: 'Today: pendientes con vencimiento hasta el día civil, incluidas vencidas. No equivale a My Day (planificadas). Crear no asigna fechas.' },
    { id: 'note', title: 'Notes', icon: '▤', views: [['notes_all','Notes'],['notes_inbox','Inbox']], pending: 'Fav., Clips, Voice, Journal, Meetings, All diferenciada.', hint: 'Notes incluye todas las notas activas; Inbox exige su marca explícita.' },
    { id: 'project', title: 'Projects', icon: '◈', views: [['projects_active','Active'],['projects_planned','Planned'],['projects_board','Board']], pending: 'Paridad con fórmulas/campos de Notion no inspeccionados.', hint: 'Active: Doing/Ongoing; Active Projects lateral también incluye Planned. Progreso = completas/total de tareas activas relacionadas; sin tareas no es 0%.' },
    { id: 'tag', title: 'Tags', icon: '#', views: [['tags_recent','Recent'],['tags_alphabetical','A–Z'],['tags_types','Types']], pending: 'Fav., tipo Entity de la referencia; Topic conserva su clasificación.', hint: 'Recent ordena modificación del tag, no última nota. A–Z: orden Unicode sin collation local; resúmenes de todas sus notas activas.' },
    { id: 'person', title: 'People', icon: '♙', views: [['people_contacts','Contacts'],['people_new','New'],['people_updated','Updated']], pending: 'Fotos, organización, Check-Ins, Birthdays, Pipeline, Active Projects.', hint: 'Contacts ordena por título; New por creación y Updated por modificación. No son filtros de período ni un directorio remoto.' },
  ];
  let states = $state<Record<string, HomeReadState>>({});
  const readers = new Map<string, HomeReader>();
  const queue = new ReadQueue(2);
  onMount(() => {
    for (const section of sections) {
      const reader = new HomeReader(section.views[0][0], queue, query => operate<HomePage>({ operation: 'home.query', input: query }, workspace), state => { states = { ...states, [section.id]: state }; });
      readers.set(section.id, reader); void reader.load();
    }
    return () => { for (const reader of readers.values()) reader.dispose(); readers.clear(); };
  });
  export async function refreshPanel() { await Promise.all([...readers.values()].map(r => r.load())); }
  async function load(id: string, view?: HomeView, next = false, retry = false) {
    const reader = readers.get(id); if (!reader) return;
    const focused = document.activeElement instanceof HTMLButtonElement ? document.activeElement : null;
    const focusLabel = focused?.textContent;
    const page = reader.state.page;
    if (retry) await reader.retry(); else await reader.load(view ?? reader.state.view, next ? page?.next_cursor ?? undefined : undefined, next ? page?.date : undefined);
    await tick();
    if (focused && document.activeElement === document.body) {
      if (focused.isConnected && !focused.disabled) focused.focus();
      else {
        const section = document.querySelector(`[data-home-section="${id}"]`);
        const replacement = [...(section?.querySelectorAll<HTMLButtonElement>('button') ?? [])].find(b => b.textContent === focusLabel && !b.disabled)
          ?? section?.querySelector<HTMLButtonElement>('.home-tabs [aria-pressed="true"]');
        replacement?.focus();
      }
    }
  }
  function createIn(kind: EntityKind, group: HomeGroup) {
    return oncreate(kind, group.project_id ? { kind: 'project', target_id: group.project_id } : undefined,
      group.project_state ? { state: group.project_state } : group.classification ? { classification: group.classification } : undefined);
  }
</script>

<main class="home" aria-label="Home">
  <div class="home-cover" aria-hidden="true"></div>
  <div class="home-content">
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <div class="home-heading"><span aria-hidden="true" class="home-house">⌂</span><h1>Home</h1><button onclick={() => void refreshPanel()}>Actualizar Home</button></div>
    <details class="home-nav"><summary>Navegación</summary><div>{#each sections.filter(s => s.id !== 'person') as section}<button onclick={() => void onnavigate(section.id)}>{section.title}</button>{/each}<span>Content pendiente</span></div></details>
    <div class="home-weather" role="note"><span aria-hidden="true">☁</span><div><strong>Clima pendiente</strong><p>Clima no implementado; no se realizan consultas remotas. Localidad, proveedor y siete días requieren una entrega opt-in.</p></div></div>
    {#each sections as section}
      {@const state = states[section.id]}
      <section class="home-section" data-home-section={section.id} aria-label={section.title}>
        <header><h2><span aria-hidden="true">{section.icon}</span><button onclick={() => void onnavigate(section.id)}>{section.title}</button></h2><button onclick={() => void oncreate(section.id)} aria-label={`Crear en ${section.title}`}>+ New page</button></header>
        <div class="home-tabs">
          {#each section.views.filter(v => v[0] !== 'tasks_done') as [view, label]}<button aria-pressed={state?.view === view} class:active={state?.view === view} onclick={() => void load(section.id, view)}>{label}</button>{/each}
          {#if section.id === 'task'}<details><summary>Más vistas</summary><button aria-pressed={state?.view === 'tasks_done'} onclick={() => void load(section.id, 'tasks_done')}>Done</button></details>{/if}
          <button disabled={state?.busy} onclick={() => void load(section.id)} aria-label={`Actualizar ${section.title}`}>↻</button>
        </div>
        <details class="home-rules"><summary>Reglas y cobertura</summary><p>{section.hint}</p><p>Pendiente: {section.pending}</p></details>
        {#if state?.busy}<p role="status">Consultando {section.title}…</p>{/if}
        {#if state?.error}<p class="error" role="alert">Lectura fallida: {state.error}</p><button onclick={() => void load(section.id, undefined, false, true)}>Reintentar {section.title}</button>{/if}
        {#if state?.page}
          {@const page = state.page}
          <p class="home-coverage">{page.items.length} elementos únicos de {page.matching_count} · Página desde {page.items.length ? page.offset + 1 : 0} · Grupos cubren sólo esta página.<br />Consulta {page.date} · {page.timezone} · Snapshot independiente por sección.</p>
          {#if page.unavailable_count}<p class="home-unavailable">{page.unavailable_count} entidades sin metadatos interpretables para decidir este filtro.</p>{/if}
          {#if page.groups.length}
            <div class:home-board={state.view === 'projects_board'} class="home-groups">
              {#each page.groups as group (group.key)}
                <details open class="home-group" data-group-key={group.key}>
                  <summary>{group.label}{group.project_archived ? ' · archivado' : ''} <span>{group.entity_ids.length} apariciones de página</span></summary>
                  {#each group.entity_ids as id (id)}{@const row = page.items.find(i => i.entity.id === id)}{#if row}<HomeRow item={row} {onopen} />{/if}{/each}
                  {#if !group.entity_ids.length}<p class="home-empty">Sin registros en esta página.</p>{/if}
                  {#if group.key !== 'unavailable'}<button class="home-new" onclick={() => void createIn(section.id, group)}>+ New page en {group.label}</button>{/if}
                </details>
              {/each}
            </div>
          {:else}
            <div class:home-gallery={section.id === 'person'} class="home-rows">
              {#each page.items as row (row.entity.id)}<HomeRow item={row} {onopen} />{/each}
              {#if !page.items.length}<p class="home-empty">Sin {section.title} en esta vista.</p>{/if}
            </div>
          {/if}
          <div class="home-pagination"><button disabled={state.busy || !page.next_cursor} onclick={() => void load(section.id, undefined, true)}>Siguiente {section.title}</button>{#if page.offset}<button disabled={state.busy} onclick={() => void load(section.id)}>Inicio {section.title}</button>{/if}</div>
        {/if}
      </section>
      {#if section.id === 'tag'}<section class="home-section home-pending" aria-label="Goals"><header><h2>◎ Goals</h2></header><p>Goals pendiente: entidad, estados Active/Dream/Achieved, relaciones, fechas/progreso y vistas By Activity/By Deadline/Achieved requieren su propio cambio. No se presentan porcentajes ni objetivos ficticios.</p></section>{/if}
    {/each}
    <p class="home-footnote">Datos originales locales · Sin LLM · Content y capacidades pendientes no están habilitados. Este primer Home no es paridad total con Ultimate Brain.</p>
  </div>
</main>
