<script lang="ts">
  import type { Snippet } from 'svelte';
  import { liquidSurface, liquidTransition } from './liquid-overlay';
  let { children }: { children: Snippet } = $props();
  let expanded = $state(false), present = $state(false);
  let trigger: HTMLElement;
  let root: HTMLDetailsElement;
  function toggle() { expanded = !expanded; if (expanded) present = true; }
  function closed() { if (!expanded) present = false; }
  function closeOnChoice(node: HTMLElement) {
    const close = (event: Event) => { if (event.target instanceof Element && event.target.closest('button')) expanded = false; };
    node.addEventListener('click', close);
    return { destroy() { node.removeEventListener('click', close); } };
  }
</script>

<svelte:window onkeydown={event => { if (event.key === 'Escape' && expanded && root?.contains(document.activeElement)) { event.preventDefault(); expanded = false; trigger.focus(); } }} onpointerdown={event => { if (expanded && root && event.target instanceof Node && !root.contains(event.target)) expanded = false; }} />
<details open={present} bind:this={root}>
  <summary bind:this={trigger} aria-expanded={expanded} onclick={event => { event.preventDefault(); toggle(); }}>Nav</summary>
  {#if expanded}
    <div class="quick-nav-options" use:closeOnChoice use:liquidSurface={{ anchor: () => trigger }} in:liquidTransition out:liquidTransition onoutroend={closed}>
      {@render children()}
    </div>
  {/if}
</details>
