import { operate } from './api';
import type { Entity } from './contracts.generated';

export type TaskState = 'todo' | 'doing' | 'done';
export const TASK_BOARD_STATES: TaskState[] = ['todo', 'doing', 'done'];
export const TASK_BOARD_LABELS: Record<TaskState, string> = { todo: 'To Do', doing: 'Doing', done: 'Done' };

export function taskStateOf(entity: Entity): TaskState | null {
  const value = (entity.payload as Record<string, unknown> | null)?.state;
  return TASK_BOARD_STATES.includes(value as TaskState) ? value as TaskState : null;
}

/** The UI reads the fresh revision before writing; Rust still owns CAS and payload merging. */
export async function updateTaskState(workspace: string, task: Entity, state: TaskState): Promise<boolean> {
  if (!TASK_BOARD_STATES.includes(state)) throw new Error('Unsupported task status.');
  const fresh = await operate<Entity>({ operation: 'entity.get', input: { id: task.id } }, workspace);
  if (fresh.kind !== 'task' || fresh.archived_at) throw new Error('This task is no longer available.');
  const current = taskStateOf(fresh);
  if (current === state) return false;
  await operate<Entity>({
    operation: 'entity.update',
    input: { id: fresh.id, expected_revision: fresh.revision, payload: { state } },
  }, workspace);
  return true;
}
