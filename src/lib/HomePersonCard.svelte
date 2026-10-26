<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import PersonPhoto from './PersonPhoto.svelte';
  import type { HomeItem } from './contracts.generated';
  let { item, onopen, layout = 'gallery', checkIn = false }: { item: HomeItem; onopen: (id: string) => Promise<void>; layout?: 'gallery' | 'list' | 'pipeline'; checkIn?: boolean } = $props();
  const person = $derived(item.metadata.person);
</script>

<article class="person-card" data-layout={layout} data-person-id={item.entity.id}>
  <button class="person-card-open" onclick={() => void onopen(item.entity.id)} aria-label={`Open ${item.entity.title}`}>
    {#if layout === 'gallery'}<PersonPhoto id={person?.photo} position={person?.photo_position} />{/if}
    <span class="person-card-content">
      <span class="person-card-title"><AppIcon name={item.entity.icon ?? 'circle-user'} color={item.entity.icon_color ?? '#3183a6'} size={19} /><span>{item.entity.title}</span></span>
      {#if person?.relationships?.length}<span class="person-relationships">{#each person.relationships as label}<span class="person-relationship" data-relationship={label}>{label}</span>{/each}</span>{/if}
      {#if person?.company}<span class="person-company">{person.company}</span>{/if}
      {#if checkIn && person?.check_in}<time class="person-check-in" datetime={person.check_in}>{person.check_in}</time>{/if}
      {#if layout === 'pipeline' && person?.last_check_in}<span class="person-last-check-in">Last Check-In <time datetime={person.last_check_in}>{person.last_check_in}</time></span>{/if}
    </span>
  </button>
</article>
