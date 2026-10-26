import { play } from '@animateicons/vanilla';
import { SIDEBAR_ICONS } from './sidebar-icons.generated';
let activeStop: (() => void) | undefined;
export function startSidebarAnimation(svg: SVGSVGElement): () => void {
  activeStop?.();
  const data = SIDEBAR_ICONS[svg.dataset.appIcon ?? ''];
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  if (!data || reduced.matches || document.hidden) return () => {};
  // Restore the exact resting tree even for animations whose last frame differs.
  const original = [svg, ...svg.querySelectorAll<SVGElement>('*')].map(node => ({
    node, attrs: [...node.attributes].map(attribute => [attribute.name, attribute.value] as const),
  }));
  const player = play(svg, data, { duration: 0.65 });
  let stopped = false;
  let timer: ReturnType<typeof setTimeout>;
  function restore() {
    if (stopped) return;
    stopped = true; clearTimeout(timer); player.destroy();
    for (const {node, attrs} of original) {
      for (const attr of [...node.attributes]) node.removeAttribute(attr.name);
      for (const [name,value] of attrs) node.setAttribute(name,value);
    }
    reduced.removeEventListener('change', preferenceChanged);
    document.removeEventListener('visibilitychange', visibilityChanged);
    if (activeStop === restore) activeStop = undefined;
  }
  function preferenceChanged() { if (reduced.matches) restore(); }
  function visibilityChanged() { if (document.hidden) restore(); }
  activeStop = restore;
  reduced.addEventListener('change', preferenceChanged);
  document.addEventListener('visibilitychange', visibilityChanged);
  timer = setTimeout(restore, 800);
  try { svg.dataset.iconPlaying = 'true'; player.start(); } catch { restore(); }
  return restore;
}
