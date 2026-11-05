import { SIDEBAR_ICONS } from './sidebar-icons.generated';
let activeStop: (() => void) | undefined;
let sequence = 0;

export function startSidebarAnimation(svg: SVGSVGElement): () => void {
  const data = SIDEBAR_ICONS[svg.dataset.appIcon ?? ''];
  if (!data) return () => {};
  return play(svg, data, false);
}

/** Same Jelly recipe, but wraps the current icon so any AppIcon can play it. */
export function startGenericJelly(svg: SVGSVGElement): () => void {
  return play(svg, SIDEBAR_ICONS.task, true);
}

function play(svg: SVGSVGElement, data: (typeof SIDEBAR_ICONS)[string], generic: boolean): () => void {
  activeStop?.();
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  if (reduced.matches || document.hidden || !svg.isConnected) return () => {};
  // Keep the actual Svelte-owned nodes, including its anchors, for exact restoration.
  const original = [...svg.childNodes];
  const originalClass = svg.getAttribute('class');
  const scope = `noteka-motion-${++sequence}`;
  const scoped = (value: string) => value.replaceAll('__NOTEKA_ICON_SCOPE__', scope);
  const motionClass = scoped(data.scope);
  const scene = document.createElementNS('http://www.w3.org/2000/svg', 'g');
  if (generic) {
    const group = document.createElementNS('http://www.w3.org/2000/svg', 'g');
    group.setAttribute('class', 'fx');
    group.append(...original);
    scene.append(group);
  } else {
    scene.innerHTML = scoped(data.body);
  }
  const style = document.createElementNS('http://www.w3.org/2000/svg', 'style');
  style.textContent = scoped(data.css);
  let stopped = false;
  let timer: ReturnType<typeof setTimeout>;
  function restore() {
    if (stopped) return;
    stopped = true; clearTimeout(timer);
    svg.replaceChildren(...original);
    if (originalClass === null) svg.removeAttribute('class');
    else svg.setAttribute('class', originalClass);
    delete svg.dataset.iconPlaying;
    reduced.removeEventListener('change', preferenceChanged);
    document.removeEventListener('visibilitychange', visibilityChanged);
    if (activeStop === restore) activeStop = undefined;
  }
  function preferenceChanged() { if (reduced.matches) restore(); }
  function visibilityChanged() { if (document.hidden) restore(); }
  activeStop = restore;
  reduced.addEventListener('change', preferenceChanged);
  document.addEventListener('visibilitychange', visibilityChanged);
  timer = setTimeout(restore, data.durationMs + 50);
  try {
    svg.classList.add(motionClass);
    svg.replaceChildren(style, ...scene.childNodes);
    svg.dataset.iconPlaying = 'true';
  } catch { restore(); }
  return restore;
}
