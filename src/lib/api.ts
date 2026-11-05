import { invoke, isTauri } from '@tauri-apps/api/core';
import type { Request, Response, CoreError } from './contracts.generated';
import { workspaceReadPool } from './workspace-reads';

export class OperationError extends Error {
  constructor(public detail: CoreError) { super(detail.message); }
}
const coordinatedReads = new Set<Request['command']['operation']>([
  'home.query', 'entity.get', 'entity.list', 'relation.list', 'workflow.query', 'search.query',
  'collection.get', 'collection.list', 'database.query', 'template.list', 'extraction.search',
]);

export async function operate<T>(command: Request['command'], workspace?: string, options?: { signal?: AbortSignal }): Promise<T> {
  if (!isTauri()) throw new Error("Open Noteka with npm run tauri dev. The browser has no desktop storage.");
  const call = async () => {
    const response = await invoke<Response>('operate', { request: { workspace, command } });
    if (!response.ok) throw new OperationError(response.error!);
    return response.data as T;
  };
  return workspace && coordinatedReads.has(command.operation)
    ? workspaceReadPool.run(workspace, call, options?.signal)
    : call();
}
export async function setPending(pending: boolean) {
  if (isTauri()) await invoke('pending_edits', { pending });
}
export async function closeConfirmed() {
  if (isTauri()) await invoke('close_confirmed');
}

export type GooglePickedFile = {
  id: string;
  name?: string;
  mimeType?: string;
  modifiedTime?: string;
  webViewLink?: string;
};
export type GooglePickResult = {
  account_id: string;
  picked_file_ids: string[];
  files: GooglePickedFile[];
};
export async function pickGoogleDrive(): Promise<GooglePickResult> {
  if (!isTauri()) throw new Error("Google Picker requires the desktop app.");
  return invoke<GooglePickResult>('google_pick');
}
export async function openExternal(url: string): Promise<void> {
  if (!isTauri()) throw new Error("Opening in a browser requires the desktop app.");
  await invoke('open_external', { url });
}

export async function currentWeatherLocation(): Promise<unknown> {
  if (!isTauri()) throw new Error('Current location requires the desktop app.');
  return invoke('weather_current_location');
}

export type GooglePreview = {
  id: string;
  name: string;
  mime_type: string;
  preview_mime: 'text/plain' | 'text/csv';
  text: string;
};
export async function googlePreview(fileId: string): Promise<GooglePreview> {
  if (!isTauri()) throw new Error("Preview requires the desktop app.");
  return invoke<GooglePreview>('google_preview', { fileId });
}
