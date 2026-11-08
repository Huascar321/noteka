<script lang="ts">
  import { liquidSurface, liquidTransition } from './liquid-overlay';
  import { tick } from 'svelte';
  import AppIcon from './AppIcon.svelte';
  import { PEEK_MODES, type PeekMode, type PeekPresentation } from './peek-mode';
  let { mode = 'full', defaultMode, preferenceError = '', disabled = false, onchange, ondefault }: {
    mode?: PeekPresentation; defaultMode: PeekMode; preferenceError?: string; disabled?: boolean;
    onchange: (mode: PeekMode) => void; ondefault: (mode: PeekMode) => void;
  } = $props();
  const id = $props.id();
  let root: HTMLDivElement, trigger: HTMLButtonElement;
  let menu = $state<HTMLDivElement>();
  let open = $state(false), defaults = $state(false);
  let position = $state({ left: 12, top: 60, maxHeight: 400 });
  function place() {
    if (!open || !trigger) return;
    const rect = trigger.getBoundingClientRect();
    const height = Math.min(menu?.scrollHeight || 286, innerHeight - 24), width = Math.min(280, innerWidth - 24);
    const top = rect.bottom + 6 + height <= innerHeight - 12 ? rect.bottom + 6 : Math.max(12, rect.top - height - 6);
    position = { left: Math.max(12, Math.min(rect.left, innerWidth - width - 12)), top, maxHeight: innerHeight - top - 12 };
  }
  async function focusChoice() {
    await tick(); place();
    (menu?.querySelector<HTMLButtonElement>('[aria-checked="true"]') ?? menu?.querySelector<HTMLButtonElement>('button'))?.focus();
  }
  async function toggle() { open = !open; defaults = false; if (open) await focusChoice(); }
  function close(restore = true) { open = false; if (restore) trigger?.focus(); }
  function choose(next: PeekMode) { if (defaults) { ondefault(next); void focusChoice(); } else { close(); onchange(next); } }
  function keydown(event: KeyboardEvent) {
    if (!open) return;
    if (event.key === 'Escape') { event.preventDefault(); close(); return; }
    if (event.key === 'ArrowLeft' && defaults) { event.preventDefault(); defaults = false; void focusChoice(); return; }
    if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) return;
    event.preventDefault();
    const buttons = [...(menu?.querySelectorAll<HTMLButtonElement>('button') ?? [])], index = buttons.indexOf(document.activeElement as HTMLButtonElement);
    buttons[event.key === 'Home' ? 0 : event.key === 'End' ? buttons.length - 1 : (index + (event.key === 'ArrowDown' ? 1 : -1) + buttons.length) % buttons.length]?.focus();
  }
  $effect(() => { if (disabled) open = false; });
</script>

<svelte:window onresize={place} onpointerdown={event => { if (open && !root?.contains(event.target as Node)) close(false); }} />
<div class="peek-mode-control" bind:this={root}>
  <button bind:this={trigger} class="icon-button" type="button" aria-label="Switch peek mode" title="Switch peek mode" aria-haspopup="menu" aria-expanded={open} aria-controls={open ? id : undefined} {disabled} onclick={toggle} onkeydown={event => { if (event.key === 'ArrowDown') { event.preventDefault(); open = true; defaults = false; void focusChoice(); } }}><AppIcon name={PEEK_MODES.find(item => item.id === mode)?.icon ?? 'expand'} size={18} /></button>
  {#if open}
    <div bind:this={menu} id={id} class="peek-mode-menu glass-menu" use:liquidSurface={{ anchor: () => trigger }} in:liquidTransition|global out:liquidTransition|global role="menu" tabindex="-1" aria-label={defaults ? 'Default opening mode' : 'Peek mode'} onkeydown={keydown} style:left={`${position.left}px`} style:top={`${position.top}px`} style:max-height={`${position.maxHeight}px`}>
      {#if defaults}<button type="button" role="menuitem" tabindex="-1" class="peek-menu-back" onclick={() => { defaults = false; void focusChoice(); }}><AppIcon name="arrow-left" size={17} />Default opening mode</button><p class="peek-default-scope">All views · New openings</p>{/if}
      {#each PEEK_MODES as item}
        <button type="button" role={item.id === 'tab' && !defaults ? 'menuitem' : 'menuitemradio'} aria-checked={item.id === 'tab' && !defaults ? undefined : (defaults ? defaultMode : mode) === item.id} tabindex="-1" onclick={() => choose(item.id)}><AppIcon name={item.icon} size={19} /><span>{item.label}</span>{#if (defaults ? defaultMode : mode) === item.id}<AppIcon name="check" size={17} />{/if}</button>
      {/each}
      {#if !defaults}<div class="peek-menu-divider"></div><button type="button" role="menuitem" tabindex="-1" onclick={() => { defaults = true; void focusChoice(); }}><AppIcon name="sliders" size={19} /><span>Default opening mode</span><AppIcon name="chevron-right" size={16} /></button>{/if}
      {#if preferenceError}<p class="peek-preference-error" role="status">{preferenceError}</p>{/if}
    </div>
  {/if}
</div>
