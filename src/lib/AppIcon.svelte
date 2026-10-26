<script lang="ts">
  import { getContext } from 'svelte';
  import { ICONS, iconName } from './icon-set';
  import { imageIconId, loadIconImage, WORKSPACE_ICON_CONTEXT } from './visual-icon';
  let {
    name = 'file-text',
    size = 18,
    color = undefined,
    strokeWidth = 1.8,
    class: className = '',
    workspace = undefined,
  }: { name?: string; size?: number; color?: string; strokeWidth?: number; class?: string; workspace?: string } = $props();
  const Icon = $derived(ICONS[iconName(name)]);
  const instanceId = $props.id();
  const scope = `noteka-${instanceId.replace(/[^a-zA-Z0-9_-]/g, '')}`;
  const attributes = $derived(Object.fromEntries(Object.entries(Icon.attrs).map(([key,value]) => [key, value.replaceAll('__NOTEKA_ICON_SCOPE__', scope)])));
  const markup = $derived(Icon.body.replaceAll('__NOTEKA_ICON_SCOPE__', scope));
  const currentWorkspace = getContext<(() => string | undefined) | undefined>(WORKSPACE_ICON_CONTEXT);
  let source = $state('');
  $effect(() => {
    const id = imageIconId(name);
    const scope = workspace ?? currentWorkspace?.();
    source = '';
    let alive = true;
    if (id && scope) void loadIconImage(scope, id).then(url => { if (alive) source = url; }).catch(() => {});
    return () => { alive = false; };
  });
</script>

{#if source}<img src={source} alt="" width={size} height={size} class={className} style:object-fit="contain" style:flex-shrink="0" onerror={() => source = ''} />{:else}<svg {...attributes} width={size} height={size} stroke-width={strokeWidth} style:color={color} class={className} data-app-icon={iconName(name)} aria-hidden="true">{@html markup}</svg>{/if}
