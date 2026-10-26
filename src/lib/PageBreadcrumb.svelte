<script lang="ts">
  import SectionIcon from './SectionIcon.svelte';
  import AppIcon from './AppIcon.svelte';
  import { sectionInfo, type Section } from './navigation';
  import type { EntityKind } from './contracts.generated';
  let { section, record, disabled = false, onnavigate }: {
    section: Section; record?: { kind: EntityKind; owner?: Section; title: string; icon: string; color?: string } | null;
    disabled?: boolean; onnavigate: (section: Section) => Promise<boolean>;
  } = $props();
  const owner = $derived(record?.owner ?? record?.kind ?? section);
  const info = $derived(sectionInfo(owner));
</script>

<nav class="page-breadcrumb" aria-label="Breadcrumb">
  <ol>
    <li class="breadcrumb-home">{#if owner === 'home' && !record}<span aria-current="page"><SectionIcon section="home" size={16} /><span>Home</span></span>{:else}<button {disabled} onclick={() => void onnavigate('home')} title="Home"><SectionIcon section="home" size={16} /><span>Home</span></button>{/if}</li>
    {#if owner !== 'home'}<li class="breadcrumb-section"><span class="breadcrumb-separator" aria-hidden="true">/</span>{#if record}<button {disabled} onclick={() => void onnavigate(owner)} title={info.label}><SectionIcon section={owner} size={16} /><span>{info.label}</span></button>{:else}<span aria-current="page"><SectionIcon section={owner} size={16} /><span>{info.label}</span></span>{/if}</li>{/if}
    {#if record}<li class="breadcrumb-record"><span class="breadcrumb-separator" aria-hidden="true">/</span><span aria-current="page" title={record.title}><AppIcon name={record.icon} color={record.color} size={17} /><span>{record.title}</span></span></li>{/if}
  </ol>
</nav>
