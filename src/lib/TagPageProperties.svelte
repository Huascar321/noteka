<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import type { HomeSummary } from './contracts.generated';
  let { payload, summary, busy = false, error = '', disabled = false, onchange, onretry, onnotes }: {
    payload: Record<string, unknown>;
    summary: Extract<HomeSummary, { kind: 'tag' }> | null;
    busy?: boolean; error?: string; disabled?: boolean;
    onchange: () => void; onretry: () => void; onnotes: () => void;
  } = $props();
  const types = [['area','Area'],['resource','Resource'],['entity','Entity'],['topic','Topic']];
  const classification = $derived(String(payload.classification ?? 'topic'));
  function set(key: string, value: unknown) { payload[key] = value; onchange(); }
</script>

<fieldset class="tag-page-properties" disabled={disabled}>
  <legend class="sr-only">Tag properties</legend>
  <label><span class="tag-property-label"><AppIcon name="circle" size={14} />Type</span><span class="tag-detail-type" data-type={classification}><span aria-hidden="true"></span><select aria-label="Classification" value={classification} onchange={event => set('classification', event.currentTarget.value)}>{#if !types.some(([value]) => value === classification)}<option value={classification} disabled>Unrecognized type</option>{/if}{#each types as [value,label]}<option {value}>{label}</option>{/each}</select></span></label>
  <label><span class="tag-property-label"><AppIcon name="star" size={14} />Favorite</span><input aria-label="Favorite tag" type="checkbox" checked={payload.favorite === true} onchange={event => set('favorite', event.currentTarget.checked)} /></label>
  <div class="tag-note-count-property" role="group" aria-label="Tag note count"><span class="tag-property-label"><AppIcon name="file-text" size={14} />Note Count</span>
    {#if busy}<span class="tag-property-empty" role="status">Loading…</span>
    {:else if error}<span class="tag-property-empty" role="alert">{error}</span><button class="tag-summary-retry" onclick={onretry}>Retry note count</button>
    {:else if summary?.available && typeof summary.note_count === 'number'}<button class="tag-note-count" aria-label={`Ver ${summary.note_count} related notes`} onclick={onnotes}>{summary.note_count} notes</button>
    {:else}<span class="tag-property-empty">Unavailable</span>{/if}
  </div>
</fieldset>
