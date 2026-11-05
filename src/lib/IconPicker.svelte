<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import { ICON_COLORS, ICON_OPTIONS, iconColor } from './icon-set';
  import { visualIconName } from './visual-icon';
  let {
    icon,
    color = null,
    size = 38,
    disabled = false,
    onchange,
  }: {
    icon?: string | null;
    color?: string | null;
    size?: number;
    disabled?: boolean;
    onchange: (value: { icon: string; color: string | null }) => void;
  } = $props();
  let open = $state(false);
  let root: HTMLDivElement;
  const current = $derived(visualIconName(icon));
  const currentColor = $derived(iconColor(color) ?? null);
  function choose(name: string) {
    if (disabled) return;
    onchange({ icon: name, color: currentColor });
  }
  function paint(value: string | null) {
    if (disabled) return;
    onchange({ icon: current, color: value });
  }
  $effect(() => { if (disabled) open = false; });
</script>

<svelte:window onkeydown={(event) => { if (event.key === 'Escape' && open) { event.preventDefault(); open = false; } }} onclick={(event) => { if (open && root && event.target instanceof Node && !root.contains(event.target)) open = false; }} />

<div class="icon-picker" bind:this={root}>
  <button
    type="button"
    class="icon-trigger"
    aria-label="Choose icon and color"
    aria-expanded={open}
    {disabled}
    onclick={() => open = !open}
    style:width={`${size + 14}px`} style:height={`${size + 14}px`}
  ><AppIcon name={current} color={currentColor ?? undefined} {size} /></button>
  {#if open}
    <div class="icon-panel" role="dialog" aria-label="Page icon">
      <p class="icon-caption">Icon</p>
      <div class="icon-grid">
        {#each ICON_OPTIONS as option}
          <button
            type="button"
            class:selected={current === option.name}
            aria-label={option.label}
            aria-pressed={current === option.name}
            title={option.label}
            onclick={() => choose(option.name)}
          ><AppIcon name={option.name} size={18} /></button>
        {/each}
      </div>
      <p class="icon-caption">Color</p>
      <div class="color-grid">
        {#each ICON_COLORS as option}
          <button
            type="button"
            class:selected={currentColor === option.value}
            class:default-color={option.value === null}
            aria-label={option.label}
            aria-pressed={currentColor === option.value}
            title={option.label}
            style:background={option.value ?? 'transparent'}
            onclick={() => paint(option.value)}
          ></button>
        {/each}
      </div>
      <div class="icon-actions">
        <button type="button" onclick={() => { paint(null); open = false; }}>Restablecer color</button>
        <button type="button" class="primary" onclick={() => open = false}>Done</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .icon-picker { position: relative; flex: 0 0 auto; }
  .icon-trigger { display: grid; place-items: center; width: 58px; height: 58px; padding: 0; border: 1px solid transparent; border-radius: 10px; background: transparent; color: var(--text); }
  .icon-trigger:hover { background: var(--hover); border-color: var(--border); }
  .icon-trigger:disabled { opacity:1; cursor:default; background:transparent; border-color:transparent; }
  .icon-panel { position: absolute; z-index: 30; top: calc(100% + 6px); left: 0; width: 292px; padding: 12px; background: var(--surface); border: 1px solid var(--border); border-radius: 9px; box-shadow: 0 14px 35px #0005; }
  .icon-caption { margin: 0 0 8px; color: var(--muted); font-size: 10px; letter-spacing: .08em; text-transform: uppercase; }
  .icon-grid { display: grid; grid-template-columns: repeat(8, 1fr); gap: 3px; margin-bottom: 15px; }
  .icon-grid button { display: grid; place-items: center; width: 31px; height: 31px; padding: 0; border: 1px solid transparent; background: transparent; color: var(--text); }
  .icon-grid button:hover, .icon-grid button.selected { background: var(--selected); border-color: var(--border); }
  .color-grid { display: flex; gap: 7px; flex-wrap: wrap; margin-bottom: 14px; }
  .color-grid button { width: 25px; height: 25px; padding: 0; border: 2px solid var(--border); border-radius: 50%; }
  .color-grid button.selected { outline: 2px solid var(--focus); outline-offset: 2px; }
  .color-grid button.default-color { position: relative; }
  .color-grid button.default-color::after { content: ""; position: absolute; inset: 4px; border-top: 2px solid var(--error); transform: rotate(-45deg); }
  .icon-actions { display: flex; justify-content: space-between; gap: 8px; }
  .icon-actions button { font-size: 11px; padding: 6px 8px; }
</style>
