// IPC en curso no es cancelable: cancelar descarta su resultado, pero conserva
// el cupo hasta terminar. Cancelar un trabajo en cola evita ejecutarlo.
export class ReadQueue {
  private active = 0;
  private pending: Array<() => void> = [];
  private readonly capacity: number;
  private readonly onIdle?: () => void;
  constructor(capacity = 2, onIdle?: () => void) {
    if (!Number.isInteger(capacity) || capacity < 1) throw new Error("Invalid read capacity.");
    this.capacity = capacity;
    this.onIdle = onIdle;
  }
  private drain() {
    while (this.active < this.capacity && this.pending.length) this.pending.shift()!();
    if (!this.active && !this.pending.length) this.onIdle?.();
  }
  run<T>(work: () => Promise<T>, signal: AbortSignal): Promise<T> {
    return new Promise((resolve, reject) => {
      let started = false;
      const abort = () => {
        if (!started) { this.pending = this.pending.filter(start => start !== startJob); this.drain(); }
        reject(new DOMException("Read discarded", 'AbortError'));
      };
      const startJob = () => {
        if (signal.aborted) { abort(); return; }
        started = true; this.active++;
        void Promise.resolve().then(() => {
          if (signal.aborted) throw new DOMException("Read discarded", 'AbortError');
          return work();
        }).then(resolve, reject).finally(() => {
          signal.removeEventListener('abort', abort); this.active--; this.drain();
        });
      };
      if (signal.aborted) { abort(); return; }
      signal.addEventListener('abort', abort, { once: true });
      this.pending.push(startJob); this.drain();
    });
  }
}

export class ReadLifetime {
  private generation = 0;
  private controller = new AbortController();
  private disposed = false;
  begin() {
    this.controller.abort(); this.controller = new AbortController();
    if (this.disposed) this.controller.abort();
    return { token: ++this.generation, signal: this.controller.signal };
  }
  current(token: number) { return !this.disposed && !this.controller.signal.aborted && this.generation === token; }
  dispose() { this.disposed = true; this.controller.abort(); this.generation++; }
}
