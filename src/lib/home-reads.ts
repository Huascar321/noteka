import type { HomeGroup, HomePage, HomeQuery, HomeView } from './contracts.generated';
import { ReadLifetime, ReadQueue } from './embedded-reads.ts';

export type HomeReadState = { view: HomeView; page: HomePage | null; busy: boolean; error: string; stale: boolean };
type RefreshRun = { key: string; started: boolean; pending: boolean; promise: Promise<void> };

// Translate system-generated empty groups by identity, never entity titles.
export function homeGroupLabel(group: HomeGroup, view?: HomeView): string {
  if (group.key === 'unavailable') return 'Unavailable';
  if (group.key === 'none' && !group.project_id && !group.related_entity) {
    if (view?.startsWith('tasks_')) return 'No project';
    if (view === 'projects_by_tag') return 'No tag';
    if (view === 'projects_by_people') return 'No person';
    if (view === 'people_pipeline') return 'No status';
  }
  if (view === 'goals_by_tag' && group.key === 'tag:' && !group.related_entity) return 'No tag';
  return group.label;
}
export class HomeReader {
  private lifetime = new ReadLifetime();
  private queue: ReadQueue;
  private read: (query: HomeQuery, signal: AbortSignal) => Promise<HomePage>;
  private changed: (state: HomeReadState) => void;
  private query: HomeQuery;
  private identity = '';
  private selectedDate: string | undefined;
  private refresh: RefreshRun | null = null;
  private disposed = false;
  private pageSize: number;
  private appendPages: boolean;
  private viewPageSizes: Partial<Record<HomeView, number>>;
  state: HomeReadState;
  constructor(view: HomeView, queue: ReadQueue, read: (query: HomeQuery, signal: AbortSignal) => Promise<HomePage>, changed: (state: HomeReadState) => void, pageSize = 50, appendPages = false, viewPageSizes: Partial<Record<HomeView, number>> = {}) {
    this.queue = queue; this.read = read; this.changed = changed;
    this.pageSize = pageSize; this.appendPages = appendPages;
    this.viewPageSizes = viewPageSizes;
    this.query = { view };
    this.state = { view, page: null, busy: false, error: '', stale: false };
  }
  load(view?: HomeView, cursor?: string, date?: string): Promise<void> {
    if (this.disposed) return Promise.resolve();
    const selected = view ?? this.state.view;
    // An explicit view selection can reset the day (for example Birthdays → Today).
    // An ordinary refresh retains a previously selected explicit date.
    const samePageDate = !!cursor && selected === this.state.view && (date === undefined || date === this.state.page?.date);
    const selectedDate = samePageDate ? this.selectedDate : date ?? (view === undefined ? this.selectedDate : undefined);
    // A cursor can pin the returned page's civil day without turning an
    // implicit "today" selection into a permanently fixed date.
    const asOf = cursor ? date ?? this.query.as_of ?? undefined : selectedDate;
    const query: HomeQuery = { view: selected, cursor, as_of: asOf, limit: this.viewPageSizes[selected] ?? this.pageSize };
    const key = JSON.stringify([selected, selectedDate ?? null, query.limit]);
    if (cursor) {
      if (this.state.busy || this.state.stale) return Promise.resolve();
      this.refresh = null;
      return this.readOnce(query, key, selectedDate);
    }
    if (this.refresh?.key === key) {
      if (this.refresh.started) this.refresh.pending = true;
      return this.refresh.promise;
    }
    const run: RefreshRun = { key, started: false, pending: false, promise: Promise.resolve() };
    this.refresh = run;
    run.promise = this.refreshLoop(run, query, selectedDate);
    return run.promise;
  }
  private async refreshLoop(run: RefreshRun, query: HomeQuery, selectedDate: string | undefined) {
    try {
      do {
        run.pending = false;
        run.started = false;
        await this.readOnce(query, run.key, selectedDate, run);
      } while (!this.disposed && this.refresh === run && run.pending);
    } finally {
      if (this.refresh === run) this.refresh = null;
    }
  }
  private async readOnce(query: HomeQuery, key: string, selectedDate: string | undefined, run?: RefreshRun) {
    const { token, signal } = this.lifetime.begin();
    if (!this.lifetime.current(token)) return;
    const { view, cursor, as_of: date } = query;
    const same = this.identity === key;
    const previous = same && (!cursor || this.appendPages) ? this.state.page : null;
    this.query = query; this.identity = key; this.selectedDate = selectedDate;
    this.state = { view, page: previous, busy: true, error: '', stale: !!previous }; this.changed(this.state);
    try {
      const page = await this.queue.run(() => {
        if (run) run.started = true;
        return this.read(query, signal);
      }, signal);
      if (!this.lifetime.current(token)) return;
      if (page.view !== view || (date && page.date !== date)) throw new Error("Home response does not match the query.");
      const append = !!cursor && !!previous;
      const groups = append ? [...previous.groups, ...page.groups.filter(group => !previous.groups.some(old => old.key === group.key))].map(group => ({ ...group, entity_ids: [...new Set([...group.entity_ids, ...(page.groups.find(next => next.key === group.key)?.entity_ids ?? [])])] })) : page.groups;
      const combined = append ? { ...page, groups, offset: previous.offset, items: [...previous.items, ...page.items.filter(item => !previous.items.some(old => old.entity.id === item.entity.id))] } : page;
      const again = !!run?.pending;
      this.state = { view, page: combined, busy: again, error: '', stale: again }; this.changed(this.state);
    } catch (error) {
      if (!this.lifetime.current(token)) return;
      const again = !!run?.pending;
      this.state = { view, page: previous, busy: again, stale: !!previous, error: error instanceof Error ? error.message : "Could not query Home." }; this.changed(this.state);
    }
  }
  retry() {
    if (this.disposed) return Promise.resolve();
    // A failed append may retry its own cursor, but a failed refresh restarts.
    return this.query.cursor ? this.readOnce(this.query, this.identity, this.selectedDate) : this.load();
  }
  dispose() { this.disposed = true; this.refresh = null; this.lifetime.dispose(); }
}
