<script lang="ts">
  import { liquidSurface, liquidTransition } from './liquid-overlay';
  import { onMount, tick, type Snippet } from 'svelte';
  import AppIcon from './AppIcon.svelte';
  let { groups, active = $bindable(null), content, placement = 'footer', actions = [] }: {
    groups: { id: string; label: string; icon: string; available: boolean; reason?: string }[];
    active?: string | null;
    content: Snippet<[string]>;
    placement?: 'footer' | 'header';
    actions?: { id: string; label: string; icon: string; disabled: boolean; checked?: boolean; reason?: string; onselect: () => void }[];
  } = $props();
  const id = $props.id();
  let root: HTMLDivElement;
  let buttons: Record<string, HTMLButtonElement> = {};
  let menuTrigger = $state<HTMLButtonElement>();
  let menu = $state<HTMLDivElement>();
  let menuOpen = $state(false);
  let menuPosition = $state({ left: 12, top: 100, maxHeight: 400 });
  let panelPosition = $state({ left: 12, top: 100, maxHeight: 400 });
  function positionPanel() {
    if (placement !== 'header' || !active || !root?.getClientRects().length) return;
    const rect = root.getBoundingClientRect(), width = Math.min(620, innerWidth - 24);
    const top = Math.max(12, Math.min(rect.bottom + 8, innerHeight - 180));
    panelPosition = { left: Math.max(12, Math.min(rect.right - width, innerWidth - width - 12)), top, maxHeight: innerHeight - top - 12 };
  }
  function close() {
    const previous = active;
    active = null; menuOpen = false;
    if (placement === 'header') menuTrigger?.focus();
    else if (previous) buttons[previous]?.focus();
  }
  function positionMenu() {
    if (!menuOpen || !root) return;
    const rect = root.getBoundingClientRect(), width = Math.min(260, innerWidth - 24);
    const height = Math.min(menu?.scrollHeight || 190, innerHeight - 24);
    const top = rect.bottom + 8 + height <= innerHeight - 12 ? rect.bottom + 8 : Math.max(12, rect.top - height - 8);
    menuPosition = { left: Math.max(12, Math.min(rect.right - width, innerWidth - width - 12)), top, maxHeight: innerHeight - top - 12 };
  }
  async function toggleMenu(last = false) {
    active = null; menuOpen = !menuOpen;
    if (!menuOpen) return;
    await tick(); positionMenu();
    const items = menu?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)');
    items?.[last ? items.length - 1 : 0]?.focus();
  }
  async function chooseTool(name: string) {
    menuOpen = false; active = name;
    await tick(); positionPanel();
    const panel = root.querySelector<HTMLElement>('.page-tool-panel:not([hidden])');
    (panel?.querySelector<HTMLElement>('input:not(:disabled),select:not(:disabled),textarea:not(:disabled)')
      ?? panel?.querySelector<HTMLButtonElement>(':scope > :not(header) button:not(:disabled)')
      ?? panel?.querySelector<HTMLButtonElement>('button:not(:disabled)'))?.focus();
  }
  function menuKeydown(event: KeyboardEvent) {
    if (event.isComposing) return;
    if (event.key === 'Escape') { event.preventDefault(); close(); return; }
    if (event.key === 'Tab') { close(); return; }
    if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) return;
    event.preventDefault();
    const items = [...(menu?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)') ?? [])];
    const index = items.indexOf(document.activeElement as HTMLButtonElement);
    items[event.key === 'Home' ? 0 : event.key === 'End' ? items.length - 1 : (index + (event.key === 'ArrowDown' ? 1 : -1) + items.length) % items.length]?.focus();
  }
  function positionOverlays() { positionPanel(); positionMenu(); }
  function outside(event: PointerEvent) {
    if (placement === 'header' && !root?.contains(event.target as Node)) { active = null; menuOpen = false; }
  }
  $effect(() => { if (active && !groups.some(group => group.id === active && group.available)) active = null; });
  $effect(() => { if (active && placement === 'header') positionPanel(); });
  $effect(() => { if (menuOpen) positionMenu(); });
  onMount(() => {
    function scroll(event: Event) { if (!(event.target instanceof Element && event.target.closest('.page-tool-panel,.page-actions-menu'))) positionOverlays(); }
    document.addEventListener('scroll', scroll, true);
    return () => document.removeEventListener('scroll', scroll, true);
  });
</script>

<svelte:window onresize={positionOverlays} onpointerdown={outside} onkeydown={(event) => { if ((active || menuOpen) && event.key === 'Escape' && !event.defaultPrevented && !event.isComposing && root?.contains(document.activeElement)) { event.preventDefault(); close(); } }} />

<div class="page-tools" class:page-tools-header={placement === 'header'} bind:this={root}>
  {#if placement === 'header'}
    <button bind:this={menuTrigger} type="button" class="page-tool-trigger page-actions-trigger" aria-label="Page actions" title="Page actions" aria-haspopup="menu" aria-expanded={menuOpen} aria-controls={menuOpen ? `${id}-menu` : undefined} onclick={() => void toggleMenu()} onkeydown={event => { if (event.key === 'ArrowDown' || event.key === 'ArrowUp') { event.preventDefault(); if (!menuOpen) void toggleMenu(event.key === 'ArrowUp'); } }}><AppIcon name="ellipsis" size={20} /></button>
    {#if menuOpen}
      <div bind:this={menu} id={`${id}-menu`} class="page-actions-menu glass-menu" use:liquidSurface={{ anchor: () => menuTrigger }} in:liquidTransition|global out:liquidTransition|global role="menu" aria-label="Page actions" tabindex="-1" onkeydown={menuKeydown} style:left={`${menuPosition.left}px`} style:top={`${menuPosition.top}px`} style:max-height={`${menuPosition.maxHeight}px`}>
        {#each groups as group (group.id)}
          <button type="button" role="menuitem" tabindex="-1" aria-label={group.label} aria-controls={`${id}-${group.id}-panel`} aria-describedby={!group.available && group.reason ? `${id}-${group.id}-reason` : undefined} disabled={!group.available} onclick={() => void chooseTool(group.id)}><AppIcon name={group.icon} size={18} /><span>{group.label}{#if !group.available && group.reason}<small id={`${id}-${group.id}-reason`}>{group.reason}</small>{/if}</span></button>
        {/each}
        {#if actions.length}<div class="page-actions-divider" role="separator"></div>{/if}
        {#each actions as action (action.id)}
          <button type="button" role={action.checked === undefined ? 'menuitem' : 'menuitemcheckbox'} aria-checked={action.checked} aria-label={action.label} title={action.disabled ? action.reason : undefined} tabindex="-1" disabled={action.disabled} onclick={() => { close(); action.onselect(); }}><AppIcon name={action.icon} size={18} /><span>{action.label}</span>{#if action.checked !== undefined}<span class="page-action-switch" class:on={action.checked} aria-hidden="true"></span>{/if}</button>
        {/each}
      </div>
    {/if}
  {:else}
  <div class="page-tool-buttons" role="group" aria-label="Page tools">
    {#each groups as group (group.id)}
      <button type="button" class="page-tool-trigger" data-page-tool={group.id} id={`${id}-${group.id}-trigger`} aria-label={group.label} aria-controls={`${id}-${group.id}-panel`} aria-expanded={active === group.id} aria-describedby={`${id}-${group.id}-tip`} disabled={!group.available} bind:this={buttons[group.id]} onclick={() => active = active === group.id ? null : group.id}>
        <AppIcon name={group.icon} size={17} />
        <span class="page-tool-tooltip" role="tooltip" id={`${id}-${group.id}-tip`}>{group.label}{!group.available && group.reason ? ` · ${group.reason}` : ''}</span>
      </button>
    {/each}
  </div>
  {/if}
  {#each groups as group (group.id)}
    <section class="page-tool-panel" use:liquidSurface={{ anchor: () => placement === 'header' ? menuTrigger : buttons[group.id], visible: active === group.id && group.available }} class:glass-menu={placement === 'header'} id={`${id}-${group.id}-panel`} aria-labelledby={`${id}-${group.id}-heading`} hidden={active !== group.id || !group.available} style:left={placement === 'header' ? `${panelPosition.left}px` : undefined} style:top={placement === 'header' ? `${panelPosition.top}px` : undefined} style:max-height={placement === 'header' ? `${panelPosition.maxHeight}px` : undefined}>
      <header><h3 id={`${id}-${group.id}-heading`}>{group.label}</h3><button type="button" class="icon-button" aria-label={`Close ${group.label}`} onclick={close}><AppIcon name="close" size={16} /></button></header>
      {@render content(group.id)}
    </section>
  {/each}
</div>
