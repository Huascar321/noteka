import { tick } from 'svelte';
import type { TransitionConfig } from 'svelte/transition';
import { tweened } from 'svelte/motion';
import { LIQUID_OPEN_MS, LIQUID_CLOSE_MS, liquidMaterialFrames } from './liquid-motion';

type Rect = { left: number; top: number; width?: number; height?: number; right?: number; bottom?: number };
export type LiquidOptions = {
  anchor?: () => Element | Rect | null | undefined;
  enabled?: boolean;
  /** Presence for panels which must remain mounted (PageTools). */
  visible?: boolean;
};
type Content = { node: HTMLElement; opacity: string; clip: string; clipPriority: string; animation?: Animation };
type Motion = { glass: HTMLDivElement; material: Animation; content: Content[]; direction: 'in' | 'out'; radius: string; reveal: () => void; driveStop?: () => void; stop: () => void; dispose: () => void };
const options = new WeakMap<HTMLElement, LiquidOptions>();
const motions = new WeakMap<HTMLElement, Motion>();
const visibility = new WeakMap<HTMLElement, string>();
const reduced = () => matchMedia('(prefers-reduced-motion: reduce)').matches;

function restoreVisibility(node: HTMLElement) {
  if (!visibility.has(node)) return;
  node.style.visibility = visibility.get(node)!;
  visibility.delete(node);
}

export function hasLiquidOverlayWithin(root: HTMLElement | null): boolean {
  return [...(root?.querySelectorAll<HTMLElement>('.liquid-surface:not([hidden]):not([inert])') ?? [])]
    .some(node => node.getClientRects().length > 0);
}

/** Navigation waits for modal teardown so its focus restoration cannot steal
 * focus back from the destination which has just opened. Register before closing. */
export function liquidExitFinished(node: HTMLElement | null): Promise<void> {
  if (!node || reduced()) return Promise.resolve();
  return new Promise(resolve => {
    const finish = () => { clearTimeout(timer); node.removeEventListener('outroend', finish); resolve(); };
    const timer = setTimeout(finish, LIQUID_CLOSE_MS + 80);
    node.addEventListener('outroend', finish, { once: true });
  });
}

function sourceFrame(node: HTMLElement, getAnchor?: LiquidOptions['anchor']): Keyframe {
  const rect = node.getBoundingClientRect();
  let anchor: ReturnType<NonNullable<LiquidOptions['anchor']>>;
  try { anchor = getAnchor?.(); } catch { anchor = null; } // Caret owners can be disposed before an ancestor outro.
  const source = anchor instanceof Element ? anchor.getBoundingClientRect() : anchor;
  const rawWidth = source ? source.width ?? (source.right! - source.left) : 0;
  const width = source && !(anchor instanceof Element) ? Math.max(12, rawWidth) : rawWidth;
  const height = source ? source.height ?? (source.bottom! - source.top) : 0;
  if (source && width > 0 && height > 0 && source.left + width > 0 && source.top + height > 0 && source.left < innerWidth && source.top < innerHeight && (!(anchor instanceof Element) || anchor.isConnected)) {
    const sx = width / rect.width, sy = height / rect.height;
    const radius = Math.min(anchor instanceof Element ? parseFloat(getComputedStyle(anchor).borderTopLeftRadius) || 12 : 8, height / 2);
    return { transform: `translate(${source.left - rect.left}px, ${source.top - rect.top}px) scale(${sx}, ${sy})`, borderRadius: `${radius / sx}px / ${radius / sy}px` };
  }
  return { transform: `translate(${rect.width * .07}px, ${rect.height * .07}px) scale(.86, .86)`, borderRadius: getComputedStyle(node).borderRadius };
}

function begin(node: HTMLElement, direction: 'in' | 'out'): Motion | null {
  if (direction === 'in') restoreVisibility(node);
  const inline = node instanceof HTMLDialogElement || node.classList.contains('peek-center');
  if (options.get(node)?.enabled === false || reduced() || !node.isConnected || !node.getClientRects().length || (!inline && typeof node.showPopover !== 'function')) return null;
  let previous = motions.get(node);
  const source = sourceFrame(node, options.get(node)?.anchor);
  const rect = node.getBoundingClientRect();
  const from = previous ? { transform: getComputedStyle(previous.glass).transform, borderRadius: getComputedStyle(previous.glass).borderRadius } : null;
  const opacities = previous?.content.map(item => getComputedStyle(item.node).opacity);
  const glass = previous?.glass ?? document.createElement('div');
  const radius = previous?.radius ?? getComputedStyle(node).borderRadius;
  if (previous) {
    previous.stop();
  } else {
    const style = getComputedStyle(node);
    glass.className = `liquid-morph-glass ${node.classList.contains('glass-popup') ? 'glass-popup' : 'glass-menu'}${inline ? ' liquid-morph-inline' : ''}`;
    if (!inline) glass.setAttribute('popover', 'manual');
    glass.setAttribute('aria-hidden', 'true');
    glass.inert = true;
    const sheen = document.createElement('span');
    sheen.className = 'liquid-morph-sheen'; glass.append(sheen);
    for (const property of ['background', 'border', 'border-radius', 'box-shadow', 'backdrop-filter', '-webkit-backdrop-filter', 'color']) glass.style.setProperty(property, style.getPropertyValue(property));
    node.append(glass);
    if (!inline) glass.showPopover();
  }
  Object.assign(glass.style, { left: `${inline ? -node.clientLeft : rect.left}px`, top: `${inline ? -node.clientTop : rect.top}px`, width: `${rect.width}px`, height: `${rect.height}px` });
  const sheen = glass.querySelector<HTMLElement>('.liquid-morph-sheen');
  if (sheen) sheen.hidden = direction === 'out';
  const children: Content[] = previous?.content ?? [...node.children].filter((child): child is HTMLElement => child instanceof HTMLElement && child !== glass).map(child => ({ node: child, opacity: getComputedStyle(child).opacity, clip: child.style.getPropertyValue('clip-path'), clipPriority: child.style.getPropertyPriority('clip-path') }));
  node.classList.add('liquid-morph-active');
  if (inline) node.classList.add('liquid-inline-active');
  const duration = direction === 'in' ? LIQUID_OPEN_MS : LIQUID_CLOSE_MS;
  const material = glass.animate(direction === 'in'
    ? liquidMaterialFrames(from ?? source, radius, -3, { x: Math.max(1, Math.min(1.035, (innerWidth - rect.left - 6) / rect.width)), y: Math.max(1, Math.min(1.025, (innerHeight - rect.top - 6) / rect.height)) })
    : [from ?? { transform: 'none', borderRadius: radius }, source],
  { duration, easing: direction === 'out' ? 'cubic-bezier(.4,0,.25,1)' : 'linear', fill: 'both' });
  material.id = `liquid-overlay-${direction}`;
  material.pause(); material.currentTime = 0;
  children.forEach((item, index) => {
    const entry: Keyframe[] = inline ? [
      { opacity: opacities?.[index] ?? Number(item.opacity) * .24, offset: 0, easing: 'ease-out' },
      { opacity: item.opacity, offset: .55 }, { opacity: item.opacity, offset: 1 },
    ] : [
      { opacity: opacities?.[index] ?? 0, offset: 0 }, { opacity: opacities?.[index] ?? 0, offset: .3, easing: 'ease-out' }, { opacity: item.opacity, offset: .8 }, { opacity: item.opacity, offset: 1 },
    ];
    item.animation = item.node.animate(direction === 'in' ? entry : [{ opacity: opacities?.[index] ?? getComputedStyle(item.node).opacity }, { opacity: 0 }],
    { duration: direction === 'in' ? duration : duration / 2, fill: 'both' });
    item.animation.id = 'liquid-overlay-content';
    item.animation.pause(); item.animation.currentTime = 0;
  });
  // Reveal the actual controls inside the moving shape. Keeping the glass
  // behind them prevents its backdrop blur from veiling the dialog's own text.
  const reveal = () => {
    if (!inline) return;
    const bounds = glass.getBoundingClientRect(), style = getComputedStyle(glass);
    const matrix = new DOMMatrixReadOnly(style.transform);
    const radii = style.borderTopLeftRadius.split(' ');
    const rx = parseFloat(radii[0]) * Math.abs(matrix.a), ry = parseFloat(radii[1] ?? radii[0]) * Math.abs(matrix.d);
    const clips = children.map(item => { const r = item.node.getBoundingClientRect(); return `inset(${bounds.top-r.top}px ${r.right-bounds.right}px ${r.bottom-bounds.bottom}px ${bounds.left-r.left}px round ${rx}px / ${ry}px)`; });
    children.forEach((item, index) => item.node.style.setProperty('clip-path', clips[index]));
  };
  const place = () => {
    if (!node.isConnected || !node.getClientRects().length) { motion.dispose(); return; }
    const bounds = node.getBoundingClientRect();
    Object.assign(glass.style, { left: `${inline ? -node.clientLeft : bounds.left}px`, top: `${inline ? -node.clientTop : bounds.top}px`, width: `${bounds.width}px`, height: `${bounds.height}px` });
    reveal();
  };
  const observer = new ResizeObserver(place);
  const motion: Motion = { glass, material, content: children, direction, radius, reveal, stop() {
    motion.driveStop?.();
    material.cancel(); children.forEach(item => item.animation?.cancel());
    observer.disconnect();
    document.removeEventListener('scroll', place, true);
    window.removeEventListener('resize', place);
  }, dispose() {
    motion.stop();
    if (inline) children.forEach(item => { if (item.clip) item.node.style.setProperty('clip-path', item.clip, item.clipPriority); else item.node.style.removeProperty('clip-path'); });
    glass.remove();
    if (motions.get(node) === motion) { motions.delete(node); node.classList.remove('liquid-morph-active', 'liquid-inline-active'); }
  } };
  motions.set(node, motion);
  reveal();
  observer.observe(node);
  document.addEventListener('scroll', place, true);
  window.addEventListener('resize', place);
  return motion;
}

/** Lifecycle and persistent presence. No wrapping/reparenting of application content. */
export function liquidSurface(node: HTMLElement, initial: LiquidOptions = {}) {
  let current = initial, generation = 0, disposed = false;
  options.set(node, current);
  node.classList.toggle('liquid-surface', current.enabled !== false);
  const media = ['(prefers-reduced-motion: reduce)', '(prefers-contrast: more)', '(prefers-reduced-transparency: reduce)'].map(query => matchMedia(query));
  const clearExit = () => { if (!node.hasAttribute('data-liquid-exiting')) return; node.removeAttribute('data-liquid-exiting'); node.removeAttribute('aria-hidden'); node.inert = false; };
  const preferenceChanged = () => {
    const motion = motions.get(node);
    if (motion?.direction === 'out' && media[0].matches && current.visible === undefined) {
      visibility.set(node, node.style.visibility); node.style.visibility = 'hidden';
    }
    motion?.dispose();
    if (current.visible === false) clearExit();
  };
  media.forEach(query => query.addEventListener('change', preferenceChanged));
  function presence(visible: boolean) {
    const token = ++generation;
    if (!visible) {
      node.setAttribute('data-liquid-exiting', '');
      node.setAttribute('aria-hidden', 'true'); node.inert = true;
    } else clearExit();
    void tick().then(() => {
      if (disposed || token !== generation) return;
      const motion = begin(node, visible ? 'in' : 'out');
      if (!motion) { if (!visible) clearExit(); return; }
      // Use Svelte's finite clock for persistent panels too, so clipping and
      // material share the same progress without a separate perpetual renderer.
      const duration = visible ? LIQUID_OPEN_MS : LIQUID_CLOSE_MS;
      const clock = tweened(0, { duration });
      let value = 0, stopped = false;
      const unsubscribe = clock.subscribe(progress => {
        value = progress;
        motion.material.currentTime = progress * duration;
        motion.content.forEach(item => { if (item.animation) item.animation.currentTime = progress * duration; });
        motion.reveal();
      });
      motion.driveStop = () => { stopped = true; unsubscribe(); void clock.set(value, { duration: 0 }); };
      void clock.set(1).then(() => {
        if (stopped || token !== generation) return;
        motion.dispose(); if (!visible) clearExit();
      });
    });
  }
  if (current.visible === true && current.enabled !== false) presence(true);
  return {
    update(next: LiquidOptions) {
      const before = current; current = next; options.set(node, next);
      node.classList.toggle('liquid-surface', next.enabled !== false);
      if (next.enabled === false) { generation++; motions.get(node)?.dispose(); clearExit(); restoreVisibility(node); }
      else if (next.visible !== undefined && (next.visible !== before.visible || before.enabled === false)) presence(next.visible);
      else if (before.enabled === false) presence(true);
    },
    destroy() { disposed = true; generation++; motions.get(node)?.dispose(); media.forEach(query => query.removeEventListener('change', preferenceChanged)); options.delete(node); restoreVisibility(node); },
  };
}

/** Deferred so onMount/tick can position a menu or call dialog.showModal(). */
export function liquidTransition(node: HTMLElement, _params: unknown, _context: { direction: 'in' | 'out' | 'both' }) {
  return ({ direction = 'in' }: { direction?: 'in' | 'out' } = {}): TransitionConfig => {
    const motion = begin(node, direction);
    if (!motion) return { duration: 0 };
    const duration = direction === 'in' ? LIQUID_OPEN_MS : LIQUID_CLOSE_MS;
    return { duration, tick(t) {
      if (motions.get(node) !== motion) return;
      const progress = direction === 'in' ? t : 1 - t;
      motion.material.currentTime = progress * duration;
      motion.content.forEach(item => { if (item.animation) item.animation.currentTime = progress * duration; });
      motion.reveal();
      if ((direction === 'in' && t === 1) || (direction === 'out' && t === 0)) motion.dispose();
    } };
  };
}
