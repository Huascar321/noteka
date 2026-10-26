import type { HomeGroup, HomePage, HomeQuery, HomeView } from './contracts.generated';
import { ReadLifetime, ReadQueue } from './embedded-reads.ts';

export type HomeReadState = { view: HomeView; page: HomePage | null; busy: boolean; error: string };

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
  private read: (query: HomeQuery) => Promise<HomePage>;
  private changed: (state: HomeReadState) => void;
  private query: HomeQuery;
  private appendBase: HomePage | null = null;
  private pageSize: number;
  private appendPages: boolean;
  private viewPageSizes: Partial<Record<HomeView, number>>;
  state: HomeReadState;
  constructor(view: HomeView, queue: ReadQueue, read: (query: HomeQuery) => Promise<HomePage>, changed: (state: HomeReadState) => void, pageSize = 50, appendPages = false, viewPageSizes: Partial<Record<HomeView, number>> = {}) {
    this.queue = queue; this.read = read; this.changed = changed;
    this.pageSize = pageSize; this.appendPages = appendPages;
    this.viewPageSizes = viewPageSizes;
    this.query = { view };
    this.state = { view, page: null, busy: false, error: '' };
  }
  async load(view = this.state.view, cursor?: string, date?: string) {
    const { token, signal } = this.lifetime.begin();
    if (!this.lifetime.current(token)) return;
    this.query = { view, cursor, as_of: date, limit: this.viewPageSizes[view] ?? this.pageSize };
    const query = this.query;
    const previous = this.appendPages && cursor && this.state.view === view ? this.state.page ?? this.appendBase : null;
    this.appendBase = previous;
    this.state = { view, page: previous, busy: true, error: '' }; this.changed(this.state);
    try {
      const page = await this.queue.run(() => this.read(query), signal);
      if (!this.lifetime.current(token)) return;
      if (page.view !== view || (date && page.date !== date)) throw new Error("Home response does not match the query.");
      const groups = previous ? [...previous.groups, ...page.groups.filter(group => !previous.groups.some(old => old.key === group.key))].map(group => ({ ...group, entity_ids: [...new Set([...group.entity_ids, ...(page.groups.find(next => next.key === group.key)?.entity_ids ?? [])])] })) : page.groups;
      const combined = previous ? { ...page, groups, offset: previous.offset, items: [...previous.items, ...page.items.filter(item => !previous.items.some(old => old.entity.id === item.entity.id))] } : page;
      this.appendBase = null;
      this.state = { view, page: combined, busy: false, error: '' }; this.changed(this.state);
    } catch (error) {
      if (!this.lifetime.current(token)) return;
      this.state = { view, page: previous, busy: false, error: error instanceof Error ? error.message : "Could not query Home." }; this.changed(this.state);
    }
  }
  retry() { return this.load(this.query.view, this.query.cursor ?? undefined, this.query.as_of ?? undefined); }
  dispose() { this.lifetime.dispose(); }
}
