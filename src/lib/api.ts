import { invoke, isTauri } from '@tauri-apps/api/core';
import type { Request, Response, CoreError } from './contracts.generated';

export class OperationError extends Error {
  constructor(public detail: CoreError) { super(detail.message); }
}
export async function operate<T>(command: Request['command'], workspace?: string): Promise<T> {
  if (!isTauri()) throw new Error("Open Noteka with npm run tauri dev. The browser has no desktop storage.");
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
