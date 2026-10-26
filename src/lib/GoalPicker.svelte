<script lang="ts">
  import { onMount } from 'svelte';
  import AppIcon from './AppIcon.svelte';
  import { operate } from './api';
  import { HomeReader, type HomeReadState } from './home-reads';
  import { ReadQueue } from './embedded-reads';
  import type { HomePage } from './contracts.generated';
  let { workspace, value, disabled = false, onchange }: {
    workspace: string; value?: string | null; disabled?: boolean; onchange: (id: string | null) => void;
  } = $props();
  let readState = $state<HomeReadState>();
  let reader: HomeReader | undefined;
  onMount(() => {
    reader = new HomeReader('goals_activity', new ReadQueue(1), query => operate<HomePage>({ operation: 'home.query', input: query }, workspace), state => readState = state, 100, true);
    void reader.load(); return () => reader?.dispose();
  });
</script>

<div class="goal-picker"><label>Goal<select aria-label="Milestone Goal" {disabled} value={value ?? ''} onchange={(event) => onchange(event.currentTarget.value || null)}><option value="">No Goal</option>{#if value && !readState?.page?.items.some(item => item.entity.id === value)}<option value={value}>Current Goal · outside this page</option>{/if}{#each readState?.page?.items ?? [] as item (item.entity.id)}<option value={item.entity.id}>{item.entity.title}</option>{/each}</select></label><button type="button" class="task-icon-action" aria-label="Refresh Goal options" title="Refresh available Goals" disabled={disabled || readState?.busy} onclick={() => void reader?.load()}><AppIcon name="refresh" size={14} /></button>{#if readState?.busy}<span class="metadata" role="status">Loading Goals…</span>{/if}{#if readState?.error}<p class="error" role="alert">{readState.error}</p><button type="button" onclick={() => void reader?.retry()}>Retry Goals</button>{/if}{#if readState?.page?.next_cursor}<button type="button" disabled={readState.busy} onclick={() => void reader?.load(undefined, readState?.page?.next_cursor ?? undefined)}>Load more Goals</button>{/if}</div>
