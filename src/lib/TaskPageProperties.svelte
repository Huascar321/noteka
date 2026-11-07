<script lang="ts">
  import { tick } from 'svelte';
  import AppIcon from './AppIcon.svelte';
  import DueDatePicker from './DueDatePicker.svelte';
  import type { Entity } from './contracts.generated';
  import { entityIcon, entityIconColor } from './entity-appearance';
  import { validCivilDate } from './project-task-calendar';
  const dueFormat = new Intl.DateTimeFormat('en-US', { month: 'short', day: 'numeric', year: 'numeric', timeZone: 'UTC' });
  let { payload, projects, disabled = false, canRelate = false, captureProject = false, timezone = 'UTC', onchange, onopen, oneditrelations }: {
    payload: Record<string, unknown>;
    projects: Entity[];
    disabled?: boolean;
    canRelate?: boolean;
    captureProject?: boolean;
    timezone?: string;
    onchange: () => void;
    onopen: (entity: Entity) => Promise<void>;
    oneditrelations: (anchor: HTMLElement) => void;
  } = $props();
  const taskState = $derived(String(payload.state ?? 'todo'));
  const dueValue = $derived(String(payload.due_date ?? ''));
  const dueLabel = $derived.by(() => {
    if (!dueValue) return 'Empty';
    if (!validCivilDate(dueValue)) return dueValue;
    return dueFormat.format(new Date(`${dueValue}T12:00:00Z`));
  });
  let dueButton = $state<HTMLButtonElement>();
  let dueOpen = $state(false);
  let dueFocus = $state(false);
  function set(key: string, value: unknown) { payload[key] = value; onchange(); }
  function closeDue(focus: boolean) {
    dueOpen = false;
    if (focus) void tick().then(() => dueButton?.focus());
  }
  function toggleDue(focus: boolean) {
    if (disabled) return;
    if (dueOpen) { closeDue(false); return; }
    dueFocus = focus;
    dueOpen = true;
  }
  function pickDue(value: string) { set('due_date', value || null); closeDue(true); }
  function dueKeys(event: KeyboardEvent) {
    if (event.isComposing) return;
    if (event.altKey && event.key === 'ArrowDown') { event.preventDefault(); toggleDue(true); return; }
    if (!dueOpen) return;
    if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); closeDue(true); }
  }
  $effect(() => { if (disabled) dueOpen = false; });
</script>

<fieldset class="task-page-properties" disabled={disabled}>
  <legend class="sr-only">Task properties</legend>
  <label class="task-status-property"><span class="task-property-label"><AppIcon name="circle" size={14} />Status</span>
    <span class="task-status-control" data-state={taskState}><span class="task-status-dot" aria-hidden="true"></span><select aria-label="Task status" value={taskState} onchange={(event) => set('state', event.currentTarget.value)}>{#if !['todo','doing','done'].includes(taskState)}<option value={taskState} disabled>Estado no reconocido</option>{/if}<option value="todo">To Do</option><option value="doing">Doing</option><option value="done">Done</option></select></span>
  </label>
  <div class="task-project-property" role="group" aria-label="Task project">
    <span class="task-property-label"><AppIcon name="arrow-up-right" size={14} />Project</span>
    <div class="task-project-value">
      <div class="compact-relation-list">{#each projects as project (project.id)}<button class="task-detail-project" title={project.title} onclick={() => void onopen(project)}><AppIcon name={entityIcon(project.kind, project.title, project.payload)} color={entityIconColor(project.payload)} size={15} /><span>{project.title}</span></button>{/each}
      {#if !projects.length}<span class="task-property-empty">{captureProject ? "Capture project · applied on save" : 'Empty'}</span>{/if}</div>
      {#if canRelate}<button class="task-project-edit icon-button" aria-label="Edit task project" title="Edit relations" aria-haspopup="dialog" onclick={event => oneditrelations(event.currentTarget)}><AppIcon name="chevron-down" size={13} /></button>{/if}
    </div>
  </div>
  <div class="task-due-property">
    <span class="task-property-label"><AppIcon name="calendar" size={14} />Due</span>
    <button type="button" bind:this={dueButton} class="task-due-trigger" class:empty={!dueValue} data-date={dueValue} aria-label="Due date" aria-haspopup="dialog" aria-expanded={dueOpen} onclick={() => toggleDue(false)} onkeydown={dueKeys}><span class="task-due-value">{dueLabel}</span><AppIcon name="calendar" size={14} /></button>
  </div>
</fieldset>

{#if dueOpen && dueButton}
  <DueDatePicker anchor={dueButton} value={dueValue} {timezone} focusOnOpen={dueFocus} onpick={pickDue} onclose={closeDue} />
{/if}
