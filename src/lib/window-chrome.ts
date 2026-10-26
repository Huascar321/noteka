import { isTauri } from '@tauri-apps/api/core';

export type DesktopPlatform = 'macos' | 'windows' | 'other' | 'web';

// This selects presentation only. Native authority remains in Tauri permissions.
export function desktopPlatform(): DesktopPlatform {
  if (!isTauri()) return 'web';
  const platform = navigator.platform || navigator.userAgent;
  if (/Mac/i.test(platform)) return 'macos';
  if (/Win/i.test(platform)) return 'windows';
  return 'other';
}
