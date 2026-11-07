<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
  import AppIcon from './AppIcon.svelte';
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
  let ready = $state(false);
  let position = $state({ left: -9999, top: -9999, width: 292 });

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

  function positionPanel() {
    if (!anchor.isConnected) { onclose(false); return; }
    const rect = anchor.getBoundingClientRect();
    if (rect.bottom < 0 || rect.top > innerHeight) { onclose(false); return; }
    const width = Math.min(292, Math.max(240, innerWidth - 24));
    const height = panel?.offsetHeight || 300;
    const below = rect.bottom + 8 + height <= innerHeight - 10;
    const top = below ? rect.bottom + 8 : Math.max(10, rect.top - height - 8);
    const left = Math.max(10, Math.min(rect.right - width, innerWidth - width - 10));
    position = { left, top, width };
    ready = true;
  }
  function focusDay(day: string) {
    void tick().then(() => panel?.querySelector<HTMLButtonElement>(`[data-date="${day}"]`)?.focus());
  }
  function navigate(delta: number) {
    const next = shiftMonth(view, delta);
    if (!next) return;
    view = next;
    focusDate = clampDay(next, Number(focusDate.slice(8)));
    void tick().then(() => positionPanel());
  }
  function jumpTo(day: string) {
    if (!day) return;
    focusDate = day;
    if (day.slice(0, 7) !== view) view = day.slice(0, 7);
    void tick().then(() => { positionPanel(); focusDay(day); });
  }
  function focusOut(event: FocusEvent) {
    const next = event.relatedTarget as Node | null;
    if (!next) return;
    if (!panel?.contains(next) && next !== anchor) onclose(false);
  }
  function keys(event: KeyboardEvent) {
    if (event.isComposing) return;
    if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); onclose(true); return; }
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

  onMount(() => {
    positionPanel();
    if (focusOnOpen) focusDay(selected || today);
    const scroll = (event: Event) => { if (!(event.target instanceof Node && panel?.contains(event.target))) positionPanel(); };
    const outside = (event: PointerEvent) => {
      if (panel && !panel.contains(event.target as Node) && !anchor.contains(event.target as Node)) onclose(false);
    };
    const resize = () => positionPanel();
    document.addEventListener('scroll', scroll, true);
    document.addEventListener('pointerdown', outside, true);
    window.addEventListener('resize', resize);
    return () => {
      document.removeEventListener('scroll', scroll, true);
      document.removeEventListener('pointerdown', outside, true);
      window.removeEventListener('resize', resize);
    };
  });
</script>

<div bind:this={panel} class="date-picker glass-menu" class:ready role="dialog" tabindex="-1" aria-label="Choose due date"
  style:left={`${position.left}px`} style:top={`${position.top}px`} style:width={`${position.width}px`} onkeydown={keys} onfocusout={focusOut}>
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
                    onclick={() => onpick(day)} onfocus={() => focusDate = day}>{Number(day.slice(8))}</button>
                {/if}
              </div>
            {/each}
          </div>
        {/each}
      </div>
    {/key}
  </div>
  <div class="date-picker-footer">
    <button type="button" onclick={() => onpick(today)}>Today</button>
    <button type="button" disabled={!selected} onclick={() => onpick('')}>Clear</button>
  </div>
</div>
