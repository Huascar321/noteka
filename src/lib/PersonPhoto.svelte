<script lang="ts">
  import { getContext } from 'svelte';
  import { loadIconImage, WORKSPACE_ICON_CONTEXT } from './visual-icon';
  let { id, position = 50 }: { id?: string | null; position?: number | null } = $props();
  const currentWorkspace = getContext<(() => string | undefined) | undefined>(WORKSPACE_ICON_CONTEXT);
  let source = $state('');
  $effect(() => {
    const workspace = currentWorkspace?.();
    const photo = id;
    source = '';
    let alive = true;
    if (photo && workspace) void loadIconImage(workspace, photo).then(url => { if (alive) source = url; }).catch(() => {});
    return () => { alive = false; };
  });
</script>

<span class="person-photo" class:person-photo-empty={!source} aria-hidden="true">{#if source}<img src={source} alt="" style:object-position={`center ${position ?? 50}%`} onerror={() => source = ''} />{/if}</span>
