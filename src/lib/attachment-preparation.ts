import type { AttachmentIndexResult, Contract, Document, EnqueuedExtraction, ExtractionControl, ExtractionJob, ExtractionRun, IndexedExtractor, Request } from './contracts.generated';
import { ReadLifetime, ReadQueue } from './embedded-reads.ts';

export type IndexSnapshot = Extract<AttachmentIndexResult, { action: 'get' }>;
export type PreparationState = {
  attachmentId: string; jobId: string; jobs: Contract['extraction_jobs'] | null;
  pageNumber: number; job: ExtractionJob | null; control: ExtractionControl | null;
  index: IndexSnapshot | null; busy: boolean; needsConsult: boolean;
  receipt: { operation: string; data: unknown } | null;
  error: { code: string; message: string; details?: unknown; uncertain: boolean } | null;
};
export type PreparationCall = (command: Request['command']) => Promise<unknown>;
const pool = new ReadQueue(1);
const initial = (attachmentId = '', jobId = ''): PreparationState => ({ attachmentId, jobId, jobs: null, pageNumber: 0, job: null, control: null, index: null, busy: false, needsConsult: false, receipt: null, error: null });

// Sólo referencias semánticas conocidas; ningún dato del bloque se hace pasar por
// metadatos de attachment. No se recorren atributos de bloques futuros.
export function attachmentReferences(document: Document | null | undefined) {
  const refs = new Map<string, string>();
  function walk(blocks: Document['blocks']) {
    for (const block of blocks) {
      const attrs = block.attrs && typeof block.attrs === 'object' && !Array.isArray(block.attrs) ? block.attrs as Record<string, unknown> : null;
      if ((block.type === 'file' || block.type === 'image') && typeof attrs?.attachment_id === 'string') {
        const label = block.type === 'file' ? attrs.name : attrs.alt;
        if (!refs.has(attrs.attachment_id)) refs.set(attrs.attachment_id, typeof label === 'string' ? label : 'Sin rótulo');
      }
      if (block.children) walk(block.children);
    }
  }
  if (document?.blocks) walk(document.blocks);
  return [...refs].map(([id, label]) => ({ id, label }));
}

export class AttachmentPreparation {
  state = initial();
  private lifetime = new ReadLifetime();
  private disposed = false;
  private call: PreparationCall;
  private changed: (state: PreparationState) => void;
  constructor(call: PreparationCall, changed: (state: PreparationState) => void) { this.call = call; this.changed = changed; }
  private publish(state: PreparationState) { this.state = state; this.changed(state); }
  select(attachmentId: string) {
    if (this.disposed) return;
    this.lifetime.begin(); this.publish(initial(attachmentId.trim()));
  }
  chooseJob(jobId: string) {
    if (this.disposed) return;
    this.lifetime.begin(); this.publish({ ...initial(this.state.attachmentId, jobId), jobs: this.state.jobs, pageNumber: this.state.pageNumber });
  }
  private async execute(work: (signal: AbortSignal) => Promise<Partial<PreparationState>>, mutation = false) {
    if (this.disposed || this.state.busy) return;
    const { token, signal } = this.lifetime.begin();
    this.publish({ ...this.state, busy: true, error: null, receipt: null, job: null, control: null, index: null,
      ...(mutation ? { jobs: null, pageNumber: 0, needsConsult: true } : {}) });
    try {
      const result = await pool.run(() => { signal.throwIfAborted(); return work(signal); }, signal);
      if (this.lifetime.current(token)) this.publish({ ...this.state, ...result, busy: false });
    } catch (error) {
      if (!this.lifetime.current(token)) return;
      const detail = error instanceof Error && 'detail' in error ? error.detail as { code: string; message: string; details: unknown } : null;
      this.publish({ ...this.state, busy: false, jobs: null, pageNumber: 0, needsConsult: mutation || this.state.needsConsult,
        error: { code: detail?.code ?? 'transport_error', message: detail?.message ?? (error instanceof Error ? error.message : 'No se pudo completar la operación local.'), details: detail?.details, uncertain: mutation } });
    }
  }
  list(next = false) {
    const { attachmentId, jobs, pageNumber } = this.state;
    if (!attachmentId || (next && !jobs?.next_cursor)) return Promise.resolve();
    return this.execute(async () => {
      const page = await this.call({ operation: 'extraction.list', input: { attachment_id: attachmentId, limit: 20, ...(next ? { cursor: jobs!.next_cursor } : {}) } }) as Contract['extraction_jobs'];
      if (page.items.length > 20 || page.items.some(job => job.attachment_id !== attachmentId)) throw new Error('Página de trabajos incompatible. Consultá de nuevo.');
      return { jobs: page, pageNumber: next ? pageNumber + 1 : 1, jobId: '', needsConsult: false };
    });
  }
  inspect() {
    const { jobId, attachmentId } = this.state;
    if (!jobId) return Promise.resolve();
    return this.execute(async signal => {
      const job = await this.call({ operation: 'extraction.get', input: { id: jobId } }) as ExtractionJob;
      if (job.attachment_id !== attachmentId) throw new Error('El trabajo no pertenece al adjunto elegido.');
      signal.throwIfAborted();
      const control = await this.call({ operation: 'extraction.control', input: { action: 'get' } }) as ExtractionControl;
      signal.throwIfAborted();
      const index = await this.call({ operation: 'extraction.index', input: { action: 'get', job_id: jobId, limit: 1 } }) as IndexSnapshot;
      if (index.action !== 'get' || index.items.length > 1 || index.items.some(entry => entry.source.job_id !== jobId)) throw new Error('Selección de índice incompatible. Consultá de nuevo.');
      return { job, control, index, needsConsult: false };
    });
  }
  private mutate(command: Request['command']) {
    return this.execute(async () => {
      const data = await this.call(command);
      let jobId = this.state.jobId;
      if (command.operation === 'extraction.enqueue') jobId = (data as EnqueuedExtraction).job.id;
      if (command.operation === 'extraction.run') jobId = (data as ExtractionRun).job?.id ?? jobId;
      return { receipt: { operation: command.operation, data }, jobId };
    }, true);
  }
  enqueue(extractor: IndexedExtractor) {
    if (!this.state.attachmentId || this.state.needsConsult) return Promise.resolve();
    return this.mutate({ operation: 'extraction.enqueue', input: { attachment_id: this.state.attachmentId, extractor } });
  }
  setPaused() {
    const c = this.state.control;
    if (!c || this.state.needsConsult) return Promise.resolve();
    return this.mutate({ operation: 'extraction.control', input: { action: 'set', paused: !c.paused, expected_revision: c.revision } });
  }
  jobAction(action: 'run' | 'cancel' | 'retry') {
    const job = this.state.job;
    if (!job || this.state.needsConsult) return Promise.resolve();
    if (action === 'run') return this.mutate({ operation: 'extraction.run', input: { id: job.id } });
    return this.mutate({ operation: action === 'cancel' ? 'extraction.cancel' : 'extraction.retry', input: { id: job.id, expected_revision: job.revision } });
  }
  get canSelectGeneration() {
    const { job, index, needsConsult } = this.state;
    return !needsConsult && !!job?.generation && !!index && index.coverage.unsupported_generations === 0 && index.coverage.indexed_generations + index.coverage.unindexed_supported_generations > 0;
  }
  selectGeneration() {
    const { job, index } = this.state;
    if (!this.canSelectGeneration) return Promise.resolve();
    return this.mutate({ operation: 'extraction.index', input: { action: 'put', job_id: job!.id, expected_generation_id: job!.generation!.id, expected_revision: index!.revision } });
  }
  dispose() { this.disposed = true; this.lifetime.dispose(); }
}
