<script lang="ts">
  import type { Entity } from './contracts.generated';
  import { taskStateOf, type TaskState } from './task-board';

  let { task, disabled = false, onopen, ontoggle }: {
    task: Entity;
    disabled?: boolean;
    onopen: (task: Entity) => Promise<void>;
    ontoggle: (task: Entity, state: TaskState) => Promise<void>;
  } = $props();
  const state = $derived(taskStateOf(task));
  const done = $derived(state === 'done');
  const due = $derived((task.payload as Record<string, unknown>)?.due_date);
  function dateLabel(value: string): string {
    const date = new Date(`${value}T12:00:00Z`);
    return Number.isFinite(date.getTime()) ? new Intl.DateTimeFormat('en-US', { month: 'short', day: 'numeric', year: 'numeric', timeZone: 'UTC' }).format(date) : value;
  }
</script>

<article class="project-task-row" data-entity-id={task.id}>
  <button type="button" class="task-status-toggle" role="checkbox" aria-checked={done} aria-label={`${done ? 'Mark pending' : 'Mark complete'}: ${task.title}`} title={done ? 'Completed · mark pending' : 'Pending · mark complete'} disabled={disabled} onclick={() => void ontoggle(task, done ? 'todo' : 'done')}>
    <span class="task-status-mark" class:complete={done} aria-hidden="true">{done ? '✓' : ''}</span>
  </button>
  <button class="project-task-open" aria-label={`Open ${task.title}`} onclick={() => void onopen(task)}>{task.title}</button>
  {#if typeof due === 'string' && due}<time datetime={due}>{dateLabel(due)}</time>{/if}
</article>
