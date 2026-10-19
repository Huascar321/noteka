import type { HomePage, HomeQuery, HomeView } from './contracts.generated';
import { ReadLifetime, ReadQueue } from './embedded-reads.ts';

export type HomeReadState = { view: HomeView; page: HomePage | null; busy: boolean; error: string };
export class HomeReader {
  private lifetime = new ReadLifetime();
  private queue: ReadQueue;
  private read: (query: HomeQuery) => Promise<HomePage>;
  private changed: (state: HomeReadState) => void;
  private query: HomeQuery;
  state: HomeReadState;
  constructor(view: HomeView, queue: ReadQueue, read: (query: HomeQuery) => Promise<HomePage>, changed: (state: HomeReadState) => void) {
    this.queue = queue; this.read = read; this.changed = changed;
    this.query = { view };
    this.state = { view, page: null, busy: false, error: '' };
  }
  async load(view = this.state.view, cursor?: string, date?: string) {
    const { token, signal } = this.lifetime.begin();
    if (!this.lifetime.current(token)) return;
    this.query = { view, cursor, as_of: date, limit: 50 };
    const query = this.query;
    this.state = { view, page: null, busy: true, error: '' }; this.changed(this.state);
    try {
      const page = await this.queue.run(() => this.read(query), signal);
      if (!this.lifetime.current(token)) return;
      if (page.view !== view || (date && page.date !== date)) throw new Error('Respuesta Home incompatible con la consulta.');
      this.state = { view, page, busy: false, error: '' }; this.changed(this.state);
    } catch (error) {
      if (!this.lifetime.current(token)) return;
      this.state = { view, page: null, busy: false, error: error instanceof Error ? error.message : 'No se pudo consultar Home.' }; this.changed(this.state);
    }
  }
  retry() { return this.load(this.query.view, this.query.cursor ?? undefined, this.query.as_of ?? undefined); }
  dispose() { this.lifetime.dispose(); }
}
