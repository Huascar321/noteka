<script lang="ts">
  import { onMount } from 'svelte';
  import AppIcon from './AppIcon.svelte';
  import PrimaryColorPicker from './PrimaryColorPicker.svelte';
  import type { ThemePreference } from './theme';

  let { workspace, theme, themeError, smoothWheel, smoothWheelError, onsmoothwheelchange, primaryColor, primaryColorError, onprimarycolorchange, blocked, onthemechange, onworkspacechange, onclose }: {
    workspace: string;
    theme: ThemePreference;
    themeError: string;
    smoothWheel: boolean;
    smoothWheelError: string;
    onsmoothwheelchange: (enabled: boolean) => void;
    primaryColor: string;
    primaryColorError: string;
    onprimarycolorchange: (color: string) => void;
    blocked: boolean;
    onthemechange: (theme: ThemePreference) => void;
    onworkspacechange: () => void;
    onclose: () => void;
  } = $props();

  const sections = [
    { id: 'preferences', label: "Preferences", group: 'Personal', icon: 'sliders', title: "Preferences", description: "Choose how Noteka looks." },
    { id: 'workspace', label: 'General', group: 'Workspace', icon: 'settings', title: 'Workspace', description: "Manage your workspace." },
  ] as const;
  const items = [
    { id: 'theme', section: 'preferences', terms: 'preferences appearance theme system light dark' },
    { id: 'primary-color', section: 'preferences', terms: 'preferences appearance primary accent color colour custom hue saturation lightness blue violet pink orange green teal' },
    { id: 'smooth-wheel', section: 'preferences', terms: 'preferences interaction mouse wheel smooth scrolling scroll movement' },
    { id: 'path', section: 'workspace', terms: "workspace general location path" },
    { id: 'change-workspace', section: 'workspace', terms: "workspace general change open another" },
  ] as const;
  type Section = typeof sections[number]['id'];
  let active = $state<Section>('preferences');
  let query = $state('');
  let dialog: HTMLDialogElement;
  let search: HTMLInputElement;

  function normalize(value: string) { return value.toLocaleLowerCase('en').normalize('NFD').replace(/[\u0300-\u036f]/g, '').trim(); }
  const terms = $derived(normalize(query).split(/\s+/).filter(Boolean));
  const matches = $derived(items.filter(item => terms.every(term => item.terms.includes(term))));
  const visibleSections = $derived(sections.filter(section => matches.some(item => item.section === section.id)));
  const current = $derived(sections.find(section => section.id === active)!);
  const visible = (id: typeof items[number]['id']) => matches.some(item => item.id === id);

  $effect(() => {
    if (visibleSections.length && !visibleSections.some(section => section.id === active)) active = visibleSections[0].id;
  });
  function clearSearch() { query = ''; search.focus(); }
  function keydown(event: KeyboardEvent) {
    if (event.key !== 'Tab') return;
    const controls = [...dialog.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled), select:not(:disabled), a[href], [tabindex]:not([tabindex="-1"])')].filter(element => element.getClientRects().length);
    const first = controls[0], last = controls.at(-1);
    if (!first || !last) return;
    if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
    else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
  }

  onMount(() => {
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    dialog.showModal();
    return () => {
      dialog.close();
      if (previous?.isConnected) previous.focus();
    };
  });
</script>

<dialog bind:this={dialog} id="settings-dialog" class="modal settings-dialog glass-popup" aria-labelledby="settings-title" onkeydown={keydown} oncancel={(event) => { event.preventDefault(); onclose(); }}>
  <h2 id="settings-title" class="sr-only">Settings</h2>
  <aside class="settings-sidebar" aria-label="Settings navigation">
    <div class="settings-sidebar-heading"><AppIcon name="settings" size={18} /><span>Settings</span></div>
    <div class="settings-search">
      <AppIcon name="search" size={16} />
      <input bind:this={search} bind:value={query} aria-label="Search settings" placeholder="Search" autocomplete="off" type="search" />
      {#if query}<button class="icon-button" aria-label="Clear search" title="Clear search" onclick={clearSearch}><AppIcon name="close" size={13} /></button>{/if}
    </div>
    <nav class="settings-navigation" aria-label="Settings sections">
      {#each visibleSections as section (section.id)}
        <div class="settings-navigation-group">
          <p>{section.group}</p>
          <button class:active={active === section.id} aria-current={active === section.id ? 'page' : undefined} aria-controls="settings-page" onclick={() => { active = section.id; }}><AppIcon name={section.icon} size={18} /><span>{section.label}</span></button>
        </div>
      {/each}
    </nav>
  </aside>
  <section class="settings-main" id="settings-page" aria-labelledby="settings-page-title">
    <button class="icon-button settings-close" aria-label="Close Settings" title="Close (Esc)" onclick={onclose}><AppIcon name="close" size={18} /></button>
    {#if visibleSections.length}
      <header class="settings-page-header"><h2 id="settings-page-title">{current.title}</h2><p>{current.description}</p></header>
      {#if active === 'preferences'}
        {#if visible('primary-color') || visible('theme')}<section class="settings-group" aria-labelledby="settings-appearance-title">
          <h3 id="settings-appearance-title">Appearance</h3>
          {#if visible('primary-color')}
            <div class="settings-primary-color"><h4>Primary color</h4><p>Choose an accent for Noteka. Shades adapt to your theme for readability.</p><PrimaryColorPicker color={primaryColor} error={primaryColorError} onchange={onprimarycolorchange} /></div>
          {/if}
          {#if visible('theme')}
            <div class="settings-row settings-theme-row">
              <div><label for="settings-theme">Theme</label><p id="settings-theme-description">Follow your system or choose light or dark.</p></div>
              <select id="settings-theme" aria-label="Theme" aria-describedby={themeError ? 'settings-theme-description settings-theme-error' : 'settings-theme-description'} value={theme} onchange={(event) => onthemechange(event.currentTarget.value as ThemePreference)}><option value="system">System</option><option value="dark">Dark</option><option value="light">Light</option></select>
              {#if themeError}<p id="settings-theme-error" class="settings-error" role="status">{themeError}</p>{/if}
            </div>
          {/if}
        </section>{/if}
        {#if visible('smooth-wheel')}
          <section class="settings-group" aria-labelledby="settings-interaction-title">
            <h3 id="settings-interaction-title">Interaction</h3>
            <div class="settings-row settings-scroll-row">
              <div><label for="settings-smooth-wheel">Smooth mouse scrolling</label><p id="settings-smooth-wheel-description">Soften mouse wheel steps. Turn off to use system scrolling.</p></div>
              <input id="settings-smooth-wheel" type="checkbox" checked={smoothWheel} aria-describedby={smoothWheelError ? 'settings-smooth-wheel-description settings-smooth-wheel-error' : 'settings-smooth-wheel-description'} onchange={event => onsmoothwheelchange(event.currentTarget.checked)} />
              {#if smoothWheelError}<p id="settings-smooth-wheel-error" class="settings-error" role="status">{smoothWheelError}</p>{/if}
            </div>
          </section>
        {/if}
      {:else}
        <section class="settings-group" aria-labelledby="settings-workspace-title">
          <h3 id="settings-workspace-title">General</h3>
          {#if visible('path')}<div class="settings-location"><h4>Workspace path</h4><p class="settings-path" title={workspace}>{workspace}</p></div>{/if}
          {#if visible('change-workspace')}<div class="settings-row settings-workspace-row"><div><h4>Workspace</h4><p>Open another workspace.</p></div><button class="settings-change-workspace" disabled={blocked} onclick={onworkspacechange}>Change workspace</button></div>{/if}
        </section>
      {/if}
    {:else}
      <div class="settings-empty"><AppIcon name="search" size={28} /><h2 id="settings-page-title">No results</h2><p role="status">No settings match “{query.trim()}”.</p><button onclick={clearSearch}>Clear search</button></div>
    {/if}
  </section>
</dialog>
