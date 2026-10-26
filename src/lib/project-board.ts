import { operate } from './api';
import type { BatchApplied, BatchPlan, BatchPreview, Entity, HomePage, ProjectState } from './contracts.generated';

export const PROJECT_BOARD_LIMIT = 100;
export const PROJECT_STATES: ProjectState[] = ['planned', 'doing', 'ongoing', 'done', 'on_hold'];
export type ProjectPlacement = { id: string; state: ProjectState; beforeId: string | null };
const changed = () => new Error("The board changed. Refresh and move the card again.");

export function boardIsComplete(page?: HomePage | null): boolean {
  return !!page && page.view === 'projects_board' && !page.next_cursor && page.offset === 0
    && page.matching_count === page.items.length && page.items.length <= PROJECT_BOARD_LIMIT
    && page.unavailable_count === 0 && page.items.every(row => row.entity.kind === 'project'
      && !row.entity.archived_at && row.metadata.issues.length === 0
      && PROJECT_STATES.includes(row.metadata.state as ProjectState));
}

function stamp(page: HomePage): string {
  return JSON.stringify(page.items.map(row => [row.entity.id, row.entity.revision, row.metadata.state]));
}

export function projectPlacement(page: HomePage, placement: ProjectPlacement) {
  if (!boardIsComplete(page) || !PROJECT_STATES.includes(placement.state)) throw changed();
  const source = page.items.find(row => row.entity.id === placement.id);
  if (!source) throw changed();
  if (placement.beforeId === source.entity.id) return [];
  const target = page.items.filter(row => row.metadata.state === placement.state && row.entity.id !== source.entity.id);
  const index = placement.beforeId === null ? target.length : target.findIndex(row => row.entity.id === placement.beforeId);
  if (index < 0) throw changed();
  target.splice(index, 0, source);
  const previous = page.items.filter(row => row.metadata.state === placement.state).map(row => row.entity.id);
  if (source.metadata.state === placement.state && target.every((row, i) => row.entity.id === previous[i])) return [];
  return target.map((row, boardOrder) => ({ row, boardOrder, state: placement.state }));
}

/** Layout selection stays in the UI; validation and atomic CAS belong to Rust. */
export async function moveProject(workspace: string, page: HomePage, placement: ProjectPlacement, alive: () => boolean): Promise<boolean> {
  const desired = projectPlacement(page, placement);
  if (!desired.length) return false;
  const plan: BatchPlan = { version: 1, batch_id: crypto.randomUUID(), steps: [], checks: [] };
  for (const { row, boardOrder, state } of desired) {
    if (!alive()) return false;
    const entity = await operate<Entity>({ operation: 'entity.get', input: { id: row.entity.id } }, workspace);
    const payload = entity.payload;
    if (entity.kind !== 'project' || entity.archived_at || entity.revision !== row.entity.revision
      || !payload || typeof payload !== 'object' || Array.isArray(payload)) throw changed();
    const original = payload as Record<string, unknown>;
    if ((original.state ?? 'planned') !== row.metadata.state) throw changed();
    const provenance = original.source;
    if (provenance && typeof provenance === 'object' && (provenance as Record<string, unknown>).archived === true) throw changed();
    if ((original.state ?? 'planned') === state && original.board_order === boardOrder) continue;
    plan.steps.push({ key: `move_${plan.steps.length}`, operation: 'entity.update', bindings: [], input: {
      id: entity.id, expected_revision: row.entity.revision,
      payload: { ...original, state, board_order: boardOrder },
    } });
  }
  if (!alive() || !plan.steps.length) return false;
  const preview = await operate<BatchPreview>({ operation: 'changes.preview', input: plan }, workspace);
  if (!alive()) return false;
  const current = await operate<HomePage>({ operation: 'home.query', input: { view: 'projects_board', limit: PROJECT_BOARD_LIMIT, as_of: page.date } }, workspace);
  if (!boardIsComplete(current) || stamp(current) !== stamp(page)) throw changed();
  if (!alive()) return false;
  const applied = await operate<BatchApplied>({ operation: 'changes.apply', input: { plan, expected: preview.receipt } }, workspace);
  if (!applied.applied || applied.batch_id !== plan.batch_id) throw new Error("The move was not confirmed. Refresh the board before moving again.");
  return true;
}
