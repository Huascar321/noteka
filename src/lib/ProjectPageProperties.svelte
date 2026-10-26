<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import type { HomeSummary } from './contracts.generated';
  let { payload, summary, busy = false, error = '', disabled = false, onchange, onretry }: {
    payload: Record<string, unknown>;
    summary: Extract<HomeSummary, { kind: 'project' }> | null;
    busy?: boolean; error?: string; disabled?: boolean;
    onchange: () => void; onretry: () => void;
  } = $props();
  const states = [['planned','Planned'],['doing','Doing'],['ongoing','Ongoing'],['on_hold','On Hold'],['done','Done']];
  const state = $derived(String(payload.state ?? 'planned'));
  const progress = $derived(summary?.available && summary.total && (typeof summary.progress_tenths === 'number' || typeof summary.progress_percent === 'number') ? (summary.progress_tenths ?? summary.progress_percent! * 10) / 10 : null);
  function set(key: string, value: unknown) { payload[key] = value; onchange(); }
</script>

<fieldset class="project-page-properties" disabled={disabled}>
  <legend class="sr-only">Project properties</legend>
  <label><span class="project-property-label"><AppIcon name="circle" size={14} />Status</span><span class="project-detail-state" data-state={state}><span aria-hidden="true"></span><select aria-label="Project status" value={state} onchange={event => set('state', event.currentTarget.value)}>{#if !states.some(([value]) => value === state)}<option value={state} disabled>Estado no reconocido</option>{/if}{#each states as [value,label]}<option {value}>{label}</option>{/each}</select></span></label>
  <label><span class="project-property-label"><AppIcon name="calendar" size={14} />Target Deadline</span><input aria-label="Project end" type="date" value={String(payload.end_date ?? '')} onchange={event => set('end_date', event.currentTarget.value || null)} /></label>
  <div class="project-progress-property" role="group" aria-label="Project progress"><span class="project-property-label"><AppIcon name="sliders" size={14} />Progress</span>
    {#if busy}<span class="project-property-empty" role="status">Loading…</span>
    {:else if error}<span class="project-property-empty" role="alert">{error}</span><button class="project-summary-retry" onclick={onretry}>Retry progress</button>
    {:else if summary?.available && summary.total === 0}<span class="project-property-empty">No tasks</span>
    {:else if progress !== null}<span class="project-detail-progress"><span>{progress}%</span><progress max="100" value={progress} aria-label="Project task progress"></progress><small>{summary?.done}/{summary?.total}</small></span>
    {:else}<span class="project-property-empty">Unavailable</span>{/if}
  </div>
</fieldset>
