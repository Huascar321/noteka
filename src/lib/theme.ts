export type Theme = 'dark' | 'light';
export const THEME_KEY = 'noteka.theme.v1';
type Store = Pick<Storage, 'getItem' | 'setItem'>;
export function readTheme(store: () => Store): Theme {
  try { return store().getItem(THEME_KEY) === 'light' ? 'light' : 'dark'; } catch { return 'dark'; }
}
export function saveTheme(theme: Theme, store: () => Store): string {
  try { store().setItem(THEME_KEY, theme); return ''; } catch { return 'Tema aplicado; no se pudo guardar la preferencia local.'; }
}
