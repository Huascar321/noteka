<script lang="ts">
  import { cubicOut } from 'svelte/easing';
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

  function collapse(node: HTMLElement, { target, duration = 280 }: { target: string; duration?: number }) {
    const root = node.closest('.page-breadcrumb');
    const targetPill = root?.querySelector<HTMLElement>(target);
    const sourcePill = node.querySelector<HTMLElement>('button, [aria-current="page"]') ?? node;
    const from = sourcePill.getBoundingClientRect();
    const to = targetPill?.getBoundingClientRect();
    const dx = to ? to.left - from.left : -from.width;
    const dy = to ? to.top + to.height / 2 - (from.top + from.height / 2) : 0;
    return {
      duration,
      easing: cubicOut,
      css: (t: number) => `transform:translate(${dx * (1 - t)}px,${dy * (1 - t)}px) scale(${.94 + .06 * t});opacity:${t};`
    };
  }

  function spawn(node: HTMLElement, { target, duration = 280 }: { target: string; duration?: number }) {
    const root = node.closest('.page-breadcrumb');
    const targetPill = root?.querySelector<HTMLElement>(target);
    const sourcePill = node.querySelector<HTMLElement>('button, [aria-current="page"]') ?? node;
    const to = sourcePill.getBoundingClientRect();
    const from = targetPill?.getBoundingClientRect();
    const dx = from ? from.left - to.left : 0;
    const dy = from ? from.top + from.height / 2 - (to.top + to.height / 2) : 0;
    return {
      duration,
      easing: cubicOut,
      css: (t: number) => `transform:translate(${dx * (1 - t)}px,${dy * (1 - t)}px) scale(${.94 + .06 * t});opacity:${t};`
    };
  }

  function pop(node: HTMLElement, { duration = 280 }: { duration?: number } = {}) {
    return { duration, easing: cubicOut, css: (t: number) => `transform:scale(${.94 + .06 * t});` };
  }
</script>

<nav class="page-breadcrumb" aria-label="Breadcrumb">
  <ol>
    <li class="breadcrumb-home">{#if owner === 'home' && !record}<span aria-current="page" in:pop><SectionIcon section="home" size={16} /><span>Home</span></span>{:else}<button {disabled} onclick={() => void onnavigate('home')} title="Home"><SectionIcon section="home" size={16} /><span>Home</span></button>{/if}</li>
    {#if owner !== 'home'}<li class="breadcrumb-section" in:spawn={{ target: '.breadcrumb-home button, .breadcrumb-home [aria-current="page"]' }} out:collapse={{ target: '.breadcrumb-home button, .breadcrumb-home [aria-current="page"]' }}><span class="breadcrumb-separator" aria-hidden="true">/</span>{#if record}<button {disabled} onclick={() => void onnavigate(owner)} title={info.label}><SectionIcon section={owner} size={16} /><span>{info.label}</span></button>{:else}<span aria-current="page" in:pop><SectionIcon section={owner} size={16} /><span>{info.label}</span></span>{/if}</li>{/if}
    {#if record}<li class="breadcrumb-record" out:collapse={{ target: owner === 'home' ? '.breadcrumb-home button, .breadcrumb-home [aria-current="page"]' : '.breadcrumb-section button, .breadcrumb-section [aria-current="page"]' }}><span class="breadcrumb-separator" aria-hidden="true">/</span><span aria-current="page" title={record.title}><AppIcon name={record.icon} color={record.color} size={17} /><span>{record.title}</span></span></li>{/if}
  </ol>
</nav>
