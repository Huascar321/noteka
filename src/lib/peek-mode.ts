export type PeekMode = 'side' | 'center' | 'full' | 'tab';
export type PeekPresentation = Exclude<PeekMode, 'tab'>;
export const PEEK_MODE_KEY = 'noteka.peek-mode.v1';
export const PEEK_MODES: { id: PeekMode; label: string; icon: string }[] = [
  { id: 'side', label: 'Side peek', icon: 'sidebar-right' },
  { id: 'center', label: 'Center peek', icon: 'app-window' },
  { id: 'full', label: 'Full page', icon: 'expand' },
  { id: 'tab', label: 'New tab', icon: 'plus' },
];
type Store = Pick<Storage, 'getItem' | 'setItem'>;
export function readPeekMode(store: () => Store): PeekMode {
  try { const mode = store().getItem(PEEK_MODE_KEY); return PEEK_MODES.some(item => item.id === mode) ? mode as PeekMode : 'side'; }
  catch { return 'side'; }
}
export function savePeekMode(mode: PeekMode, store: () => Store): string {
  try { store().setItem(PEEK_MODE_KEY, mode); return ''; }
  catch { return 'Opening mode applied. Your preference could not be saved on this device.'; }
}
