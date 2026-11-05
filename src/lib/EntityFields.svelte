<script lang="ts">
  import { noteTypes } from './note-types';
  import type { EntityKind } from './contracts.generated';
  import GoalPicker from './GoalPicker.svelte';
  let { kind, payload, workspace = '', disabled = false, taskSecondaryOnly = false, noteSecondaryOnly = false, projectSecondaryOnly = false, tagSecondaryOnly = false, goalSecondaryOnly = false, onchange }: { kind: EntityKind; payload: Record<string, unknown>; workspace?: string; disabled?: boolean; taskSecondaryOnly?: boolean; noteSecondaryOnly?: boolean; projectSecondaryOnly?: boolean; tagSecondaryOnly?: boolean; goalSecondaryOnly?: boolean; onchange: () => void } = $props();
  const visual = ['icon', 'icon_color'];
  const contactFields = [{key:'job_title',label:'Title'}, {key:'location',label:'Location'}, {key:'phone',label:'Phone'}, {key:'email',label:'Email'}, {key:'instagram',label:'Instagram'}, {key:'linkedin',label:'LinkedIn'}, {key:'twitter',label:'Twitter/X'}, {key:'website',label:'Website'}] as const;
  const allowed: Record<EntityKind, string[]> = { note: ['inbox', 'note_type', 'favorite', 'source', 'goal', 'milestone', ...visual], task: ['state', 'priority', 'inbox', 'planned_date', 'due_date', ...visual], project: ['state', 'purpose', 'start_date', 'end_date', 'source', 'board_order', ...visual], tag: ['classification', 'favorite', 'source', 'source_activity_at', ...visual], person: ['relationships', 'company', 'photo', 'photo_position', 'birthday', 'check_in', 'last_check_in', 'pipeline_status', 'source', 'gallery_order', 'interests', ...contactFields.map(field => field.key), ...visual] };
  const unrepresented = $derived(Object.keys(payload).filter(k => !allowed[kind].includes(k)));
  function set(key: string, value: unknown) { payload[key] = value; onchange(); }
  let interestInput = $state('');
  let interestError = $state('');
  const interests = $derived(Array.isArray(payload.interests) ? payload.interests : []);
  const interestsEditable = $derived(payload.interests == null || (Array.isArray(payload.interests) && payload.interests.every(value => typeof value === 'string')));
  function addInterest() {
    const label = interestInput.trim();
    if (!interestsEditable) { interestError = 'Existing interests cannot be interpreted.'; return; }
    if (!label) return;
    if (new TextEncoder().encode(label).length > 128 || /[\u0000-\u001f\u007f-\u009f]/.test(label)) { interestError = 'Use at most 128 UTF-8 bytes without control characters.'; return; }
    if (interests.includes(label)) { interestError = 'This interest is already listed.'; return; }
    if (interests.length >= 32) { interestError = 'A person can have at most 32 interests.'; return; }
    set('interests', [...interests, label]); interestInput = ''; interestError = '';
  }
  function removeInterest(index: number) { if (interestsEditable) { set('interests', interests.filter((_, i) => i !== index)); interestError = ''; } }
  const goal = $derived(kind === 'note' && payload.goal && typeof payload.goal === 'object' && !Array.isArray(payload.goal) ? payload.goal as Record<string, unknown> : null);
  const milestone = $derived(kind === 'note' && payload.milestone && typeof payload.milestone === 'object' && !Array.isArray(payload.milestone) ? payload.milestone as Record<string, unknown> : null);
  function setNested(type: 'goal' | 'milestone', key: string, value: unknown) { set(type, { ...payload[type] as Record<string, unknown>, [key]: value }); }
</script>

<fieldset class="entity-fields" disabled={disabled}>
  <legend class="sr-only">Properties</legend>
  {#if goal}
    {#if !goalSecondaryOnly}<label>Status<select aria-label="Goal status" value={String(goal.state ?? 'dream')} onchange={(event) => setNested('goal', 'state', event.currentTarget.value)}><option value="active">Active</option><option value="dream">Dream</option><option value="achieved">Achieved</option></select></label>{/if}
    <label>Goal Set<input aria-label="Goal Set" type="date" value={String(goal.goal_set ?? '')} onchange={(event) => setNested('goal', 'goal_set', event.currentTarget.value || null)} /></label>
    <label>Target Deadline<input aria-label="Goal Target Deadline" type="date" value={String(goal.target_deadline ?? '')} onchange={(event) => setNested('goal', 'target_deadline', event.currentTarget.value || null)} /></label>
    <label>Achieved<input aria-label="Date achieved" type="date" value={String(goal.achieved_on ?? '')} onchange={(event) => setNested('goal', 'achieved_on', event.currentTarget.value || null)} /></label>
  {:else if milestone}
    {#if workspace}<GoalPicker {workspace} value={typeof milestone.goal_id === 'string' ? milestone.goal_id : null} {disabled} onchange={(id) => setNested('milestone', 'goal_id', id)} />{/if}
    <label>Target Deadline<input aria-label="Milestone Target Deadline" type="date" value={String(milestone.target_deadline ?? '')} onchange={(event) => setNested('milestone', 'target_deadline', event.currentTarget.value || null)} /></label>
    <label>Date Completed<input aria-label="Date Completed" type="date" value={String(milestone.completed_on ?? '')} onchange={(event) => setNested('milestone', 'completed_on', event.currentTarget.value || null)} /></label>
  {:else if kind === 'note' && !noteSecondaryOnly}
    <label>Type<select aria-label="Note type" value={String(payload.note_type ?? '')} onchange={(e) => set('note_type', e.currentTarget.value || null)}><option value="">No type</option>{#each noteTypes as [value,label]}<option {value}>{label}</option>{/each}</select></label>
    <label class="checkbox-label"><input aria-label="Favorite note" type="checkbox" checked={payload.favorite === true} onchange={(e) => set('favorite', e.currentTarget.checked)} /> Favorite</label>
  {/if}
  {#if kind === 'task'}
    {#if !taskSecondaryOnly}<label>Status<select aria-label="Task status" value={String(payload.state ?? 'todo')} onchange={(e) => set('state', e.currentTarget.value)}><option value="todo">To Do</option><option value="doing">Doing</option><option value="done">Done</option></select></label>{/if}
    <label>Priority<select aria-label="Priority" value={String(payload.priority ?? 'normal')} onchange={(e) => set('priority', e.currentTarget.value)}><option value="low">Low</option><option value="normal">Normal</option><option value="high">High</option></select></label>
    <label>Planned<input aria-label="Planned date" type="date" value={String(payload.planned_date ?? '')} onchange={(e) => set('planned_date', e.currentTarget.value || null)} /></label>
    {#if !taskSecondaryOnly}<label>Due date<input aria-label="Due date" type="date" value={String(payload.due_date ?? '')} onchange={(e) => set('due_date', e.currentTarget.value || null)} /></label>{/if}
  {:else if kind === 'project'}
    {#if !projectSecondaryOnly}<label>Status<select aria-label="Project status" value={String(payload.state ?? 'planned')} onchange={(e) => set('state', e.currentTarget.value)}><option value="planned">Planned</option><option value="doing">In progress</option><option value="ongoing">Ongoing</option><option value="on_hold">On hold</option><option value="done">Completed</option></select></label>{/if}
    <label>Outcome or purpose<textarea aria-label="Purpose" value={String(payload.purpose ?? '')} onchange={(e) => set('purpose', e.currentTarget.value)}></textarea></label>
    <label>Start<input aria-label="Project start" type="date" value={String(payload.start_date ?? '')} onchange={(e) => set('start_date', e.currentTarget.value || null)} /></label>
    {#if !projectSecondaryOnly}<label>End<input aria-label="Project end" type="date" value={String(payload.end_date ?? '')} onchange={(e) => set('end_date', e.currentTarget.value || null)} /></label>{/if}
  {:else if kind === 'tag' && !tagSecondaryOnly}
    <label>Classification<select aria-label="Classification" value={String(payload.classification ?? 'topic')} onchange={(e) => set('classification', e.currentTarget.value)}><option value="area">Area</option><option value="resource">Resource</option><option value="entity">Entity</option><option value="topic">Topic</option></select></label>
  {:else if kind === 'person'}
    <label>Company<input aria-label="Company" maxlength="512" value={String(payload.company ?? '')} onchange={(e) => set('company', e.currentTarget.value || null)} /></label>
    {#each contactFields as field (field.key)}<label>{field.label}<input aria-label={field.label} maxlength="2048" value={String(payload[field.key] ?? '')} onchange={(e) => set(field.key, e.currentTarget.value || null)} /></label>{/each}
    <div class="person-interests" role="group" aria-label="Interests"><span>Interests</span><div class="person-interest-list">{#each interests as interest, index (index)}{#if typeof interest === 'string'}<span class="person-interest-chip">{interest}<button type="button" aria-label={`Remove interest ${interest}`} onclick={() => removeInterest(index)}>×</button></span>{/if}{/each}</div><div class="person-interest-add"><input class="glass-field" aria-label="New interest" placeholder="Add an interest" bind:value={interestInput} disabled={!interestsEditable} oninput={() => interestError = ''} onkeydown={(event) => { if (event.key === 'Enter') { event.preventDefault(); addInterest(); } }} /><button type="button" class="glass-secondary" aria-label="Add interest" disabled={!interestsEditable || !interestInput.trim()} onclick={addInterest}>Add</button></div>{#if !interestsEditable}<p class="error">Existing interests cannot be interpreted and are preserved.</p>{:else if interestError}<p class="error" role="alert">{interestError}</p>{/if}</div>
    <label>Relations<input aria-label="Person relationships" placeholder="Friend, Colleague" value={Array.isArray(payload.relationships) ? payload.relationships.join(', ') : ''} onchange={(e) => set('relationships', [...new Set(e.currentTarget.value.split(',').map(v => v.trim()).filter(Boolean))])} /></label>
    <label>Birthday<input aria-label="Birthday" type="date" value={String(payload.birthday ?? '')} onchange={(e) => set('birthday', e.currentTarget.value || null)} /></label>
    <label>Next check-in<input aria-label="Next check-in" type="date" value={String(payload.check_in ?? '')} onchange={(e) => set('check_in', e.currentTarget.value || null)} /></label>
    <label>Last check-in<input aria-label="Last check-in" type="date" value={String(payload.last_check_in ?? '')} onchange={(e) => set('last_check_in', e.currentTarget.value || null)} /></label>
    <label>Pipeline<select aria-label="Person pipeline" value={String(payload.pipeline_status ?? '')} onchange={(e) => set('pipeline_status', e.currentTarget.value || null)}><option value="">No status</option>{#each [['prospect','Prospect'],['contacted','Contacted'],['negotiating','Negotiating'],['rejected','Rejected'],['closed','Closed']] as [value,label]}<option {value}>{label}</option>{/each}</select></label>
  {/if}
  {#if kind === 'tag'}{#if !tagSecondaryOnly}<label class="checkbox-label"><input aria-label="Favorite tag" type="checkbox" checked={payload.favorite === true} onchange={(e) => set('favorite', e.currentTarget.checked)} /> Favorite</label>{/if}{#if payload.source_activity_at}<p class="metadata">Source activity: {String(payload.source_activity_at)}</p>{/if}{/if}
  {#if (kind === 'note' || kind === 'project' || kind === 'tag' || kind === 'person') && payload.source && typeof payload.source === 'object'}<details class="note-source"><summary>Imported source</summary><pre>{JSON.stringify(payload.source, null, 2)}</pre></details>{/if}
  {#if unrepresented.length}<p class="conflict">Unrepresented fields preserved: {unrepresented.join(', ')}. These are not converted or removed; you can only change the title without replacing this payload.</p>{/if}
</fieldset>
