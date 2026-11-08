<script lang="ts">
  import type { HomeReadState } from './home-reads';
  let { state, updating = false }: { state?: HomeReadState; updating?: boolean } = $props();
</script>

{#if updating}
  {#if state?.stale && state.page && state.busy}<span class="task-refresh-status" role="status" title="Updating; showing previous results.">Updating; showing previous results.</span>{/if}
{:else}
  {#if state?.stale && state.page && !state.busy}<p class="task-read-status" role="status">Showing previous results; they may be out of date.</p>{/if}
{/if}
