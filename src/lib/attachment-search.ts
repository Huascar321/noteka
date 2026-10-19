import type { AttachmentSearchInput, AttachmentSearchPage, CoreError, IndexedExtractor } from './contracts.generated';
import { ReadLifetime, ReadQueue } from './embedded-reads.ts';

export type AttachmentCriteria = { query: string; extractor: IndexedExtractor | ''; job_id: string; attachment_id: string };
export type AttachmentFailure = { code: string; reason?: string; message: string };
export type AttachmentReadState = { page: AttachmentSearchPage | null; pageNumber: number; busy: boolean; error: AttachmentFailure | null };
export const ATTACHMENT_PAGE_SIZE = 20;
// Compartido entre montajes/workspaces: descartar invoke no libera su cupo real.
const pool = new ReadQueue(1);

function failure(error: unknown): AttachmentFailure {
  if (error instanceof Error && 'detail' in error) {
    const detail = error.detail as CoreError;
    if (detail && typeof detail.code === 'string' && typeof detail.message === 'string') {
      const details = detail.details;
      const reason = details && typeof details === 'object' && !Array.isArray(details) && 'reason' in details && typeof details.reason === 'string' ? details.reason : undefined;
      return { code: detail.code, message: detail.message, ...(reason ? { reason } : {}) };
    }
  }
  return { code: 'transport_error', message: error instanceof Error ? error.message : 'No se pudo consultar evidencia local.' };
}

export class AttachmentReader {
  private lifetime = new ReadLifetime();
  private disposed = false;
  private committed: AttachmentSearchInput | null = null;
  private read: (input: AttachmentSearchInput) => Promise<AttachmentSearchPage>;
  private changed: (state: AttachmentReadState) => void;
  state: AttachmentReadState = { page: null, pageNumber: 0, busy: false, error: null };

  constructor(read: (input: AttachmentSearchInput) => Promise<AttachmentSearchPage>, changed: (state: AttachmentReadState) => void) {
    this.read = read; this.changed = changed;
  }
  private publish(state: AttachmentReadState) { this.state = state; this.changed(state); }
  edit() {
    if (this.disposed) return;
    this.lifetime.begin(); this.committed = null;
    this.publish({ page: null, pageNumber: 0, busy: false, error: null });
  }
  search(criteria: AttachmentCriteria) {
    if (this.disposed || this.state.busy || !criteria.query.trim()) return Promise.resolve();
    const query: AttachmentSearchInput = { query: criteria.query, limit: ATTACHMENT_PAGE_SIZE,
      ...(criteria.extractor ? { extractor: criteria.extractor } : {}),
      ...(criteria.job_id.trim() ? { job_id: criteria.job_id.trim() } : {}),
      ...(criteria.attachment_id.trim() ? { attachment_id: criteria.attachment_id.trim() } : {}) };
    return this.load(query, 1);
  }
  next() {
    const page = this.state.page;
    if (this.disposed || this.state.busy || !page?.next_cursor || !this.committed) return Promise.resolve();
    return this.load({ ...this.committed, cursor: page.next_cursor }, this.state.pageNumber + 1, page);
  }
  private async load(query: AttachmentSearchInput, pageNumber: number, previous?: AttachmentSearchPage) {
    const { token, signal } = this.lifetime.begin();
    this.committed = { ...query };
    this.publish({ page: null, pageNumber: 0, busy: true, error: null });
    try {
      const page = await pool.run(() => this.read(query), signal);
      if (!this.lifetime.current(token)) return;
      if (page.items.length > ATTACHMENT_PAGE_SIZE || (previous && (page.revision !== previous.revision || page.profile !== previous.profile || page.version !== previous.version))) {
        throw new Error('Página incompatible con la consulta. Buscá de nuevo.');
      }
      this.publish({ page, pageNumber, busy: false, error: null });
    } catch (error) {
      if (!this.lifetime.current(token)) return;
      this.committed = null;
      this.publish({ page: null, pageNumber: 0, busy: false, error: failure(error) });
    }
  }
  dispose() { this.disposed = true; this.committed = null; this.lifetime.dispose(); }
}
