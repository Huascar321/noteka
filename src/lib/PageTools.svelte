<script lang="ts">
  import { onMount, type Snippet } from 'svelte';
  import AppIcon from './AppIcon.svelte';
  let { groups, active = $bindable(null), content, placement = 'footer' }: {
    groups: { id: string; label: string; icon: string; available: boolean; reason?: string }[];
    active?: string | null;
    content: Snippet<[string]>;
    placement?: 'footer' | 'header';
  } = $props();
  const id = $props.id();
  let root: HTMLDivElement;
  let buttons: Record<string, HTMLButtonElement> = {};
  let panelPosition = $state({ left: 12, top: 100, maxHeight: 400 });
  function positionPanel() {
    if (placement !== 'header' || !active || !root?.getClientRects().length) return;
    const rect = root.getBoundingClientRect(), width = Math.min(620, innerWidth - 24);
    const top = Math.max(12, Math.min(rect.bottom + 8, innerHeight - 180));
    panelPosition = { left: Math.max(12, Math.min(rect.right - width, innerWidth - width - 12)), top, maxHeight: innerHeight - top - 12 };
  }
  function close() {
    const previous = active;
    active = null;
    if (previous) buttons[previous]?.focus();
  }
  $effect(() => { if (active && !groups.some(group => group.id === active && group.available)) active = null; });
  $effect(() => { if (active && placement === 'header') positionPanel(); });
  onMount(() => {
    function scroll(event: Event) { if (!(event.target instanceof Element && event.target.closest('.page-tool-panel'))) positionPanel(); }
    document.addEventListener('scroll', scroll, true);
    return () => document.removeEventListener('scroll', scroll, true);
  });
</script>

<svelte:window onresize={positionPanel} onpointerdown={(event) => { if (placement === 'header' && active && !root?.contains(event.target as Node)) active = null; }} onkeydown={(event) => { if (active && event.key === 'Escape' && !event.defaultPrevented && !event.isComposing && root?.contains(document.activeElement)) { event.preventDefault(); close(); } }} />

<div class="page-tools" class:page-tools-header={placement === 'header'} bind:this={root}>
  <div class="page-tool-buttons" role="group" aria-label="Page tools">
    {#each groups as group (group.id)}
      <button type="button" class="page-tool-trigger" data-page-tool={group.id} id={`${id}-${group.id}-trigger`} aria-label={group.label} aria-controls={`${id}-${group.id}-panel`} aria-expanded={active === group.id} aria-describedby={`${id}-${group.id}-tip`} disabled={!group.available} bind:this={buttons[group.id]} onclick={() => active = active === group.id ? null : group.id}>
        <AppIcon name={group.icon} size={17} />
        <span class="page-tool-tooltip" role="tooltip" id={`${id}-${group.id}-tip`}>{group.label}{!group.available && group.reason ? ` · ${group.reason}` : ''}</span>
      </button>
    {/each}
  </div>
  {#each groups as group (group.id)}
    <section class="page-tool-panel" id={`${id}-${group.id}-panel`} aria-labelledby={`${id}-${group.id}-trigger`} hidden={active !== group.id || !group.available} style:left={placement === 'header' ? `${panelPosition.left}px` : undefined} style:top={placement === 'header' ? `${panelPosition.top}px` : undefined} style:max-height={placement === 'header' ? `${panelPosition.maxHeight}px` : undefined}>
      <header><h3>{group.label}</h3><button type="button" class="icon-button" aria-label={`Close ${group.label}`} onclick={close}><AppIcon name="close" size={16} /></button></header>
      {@render content(group.id)}
    </section>
  {/each}
</div>
