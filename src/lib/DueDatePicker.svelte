<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
  import AppIcon from './AppIcon.svelte';
  import { LIQUID_OPEN_MS as OPEN_MS, LIQUID_CLOSE_MS as CLOSE_MS, liquidMaterialFrames, liquidContentFrames } from './liquid-motion';
  import { calendarDays, civilToday, shiftDay, shiftMonth, validCivilDate } from './project-task-calendar';

  let { anchor, value, timezone = 'UTC', focusOnOpen = false, onpick, onclose }: {
    anchor: HTMLElement;
    value: string;
    timezone?: string;
    focusOnOpen?: boolean;
    onpick: (value: string) => void;
    onclose: (focus: boolean) => void;
  } = $props();

  const today = untrack(() => civilToday(timezone));
  const selected = $derived(validCivilDate(value) ? value : '');
  let view = $state(untrack(() => (selected || today).slice(0, 7)));
  let focusDate = $state(untrack(() => selected || today));
  let panel: HTMLDivElement;
  let material: HTMLDivElement;
  let content: HTMLDivElement;
  let materialAnimation: Animation | undefined;
  let contentAnimation: Animation | undefined;
  let ready = $state(false);
  let closing = $state(false);
  let closeFocus = false;
  let closeTimer: ReturnType<typeof setTimeout> | undefined;
  let position = $state({ left: -9999, top: -9999, width: 292, below: true });

  const weekdays = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
  const dayFormat = new Intl.DateTimeFormat('en-US', { month: 'long', day: 'numeric', year: 'numeric', timeZone: 'UTC' });
  const monthFormat = new Intl.DateTimeFormat('en-US', { month: 'long', year: 'numeric', timeZone: 'UTC' });
  const monthLabel = $derived(formatMonth(view));
  const days = $derived(calendarDays(view));
  const rows = $derived(Array.from({ length: Math.ceil(days.length / 7) }, (_, index) => days.slice(index * 7, index * 7 + 7)));

  function formatMonth(month: string) {
    return monthFormat.format(new Date(`${month}-01T12:00:00Z`));
  }
  function formatDay(day: string) {
    return dayFormat.format(new Date(`${day}T12:00:00Z`));
  }
  function daysIn(month: string) {
    const [year, monthNumber] = month.split('-').map(Number);
    return new Date(Date.UTC(year, monthNumber, 0)).getUTCDate();
  }
  function clampDay(month: string, day: number) {
    return `${month}-${String(Math.min(day, daysIn(month))).padStart(2, '0')}`;
  }
  function weekOffset(day: string) {
    return (new Date(`${day}T12:00:00Z`).getUTCDay() + 6) % 7;
  }
  function prefersReducedMotion() {
    return typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;
  }

  function positionPanel() {
    if (closing) return;
    if (!anchor.isConnected) { onclose(false); return; }
    const rect = anchor.getBoundingClientRect();
    if (rect.bottom < 0 || rect.top > innerHeight) { onclose(false); return; }
    const width = Math.min(292, Math.max(240, innerWidth - 24));
    const height = panel?.offsetHeight || 300;
    const below = rect.bottom + 8 + height <= innerHeight - 10;
    const top = below ? rect.bottom + 8 : Math.max(10, rect.top - height - 8);
    const left = Math.max(10, Math.min(rect.right - width, innerWidth - width - 10));
    position = { left, top, width, below };
    ready = true;
  }
  // Only the glass changes shape. Text lives in a sibling layer and never
  // inherits the highly nonuniform scale of the source capsule.
  function sourceFrame(): Keyframe {
    const source = anchor.getBoundingClientRect();
    const sx = Math.max(1, source.width) / panel.offsetWidth;
    const sy = Math.max(1, source.height) / panel.offsetHeight;
    const radius = Math.min(parseFloat(getComputedStyle(anchor).borderTopLeftRadius) || 16, source.height / 2);
    return {
      transform: `translate(${source.left - position.left}px, ${source.top - position.top}px) scale(${sx}, ${sy})`,
      borderRadius: `${radius / sx}px / ${radius / sy}px`,
    };
  }
  function cancelMotion() {
    materialAnimation?.cancel();
    contentAnimation?.cancel();
    materialAnimation = contentAnimation = undefined;
  }
  function arrive() {
    if (closing || !panel?.isConnected || prefersReducedMotion() || typeof material.animate !== 'function') return;
    const drift = position.below ? -3 : 3;
    materialAnimation = material.animate(liquidMaterialFrames(sourceFrame(), '14px', drift), { duration: OPEN_MS, fill: 'both' });
    materialAnimation.id = 'date-picker-arrive';
    contentAnimation = content.animate(liquidContentFrames(drift), { duration: OPEN_MS, fill: 'both' });
    // Remove finished effects so a month change or resize uses natural layout.
    void materialAnimation.finished.then(() => { if (!closing) cancelMotion(); }).catch(() => {});
  }
  function focusDay(day: string) {
    void tick().then(() => panel?.querySelector<HTMLButtonElement>(`[data-date="${day}"]`)?.focus());
  }
  function navigate(delta: number) {
    if (closing) return;
    const next = shiftMonth(view, delta);
    if (!next) return;
    view = next;
    focusDate = clampDay(next, Number(focusDate.slice(8)));
    void tick().then(() => positionPanel());
  }
  function jumpTo(day: string) {
    if (closing || !day) return;
    focusDate = day;
    if (day.slice(0, 7) !== view) view = day.slice(0, 7);
    void tick().then(() => { positionPanel(); focusDay(day); });
  }
  function focusOut(event: FocusEvent) {
    if (closing) return;
    const next = event.relatedTarget as Node | null;
    if (!next) return;
    if (!panel?.contains(next) && next !== anchor) requestClose(false);
  }
  function keys(event: KeyboardEvent) {
    if (closing || event.isComposing) return;
    const from = (document.activeElement as HTMLElement | null)?.closest<HTMLElement>('[data-date]')?.dataset.date;
    if (!from) return;
    let next = '';
    switch (event.key) {
      case 'ArrowLeft': next = shiftDay(from, -1) || ''; break;
      case 'ArrowRight': next = shiftDay(from, 1) || ''; break;
      case 'ArrowUp': next = shiftDay(from, -7) || ''; break;
      case 'ArrowDown': next = shiftDay(from, 7) || ''; break;
      case 'Home': next = shiftDay(from, -weekOffset(from)) || ''; break;
      case 'End': next = shiftDay(from, 6 - weekOffset(from)) || ''; break;
      case 'PageUp': { const month = shiftMonth(from.slice(0, 7), -1); next = month ? clampDay(month, Number(from.slice(8))) : ''; break; }
      case 'PageDown': { const month = shiftMonth(from.slice(0, 7), 1); next = month ? clampDay(month, Number(from.slice(8))) : ''; break; }
      default: return;
    }
    if (!next) return;
    event.preventDefault();
    jumpTo(next);
  }
  function requestClose(focus: boolean) {
    if (closing) return;
    if (prefersReducedMotion() || typeof material?.animate !== 'function' || !anchor.isConnected) { onclose(focus); return; }
    // Read the rendered frame before cancelling an unfinished opening.
    const glassStyle = getComputedStyle(material);
    const fromGlass = { transform: glassStyle.transform, borderRadius: glassStyle.borderRadius };
    const contentStyle = getComputedStyle(content);
    const fromContent = { opacity: contentStyle.opacity, transform: contentStyle.transform };
    const destination = sourceFrame();
    cancelMotion();
    closing = true;
    closeFocus = focus;
    materialAnimation = material.animate([fromGlass, destination], {
      duration: CLOSE_MS, easing: 'cubic-bezier(.4,0,.25,1)', fill: 'both',
    });
    materialAnimation.id = 'date-picker-dismiss';
    contentAnimation = content.animate([fromContent, { opacity: 0, transform: `translateY(${position.below ? -4 : 4}px)` }], {
      duration: CLOSE_MS * .5, easing: 'ease-out', fill: 'both',
    });
    void materialAnimation.finished.then(finishClose).catch(() => {});
    closeTimer = setTimeout(finishClose, CLOSE_MS + 80);
  }
  function finishClose() {
    if (!closing) return;
    if (closeTimer) { clearTimeout(closeTimer); closeTimer = undefined; }
    onclose(closeFocus);
  }
  function pick(next: string) {
    if (closing) return;
    onpick(next);
    requestClose(true);
  }
  onMount(() => {
    positionPanel();
    void tick().then(arrive);
    if (focusOnOpen) focusDay(selected || today);
    else anchor.focus({ preventScroll: true }); // WebKit doesn't focus buttons on pointer activation.
    const scroll = (event: Event) => { if (!(event.target instanceof Node && panel?.contains(event.target))) positionPanel(); };
    const outside = (event: PointerEvent) => {
      if (closing) return;
      if (panel && !panel.contains(event.target as Node) && !anchor.contains(event.target as Node)) requestClose(false);
    };
    const anchorClick = (event: MouseEvent) => {
      if (closing || !anchor.contains(event.target as Node)) return;
      event.preventDefault();
      event.stopPropagation();
      requestClose(false);
    };
    const escape = (event: KeyboardEvent) => {
      if (event.key !== 'Escape' || event.isComposing) return;
      const target = event.target as Node | null;
      if (!target || (!anchor.contains(target) && !panel?.contains(target))) return;
      event.preventDefault();
      event.stopPropagation();
      requestClose(true);
    };
    const resize = () => positionPanel();
    const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
    const motionChanged = () => {
      if (!reducedMotion.matches) return;
      cancelMotion();
      if (closing) finishClose();
    };
    reducedMotion.addEventListener('change', motionChanged);
    document.addEventListener('scroll', scroll, true);
    document.addEventListener('pointerdown', outside, true);
    document.addEventListener('click', anchorClick, true);
    document.addEventListener('keydown', escape, true);
    window.addEventListener('resize', resize);
    return () => {
      if (closeTimer) clearTimeout(closeTimer);
      cancelMotion();
      reducedMotion.removeEventListener('change', motionChanged);
      document.removeEventListener('scroll', scroll, true);
      document.removeEventListener('pointerdown', outside, true);
      document.removeEventListener('click', anchorClick, true);
      document.removeEventListener('keydown', escape, true);
      window.removeEventListener('resize', resize);
    };
  });
</script>

<div bind:this={panel} class="date-picker" class:ready class:closing role="dialog" tabindex="-1" aria-label="Choose due date" inert={closing}
  style={`left:${position.left}px;top:${position.top}px;width:${position.width}px`}
  onkeydown={keys} onfocusout={focusOut}>
  <div bind:this={material} class="date-picker-material glass-menu" aria-hidden="true"><span class="date-picker-sheen"></span></div>
  <div bind:this={content} class="date-picker-content">
  <div class="date-picker-header">
    <button type="button" class="date-picker-nav" aria-label="Previous month" onclick={() => navigate(-1)}><AppIcon name="arrow-left" size={15} /></button>
    <span class="date-picker-title" aria-live="polite">{monthLabel}</span>
    <button type="button" class="date-picker-nav" aria-label="Next month" onclick={() => navigate(1)}><AppIcon name="arrow-right" size={15} /></button>
  </div>
  <div class="date-picker-weekdays" aria-hidden="true">{#each weekdays as weekday}<span>{weekday}</span>{/each}</div>
  <div class="date-picker-calendar">
    {#key view}
      <div class="date-picker-grid" role="grid" aria-label={monthLabel}>
        {#each rows as row, rowIndex (rowIndex)}
          <div class="date-picker-row" role="row">
            {#each row as day, cellIndex (cellIndex)}
              <div class="date-picker-cell" role="gridcell" aria-selected={!!day && day === selected} aria-disabled={!day}>
                {#if day}
                  <button type="button" class="date-picker-day" data-date={day} data-selected={day === selected} tabindex={day === focusDate ? 0 : -1}
                    aria-label={formatDay(day)} aria-current={day === today ? 'date' : undefined}
                    onclick={() => pick(day)} onfocus={() => focusDate = day}>{Number(day.slice(8))}</button>
                {/if}
              </div>
            {/each}
          </div>
        {/each}
      </div>
    {/key}
  </div>
  <div class="date-picker-footer">
    <button type="button" onclick={() => pick(today)}>Today</button>
    <button type="button" disabled={!selected} onclick={() => pick('')}>Clear</button>
  </div>
  </div>
</div>
