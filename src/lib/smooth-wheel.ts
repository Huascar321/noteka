/** Local presentation preference; workspace data is not involved. */
export const SMOOTH_WHEEL_KEY = 'noteka.smooth-wheel.v1';
type Store = Pick<Storage, 'getItem' | 'setItem'>;
export function readSmoothWheel(store: () => Store): boolean {
  try { return store().getItem(SMOOTH_WHEEL_KEY) !== 'off'; } catch { return true; }
}
export function saveSmoothWheel(enabled: boolean, store: () => Store): string {
  try { store().setItem(SMOOTH_WHEEL_KEY, enabled ? 'on' : 'off'); return ''; }
  catch { return 'Scrolling preference applied. It could not be saved on this device.'; }
}

/** A conservative signal, not reliable identification of the physical device. */
export function isDiscreteWheel(event: Pick<WheelEvent, 'deltaMode' | 'deltaX' | 'deltaY' | 'ctrlKey' | 'metaKey' | 'altKey' | 'shiftKey'> & { wheelDeltaY?: number }): boolean {
  if (event.ctrlKey || event.metaKey || event.altKey || event.shiftKey || event.deltaX !== 0 || !Number.isFinite(event.deltaY) || event.deltaY === 0) return false;
  if (event.deltaMode === 1) return true;
  if (event.deltaMode !== 0) return false;
  const legacy = event.wheelDeltaY;
  return typeof legacy === 'number' && Number.isFinite(legacy) && Math.abs(legacy) >= 120 && legacy % 120 === 0;
}

export function installSmoothWheel(doc: Document): () => void {
  type Pending = { element: HTMLElement; target: number; direction: number; time: number };
  let pending: Pending | null = null;
  let expiry: ReturnType<typeof setTimeout> | undefined;
  function stop() {
    if (expiry !== undefined) { clearTimeout(expiry); expiry = undefined; }
    if (!pending) return;
    const el = pending.element; pending = null;
    if (el.isConnected) el.scrollTo({ top: el.scrollTop, behavior: 'instant' });
  }
  function findScroller(event: WheelEvent): HTMLElement | null {
    for (const node of event.composedPath()) {
      if (!(node instanceof HTMLElement)) continue;
      if (node instanceof HTMLSelectElement || node instanceof HTMLTextAreaElement || (node instanceof HTMLInputElement && ['number', 'range', 'date', 'time'].includes(node.type))) return null;
      const style = getComputedStyle(node);
      if (!['auto', 'scroll'].includes(style.overflowY)) continue;
      const max = node.scrollHeight - node.clientHeight;
      if (max <= 1) {
        if (['contain', 'none'].includes(style.overscrollBehaviorY)) return null;
        continue;
      }
      const current = node.scrollTop;
      if ((event.deltaY < 0 && current <= 0) || (event.deltaY > 0 && current >= max - 1)) return null;
      return node;
    }
    return null;
  }
  function wheel(event: WheelEvent) {
    if (event.defaultPrevented || !event.cancelable || !isDiscreteWheel(event)) { stop(); return; }
    const element = findScroller(event);
    if (!element) { stop(); return; }
    const lineHeight = Number.parseFloat(getComputedStyle(element).lineHeight) || 16;
    const delta = event.deltaY * (event.deltaMode === 1 ? lineHeight : 1);
    const direction = Math.sign(delta), now = performance.now();
    if (pending && (pending.element !== element || pending.direction !== direction)) stop();
    const base = pending && now - pending.time < 350 ? pending.target : element.scrollTop;
    const target = Math.max(0, Math.min(element.scrollHeight - element.clientHeight, base + delta));
    if (Math.abs(target - element.scrollTop) < 1) { stop(); return; }
    event.preventDefault();
    pending = { element, target, direction, time: now };
    if (expiry !== undefined) clearTimeout(expiry);
    expiry = setTimeout(() => { pending = null; expiry = undefined; }, 800);
    element.scrollTo({ top: target, behavior: 'smooth' });
  }
  const interrupt = () => stop();
  doc.addEventListener('wheel', wheel, { passive: false });
  doc.addEventListener('pointerdown', interrupt, { passive: true });
  doc.addEventListener('keydown', interrupt);
  return () => { doc.removeEventListener('wheel', wheel); doc.removeEventListener('pointerdown', interrupt); doc.removeEventListener('keydown', interrupt); stop(); };
}
