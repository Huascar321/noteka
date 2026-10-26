import type { ReadPdfOcrTable, PdfOcrTablePage, CoreError, FragmentPage, IndexedExtractor, PdfFragmentPage, OcrRegionPage, PdfkitPagePage, PdfOcrRegionPage, Request, XlsxRangePage, XlsxCellPage } from './contracts.generated';
import { ReadLifetime, ReadQueue } from './embedded-reads.ts';

export type GenerationPage = { kind: 'utf8_text'; data: FragmentPage } | { kind: 'pdf_text'; data: PdfFragmentPage } | { kind: 'pdfium_text'; data: PdfFragmentPage } | { kind: 'paddle_ocr'; data: PdfFragmentPage } | { kind: 'xlsx_cells'; data: XlsxCellPage } | { kind: 'image_ocr'; data: OcrRegionPage } | { kind: 'pdfkit_text'; data: PdfkitPagePage } | { kind: 'pdf_ocr'; data: PdfOcrRegionPage } | { kind: 'xlsx_range'; data: XlsxRangePage } | { kind: 'pdf_ocr_table'; data: PdfOcrTablePage };
export type GenerationKind = IndexedExtractor | 'xlsx_range' | 'pdf_ocr_table';
export type GenerationCriteria = { id: string } & ({ kind: IndexedExtractor } | { kind: 'xlsx_range'; sheet_id: number; range: string; header_row?: number } | { kind: 'pdf_ocr_table'; page_number: number; x_edges: string[]; y_min: string; y_max: string; header_row?: number });
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
    if (criteria.kind === 'xlsx_range' && (!Number.isFinite(criteria.sheet_id) || (criteria.header_row !== undefined && !Number.isFinite(criteria.header_row)))) {
      this.edit(); this.publish({ ...empty(), error: { code: 'validation_error', message: "Sheet and header row must be finite numbers." } }); return Promise.resolve();
    }
    if (criteria.kind === 'pdf_ocr_table' && (!Number.isFinite(criteria.page_number) || (criteria.header_row !== undefined && !Number.isFinite(criteria.header_row)))) {
      this.edit(); this.publish({ ...empty(), error: { code: 'validation_error', message: "Page and header row must be finite numbers." } }); return Promise.resolve();
    }
    return this.load(criteria.kind === 'pdf_ocr_table' ? { ...criteria, x_edges: [...criteria.x_edges], id: criteria.id.trim() } : { ...criteria, id: criteria.id.trim() }, 1);
  }
  next() {
    const previous = this.state.page;
    if (this.disposed || this.state.busy || !this.committed || !previous?.data.next_cursor) return Promise.resolve();
    return this.load(this.committed, this.state.pageNumber + 1, previous.data.next_cursor, previous);
  }
  private async load(criteria: GenerationCriteria, pageNumber: number, cursor?: string, previous?: GenerationPage) {
    const { token, signal } = this.lifetime.begin(); this.committed = { ...criteria };
    this.publish({ page: null, pageNumber: 0, busy: true, error: null });
    const operations = { utf8_text: 'extraction.fragments', pdf_text: 'extraction.pdf_fragments', pdfium_text: 'extraction.pdf_fragments', paddle_ocr: 'extraction.pdf_fragments', xlsx_cells: 'extraction.xlsx_cells', image_ocr: 'extraction.ocr_regions', pdfkit_text: 'extraction.pdfkit_pages', pdf_ocr: 'extraction.pdf_ocr_regions' } as const;
    const command: Request['command'] = criteria.kind === 'xlsx_range'
      ? { operation: 'extraction.xlsx_range', input: { id: criteria.id, sheet_id: criteria.sheet_id, range: criteria.range, ...(criteria.header_row === undefined ? {} : { header_row: criteria.header_row }), limit: GENERATION_PAGE_SIZE, ...(cursor ? { cursor } : {}) } }
      : criteria.kind === 'pdf_ocr_table' ? { operation: 'extraction.pdf_ocr_table', input: { id: criteria.id, page_number: criteria.page_number, x_edges: [...criteria.x_edges] as ReadPdfOcrTable['x_edges'], y_min: criteria.y_min, y_max: criteria.y_max, ...(criteria.header_row === undefined ? {} : { header_row: criteria.header_row }), limit: GENERATION_PAGE_SIZE, ...(cursor ? { cursor } : {}) } }
      : { operation: operations[criteria.kind], input: { id: criteria.id, limit: GENERATION_PAGE_SIZE, ...(cursor ? { cursor } : {}) } };
    try {
      const data = await pool.run(() => { signal.throwIfAborted(); return this.call(command); }, signal) as PdfOcrTablePage | FragmentPage | PdfFragmentPage | XlsxCellPage | XlsxRangePage | OcrRegionPage | PdfkitPagePage | PdfOcrRegionPage;
      if (!this.lifetime.current(token)) return;
      if (data.items.length > GENERATION_PAGE_SIZE || data.job_id !== criteria.id || data.generation.extractor !== (criteria.kind === 'xlsx_range' ? 'xlsx_cells' : criteria.kind === 'pdf_ocr_table' ? 'pdf_ocr' : criteria.kind) || (previous && (previous.kind !== criteria.kind || previous.data.generation.id !== data.generation.id))) {
        throw new Error("Page does not match the job or generation. Read again from the beginning.");
      }
      if (criteria.kind === 'xlsx_range') {
        const range = data as XlsxRangePage;
        if (range.sheet.id !== criteria.sheet_id || range.range !== criteria.range || range.header_row !== (criteria.header_row ?? null) || range.headers.length > 64) throw new Error("Range does not match the reading criteria.");
      }
      if (criteria.kind === 'pdf_ocr_table') {
        const table = data as PdfOcrTablePage;
        if (table.page.raster.page_number !== criteria.page_number || JSON.stringify(table.x_edges) !== JSON.stringify(criteria.x_edges) || table.y_min !== criteria.y_min || table.y_max !== criteria.y_max || table.header_row !== (criteria.header_row ?? null) || table.covers_generation !== false || table.outside.length > 512) throw new Error("Table does not match the reading criteria.");
      }
      const page = { kind: criteria.kind, data } as GenerationPage;
      this.publish({ page, pageNumber, busy: false, error: null });
    } catch (error) {
      if (!this.lifetime.current(token)) return;
      this.committed = null;
      const detail = error instanceof Error && 'detail' in error ? error.detail as CoreError : null;
      this.publish({ page: null, pageNumber: 0, busy: false, error: { code: detail?.code ?? 'transport_error', message: detail?.message ?? (error instanceof Error ? error.message : "Could not read the local output."), details: detail?.details } });
    }
  }
  dispose() { this.disposed = true; this.committed = null; this.lifetime.dispose(); }
}
