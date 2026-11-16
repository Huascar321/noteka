import { desktopPlatform } from './window-chrome';

// Presentation only: handlers still accept Meta on macOS and Control elsewhere.
export function shortcutLabel(key: string): string {
  const platform = desktopPlatform();
  const mac = platform === 'macos' || (platform === 'web' && /Mac/i.test(navigator.platform || navigator.userAgent));
  return mac ? `⌘${key}` : `Ctrl+${key}`;
}
