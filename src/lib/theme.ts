export type Theme = 'dark' | 'light';
export type ThemePreference = Theme | 'system';
export const THEME_KEY = 'noteka.theme.v1';
type Store = Pick<Storage, 'getItem' | 'setItem'>;
export function readTheme(store: () => Store): ThemePreference {
  try { const value = store().getItem(THEME_KEY); return value === 'light' || value === 'system' ? value : 'dark'; } catch { return 'dark'; }
}
export function resolveTheme(preference: ThemePreference, systemDark: boolean): Theme {
  return preference === 'system' ? (systemDark ? 'dark' : 'light') : preference;
}
export function saveTheme(theme: ThemePreference, store: () => Store): string {
  try { store().setItem(THEME_KEY, theme); return ''; } catch { return "Theme applied. Your preference could not be saved on this device."; }
}
