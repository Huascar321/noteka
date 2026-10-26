import type { Contract, Entity, EntityKind, SearchHit } from './contracts.generated';
import { ReadLifetime, ReadQueue } from './embedded-reads.ts';
import { matchingSections, type NavigationTarget, type SectionInfo } from './navigation.ts';

export type SearchCriteria = { query: string; kind: EntityKind | ''; archived: boolean; related: string };
export type ContextResult =
  | { key: string; type: 'section'; section: SectionInfo }
  | { key: string; type: 'entity'; id: string; title: string; kind: EntityKind; hit?: SearchHit; entity?: Entity };
export type ContextSearchState = {
  results: ContextResult[]; selected: number; preview: Entity | null;
  busy: boolean; previewBusy: boolean; error: string; previewError: string; cursor: string | null;
};
type Read = (command: Parameters<typeof import('./api').operate>[0]) => Promise<unknown>;
// A closed dialog does not release an IPC slot until its actual response arrives.
const reads = new ReadQueue(2);

export class ContextSearch {
  state: ContextSearchState = { results: [], selected: 0, preview: null, busy: false, previewBusy: false, error: '', previewError: '', cursor: null };
  private lifetime = new ReadLifetime();
  private previewLifetime = new ReadLifetime();
  private criteria: SearchCriteria = { query: '', kind: '', archived: false, related: '' };
  private disposed = false;
  private current: { token: number; signal: AbortSignal } | null = null;
  private read: Read;
  private changed: (state: ContextSearchState) => void;
  constructor(read: Read, changed: (state: ContextSearchState) => void) { this.read = read; this.changed = changed; }
  private publish(patch: Partial<ContextSearchState>) {
    if (this.disposed) return;
    this.state = { ...this.state, ...patch }; this.changed(this.state);
  }
  invalidate() {
    this.lifetime.begin(); this.previewLifetime.begin(); this.current = null;
    this.publish({ results: [], selected: 0, preview: null, busy: true, previewBusy: false, error: '', previewError: '', cursor: null });
  }
  async search(criteria: SearchCriteria) {
    this.criteria = { ...criteria, query: criteria.query.trim(), related: criteria.related.trim() };
    const { token, signal } = this.lifetime.begin();
    this.current = { token, signal };
    const committed = this.criteria;
    this.previewLifetime.begin();
    const sections = !criteria.kind && !criteria.archived && !criteria.related.trim()
      ? matchingSections(criteria.query).map(section => ({ key: `section:${section.id}`, type: 'section' as const, section })) : [];
    this.publish({ results: sections, selected: 0, busy: true, error: '', cursor: null, preview: null, previewBusy: false, previewError: '' });
    if (!this.criteria.query && this.criteria.related) {
      this.publish({ busy: false, error: "Enter a term to search within a context." }); return;
    }
    try {
      const page = await reads.run(() => this.query(committed), signal);
      if (!this.lifetime.current(token) || !page) return;
      const results = this.combine(sections, page.items);
      this.publish({ results, cursor: page.cursor, busy: false });
      void this.select(0);
    } catch (error) {
      if (this.lifetime.current(token)) this.publish({ busy: false, error: message(error) });
    }
  }
  private async query(criteria: SearchCriteria, cursor?: string): Promise<{ items: ContextResult[]; cursor: string | null }> {
    if (criteria.query) {
      const page = await this.read({ operation: 'search.query', input: {
        query: criteria.query, kind: criteria.kind || undefined, archived: criteria.archived,
        related_id: criteria.related || undefined, limit: 30, cursor,
      } }) as Contract['search'];
      return { items: page.items.map(hit => ({ key: `entity:${hit.entity_id}`, type: 'entity', id: hit.entity_id, kind: hit.kind, title: hit.title, hit })), cursor: page.next_cursor ?? null };
    }
    const page = await this.read({ operation: 'entity.list', input: { kind: criteria.kind || undefined, archived: criteria.archived, limit: 20, cursor } }) as Contract['entities'];
    return { items: page.items.map(entity => ({ key: `entity:${entity.id}`, type: 'entity', id: entity.id, kind: entity.kind, title: entity.title, entity })), cursor: page.next_cursor ?? null };
  }
  private combine(previous: ContextResult[], next: ContextResult[]) {
    const seen = new Set(previous.map(result => result.key));
    return [...previous, ...next.filter(result => { if (seen.has(result.key)) return false; seen.add(result.key); return true; })];
  }
  async more() {
    if (this.state.busy || !this.state.cursor || this.disposed) return;
    const current = this.current;
    if (!current) return;
    const { token, signal } = current;
    const committed = this.criteria;
    const { cursor } = this.state;
    this.publish({ busy: true, error: '' });
    try {
      const page = await reads.run(() => this.query(committed, cursor ?? undefined), signal);
      if (this.lifetime.current(token) && page) this.publish({ results: this.combine(this.state.results, page.items), cursor: page.cursor, busy: false });
    } catch (error) { if (this.lifetime.current(token)) this.publish({ busy: false, error: message(error) }); }
  }
  async select(index: number) {
    const result = this.state.results[index];
    const { token, signal } = this.previewLifetime.begin();
    this.publish({ selected: index, preview: null, previewError: '', previewBusy: result?.type === 'entity' });
    if (!result || result.type === 'section') return;
    try {
      const entity = await reads.run(() => this.read({ operation: 'entity.get', input: { id: result.id } }), signal) as Entity;
      if (this.previewLifetime.current(token)) this.publish({ preview: entity, previewBusy: false });
    } catch (error) { if (this.previewLifetime.current(token)) this.publish({ previewError: message(error), previewBusy: false }); }
  }
  move(delta: number) {
    const count = this.state.results.length;
    if (count) void this.select((this.state.selected + delta + count) % count);
  }
  dispose() { this.disposed = true; this.lifetime.dispose(); this.previewLifetime.dispose(); }
}

export function resultTarget(result: ContextResult): NavigationTarget {
  return result.type === 'section' ? { type: 'section', section: result.section.id }
    : { type: 'entity', id: result.id, kind: result.kind, title: result.title, hit: result.hit };
}

export function documentPreview(value: unknown): string {
  if (!value || typeof value !== 'object' || !('schema_version' in value) || ![1, 2, 3].includes(Number(value.schema_version)) || !('blocks' in value) || !Array.isArray(value.blocks)) return '';
  const lines: string[] = [];
  function walk(blocks: unknown[], depth = 0) {
    if (depth > 32 || lines.length >= 16) return;
    for (const raw of blocks) {
      if (!raw || typeof raw !== 'object') continue;
      const block = raw as { content?: unknown[]; children?: unknown[] };
      const text = Array.isArray(block.content) ? block.content.map(part => part && typeof part === 'object' && 'text' in part && typeof part.text === 'string' ? part.text : '').join('') : '';
      if (text) lines.push(text.slice(0, 600));
      if (Array.isArray(block.children)) walk(block.children, depth + 1);
      if (lines.length >= 16) return;
    }
  }
  walk(value.blocks); return lines.join('\n\n');
}

function message(error: unknown): string { return error instanceof Error ? error.message : "Could not read the workspace."; }
