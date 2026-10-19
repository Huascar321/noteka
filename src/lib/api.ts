import { invoke, isTauri } from '@tauri-apps/api/core';
import type { Request, Response, CoreError } from './contracts.generated';

export class OperationError extends Error {
  constructor(public detail: CoreError) { super(detail.message); }
}
export async function operate<T>(command: Request['command'], workspace?: string): Promise<T> {
  if (!isTauri()) throw new Error('Abrí Noteka con npm run tauri dev. El navegador no tiene almacenamiento de escritorio.');
  const response = await invoke<Response>('operate', { request: { workspace, command } });
  if (!response.ok) throw new OperationError(response.error!);
  return response.data as T;
}
export async function setPending(pending: boolean) {
  if (isTauri()) await invoke('pending_edits', { pending });
}
export async function closeConfirmed() {
  if (isTauri()) await invoke('close_confirmed');
}
