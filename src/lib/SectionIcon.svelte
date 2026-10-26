<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import { sectionInfo, type Section } from './navigation';
  let { section, size = 18 }: { section: Section; size?: number } = $props();
</script>

<span class="section-symbol" data-section-icon={section} aria-hidden="true"><AppIcon name={sectionInfo(section).icon} {size} /></span>
