import type { CoreError, Request, SearchPage, SearchQuery } from './contracts.generated';
import { AttachmentReader, type AttachmentReadState } from './attachment-search.ts';
import { ReadLifetime, ReadQueue } from './embedded-reads.ts';
export type NotesReadState = { page: SearchPage | null; pageNumber: number; busy: boolean; error: { code: string; message: string; details?: unknown } | null };
export type GroupedSearchState = { notes: NotesReadState; attachments: AttachmentReadState };
const notesPool = new ReadQueue(1);
const emptyNotes = (): NotesReadState => ({ page: null, pageNumber: 0, busy: false, error: null });
class NotesTextReader {
  state = emptyNotes();
  private lifetime = new ReadLifetime();
  private disposed = false;
  private committed: SearchQuery | null = null;
  private read: (input: SearchQuery) => Promise<SearchPage>;
  private changed: (state: NotesReadState) => void;
  constructor(read: (input: SearchQuery) => Promise<SearchPage>, changed: (state: NotesReadState) => void) { this.read = read; this.changed = changed; }
  private publish(state: NotesReadState) { this.state = state; this.changed(state); }
  edit() { if (this.disposed) return; this.lifetime.begin(); this.committed = null; this.publish(emptyNotes()); }
  search(query: string) { if (this.disposed || this.state.busy || !query.trim()) return Promise.resolve(); return this.load({ query, archived: false, limit: 20 }, 1); }
  next() { if (this.disposed || this.state.busy || !this.state.page?.next_cursor || !this.committed) return Promise.resolve(); return this.load({ ...this.committed, cursor: this.state.page.next_cursor }, this.state.pageNumber + 1); }
  private async load(input: SearchQuery, pageNumber: number) {
    const { token, signal } = this.lifetime.begin(); this.committed = { ...input }; this.publish({ page: null, pageNumber: 0, busy: true, error: null });
    try {
      const page = await notesPool.run(() => { signal.throwIfAborted(); return this.read(input); }, signal);
      if (!this.lifetime.current(token)) return;
      if (page.items.length > 20) throw new Error('Página de entidades excesiva. Buscá de nuevo.');
      this.publish({ page, pageNumber, busy: false, error: null });
    } catch (error) {
      if (!this.lifetime.current(token)) return;
      this.committed = null;
      const detail = error instanceof Error && 'detail' in error ? error.detail as CoreError : null;
      this.publish({ page: null, pageNumber: 0, busy: false, error: { code: detail?.code ?? 'transport_error', message: detail?.message ?? (error instanceof Error ? error.message : 'No se pudo consultar entidades.'), details: detail?.details } });
    }
  }
  dispose() { this.disposed = true; this.lifetime.dispose(); this.committed = null; }
}
export class GroupedTextSearch {
  state: GroupedSearchState = { notes: emptyNotes(), attachments: { page: null, pageNumber: 0, busy: false, error: null } };
  private notes: NotesTextReader;
  private attachments: AttachmentReader;
  private query = '';
  private disposed = false;
  private changed: (state: GroupedSearchState) => void;
  constructor(call: (command: Request['command']) => Promise<unknown>, changed: (state: GroupedSearchState) => void) {
    this.changed = changed;
    this.notes = new NotesTextReader(input => call({ operation: 'search.query', input }) as Promise<SearchPage>, notes => this.publish({ ...this.state, notes }));
    this.attachments = new AttachmentReader(input => call({ operation: 'extraction.search', input }) as Promise<import('./contracts.generated').AttachmentSearchPage>, attachments => this.publish({ ...this.state, attachments }));
  }
  private publish(state: GroupedSearchState) { this.state = state; this.changed(state); }
  edit() { this.query = ''; this.notes.edit(); this.attachments.edit(); }
  search(query: string) {
    if (this.disposed || this.state.notes.busy || this.state.attachments.busy || !query.trim()) return Promise.resolve();
    this.query = query;
    return Promise.all([this.notes.search(query), this.attachments.search({ query, extractor: '', job_id: '', attachment_id: '' })]).then(() => {});
  }
  nextNotes() { return this.notes.next(); }
  nextAttachments() { return this.attachments.next(); }
  startNotes() { return this.query ? this.notes.search(this.query) : Promise.resolve(); }
  startAttachments() { return this.query ? this.attachments.search({ query: this.query, extractor: '', job_id: '', attachment_id: '' }) : Promise.resolve(); }
  dispose() { this.disposed = true; this.notes.dispose(); this.attachments.dispose(); }
}
