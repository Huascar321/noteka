<script lang="ts">
  import type { Snippet } from 'svelte';
  import { liquidSurface, liquidTransition } from './liquid-overlay';
  let { children }: { children: Snippet } = $props();
  let expanded = $state(false), present = $state(false);
  let trigger: HTMLElement;
  let root: HTMLDetailsElement;
  function toggle() { expanded = !expanded; if (expanded) present = true; }
  function closed() { if (!expanded) present = false; }
</script>

<svelte:window onkeydown={event => { if (event.key === 'Escape' && expanded && root?.contains(document.activeElement)) { event.preventDefault(); expanded = false; trigger.focus(); } }} />
<details open={present} bind:this={root}>
  <summary bind:this={trigger} aria-expanded={expanded} onclick={event => { event.preventDefault(); toggle(); }}>Nav</summary>
  {#if expanded}
    <div class="quick-nav-options" use:liquidSurface={{ anchor: () => trigger }} in:liquidTransition out:liquidTransition onoutroend={closed}>
      {@render children()}
    </div>
  {/if}
</details>
