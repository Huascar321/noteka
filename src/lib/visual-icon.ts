import { ReadQueue } from './embedded-reads';
import { operate } from './api';
import { iconName, type IconName } from './icon-set';
import type { AttachmentData } from './contracts.generated';

export const WORKSPACE_ICON_CONTEXT = Symbol('noteka-workspace-icons');
const queue = new ReadQueue(2);
const cache = new Map<string, Promise<string>>();
const MAX_BYTES = 1_048_576;

export function imageIconId(value: unknown): string | null {
  if (typeof value !== 'string' || !/^image-[a-f0-9]{32}$/.test(value)) return null;
  const raw = value.slice(6);
  return `${raw.slice(0, 8)}-${raw.slice(8, 12)}-${raw.slice(12, 16)}-${raw.slice(16, 20)}-${raw.slice(20)}`;
}

export function visualIconName(value: unknown, fallback: IconName = 'file-text'): string {
  return imageIconId(value) ? String(value) : iconName(value, fallback);
}

export function loadIconImage(workspace: string, id: string): Promise<string> {
  const key = JSON.stringify([workspace, id]);
  const previous = cache.get(key);
  if (previous) { cache.delete(key); cache.set(key, previous); return previous; }
  const signal = new AbortController().signal;
  const request: Promise<string> = queue.run(async () => {
    const data = await operate<AttachmentData>({ operation: 'attachment.read', input: { id } }, workspace);
    if (data.attachment.id !== id || data.attachment.bytes > MAX_BYTES || data.base64.length > Math.ceil(MAX_BYTES / 3) * 4
      || !['image/png', 'image/jpeg', 'image/gif', 'image/webp'].includes(data.attachment.mime)) {
      throw new Error("The local icon must be a supported image of up to 1 MiB.");
    }
    return `data:${data.attachment.mime};base64,${data.base64}`;
  }, signal).catch(error => {
    if (cache.get(key) === request) cache.delete(key);
    throw error;
  });
  cache.set(key, request);
  while (cache.size > 32) cache.delete(cache.keys().next().value!);
  return request;
}
