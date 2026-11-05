type FocusAnchor = { element: HTMLElement; section: HTMLElement; label: string | null };

export function captureRefreshFocus(root?: HTMLElement): FocusAnchor | null {
  const element = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const section = element?.closest<HTMLElement>('[data-home-section]');
  if (!element || !section || !root?.contains(section)) return null;
  return { element, section, label: element.getAttribute('aria-label') ?? element.textContent };
}

export function restoreRefreshFocus(anchor: FocusAnchor | null) {
  if (!anchor || !anchor.section.isConnected || document.activeElement !== document.body) return;
  if (anchor.element.isConnected && !anchor.element.matches(':disabled')) { anchor.element.focus({ preventScroll: true }); return; }
  const replacement = [...anchor.section.querySelectorAll<HTMLButtonElement>('button')].find(button => (button.getAttribute('aria-label') ?? button.textContent) === anchor.label && !button.disabled)
    ?? anchor.section.querySelector<HTMLButtonElement>('.home-tabs [aria-pressed="true"]');
  replacement?.focus({ preventScroll: true });
}
