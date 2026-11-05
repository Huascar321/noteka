<script lang="ts">
  import { onDestroy } from 'svelte';
  import AppIcon from './AppIcon.svelte';
  import type { Entity } from './contracts.generated';
  import { TASK_BOARD_LABELS, TASK_BOARD_STATES, taskStateOf, type TaskState } from './task-board';

  let { tasks, movingId = '', error = '', disabled = false, onopen, onmove, oncreate }: {
    tasks: Entity[];
    movingId?: string;
    error?: string;
    disabled?: boolean;
    onopen: (task: Entity) => Promise<void>;
    onmove: (task: Entity, state: TaskState) => Promise<void>;
    oncreate: (state: TaskState) => void;
  } = $props();

  const columns = $derived([
    ...TASK_BOARD_STATES.map(state => ({
      state: state as TaskState | null,
      label: TASK_BOARD_LABELS[state],
      items: tasks.filter(task => taskStateOf(task) === state).sort(compareTasks),
    })),
    ...(tasks.some(task => taskStateOf(task) === null) ? [{
      state: null,
      label: 'Status unavailable',
      items: tasks.filter(task => taskStateOf(task) === null).sort(compareTasks),
    }] : []),
  ]);

  function compareTasks(a: Entity, b: Entity): number {
    const date = (task: Entity) => {
      const value = (task.payload as Record<string, unknown>)?.due_date;
      return typeof value === 'string' && value ? value : '9999-12-31';
    };
    return date(a).localeCompare(date(b)) || a.title.localeCompare(b.title) || a.id.localeCompare(b.id);
  }

  function dateLabel(value: string): string {
    const date = new Date(`${value}T12:00:00Z`);
    return Number.isFinite(date.getTime()) ? new Intl.DateTimeFormat('en-US', { month: 'long', day: 'numeric', year: 'numeric', timeZone: 'UTC' }).format(date) : value;
  }

  let board = $state<HTMLElement>();
  let pointer: { id: number; task: Entity; x: number; y: number; started: boolean } | null = null;
  let draggedId = $state<string | null>(null);
  let destination = $state<TaskState | null>(null);
  let ghost = $state<{ x: number; y: number; title: string } | null>(null);
  let suppressClick = false;

  function endDrag() { pointer = null; draggedId = null; destination = null; ghost = null; }
  function startPointer(task: Entity, event: PointerEvent) {
    if (disabled || movingId || pointer || event.button !== 0 || event.pointerType === 'touch') return;
    if ((event.target as HTMLElement | null)?.closest('select')) return;
    pointer = { id: event.pointerId, task, x: event.clientX, y: event.clientY, started: false };
  }
  function movePointer(event: PointerEvent) {
    if (!pointer || event.pointerId !== pointer.id) return;
    if (disabled || movingId || event.buttons !== 1) { endDrag(); return; }
    if (!pointer.started && Math.hypot(event.clientX - pointer.x, event.clientY - pointer.y) < 6) return;
    if (!pointer.started) { pointer.started = true; draggedId = pointer.task.id; }
    event.preventDefault();
    ghost = { x: event.clientX, y: event.clientY, title: pointer.task.title };
    const target = document.elementFromPoint(event.clientX, event.clientY);
    const column = target?.closest<HTMLElement>('.task-board-column[data-task-state]');
    const state = column && board?.contains(column) ? column.dataset.taskState : undefined;
    destination = TASK_BOARD_STATES.includes(state as TaskState) ? state as TaskState : null;
  }
  function finishPointer(event: PointerEvent) {
    if (!pointer) { if (suppressClick) window.setTimeout(() => { suppressClick = false; }, 0); return; }
    if (event.pointerId !== pointer.id) return;
    const task = pointer.task, started = pointer.started, target = destination;
    endDrag();
    if (!started) return;
    suppressClick = true;
    window.setTimeout(() => { suppressClick = false; }, 0);
    if (target && taskStateOf(task) !== target) void onmove(task, target);
  }
  function cancelClick(event: MouseEvent) {
    if (suppressClick) { event.preventDefault(); event.stopPropagation(); suppressClick = false; }
  }
  onDestroy(endDrag);
</script>

<svelte:window onpointermove={movePointer} onpointerup={finishPointer} onpointercancel={endDrag} onblur={endDrag} onclickcapture={cancelClick} />

<div class="task-board" bind:this={board} role="region" aria-label="Project task board" aria-busy={!!movingId}>
  {#if error}<p class="error" role="alert">{error}</p>{/if}
  {#each columns as column (column.state ?? 'unknown')}
    <section class="task-board-column" data-task-state={column.state ?? 'unknown'} class:task-board-column-destination={destination === column.state && column.state !== null} aria-label={column.label}>
      <header>
        <h3><span class="task-board-pill" data-state={column.state ?? 'unknown'}><span aria-hidden="true"></span>{column.label}</span><span class="task-board-count">{column.items.length}</span></h3>
      </header>
      <div class="task-board-cards">
        {#each column.items as task (task.id)}
          <article class="task-board-card" class:task-board-card-dragging={draggedId === task.id} data-entity-id={task.id} onpointerdown={(event) => startPointer(task, event)}>
            <button class="task-board-open" disabled={!!movingId || disabled} onclick={() => void onopen(task)} aria-label={`Open ${task.title}`}>
              <span>{task.title}</span>
            </button>
            <div class="task-board-card-meta">
              {#if typeof (task.payload as Record<string, unknown>)?.due_date === 'string' && (task.payload as Record<string, unknown>).due_date}
                <time datetime={String((task.payload as Record<string, unknown>).due_date)}>{dateLabel(String((task.payload as Record<string, unknown>).due_date))}</time>
              {/if}
              <label class="task-board-status">
                <span class="task-status-dot" data-state={taskStateOf(task) ?? 'unknown'} aria-hidden="true"></span>
                <select aria-label={`Status for ${task.title}`} value={taskStateOf(task) ?? ''} disabled={!!movingId || disabled} onchange={(event) => void onmove(task, event.currentTarget.value as TaskState)}>
                  {#if taskStateOf(task) === null}<option value="" disabled>Unavailable</option>{/if}
                  {#each TASK_BOARD_STATES as state}<option value={state}>{TASK_BOARD_LABELS[state]}</option>{/each}
                </select>
              </label>
            </div>
          </article>
        {/each}
        {#if column.state}<button class="task-board-new" disabled={disabled || !!movingId} onclick={() => oncreate(column.state!)}><AppIcon name="plus" size={16} />New task</button>{/if}
      </div>
    </section>
  {/each}
</div>
{#if ghost}<div class="task-board-ghost" style="left:{ghost.x + 12}px;top:{ghost.y + 12}px">{ghost.title}</div>{/if}
