import { ReadQueue } from './embedded-reads.ts';

// Limita las lecturas físicas compartidas entre paneles de un mismo workspace.
// Una cancelación después del inicio no retira el pool ni libera su cupo.
export class WorkspaceReadPool {
  private queues = new Map<string, ReadQueue>();
  get size() { return this.queues.size; }
  run<T>(workspace: string, work: () => Promise<T>, signal?: AbortSignal): Promise<T> {
    let queue = this.queues.get(workspace);
    if (!queue) {
      const created = new ReadQueue(2, () => {
        if (this.queues.get(workspace) === created) this.queues.delete(workspace);
      });
      queue = created;
      this.queues.set(workspace, queue);
    }
    return queue.run(work, signal ?? new AbortController().signal);
  }
}

export const workspaceReadPool = new WorkspaceReadPool();
