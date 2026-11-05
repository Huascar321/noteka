export type PanelKind = 'sidebar' | 'sidePeek';
export type PanelWidths = { sidebar: number; sidePeek: number | null };
export const PANEL_WIDTHS_KEY = 'noteka.panel-widths.v1';
export const PANEL_MOTION_MS = 260;
const defaults = (): PanelWidths => ({ sidebar: 250, sidePeek: null });
const valid = (value: unknown): value is number => typeof value === 'number' && Number.isFinite(value) && value > 0;
type Store = Pick<Storage, 'getItem' | 'setItem'>;

export function panelLimits(kind: PanelKind, viewport: number) {
  const width = Number.isFinite(viewport) ? Math.max(0, viewport) : 0;
  const max = Math.floor(width * (kind === 'sidebar' ? 35 : 65) / 100);
  return { min: Math.min(kind === 'sidebar' ? 180 : 280, max), max };
}
export function clampPanelWidth(kind: PanelKind, value: number, viewport: number): number {
  const { min, max } = panelLimits(kind, viewport);
  return Math.min(max, Math.max(min, Number.isFinite(value) ? Math.round(value) : min));
}
export function effectivePanelWidth(kind: PanelKind, preferred: PanelWidths, viewport: number): number {
  return clampPanelWidth(kind, kind === 'sidebar' ? preferred.sidebar : preferred.sidePeek ?? viewport * .45, viewport);
}
export function sidePeekExcludesSidebar(width: number, viewport: number): boolean { return width * 100 > viewport * 45; }
export function readPanelWidths(store: () => Store): PanelWidths {
  try {
    const value = JSON.parse(store().getItem(PANEL_WIDTHS_KEY) ?? 'null');
    if (!value || typeof value !== 'object' || Array.isArray(value)) return defaults();
    return { sidebar: valid(value.sidebar) ? value.sidebar : 250, sidePeek: valid(value.sidePeek) ? value.sidePeek : null };
  } catch { return defaults(); }
}
export function savePanelWidths(value: PanelWidths, store: () => Store): string {
  try { store().setItem(PANEL_WIDTHS_KEY, JSON.stringify({ sidebar: value.sidebar, sidePeek: value.sidePeek })); return ''; }
  catch { return 'Panel widths applied. Your preference could not be saved on this device.'; }
}
