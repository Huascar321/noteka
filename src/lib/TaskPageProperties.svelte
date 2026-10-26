<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import type { Entity } from './contracts.generated';
  import { entityIcon, entityIconColor } from './entity-appearance';
  let { payload, projects, disabled = false, canRelate = false, captureProject = false, onchange, onopen, oneditrelations }: {
    payload: Record<string, unknown>;
    projects: Entity[];
    disabled?: boolean;
    canRelate?: boolean;
    captureProject?: boolean;
    onchange: () => void;
    onopen: (entity: Entity) => Promise<void>;
    oneditrelations: () => void;
  } = $props();
  const state = $derived(String(payload.state ?? 'todo'));
  function set(key: string, value: unknown) { payload[key] = value; onchange(); }
</script>

<fieldset class="task-page-properties" disabled={disabled}>
  <legend class="sr-only">Task properties</legend>
  <label class="task-status-property"><span class="task-property-label"><AppIcon name="circle" size={14} />Status</span>
    <span class="task-status-control" data-state={state}><span class="task-status-dot" aria-hidden="true"></span><select aria-label="Task status" value={state} onchange={(event) => set('state', event.currentTarget.value)}>{#if !['todo','done'].includes(state)}<option value={state} disabled>Estado no reconocido</option>{/if}<option value="todo">To Do</option><option value="done">Done</option></select></span>
  </label>
  <div class="task-project-property" role="group" aria-label="Task project">
    <span class="task-property-label"><AppIcon name="arrow-up-right" size={14} />Project</span>
    <div class="task-project-value">
      {#each projects as project (project.id)}<button class="task-detail-project" title={project.title} onclick={() => void onopen(project)}><AppIcon name={entityIcon(project.kind, project.title, project.payload)} color={entityIconColor(project.payload)} size={15} /><span>{project.title}</span></button>{/each}
      {#if !projects.length}<span class="task-property-empty">{captureProject ? "Capture project · applied on save" : 'Empty'}</span>{/if}
      {#if canRelate}<button class="task-project-edit icon-button" aria-label="Edit task project" title="Edit relations" onclick={oneditrelations}><AppIcon name="chevron-down" size={13} /></button>{/if}
    </div>
  </div>
  <label class="task-due-property"><span class="task-property-label"><AppIcon name="calendar" size={14} />Due</span><input aria-label="Due date" type="date" value={String(payload.due_date ?? '')} onchange={(event) => set('due_date', event.currentTarget.value || null)} /></label>
</fieldset>
