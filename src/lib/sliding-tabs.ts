let jellyModule: Promise<typeof import('./sidebar-icon-animation')> | undefined;

export function slidingTabs(node: HTMLElement) {
  const host = node.closest<HTMLElement>('.task-view-toolbar') ?? node;
  const indicator = document.createElement('span');
  indicator.className = 'sliding-tabs-indicator';
  indicator.setAttribute('aria-hidden', 'true');
  host.prepend(indicator);
  node.classList.add('sliding-tabs');
  host.classList.add('sliding-tabs-host');

  let placed = false;
  let frame = 0;
  const activeTab = () => [...node.children].find(child =>
    child instanceof HTMLElement && child.matches('button.active, button[aria-pressed="true"]')
  ) as HTMLElement | undefined;

  const place = (animate: boolean) => {
    const target = activeTab();
    if (!target) { indicator.style.opacity = '0'; return; }
    const bounds = node.getBoundingClientRect();
    const hostRect = host.getBoundingClientRect();
    const targetRect = target.getBoundingClientRect();
    const x = targetRect.left - hostRect.left - host.clientLeft;
    const y = targetRect.top - hostRect.top - host.clientTop;
    const width = targetRect.width, height = targetRect.height;
    const visible = targetRect.right > bounds.left && targetRect.left < bounds.right;
    const skipTransition = !animate || !placed;
    if (skipTransition) indicator.style.transition = 'none';
    indicator.style.opacity = visible ? '1' : '0';
    indicator.style.transform = `translate(${x}px,${y}px)`;
    indicator.style.width = `${width}px`;
    indicator.style.height = `${height}px`;
    placed = true;
    if (skipTransition) requestAnimationFrame(() => { indicator.style.transition = ''; });
  };

  const schedule = (animate: boolean) => {
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(() => place(animate));
  };

  const mutations = new MutationObserver(() => schedule(true));
  mutations.observe(node, { subtree: true, childList: true, attributes: true, attributeFilter: ['class', 'aria-pressed'] });
  const resize = new ResizeObserver(() => schedule(false));
  resize.observe(node);
  resize.observe(host);
  const onScroll = () => schedule(false);
  node.addEventListener('scroll', onScroll, { passive: true });
  const playJelly = (event: Event) => {
    const target = event.target;
    if (!(target instanceof Element)) return;
    const button = target.closest('button');
    if (!button || !node.contains(button) || button.disabled || document.hidden || matchMedia('(prefers-reduced-motion: reduce)').matches) return;
    const svg = button.querySelector<SVGSVGElement>('svg[data-app-icon]');
    if (!svg) return;
    void (jellyModule ??= import('./sidebar-icon-animation'))
      .then(module => module.startGenericJelly(svg))
      .catch(error => console.warn('[Noteka] Could not load the tab icon animation.', error));
  };
  node.addEventListener('click', playJelly);
  const onWindowResize = () => schedule(false);
  window.addEventListener('resize', onWindowResize);
  schedule(false);

  return {
    destroy() {
      cancelAnimationFrame(frame);
      mutations.disconnect();
      resize.disconnect();
      node.removeEventListener('scroll', onScroll);
      node.removeEventListener('click', playJelly);
      window.removeEventListener('resize', onWindowResize);
      indicator.remove();
      node.classList.remove('sliding-tabs');
      host.classList.remove('sliding-tabs-host');
    }
  };
}
