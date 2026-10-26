<script lang="ts">
  import { onMount } from 'svelte';
  let { label, controls, edge, width, min, max, disabled = false, onstart, onresize, onend }: {
    label: string; controls: string; edge: 'left' | 'right'; width: number; min: number; max: number; disabled?: boolean;
    onstart: () => void; onresize: (width: number) => void; onend: (cancelled: boolean) => void;
  } = $props();
  let node: HTMLDivElement;
  let pointer = $state<number | null>(null);
  let startX = 0, startWidth = 0, latestX = 0, frame = 0;
  const bounded = (value: number) => Math.min(max, Math.max(min, Math.round(value)));
  function update() { frame = 0; if (pointer !== null) onresize(bounded(startWidth + (latestX - startX) * (edge === 'right' ? 1 : -1))); }
  function finish(cancelled = false) {
    if (pointer === null) return;
    cancelAnimationFrame(frame); frame = 0;
    if (!cancelled) update();
    const id = pointer; pointer = null;
    if (node.hasPointerCapture(id)) node.releasePointerCapture(id);
    onend(cancelled);
  }
  function start(event: PointerEvent) {
    if (disabled || event.button !== 0 || !event.isPrimary || pointer !== null) return;
    event.preventDefault(); startX = latestX = event.clientX; startWidth = width; pointer = event.pointerId;
    node.setPointerCapture(event.pointerId); node.focus({ preventScroll: true }); onstart();
  }
  function move(event: PointerEvent) { if (event.pointerId === pointer) { latestX = event.clientX; if (!frame) frame = requestAnimationFrame(update); } }
  function keydown(event: KeyboardEvent) {
    if (disabled || event.isComposing || event.metaKey || event.ctrlKey) return;
    if (event.key === 'Escape' && pointer !== null) { event.preventDefault(); finish(true); return; }
    if (pointer !== null || !['ArrowLeft','ArrowRight','Home','End'].includes(event.key)) return;
    event.preventDefault(); const amount = event.altKey ? 1 : event.shiftKey ? 50 : 10;
    const delta = (event.key === 'ArrowRight' ? 1 : -1) * (edge === 'right' ? 1 : -1) * amount;
    onstart(); onresize(bounded(event.key === 'Home' ? min : event.key === 'End' ? max : width + delta)); onend(false);
  }
  $effect(() => { if (disabled && pointer !== null) finish(true); });
  onMount(() => () => { cancelAnimationFrame(frame); if (pointer !== null) finish(true); });
</script>

<svelte:window onblur={() => finish(true)} onkeydown={event => { if (pointer !== null && event.key === 'Escape') { event.preventDefault(); finish(true); } }} />
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions (This focusable splitter resizes its panel with pointer and arrow keys.) -->
<div bind:this={node} class="panel-resize-handle" class:edge-left={edge === 'left'} class:dragging={pointer !== null} role="separator" aria-orientation="vertical" aria-label={label} aria-controls={controls} aria-valuemin={min} aria-valuemax={max} aria-valuenow={width} aria-valuetext={`${width} pixels`} aria-disabled={disabled} tabindex={disabled ? -1 : 0} onpointerdown={start} onpointermove={move} onpointerup={event => { if (event.pointerId === pointer) finish(); }} onpointercancel={() => finish(true)} onlostpointercapture={() => finish(true)} onkeydown={keydown}></div>
