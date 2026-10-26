export type CoverMode = 'ascii' | 'photo';
export const COVER_KEY = 'noteka.home-cover.v1';
type Store = Pick<Storage, 'getItem' | 'setItem'>;
export function readCover(store: () => Store): CoverMode {
  try { return store().getItem(COVER_KEY) === 'photo' ? 'photo' : 'ascii'; } catch { return 'ascii'; }
}
export function saveCover(mode: CoverMode, store: () => Store): string {
  try { store().setItem(COVER_KEY, mode); return ''; } catch { return "Cover applied. Your preference could not be saved on this device."; }
}
