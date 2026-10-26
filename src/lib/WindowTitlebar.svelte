<script lang="ts">
  import { onMount, type Snippet } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import AppIcon from './AppIcon.svelte';
  import { desktopPlatform } from './window-chrome';

  let { children, workspaceOpen, sidebarVisible, blocked, ontoggle, onclose, onerror }: {
    children?: Snippet;
    workspaceOpen: boolean;
    sidebarVisible: boolean;
    blocked: boolean;
    ontoggle: () => void;
    onclose: () => void | Promise<void>;
    onerror: (message: string) => void;
  } = $props();

  const platform = desktopPlatform();
  const dragRegion = platform === 'macos' || platform === 'windows' ? '' : undefined;
  let maximized = $state(false);

  async function windowAction(action: 'minimize' | 'maximize') {
    if (platform !== 'windows') return;
    try {
      const current = getCurrentWindow();
      if (action === 'minimize') await current.minimize();
      else await current.toggleMaximize();
    } catch {
      onerror(action === 'minimize' ? "Could not minimize the window." : "Could not resize the window.");
    }
  }

  onMount(() => {
    if (platform !== 'windows') return;
    const current = getCurrentWindow();
    let disposed = false;
    let reading = false;
    let queued = false;
    let unlisten: (() => void) | undefined;
    async function refreshMaximized() {
      if (disposed) return;
      if (reading) { queued = true; return; }
      reading = true;
      try {
        do {
          queued = false;
          const next = await current.isMaximized();
          if (!disposed) maximized = next;
        } while (queued && !disposed);
      } catch {
        if (!disposed) onerror("Could not read the window state.");
      } finally { reading = false; }
    }
    void current.onResized(() => { void refreshMaximized(); }).then(fn => {
      if (disposed) fn(); else unlisten = fn;
    }).catch(() => { if (!disposed) onerror("Could not track the window state."); });
    void refreshMaximized();
    return () => { disposed = true; unlisten?.(); };
  });
</script>

<header class="tab-bar window-titlebar" data-desktop-platform={platform} data-tauri-drag-region={dragRegion}>
  {#if platform === 'macos'}<div class="native-traffic-space" aria-hidden="true" data-tauri-drag-region={dragRegion}></div>{/if}
  {#if workspaceOpen}
    <button class="icon-button sidebar-toggle" aria-label={sidebarVisible ? "Collapse sidebar" : "Expand sidebar"} title={sidebarVisible ? "Collapse sidebar" : "Expand sidebar"} aria-controls="workspace-sidebar" aria-expanded={sidebarVisible} disabled={blocked} onclick={ontoggle}>
      <svg viewBox="0 0 24 24" width="19" height="19" fill="none" stroke="currentColor" stroke-width="1.7" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16"/></svg>
    </button>
  {/if}
  <div class="window-titlebar-content" data-tauri-drag-region={dragRegion}>
    {#if workspaceOpen}{@render children?.()}{:else}<span class="welcome-window-title" data-tauri-drag-region={dragRegion}>Noteka</span>{/if}
    <div class="titlebar-drag-space" data-tauri-drag-region={dragRegion} aria-hidden="true"></div>
  </div>
  {#if platform === 'windows'}
    <div class="window-controls" aria-label="Window controls">
      <button class="window-control" aria-label="Minimize window" title="Minimize" onclick={() => void windowAction('minimize')}><svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"><path d="M1 6h10"/></svg></button>
      <button class="window-control" aria-label={maximized ? "Restore window" : "Maximize window"} title={maximized ? "Restore" : "Maximize"} onclick={() => void windowAction('maximize')}><svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">{#if maximized}<path d="M3 3V1.5h7.5V9H9"/><rect x="1.5" y="3" width="7.5" height="7.5"/>{:else}<rect x="1.5" y="1.5" width="9" height="9"/>{/if}</svg></button>
      <button class="window-control window-close" aria-label="Close window" title="Close" disabled={blocked} onclick={() => void onclose()}><AppIcon name="close" size={16} /></button>
    </div>
  {/if}
</header>
