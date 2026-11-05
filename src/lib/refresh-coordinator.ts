// Coalesce repeated panel refreshes, including events received while blocked.
export class RefreshCoordinator {
  private pending = false;
  private running = false;
  private started = false;
  private disposed = false;
  private current: Promise<void> | null = null;
  private canRun: () => boolean;
  private work: () => Promise<void>;
  private failed: (error: unknown) => void;
  constructor(canRun: () => boolean, work: () => Promise<void>, failed: (error: unknown) => void) {
    this.canRun = canRun; this.work = work; this.failed = failed;
  }

  request(): Promise<void> {
    if (this.disposed) return Promise.resolve();
    if (!this.running || this.started) this.pending = true;
    this.notifyReady();
    return this.current ?? Promise.resolve();
  }

  notifyReady() {
    if (this.disposed || this.running || !this.pending || !this.canRun()) return;
    this.running = true;
    this.started = false;
    this.current = Promise.resolve().then(async () => {
      while (!this.disposed && this.pending && this.canRun()) {
        this.pending = false;
        this.started = true;
        try { await this.work(); } catch (error) { if (!this.disposed) this.failed(error); }
      }
    }).finally(() => {
      this.running = false;
      this.started = false;
      this.current = null;
      this.notifyReady();
    });
  }

  cancelPending() { this.pending = false; }
  dispose() { this.disposed = true; this.pending = false; }
}
