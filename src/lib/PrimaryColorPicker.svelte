<script lang="ts">
  import { untrack } from 'svelte';
  import AppIcon from './AppIcon.svelte';
  import { PRIMARY_COLORS, hexToHsl, hslToHex, normalizeColor } from './primary-color';

  let { color, error = '', onchange }: { color: string; error?: string; onchange: (color: string) => void } = $props();
  let customOpen = $state(untrack(() => !PRIMARY_COLORS.some(option => option.value === color)));
  let hsl = $state(untrack(() => hexToHsl(color)));
  let hexInput = $state(untrack(() => color));
  let hexError = $state('');
  let lastSent = $state('');
  let pointer = $state<number | null>(null);
  const marker = $derived({ x: 50 + 43 * Math.sin(hsl.h * Math.PI / 180), y: 50 - 43 * Math.cos(hsl.h * Math.PI / 180) });
  const hueColor = $derived(`hsl(${hsl.h} 100% 50%)`);

  $effect(() => {
    if (color !== lastSent) { hsl = hexToHsl(color); hexInput = color; hexError = ''; }
  });
  function choose(value: string) {
    lastSent = value; hexInput = value; hexError = ''; onchange(value);
  }
  function changeHsl(next: Partial<typeof hsl>) {
    hsl = { ...hsl, ...next }; choose(hslToHex(hsl));
  }
  function choosePreset(value: string) {
    hsl = hexToHsl(value); customOpen = false; choose(value);
  }
  function applyHex() {
    const value = normalizeColor(hexInput.trim());
    if (!value) { hexError = 'Enter six hexadecimal digits, for example #3183A6.'; return; }
    hsl = hexToHsl(value); choose(value);
  }
  function point(event: PointerEvent) {
    const bounds = event.currentTarget instanceof HTMLElement ? event.currentTarget.getBoundingClientRect() : null;
    if (!bounds) return;
    const x = event.clientX - bounds.left - bounds.width / 2;
    const y = event.clientY - bounds.top - bounds.height / 2;
    if (Math.hypot(x, y) < 5) return;
    changeHsl({ h: (Math.atan2(y, x) * 180 / Math.PI + 450) % 360 });
  }
  function start(event: PointerEvent) {
    if (event.button !== 0 || pointer !== null) return;
    const element = event.currentTarget as HTMLElement;
    pointer = event.pointerId; element.focus(); element.setPointerCapture(event.pointerId); point(event);
  }
  function hueKey(event: KeyboardEvent) {
    const steps: Record<string, number> = { ArrowRight: 1, ArrowUp: 1, ArrowLeft: -1, ArrowDown: -1, PageUp: 15, PageDown: -15 };
    if (event.key in steps) { event.preventDefault(); changeHsl({ h: (hsl.h + steps[event.key] * (event.shiftKey ? 10 : 1) + 360) % 360 }); }
    else if (event.key === 'Home' || event.key === 'End') { event.preventDefault(); changeHsl({ h: event.key === 'Home' ? 0 : 359 }); }
  }
</script>

<div class="primary-color-picker">
  <div class="primary-color-options" role="group" aria-label="Primary color suggestions">
    {#each PRIMARY_COLORS as option}
      <button class="primary-color-option" class:chosen={color === option.value} aria-pressed={color === option.value} aria-label={option.name} title={option.name} onclick={() => choosePreset(option.value)}>
        <span class="primary-color-swatch" style:background={option.value}>{#if color === option.value}<AppIcon name="check" size={15} />{/if}</span><span>{option.name}</span>
      </button>
    {/each}
    <button class="primary-color-option primary-color-custom" class:chosen={customOpen} aria-expanded={customOpen} aria-controls="custom-primary-color" onclick={() => customOpen = !customOpen}><span class="primary-color-swatch"><AppIcon name="palette" size={17} /></span><span>Custom</span></button>
  </div>
  {#if customOpen}
    <div class="custom-primary-color" id="custom-primary-color">
      <div class="color-wheel-field">
        <span id="color-hue-label">Hue</span>
        <div class="color-wheel" role="slider" tabindex="0" aria-labelledby="color-hue-label" aria-valuemin="0" aria-valuemax="359" aria-valuenow={Math.round(hsl.h) % 360} aria-valuetext={`${Math.round(hsl.h) % 360} degrees`} aria-describedby="color-wheel-help" onkeydown={hueKey} onpointerdown={start} onpointermove={(event) => { if (pointer === event.pointerId) point(event); }} onpointerup={() => pointer = null} onpointercancel={() => pointer = null} onlostpointercapture={() => pointer = null}>
          <span class="color-wheel-center" style:background={color}></span><span class="color-wheel-marker" style:left={`${marker.x}%`} style:top={`${marker.y}%`} style:background={hueColor}></span>
        </div>
        <small id="color-wheel-help">Drag or use arrow keys. Hold Shift for larger steps.</small>
      </div>
      <div class="color-precision-controls">
        <label for="primary-saturation">Saturation <output for="primary-saturation">{Math.round(hsl.s)}%</output></label>
        <input id="primary-saturation" type="range" min="0" max="100" step="1" value={hsl.s} style:background={`linear-gradient(to right, hsl(${hsl.h} 0% ${hsl.l}%), hsl(${hsl.h} 100% ${hsl.l}%))`} oninput={(event) => changeHsl({ s: Number(event.currentTarget.value) })} />
        <label for="primary-lightness">Lightness <output for="primary-lightness">{Math.round(hsl.l)}%</output></label>
        <input id="primary-lightness" type="range" min="0" max="100" step="1" value={hsl.l} style:background={`linear-gradient(to right, #000, hsl(${hsl.h} ${hsl.s}% 50%), #fff)`} oninput={(event) => changeHsl({ l: Number(event.currentTarget.value) })} />
        <label for="primary-color-hex">Hex color</label>
        <div class="primary-color-hex"><input id="primary-color-hex" type="text" bind:value={hexInput} maxlength="7" spellcheck="false" autocapitalize="off" autocomplete="off" aria-invalid={!!hexError} aria-describedby={hexError ? 'primary-color-hex-error' : undefined} oninput={() => hexError = ''} onkeydown={(event) => { if (event.key === 'Enter') { event.preventDefault(); applyHex(); } }} /><button onclick={applyHex}>Apply</button></div>
        {#if hexError}<p class="settings-error" id="primary-color-hex-error" role="status">{hexError}</p>{/if}
      </div>
    </div>
  {/if}
  <p class="primary-color-value">{color.toUpperCase()}</p>
  {#if error}<p class="settings-error" role="status">{error}</p>{/if}
</div>
