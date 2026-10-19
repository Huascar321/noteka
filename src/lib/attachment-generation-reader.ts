import type { CoreError, FragmentPage, IndexedExtractor, PdfFragmentPage, OcrRegionPage, Request, XlsxCellPage } from './contracts.generated';
import { ReadLifetime, ReadQueue } from './embedded-reads.ts';

export type GenerationPage = { kind: 'utf8_text'; data: FragmentPage } | { kind: 'pdf_text'; data: PdfFragmentPage } | { kind: 'xlsx_cells'; data: XlsxCellPage } | { kind: 'image_ocr'; data: OcrRegionPage };
export type GenerationCriteria = { id: string; kind: IndexedExtractor };
export type GenerationReadState = {
  page: GenerationPage | null; pageNumber: number; busy: boolean;
  error: { code: string; message: string; details?: unknown } | null;
};
export const GENERATION_PAGE_SIZE = 20;
const pool = new ReadQueue(1);
const empty = (): GenerationReadState => ({ page: null, pageNumber: 0, busy: false, error: null });
export class AttachmentGenerationReader {
  state = empty();
  private lifetime = new ReadLifetime();
  private disposed = false;
  private committed: GenerationCriteria | null = null;
  private call: (command: Request['command']) => Promise<unknown>;
  private changed: (state: GenerationReadState) => void;
  constructor(call: (command: Request['command']) => Promise<unknown>, changed: (state: GenerationReadState) => void) { this.call = call; this.changed = changed; }
  private publish(state: GenerationReadState) { this.state = state; this.changed(state); }
  edit() {
    if (this.disposed) return;
    this.lifetime.begin(); this.committed = null; this.publish(empty());
  }
  read(criteria: GenerationCriteria) {
    if (this.disposed || this.state.busy || !criteria.id.trim()) return Promise.resolve();
    return this.load({ id: criteria.id.trim(), kind: criteria.kind }, 1);
  }
  next() {
    const previous = this.state.page;
    if (this.disposed || this.state.busy || !this.committed || !previous?.data.next_cursor) return Promise.resolve();
    return this.load(this.committed, this.state.pageNumber + 1, previous.data.next_cursor, previous);
  }
  private async load(criteria: GenerationCriteria, pageNumber: number, cursor?: string, previous?: GenerationPage) {
    const { token, signal } = this.lifetime.begin(); this.committed = { ...criteria };
    this.publish({ page: null, pageNumber: 0, busy: true, error: null });
    const operations = { utf8_text: 'extraction.fragments', pdf_text: 'extraction.pdf_fragments', xlsx_cells: 'extraction.xlsx_cells', image_ocr: 'extraction.ocr_regions' } as const;
    const operation = operations[criteria.kind];
    try {
      const data = await pool.run(() => { signal.throwIfAborted(); return this.call({ operation, input: { id: criteria.id, limit: GENERATION_PAGE_SIZE, ...(cursor ? { cursor } : {}) } }); }, signal) as FragmentPage | PdfFragmentPage | XlsxCellPage | OcrRegionPage;
      if (!this.lifetime.current(token)) return;
      if (data.items.length > GENERATION_PAGE_SIZE || data.job_id !== criteria.id || data.generation.extractor !== criteria.kind || (previous && (previous.kind !== criteria.kind || previous.data.generation.id !== data.generation.id))) {
        throw new Error('Página incompatible con el trabajo o generación. Volvé a leer desde el inicio.');
      }
      const page = { kind: criteria.kind, data } as GenerationPage;
      this.publish({ page, pageNumber, busy: false, error: null });
    } catch (error) {
      if (!this.lifetime.current(token)) return;
      this.committed = null;
      const detail = error instanceof Error && 'detail' in error ? error.detail as CoreError : null;
      this.publish({ page: null, pageNumber: 0, busy: false, error: { code: detail?.code ?? 'transport_error', message: detail?.message ?? (error instanceof Error ? error.message : 'No se pudo leer el derivado local.'), details: detail?.details } });
    }
  }
  dispose() { this.disposed = true; this.committed = null; this.lifetime.dispose(); }
}
