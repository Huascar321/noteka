<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import { entityIcon, entityIconColor } from './entity-appearance';
  import type { Entity, HomeSummary } from './contracts.generated';
  let { payload, tags, summary, busy = false, error = '', disabled = false, canRelate = false, onchange, onopen, oneditrelations, onretry }: {
    payload: Record<string, unknown>; tags: Entity[];
    summary: Extract<HomeSummary, { kind: 'goal' }> | null;
    busy?: boolean; error?: string; disabled?: boolean; canRelate?: boolean;
    onchange: () => void; onopen: (entity: Entity) => Promise<void>;
    oneditrelations: (anchor: HTMLElement) => void; onretry: () => void;
  } = $props();
  const goal = $derived(payload.goal as Record<string, unknown>);
  const states = [['active','Active'],['dream','Dream'],['achieved','Achieved']];
  const state = $derived(String(goal?.state ?? 'dream'));
  const progress = $derived(summary?.available && typeof summary.total === 'number' && summary.total > 0 && typeof summary.progress_tenths === 'number' ? summary.progress_tenths / 10 : null);
  function setState(value: string) { payload.goal = { ...goal, state: value }; onchange(); }
</script>

<fieldset class="goal-page-properties" disabled={disabled}>
  <legend class="sr-only">Goal properties</legend>
  <label><span class="goal-property-label"><AppIcon name="circle" size={14} />Status</span><span class="goal-detail-state" data-state={state}><span aria-hidden="true"></span><select aria-label="Goal status" value={state} onchange={event => setState(event.currentTarget.value)}>{#if !states.some(([value]) => value === state)}<option value={state} disabled>Estado no reconocido</option>{/if}{#each states as [value,label]}<option {value}>{label}</option>{/each}</select></span></label>
  <div role="group" aria-label="Goal tags"><span class="goal-property-label"><AppIcon name="arrow-up-right" size={14} />Tag</span><div class="goal-tag-value">{#each tags as tag (tag.id)}<button class="goal-detail-tag" title={tag.title} onclick={() => void onopen(tag)}><AppIcon name={entityIcon(tag.kind,tag.title,tag.payload)} color={entityIconColor(tag.payload)} size={16} /><span>{tag.title}{tag.archived_at ? " · archived" : ''}</span></button>{/each}{#if !tags.length}<span class="goal-property-empty">Empty</span>{/if}{#if canRelate}<button class="icon-button" aria-label="Edit Goal tag" title="Edit relations" aria-haspopup="dialog" onclick={event => oneditrelations(event.currentTarget)}><AppIcon name="chevron-down" size={13} /></button>{/if}</div></div>
  <div role="group" aria-label="Goal progress"><span class="goal-property-label"><AppIcon name="sliders" size={14} />Progress</span>
    {#if busy}<span class="goal-property-empty" role="status">Loading…</span>
    {:else if error}<span class="goal-property-empty" role="alert">{error}</span><button class="goal-summary-retry" onclick={onretry}>Retry Goal progress</button>
    {:else if summary?.available && summary.total === 0}<span class="goal-property-empty">No milestones</span>
    {:else if progress !== null}<span class="goal-detail-progress"><span>{progress}%</span><progress max="100" value={progress} aria-label="Goal milestone progress"></progress><small>{summary?.done}/{summary?.total}</small></span>
    {:else}<span class="goal-property-empty">Unavailable</span>{/if}
  </div>
</fieldset>
