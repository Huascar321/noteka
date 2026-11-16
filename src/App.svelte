<script lang="ts">
  import { liquidSurface, liquidTransition, hasLiquidOverlayWithin, liquidExitFinished } from './lib/liquid-overlay';
  import { onMount, tick, setContext, untrack } from 'svelte';
  import { isTauri } from '@tauri-apps/api/core';
  import { shortcutLabel } from './lib/shortcuts';
  import { listen } from '@tauri-apps/api/event';
  import { operate, setPending, closeConfirmed } from './lib/api';
  import WorkspacePane from './lib/WorkspacePane.svelte';
  import SearchDialog from './lib/SearchDialog.svelte';
  import SettingsDialog from './lib/SettingsDialog.svelte';
  import SectionIcon from './lib/SectionIcon.svelte';
  import AppIcon from './lib/AppIcon.svelte';
  import NotekaMark from './lib/NotekaMark.svelte';
  import WindowTitlebar from './lib/WindowTitlebar.svelte';
  import PanelResizeHandle from './lib/PanelResizeHandle.svelte';
  import { sidebarIconMotion } from './lib/sidebar-icon-motion';
  import { PRIMARY_SECTIONS, targetTitle, targetIcon, type NavigationTarget, type PaneState, type Section, type WorkspacePaneApi } from './lib/navigation';
  import { readPrimaryColor, savePrimaryColor, primaryPalette, normalizeColor } from './lib/primary-color';
  import { readTheme, resolveTheme, saveTheme, type ThemePreference } from './lib/theme';
  import { installSmoothWheel, readSmoothWheel, saveSmoothWheel } from './lib/smooth-wheel';
  import { readPeekMode, savePeekMode, type PeekMode, type PeekPresentation } from './lib/peek-mode';
  import { readPanelWidths, savePanelWidths, panelLimits, effectivePanelWidth, clampPanelWidth, sidePeekExcludesSidebar, PANEL_MOTION_MS, type PanelKind, type PanelWidths } from './lib/panel-widths';
  import { saveWorkspacePath } from './lib/workspace-startup';
  import type { WorkspaceInfo } from './lib/contracts.generated';
  import { WORKSPACE_ICON_CONTEXT } from './lib/visual-icon';

  type Tab = { id: string; initialTarget: NavigationTarget; state: PaneState; ownerId?: string; presentation?: PeekPresentation; returnFocus?: HTMLElement; sideHidden?: boolean; sideEpoch?: number; closing?: boolean };
  let { startupWorkspace = null, startupError = '' }: { startupWorkspace?: WorkspaceInfo | null; startupError?: string } = $props();
  // Los props de arranque se leen una vez: describen el estado resuelto antes de montar.
  const initial = untrack(() => ({ workspace: startupWorkspace, error: startupError }));
  let workspace = $state<WorkspaceInfo | null>(initial.workspace);
  setContext(WORKSPACE_ICON_CONTEXT, () => workspace?.path);
  let path = $state(initial.workspace?.path ?? '');
  let tabs = $state<Tab[]>([]);
  let activeId = $state('');
  let focusedId = $state('');
  let paneRefs = $state<Record<string, WorkspacePaneApi | undefined>>({});
  let working = $state(false);
  let preparing = $state(false);
  let error = $state(initial.error);
  let searchMode = $state<'current' | 'new' | null>(null);
  let settingsOpen = $state(false);
  let sidebarOpen = $state(false);
  let sidebarCollapsed = $state(false);
  let compactSidebar = $state(window.matchMedia('(max-width: 760px)').matches);
  let viewportWidth = $state(window.innerWidth);
  let preferredWidths = $state<PanelWidths>(readPanelWidths(() => localStorage));
  let panelWidthError = $state('');
  let resizing = $state<PanelKind | null>(null);
  let resizeSnapshot: { widths: PanelWidths; open: boolean; collapsed: boolean } | null = null;
  const sidebarWidth = $derived(effectivePanelWidth('sidebar', preferredWidths, viewportWidth));
  const sidePeekWidth = $derived(effectivePanelWidth('sidePeek', preferredWidths, viewportWidth));
  const sideWide = $derived(sidePeekExcludesSidebar(sidePeekWidth, viewportWidth));
  const sidebarLimits = $derived(panelLimits('sidebar', viewportWidth));
  const sidePeekLimits = $derived(panelLimits('sidePeek', viewportWidth));
  const sidebarVisible = $derived(compactSidebar ? sidebarOpen : !sidebarCollapsed);
  function setSidebarVisible(visible: boolean) { sidebarOpen = visible; sidebarCollapsed = !visible; }
  function toggleSidebar() {
    const opening = !sidebarVisible;
    if (opening && activePeek?.presentation === 'side' && sideWide) hideSide(activePeek);
    setSidebarVisible(opening);
  }
  function beginResize(kind: PanelKind) {
    resizeSnapshot = { widths: { ...preferredWidths }, open: sidebarOpen, collapsed: sidebarCollapsed };
    resizing = kind;
  }
  function resizePanel(kind: PanelKind, width: number) {
    preferredWidths = { ...preferredWidths, [kind]: clampPanelWidth(kind, width, viewportWidth) };
    if (kind === 'sidePeek' && sideWide && activePeek && peekVisible(activePeek)) setSidebarVisible(false);
  }
  function endResize(cancelled: boolean) {
    if (cancelled && resizeSnapshot) {
      preferredWidths = resizeSnapshot.widths; sidebarOpen = resizeSnapshot.open; sidebarCollapsed = resizeSnapshot.collapsed;
    } else if (!cancelled) panelWidthError = savePanelWidths(preferredWidths, () => localStorage);
    resizing = null; resizeSnapshot = null;
  }
  let theme = $state<ThemePreference>(readTheme(() => localStorage));
  const systemThemeMedia = typeof window.matchMedia === 'function' ? window.matchMedia('(prefers-color-scheme: dark)') : null;
  let systemDark = $state(systemThemeMedia?.matches ?? true);
  const effectiveTheme = $derived(resolveTheme(theme, systemDark));
  let themeError = $state('');
  let smoothWheel = $state(readSmoothWheel(() => localStorage));
  let smoothWheelError = $state('');
  const motionMedia = window.matchMedia('(prefers-reduced-motion: reduce)');
  let reduceMotion = $state(motionMedia.matches);
  $effect(() => { if (smoothWheel && !reduceMotion) return installSmoothWheel(document); });
  function changeSmoothWheel(enabled: boolean) { smoothWheel = enabled; smoothWheelError = saveSmoothWheel(enabled, () => localStorage); }
  let primaryColor = $state(readPrimaryColor(() => localStorage));
  let primaryColorError = $state('');
  let defaultPeekMode = $state<PeekMode>(readPeekMode(() => localStorage));
  let peekPreferenceError = $state('');
  function changeDefaultPeekMode(mode: PeekMode) { defaultPeekMode = mode; peekPreferenceError = savePeekMode(mode, () => localStorage); }
  $effect(() => {
    for (const [name, value] of Object.entries(primaryPalette(primaryColor, effectiveTheme))) document.documentElement.style.setProperty(name, value);
  });
  function changePrimaryColor(next: string) {
    const valid = normalizeColor(next);
    if (!valid) return;
    primaryColor = valid; primaryColorError = savePrimaryColor(valid, () => localStorage);
  }
  const activeTab = $derived(tabs.find(tab => tab.id === activeId));
  const activeSectionIndex = $derived(PRIMARY_SECTIONS.findIndex(section => section.id === activeTab?.state.section));
  const rootTabs = $derived(tabs.filter(tab => !tab.ownerId));
  const activePeek = $derived(tabs.find(tab => tab.ownerId === activeId));
  function peekVisible(tab?: Tab): boolean { return !!tab && !tab.closing && (tab.presentation !== 'side' || !tab.sideHidden); }
  const visiblePeek = $derived(peekVisible(activePeek) ? activePeek : undefined);
  const sideVisible = $derived(visiblePeek?.presentation === 'side');
  const focusedTab = $derived(tabs.find(tab => tab.id === focusedId) ?? activeTab);
  const guarded = $derived(tabs.some(tab => tab.state.guarded));
  const protectedState = $derived(tabs.some(tab => tab.state.dirty || tab.state.busy));
  const blocked = $derived(preparing || guarded);
  $effect(() => { if (sidebarVisible && sideWide && sideVisible) setSidebarVisible(false); });
  $effect(() => {
    const id = activeId;
    if (id) void tick().then(() => document.getElementById(`tab-${id}`)?.scrollIntoView({ block: 'nearest', inline: 'nearest' }));
  });
  let pendingWrite = Promise.resolve();
  $effect(() => { document.documentElement.dataset.theme = effectiveTheme; });
  $effect(() => {
    const pending = protectedState;
    pendingWrite = pendingWrite.then(() => setPending(pending)).catch(() => { error = "Close protection could not be enabled. Keep this window open while changes are pending."; });
  });
  function changeTheme(next: ThemePreference) { theme = next; themeError = saveTheme(next, () => localStorage); }
  function updateTab(id: string, state: PaneState) {
    const tab = tabs.find(tab => tab.id === id);
    if (tab && JSON.stringify(tab.state) !== JSON.stringify(state)) tab.state = state;
  }
  function makePane(target: NavigationTarget): Tab {
    const id = crypto.randomUUID();
    return { id, initialTarget: target, state: { section: target.type === 'section' ? target.section : target.kind, title: targetTitle(target), icon: targetIcon(target), sectionPage: target.type === 'section', dirty: false, busy: false, guarded: false } };
  }
  function addTab(target: NavigationTarget = { type: 'section', section: 'home' }) {
    const tab = makePane(target); tabs = [...tabs, tab];
    activeId = tab.id; focusedId = tab.id; sidebarOpen = false;
  }
  if (initial.workspace) addTab();
  async function selectWorkspace(create: boolean) {
    if (working) return;
    working = true; error = '';
    try {
      workspace = await operate<WorkspaceInfo>({ operation: create ? 'workspace.create' : 'workspace.open', input: {} }, path.trim() || undefined);
      path = workspace.path; tabs = []; addTab();
      const saveError = saveWorkspacePath(workspace.path, () => localStorage);
      if (saveError) error = saveError;
    } catch (e) { error = e instanceof Error ? e.message : "Could not open the workspace."; }
    finally { working = false; }
  }
  function activateTab(id: string, focus = false) {
    if (blocked || settingsOpen || !rootTabs.some(tab => tab.id === id)) return;
    activeId = id;
    const child = tabs.find(tab => tab.ownerId === id);
    focusedId = child && peekVisible(child) ? child.id : id;
    if (child?.presentation === 'side' && peekVisible(child) && sideWide) setSidebarVisible(false);
    else sidebarOpen = false;
    if (focus) void tick().then(() => document.getElementById(`tab-${id}`)?.focus());
  }
  async function navigate(section: Section) {
    if (blocked || settingsOpen) return;
    sidebarOpen = false;
    if (activePeek && !(await closePeek(activePeek.id, false))) return;
    focusedId = activeId;
    await paneRefs[activeId]?.navigate(section);
  }
  let searchAnchor = $state<HTMLElement | null>(null);
  let settingsAnchor = $state<HTMLElement | null>(null);
  function openSearch(mode: 'current' | 'new', anchor: HTMLElement | null = null) {
    searchAnchor = anchor;
    if (workspace && !blocked && !settingsOpen && (mode === 'new' || !focusedTab?.state.busy)) { searchMode = mode; sidebarOpen = false; }
  }
  function openSettings(event?: MouseEvent) {
    settingsAnchor = event?.currentTarget instanceof HTMLElement ? event.currentTarget : null;
    if (workspace && !blocked && !searchMode) settingsOpen = true;
  }
  async function chooseTarget(target: NavigationTarget, newTab: boolean) {
    const exit = liquidExitFinished(document.querySelector('.context-search'));
    searchMode = null;
    await exit;
    await tick();
    if (newTab) addTab(target);
    else if (target.type === 'section') await navigate(target.section);
    else await openDetail(activeId, target);
  }
  async function openDetail(ownerId: string, target: NavigationTarget) {
    if (blocked || settingsOpen || resizing || !workspace || ownerId !== activeId) return;
    const existing = tabs.find(tab => tab.ownerId === ownerId);
    if (existing && target.type === 'entity' && existing.state.entityId === target.id && (existing.sideHidden || !target.hit?.block_id)) { await revealPeek(existing); return; }
    if (defaultPeekMode === 'tab') { addTab(target); return; }
    if (existing) {
      if (!(await revealPeek(existing))) return;
      // Reuse the pane: its own guard resolves before replacing the buffer.
      if (await paneRefs[existing.id]?.openTarget(target)) {
        const changed = existing.presentation !== defaultPeekMode;
        existing.presentation = defaultPeekMode;
        if (changed && defaultPeekMode === 'side') existing.sideHidden = true;
        await revealPeek(existing);
      }
      return;
    }
    const tab = makePane(target);
    tab.ownerId = ownerId; tab.presentation = defaultPeekMode;
    tab.sideHidden = defaultPeekMode === 'side';
    tab.returnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : undefined;
    tabs = [...tabs, tab]; sidebarOpen = false;
    await revealPeek(tabs.find(item => item.id === tab.id)!);
  }
  function hideSide(tab: Tab) {
    tab.sideEpoch = (tab.sideEpoch ?? 0) + 1; tab.sideHidden = true;
    if (focusedId === tab.id) focusedId = tab.ownerId!;
  }
  async function revealPeek(tab: Tab, animate = true): Promise<boolean> {
    const epoch = (tab.sideEpoch ?? 0) + 1; tab.sideEpoch = epoch;
    if (tab.presentation === 'side') {
      if (sideWide) setSidebarVisible(false);
      if (animate && tab.sideHidden) {
        await tick();
        await new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));
      }
      if (tab.sideEpoch !== epoch || !tabs.some(item => item.id === tab.id) || tab.presentation !== 'side' || tab.ownerId !== activeId) return false;
    }
    tab.sideHidden = false; focusedId = tab.id;
    await tick();
    document.querySelector<HTMLElement>(`#pane-${tab.id} .editor`)?.focus({ preventScroll: true });
    return true;
  }
  function presentPane(id: string, mode: PeekMode) {
    const tab = tabs.find(tab => tab.id === id);
    if (!tab || blocked || settingsOpen) return;
    if (mode === 'tab') {
      tab.sideEpoch = (tab.sideEpoch ?? 0) + 1;
      delete tab.ownerId; delete tab.presentation; delete tab.returnFocus;
      delete tab.sideHidden;
      activeId = id; focusedId = id;
    } else {
      if (!tab.ownerId && mode !== 'full') {
        const source = makePane({ type: 'section', section: tab.state.section });
        tab.ownerId = source.id;
        tab.returnFocus = undefined;
        tabs = [...tabs, source]; activeId = source.id;
      }
      if (tab.ownerId) {
        const changed = tab.presentation !== mode; tab.presentation = mode;
        if (mode === 'side' && changed) tab.sideHidden = true;
        void revealPeek(tab);
      }
      focusedId = id;
    }
  }
  async function closePeek(id: string, restoreFocus = true): Promise<boolean> {
    const tab = tabs.find(tab => tab.id === id && tab.ownerId);
    if (!tab || blocked || searchMode || settingsOpen) return false;
    preparing = true; error = '';
    let allowed = false;
    try {
      if (!(await prepareTabs([id]))) return false;
      if (tab.presentation === 'side' && !tab.sideHidden) {
        tab.closing = true; hideSide(tab); await tick();
        if (!matchMedia('(prefers-reduced-motion: reduce)').matches) await new Promise(resolve => setTimeout(resolve, PANEL_MOTION_MS));
      }
      tabs = tabs.filter(item => item.id !== id); delete paneRefs[id];
      focusedId = tab.ownerId!; allowed = true;
    } finally { preparing = false; }
    if (allowed && restoreFocus) {
      await tick();
      const previous = tab.returnFocus;
      if (previous?.isConnected && !previous.closest('[inert]')) previous.focus({ preventScroll: true });
      else document.querySelector<HTMLElement>(`#pane-${tab.ownerId} main`)?.focus({ preventScroll: true });
    }
    return allowed;
  }
  async function prepareTabs(ids: string[]): Promise<boolean> {
    for (const id of ids) {
      const tab = tabs.find(tab => tab.id === id);
      if (!tab) continue;
      if (tab.state.busy) { error = "Wait for the operation to finish before closing."; return false; }
      activeId = tab.ownerId ?? id; focusedId = id;
      if (tab.ownerId && tab.presentation === 'side' && tab.sideHidden && tab.state.dirty) await revealPeek(tab, false);
      await tick();
      const pane = paneRefs[id];
      if (!pane || !(await pane.prepareToLeave())) return false;
    }
    return true;
  }
  async function closeTab(id: string) {
    if (blocked || searchMode || settingsOpen) return;
    preparing = true; error = '';
    const previous = activeId;
    try {
      const children = tabs.filter(tab => tab.ownerId === id).map(tab => tab.id);
      if (!(await prepareTabs([...children, id]))) return;
      const index = rootTabs.findIndex(tab => tab.id === id);
      const closing = new Set([...children, id]);
      tabs = tabs.filter(tab => !closing.has(tab.id));
      for (const paneId of closing) delete paneRefs[paneId];
      if (!rootTabs.length) addTab();
      else { activeId = previous !== id && rootTabs.some(tab => tab.id === previous) ? previous : rootTabs[Math.min(index, rootTabs.length - 1)].id; const child = tabs.find(tab => tab.ownerId === activeId); focusedId = child && peekVisible(child) ? child.id : activeId; }
      await tick(); document.getElementById(`tab-${activeId}`)?.focus();
    } finally { preparing = false; }
  }
  async function leaveWorkspace(closeHost = false) {
    if (blocked || searchMode) return;
    const exit = liquidExitFinished(document.getElementById('settings-dialog'));
    preparing = true; error = ''; settingsOpen = false;
    try {
      await exit;
      await tick();
      // Every mounted pane participates, including buffers in inactive tabs.
      const ordered = [...tabs.filter(tab => tab.id === activeId), ...tabs.filter(tab => tab.id !== activeId)];
      if (!(await prepareTabs(ordered.map(tab => tab.id)))) return;
      if (closeHost) { await pendingWrite; await closeConfirmed(); }
      else { workspace = null; tabs = []; activeId = ''; focusedId = ''; paneRefs = {}; }
    } catch (e) { error = e instanceof Error ? e.message : "Could not finish closing."; }
    finally {
      preparing = false;
    }
  }
  function keydown(event: KeyboardEvent) {
    if (!workspace || event.defaultPrevented || event.isComposing || blocked) return;
    if (resizing) { if (event.metaKey || event.ctrlKey) event.preventDefault(); return; }
    if (settingsOpen) {
      if (((event.metaKey || event.ctrlKey) && ['k','t','w'].includes(event.key.toLowerCase())) || (event.ctrlKey && event.key === 'Tab')) event.preventDefault();
      return;
    }
    if (event.key === 'Escape' && compactSidebar && sidebarOpen && !searchMode) {
      event.preventDefault(); sidebarOpen = false;
      document.querySelector<HTMLButtonElement>('.sidebar-toggle')?.focus();
      return;
    }
    const command = event.metaKey || event.ctrlKey;
    if (event.key === 'Escape' && visiblePeek && !searchMode && !document.querySelector('dialog[open]') && !hasLiquidOverlayWithin(document.getElementById(`pane-${visiblePeek.id}`))) {
      // Let editor menus consume Escape before closing their containing peek.
      const id = visiblePeek.id; queueMicrotask(() => { if (!event.defaultPrevented && !event.isComposing) void closePeek(id); });
      return;
    }
    if (event.key === 'Tab' && !command && activePeek?.presentation === 'center' && !searchMode && !document.querySelector('dialog[open]')) {
      const panel = document.getElementById(`pane-${activePeek.id}`);
      const focusable = [...(panel?.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled), textarea:not(:disabled), select:not(:disabled), a[href], [tabindex="0"], [contenteditable="true"]') ?? [])].filter(node => node.getClientRects().length && !node.closest('[hidden], [inert]'));
      const index = focusable.indexOf(document.activeElement as HTMLElement);
      if (index < 0 || (event.shiftKey && index === 0) || (!event.shiftKey && index === focusable.length - 1)) { event.preventDefault(); focusable[event.shiftKey ? focusable.length - 1 : 0]?.focus(); }
    }
    if (command && ['k', 't'].includes(event.key.toLowerCase())) {
      event.preventDefault(); if (!searchMode) openSearch(event.key.toLowerCase() === 't' ? 'new' : 'current');
    } else if (command && event.key.toLowerCase() === 'w' && !searchMode) {
      event.preventDefault(); if (visiblePeek && focusedId === visiblePeek.id) void closePeek(visiblePeek.id); else void closeTab(activeId);
    } else if (event.ctrlKey && event.key === 'Tab' && !searchMode) {
      event.preventDefault(); const index = rootTabs.findIndex(tab => tab.id === activeId);
      activateTab(rootTabs[(index + (event.shiftKey ? -1 : 1) + rootTabs.length) % rootTabs.length].id, true);
    }
  }
  function tabKey(event: KeyboardEvent, id: string) {
    if (event.metaKey || event.ctrlKey || !['ArrowLeft','ArrowRight','Home','End'].includes(event.key)) return;
    event.preventDefault(); const index = rootTabs.findIndex(tab => tab.id === id);
    const next = event.key === 'Home' ? 0 : event.key === 'End' ? rootTabs.length - 1 : (index + (event.key === 'ArrowRight' ? 1 : -1) + rootTabs.length) % rootTabs.length;
    activateTab(rootTabs[next].id, true);
  }
  function beforeUnload(event: BeforeUnloadEvent) { if (protectedState) { event.preventDefault(); event.returnValue = ''; } }
  onMount(() => {
    const focusRoot = document.documentElement;
    const pointerFocus = () => { focusRoot.dataset.focusModality = 'pointer'; };
    const keyboardFocus = (event: KeyboardEvent) => { if (!['Shift', 'Control', 'Alt', 'Meta'].includes(event.key)) focusRoot.dataset.focusModality = 'keyboard'; };
    focusRoot.dataset.focusModality = 'keyboard';
    document.addEventListener('pointerdown', pointerFocus, true);
    document.addEventListener('keydown', keyboardFocus, true);
    const updateMotion = () => { reduceMotion = motionMedia.matches; };
    motionMedia.addEventListener('change', updateMotion);
    updateMotion();
    const updateSystemTheme = () => { systemDark = systemThemeMedia?.matches ?? true; };
    systemThemeMedia?.addEventListener('change', updateSystemTheme);
    updateSystemTheme();
    const media = window.matchMedia('(max-width: 760px)');
    const resizeSidebar = () => { compactSidebar = media.matches; sidebarOpen = false; };
    media.addEventListener('change', resizeSidebar);
    const resized = () => { viewportWidth = window.innerWidth; };
    window.addEventListener('resize', resized);
    let disposed = false; let unlisten: (() => void) | undefined;
    if (isTauri()) void listen('noteka-close-requested', () => { void leaveWorkspace(true); }).then(fn => { if (disposed) fn(); else unlisten = fn; });
    return () => { disposed = true; unlisten?.(); document.removeEventListener('pointerdown', pointerFocus, true); document.removeEventListener('keydown', keyboardFocus, true); delete focusRoot.dataset.focusModality; motionMedia.removeEventListener('change', updateMotion); systemThemeMedia?.removeEventListener('change', updateSystemTheme); media.removeEventListener('change', resizeSidebar); window.removeEventListener('resize', resized); };
  });
</script>

<svelte:window onkeydown={keydown} onbeforeunload={beforeUnload} />

<div class="app-window" class:resizing-panels={resizing !== null} style:--sidebar-width={`${sidebarWidth}px`} style:--side-peek-width={`${sidePeekWidth}px`}>
<WindowTitlebar workspaceOpen={workspace !== null} {sidebarVisible} blocked={blocked || resizing !== null || searchMode !== null || settingsOpen || activePeek?.presentation === 'center'} ontoggle={toggleSidebar} onclose={() => leaveWorkspace(true)} onerror={(message) => { error = message; }}>
    <div class="workspace-tabs" role="tablist" aria-label="Open tabs">{#each rootTabs as tab (tab.id)}<div class="workspace-tab" class:active={tab.id === activeId}><button role="tab" id={`tab-${tab.id}`} aria-selected={tab.id === activeId} aria-controls={`pane-${tab.id}`} tabindex={tab.id === activeId ? 0 : -1} disabled={blocked || activePeek?.presentation === 'center'} onclick={() => activateTab(tab.id)} onkeydown={(event) => tabKey(event, tab.id)} title={tab.state.title}>{#if tab.state.sectionPage}<SectionIcon section={tab.state.section} size={15} />{:else}<AppIcon name={tab.state.icon} color={tab.state.color} size={15} />{/if}<span class="tab-title">{tab.state.title}</span>{#if tab.state.dirty || tabs.some(child => child.ownerId === tab.id && child.state.dirty)}<span class="tab-dirty" aria-label="Unsaved changes">●</span>{/if}</button><button class="tab-close" aria-label={`Close tab ${tab.state.title}`} title="Close tab" disabled={blocked || activePeek?.presentation === 'center'} onclick={() => void closeTab(tab.id)}><AppIcon name="close" size={14} /></button></div>{/each}</div><button class="icon-button new-tab" aria-label="New tab" title={`New tab (${shortcutLabel('T')})`} disabled={blocked || activePeek?.presentation === 'center'} onclick={event => openSearch('new', event.currentTarget)}><AppIcon name="plus" size={17} /></button>
  </WindowTitlebar>

{#if !workspace}
  <main class="welcome">
    <div class="brandmark"><NotekaMark size={68} /></div><p class="eyebrow">YOUR SPACE</p><h1 class="noteka-wordmark">Noteka</h1>
    <p>A place for your notes and what comes next.<br />No accounts. No models. Your data stays local.</p>
    <form onsubmit={(event) => { event.preventDefault(); void selectWorkspace(false); }}>
      <label for="workspace-path">Workspace path</label><input id="workspace-path" class="glass-field" bind:value={path} placeholder="Leave blank to use the Noteka data directory" disabled={working} />
      <div class="actions"><button class="primary glass-action" type="button" disabled={working} onclick={() => void selectWorkspace(true)}>Create workspace</button><button class="glass-secondary" disabled={working}>Open workspace</button></div>
    </form>
    {#if error}<p class="error" role="alert">{error}</p>{/if}<small>Local editor · No external services</small>
  </main>
{:else}
  <div class="desktop-shell" class:sidebar-open={sidebarOpen} class:sidebar-collapsed={sidebarCollapsed}>
    {#if compactSidebar}<button class="sidebar-backdrop" class:visible={sidebarOpen} aria-label="Close sidebar" aria-hidden={!sidebarOpen} inert={!sidebarOpen} tabindex="-1" onclick={() => { sidebarOpen = false; }}></button>{/if}
    <aside class="desktop-sidebar" class:sidebar-narrow={sidebarWidth < 230} id="workspace-sidebar" aria-label="Workspace" aria-hidden={!sidebarVisible} inert={!sidebarVisible || activePeek?.presentation === 'center'}>
      <div class="sidebar-scroll">
      <div class="workspace-brand"><NotekaMark size={28} /><span class="noteka-wordmark">Noteka</span></div>
      <div class="sidebar-tools"><button use:sidebarIconMotion class="sidebar-search" aria-label="Search" title={`Search (${shortcutLabel('K')})`} disabled={blocked || activeTab?.state.busy} onclick={event => openSearch('current', event.currentTarget)}><AppIcon name="search" size={15} /><span>Search</span><kbd>{shortcutLabel('K')}</kbd></button><button use:sidebarIconMotion class="icon-button collections-shortcut" aria-label="Collections" title="Collections" disabled={blocked || activeTab?.state.busy} onclick={() => void navigate('collections')}><AppIcon name="collections" size={18} /></button></div>
      <p class="sidebar-caption">WORKSPACE</p>
      <nav aria-label="Sections">{#if activeSectionIndex >= 0}<span class="sidebar-nav-indicator" style={`--active-index:${activeSectionIndex}`} aria-hidden="true"></span>{/if}{#each PRIMARY_SECTIONS as section}<button use:sidebarIconMotion class:active={activeTab?.state.section === section.id} aria-current={activeTab?.state.section === section.id ? 'page' : undefined} title={section.label} disabled={blocked || activeTab?.state.busy} onclick={() => void navigate(section.id)}><span class="nav-icon" aria-hidden="true"><SectionIcon section={section.id} size={18} /></span><span class="sidebar-nav-label">{section.label}</span></button>{/each}</nav>
      <div class="sidebar-bottom"><button class="sidebar-settings" aria-haspopup="dialog" aria-controls={settingsOpen ? 'settings-dialog' : undefined} aria-expanded={settingsOpen} disabled={blocked} onclick={openSettings}><AppIcon name="settings" size={18} /><span>Settings</span></button></div>
      </div>
      <PanelResizeHandle label="Resize sidebar" controls="workspace-sidebar" edge="right" width={sidebarWidth} min={sidebarLimits.min} max={sidebarLimits.max} disabled={!sidebarVisible || blocked || searchMode !== null || settingsOpen} onstart={() => beginResize('sidebar')} onresize={width => resizePanel('sidebar', width)} onend={endResize} />
    </aside>
    <section class="desktop-workspace" aria-label="Workspace content">

      {#if error}<div class="shell-error" role="alert">{error}<button aria-label="Dismiss notification" onclick={() => { error = ''; }}><AppIcon name="close" size={14} /></button></div>{/if}
      {#if panelWidthError}<div class="shell-error" role="status">{panelWidthError}<button aria-label="Dismiss width notification" onclick={() => { panelWidthError = ''; }}><AppIcon name="close" size={14} /></button></div>{/if}
      <div class="workspace-panels" class:has-side-peek={sideVisible}>
        {#if activePeek?.presentation === 'center'}<button class="peek-backdrop" aria-label="Close center peek" tabindex="-1" disabled={blocked || searchMode !== null || settingsOpen} onclick={() => { if (activePeek) void closePeek(activePeek.id); }}></button>{/if}
        {#each tabs as tab (tab.id)}
          {@const shown = tab.ownerId ? tab.ownerId === activeId : tab.id === activeId}
          {@const visiblyShown = shown && !(tab.ownerId && tab.presentation === 'side' && (tab.sideHidden || tab.closing))}
          {@const covered = !tab.ownerId && activePeek?.presentation === 'full'}
          <div class="workspace-tab-panel" use:liquidSurface={{ enabled: !!tab.ownerId && tab.presentation === 'center', anchor: () => tab.returnFocus }} in:liquidTransition|global out:liquidTransition|global class:active={tab.id === activeId} class:peek={!!tab.ownerId} class:peek-side={!!tab.ownerId && tab.presentation === 'side'} class:peek-center={!!tab.ownerId && tab.presentation === 'center'} class:peek-full={!!tab.ownerId && tab.presentation === 'full'} class:peek-hidden={!!tab.ownerId && tab.presentation === 'side' && !!(tab.sideHidden || tab.closing)} class:peek-narrow={sidePeekWidth < 480} id={`pane-${tab.id}`} role={tab.ownerId ? tab.presentation === 'center' ? 'dialog' : 'region' : 'tabpanel'} aria-modal={tab.ownerId && tab.presentation === 'center' ? true : undefined} aria-label={tab.ownerId ? `Details: ${tab.state.title}` : undefined} aria-labelledby={!tab.ownerId ? `tab-${tab.id}` : undefined} aria-hidden={(!visiblyShown || covered) && !tab.state.guarded ? true : undefined} hidden={!shown} inert={!visiblyShown || (!tab.ownerId && !tab.state.guarded && (covered || activePeek?.presentation === 'center'))} onfocusin={() => { focusedId = tab.id; }} onpointerdown={() => { focusedId = tab.id; }}>
            <WorkspacePane bind:this={paneRefs[tab.id]} {workspace} paneId={tab.id} active={visiblyShown && focusedId === tab.id} visible={visiblyShown && !covered} sessionActive={shown} blocked={searchMode !== null || preparing || settingsOpen} initialTarget={tab.initialTarget} onstate={updateTab} onrequestdetail={!tab.ownerId ? target => openDetail(tab.id, target) : undefined} onnavigatesection={tab.ownerId ? async section => { await navigate(section); } : undefined} peekMode={tab.presentation ?? 'full'} {defaultPeekMode} {peekPreferenceError} onpresent={mode => presentPane(tab.id, mode)} ondefaultpeek={changeDefaultPeekMode} onclosepeek={tab.ownerId ? () => { void closePeek(tab.id); } : undefined} />
            {#if tab.ownerId && tab.presentation === 'side'}<PanelResizeHandle label="Resize side peek" controls={`pane-${tab.id}`} edge="left" width={sidePeekWidth} min={sidePeekLimits.min} max={sidePeekLimits.max} disabled={!visiblyShown || blocked || searchMode !== null || settingsOpen} onstart={() => beginResize('sidePeek')} onresize={width => resizePanel('sidePeek', width)} onend={endResize} />{/if}
          </div>
        {/each}
      </div>
    </section>
  </div>
  {#if searchMode}<SearchDialog anchor={searchAnchor} workspace={workspace.path} mode={searchMode} onclose={() => { searchMode = null; }} onchoose={chooseTarget} />{/if}
  {#if settingsOpen}<SettingsDialog anchor={settingsAnchor} {primaryColor} {primaryColorError} onprimarycolorchange={changePrimaryColor} workspace={workspace.path} {theme} {themeError} {smoothWheel} {smoothWheelError} onsmoothwheelchange={changeSmoothWheel} {blocked} onthemechange={changeTheme} onworkspacechange={() => { void leaveWorkspace(); }} onclose={() => { settingsOpen = false; }} />{/if}
{/if}
</div>
