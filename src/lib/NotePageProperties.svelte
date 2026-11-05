<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import type { Entity } from './contracts.generated';
  import { entityIcon, entityIconColor } from './entity-appearance';
  import { noteTypes as types } from './note-types';
  let { payload, tags, sourceUrl = '', disabled = false, readOnly = false, canRelate = false, captureTag = false, onchange, onopen, oneditrelations }: {
    payload: Record<string, unknown>;
    tags: Entity[];
    sourceUrl?: string;
    disabled?: boolean;
    readOnly?: boolean;
    canRelate?: boolean;
    captureTag?: boolean;
    onchange: () => void;
    onopen: (entity: Entity) => Promise<void>;
    oneditrelations: (anchor: HTMLElement) => void;
  } = $props();
  const noteType = $derived(String(payload.note_type ?? ''));
  function set(key: string, value: unknown) { if (disabled || readOnly) return; payload[key] = value; onchange(); }
</script>

<fieldset class="note-page-properties" disabled={disabled}>
  <legend class="sr-only">Note properties</legend>
  <div class="note-tag-property" role="group" aria-label="Note tags">
    <span class="note-property-label"><AppIcon name="arrow-up-right" size={14} />Tag</span>
    <div class="note-tag-value">
      <div class="note-tag-list">
        {#each tags as tag (tag.id)}<button class="note-detail-tag" title={tag.title} onclick={() => void onopen(tag)}><AppIcon name={entityIcon(tag.kind, tag.title, tag.payload)} color={entityIconColor(tag.payload)} size={15} /><span>{tag.title}{tag.archived_at ? " · archived" : ''}</span></button>{/each}
        {#if !tags.length}<span class="note-property-empty" title={captureTag ? "Capture tag · applied on save" : 'Empty'}>{captureTag ? "Capture tag · applied on save" : 'Empty'}</span>{/if}
      </div>
      {#if canRelate}<button class="note-tag-edit icon-button" aria-label="Edit note tags" title="Edit relations" aria-haspopup="dialog" disabled={readOnly} onclick={event => oneditrelations(event.currentTarget)}><AppIcon name="chevron-down" size={13} /></button>{/if}
    </div>
  </div>
  <div class="note-url-property" role="group" aria-label="Source URL, read only">
    <span class="note-property-label"><AppIcon name="globe" size={14} />URL</span>
    <span class:note-property-empty={!sourceUrl} class="note-source-url" title={sourceUrl ? `Imported URL, read only: ${sourceUrl}` : "No imported URL"}>{sourceUrl || 'Empty'}</span>
  </div>
  <label class="note-favorite-property"><span class="note-property-label"><AppIcon name="star" size={14} />Favorite</span><input aria-label="Favorite note" type="checkbox" disabled={readOnly} checked={payload.favorite === true} onchange={(event) => set('favorite', event.currentTarget.checked)} /></label>
  <label class="note-type-property"><span class="note-property-label"><AppIcon name="circle" size={14} />Type</span><span class="note-type-field"><select aria-label="Note type" disabled={readOnly} value={noteType} onchange={(event) => set('note_type', event.currentTarget.value || null)}><option value="">No type</option>{#if noteType && !types.some(([value]) => value === noteType)}<option value={noteType} disabled>Unrecognized type</option>{/if}{#each types as [value, label]}<option {value}>{label}</option>{/each}</select><AppIcon name="chevron-down" size={14} /></span></label>
</fieldset>
