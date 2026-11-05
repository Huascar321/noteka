<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import { entityIcon, entityIconColor } from './entity-appearance';
  import { goalDate } from './goal-dates';
  import type { Entity } from './contracts.generated';
  let { projects, disabled = false, onopen, onadd }: {
    projects: Entity[]; timezone: string; disabled?: boolean;
    onopen: (entity: Entity) => Promise<void>; onadd: (anchor: HTMLElement) => void;
  } = $props();
  const states: Record<string, string> = { planned: 'Planned', doing: 'Doing', ongoing: 'Ongoing', done: 'Done', on_hold: 'On hold' };
  function state(project: Entity) {
    const payload = project.payload as Record<string, unknown>;
    return typeof payload?.state === 'string' && states[payload.state] ? states[payload.state] : payload && !('state' in payload) ? 'Planned' : 'Unavailable';
  }
  function deadline(project: Entity) {
    const date = (project.payload as Record<string, unknown>)?.end_date;
    if (date == null || date === '') return 'Empty';
    return typeof date === 'string' ? goalDate(date, 'UTC', true) || 'Unavailable' : 'Unavailable';
  }
</script>

<section class="goal-projects" aria-label="Goal Projects">
  <header><h2>Goal Projects</h2><button class="goal-project-add glass-secondary" type="button" disabled={disabled} aria-haspopup="dialog" onclick={event => onadd(event.currentTarget)}><AppIcon name="plus" size={15} />Add project</button></header>
  <div class="goal-projects-table-wrap">
    <table aria-label="Related projects">
      <thead><tr><th scope="col">Name</th><th scope="col">Status</th><th scope="col">Target Deadline</th></tr></thead>
      <tbody>{#each projects as project (project.id)}
        <tr><th scope="row"><button class="goal-project-row" onclick={() => void onopen(project)}><AppIcon name={entityIcon(project.kind,project.title,project.payload)} color={entityIconColor(project.payload)} size={17} /><span>{project.title || 'Untitled'}{project.archived_at ? ' · archived' : ''}</span></button></th>
          <td><span class="project-row-state" data-project-state={(project.payload as Record<string, unknown>)?.state}>{state(project)}</span></td>
          <td>{deadline(project)}</td></tr>
      {/each}</tbody>
    </table>
  </div>
  {#if !projects.length}<p class="goal-property-empty">No related projects.</p>{/if}
</section>
