// The action is attached exclusively to sidebar buttons; rendering AppIcon is static.
let latestClick = 0;
let playerModule: Promise<typeof import('./sidebar-icon-animation')> | undefined;
export function sidebarIconMotion(node: HTMLButtonElement) {
  let disposed = false;
  let clickId = 0;
  let stop: (() => void) | undefined;
  async function click() {
    clickId = ++latestClick;
    const currentClick = clickId;
    stop?.(); stop = undefined;
    const svg = node.querySelector<SVGSVGElement>('svg[data-app-icon]');
    if (!svg || node.disabled || document.hidden || matchMedia('(prefers-reduced-motion: reduce)').matches) return;
    try {
      const module = await (playerModule ??= import('./sidebar-icon-animation').catch(error => { playerModule = undefined; throw error; }));
      if (disposed || currentClick !== latestClick || !node.isConnected || !svg.isConnected
        || document.hidden || matchMedia('(prefers-reduced-motion: reduce)').matches) return;
      stop = module.startSidebarAnimation(svg);
    } catch (error) {
      // Cosmetic failure must preserve navigation while remaining diagnosable.
      console.warn("[Noteka] Could not load the sidebar animation.", error);
    }
  }
  node.addEventListener('click', click);
  return { destroy() { disposed = true; if (clickId === latestClick) latestClick++; node.removeEventListener('click', click); stop?.(); } };
}
