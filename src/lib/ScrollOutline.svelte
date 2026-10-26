<script lang="ts">
  import { tick } from 'svelte';
  type Entry = { node: HTMLElement; label: string; level: number; top: number };
  let { root, mode = 'note', active = true, disabled = false }: {
    root?: HTMLElement; mode?: 'home' | 'note'; active?: boolean; disabled?: boolean;
  } = $props();
  let entries = $state.raw<Entry[]>([]);
  let current = $state(0);
  let rail = $state<HTMLElement>();
  let position = $state({ top: 120, right: 12, height: 360, width: 500 });

  function revealCurrent() {
    const button = rail?.querySelector<HTMLButtonElement>('[aria-current="location"]');
    if (!rail || !button) return;
    if (button.offsetTop < rail.scrollTop) rail.scrollTop = button.offsetTop;
    else if (button.offsetTop + button.offsetHeight > rail.scrollTop + rail.clientHeight) rail.scrollTop = button.offsetTop + button.offsetHeight - rail.clientHeight;
  }
  $effect(() => {
    const scroller = root, enabled = active, scope = mode;
    if (!scroller || !enabled) return;
    let frame = 0, dirty = true, disposed = false;
    const observed = new Set<Element>();
    function updateCurrent() {
      if (!entries.length) { current = 0; return; }
      const threshold = scroller!.scrollTop + Math.min(110, scroller!.clientHeight * .18);
      let lo = 0, hi = entries.length;
      while (lo < hi) { const mid = (lo + hi) >>> 1; if (entries[mid].top <= threshold) lo = mid + 1; else hi = mid; }
      const maximum = scroller!.scrollHeight - scroller!.clientHeight;
      const next = maximum > 1 && maximum - scroller!.scrollTop <= 2 ? entries.length - 1 : Math.max(0, lo - 1);
      if (current !== next) { current = next; void tick().then(() => { if (!disposed) revealCurrent(); }); }
    }
    function collect() {
      if (!scroller!.isConnected || !scroller!.clientHeight) { entries = []; return; }
      const rootRect = scroller!.getBoundingClientRect();
      const selector = scope === 'home' ? '[data-home-section] > header > h2'
        : '.page-title-row > .title-input, .page-title-row > h1, .note-document .noteka-document :is(h1[data-id],h2[data-id],h3[data-id])';
      const next = [...scroller!.querySelectorAll<HTMLElement>(selector)].filter(node => node.getClientRects().length).map(node => ({
        node,
        label: (node instanceof HTMLTextAreaElement || node instanceof HTMLInputElement ? node.value : node.textContent)?.trim() || (node.matches('.title-input') ? 'Untitled' : 'Untitled heading'),
        level: scope === 'home' || node.closest('.page-title-row') ? 0 : Number(node.tagName.slice(1)),
        top: node.getBoundingClientRect().top - rootRect.top - scroller!.clientTop + scroller!.scrollTop,
      }));
      if (next.length !== entries.length || next.some((entry, index) => { const old = entries[index]; return old.node !== entry.node || old.label !== entry.label || old.level !== entry.level || Math.abs(old.top - entry.top) > .5; })) entries = next;
      const container = scroller!.closest<HTMLElement>('.workspace-pane') ?? scroller!.parentElement;
      const containerRect = container?.getBoundingClientRect() ?? rootRect;
      const offset = Math.min(140, Math.max(64, scroller!.clientHeight * .18));
      position = { top: rootRect.top - containerRect.top + offset, right: Math.max(8, containerRect.right - rootRect.right + scroller!.offsetWidth - scroller!.clientWidth + 8), height: Math.max(24, Math.min(scroller!.clientHeight * .55, scroller!.clientHeight - offset - 20)), width: scroller!.clientWidth };
      for (const node of observed) if (!node.isConnected || !scroller!.contains(node)) { resize.unobserve(node); observed.delete(node); }
      for (const node of scroller!.querySelectorAll(scope === 'home' ? '.home-content' : '.page')) if (!observed.has(node)) { observed.add(node); resize.observe(node); }
    }
    function run() {
      frame = 0;
      if (disposed) return;
      if (dirty) { dirty = false; collect(); }
      updateCurrent();
    }
    function schedule() { if (!disposed && !frame) frame = requestAnimationFrame(run); }
    function changed() { dirty = true; schedule(); }
    const resize = new ResizeObserver(changed);
    const mutations = new MutationObserver(changed);
    resize.observe(scroller);
    mutations.observe(scroller, { childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ['hidden', 'open'] });
    scroller.addEventListener('scroll', schedule, { passive: true });
    scroller.addEventListener('input', changed);
    window.addEventListener('resize', changed);
    changed();
    return () => { disposed = true; cancelAnimationFrame(frame); resize.disconnect(); mutations.disconnect(); scroller.removeEventListener('scroll', schedule); scroller.removeEventListener('input', changed); window.removeEventListener('resize', changed); };
  });
  function goTo(entry: Entry) {
    if (!root || disabled || !entry.node.isConnected) return;
    const top = entry.node.getBoundingClientRect().top - root.getBoundingClientRect().top - root.clientTop + root.scrollTop - 18;
    root.scrollTo({ top: Math.max(0, top), behavior: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'instant' : 'smooth' });
  }
  function keydown(event: KeyboardEvent) {
    if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key) || !rail) return;
    const buttons = [...rail.querySelectorAll<HTMLButtonElement>('button')];
    const index = buttons.indexOf(document.activeElement as HTMLButtonElement);
    if (index < 0) return;
    event.preventDefault();
    buttons[event.key === 'Home' ? 0 : event.key === 'End' ? buttons.length - 1 : (index + (event.key === 'ArrowDown' ? 1 : -1) + buttons.length) % buttons.length]?.focus();
  }
</script>

{#if active && entries.length}
  <nav bind:this={rail} class="scroll-outline" aria-label={mode === 'home' ? 'Home sections' : 'Note outline'} style:top={`${position.top}px`} style:right={`${position.right}px`} style:max-height={`${position.height}px`}>
    {#each entries as entry, index (entry.node)}
      <button type="button" class:current={index === current} title={entry.label} aria-label={`Go to ${entry.label}`} aria-current={index === current ? 'location' : undefined} {disabled} style:--marker-width={`${18 - entry.level * 2}px`} onkeydown={keydown} onmousedown={event => event.preventDefault()} onclick={() => goTo(entry)}>
        <span class="outline-mark" aria-hidden="true"></span>
      </button>
    {/each}
  </nav>
{/if}
