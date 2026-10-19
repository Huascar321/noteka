<script lang="ts">
  import type { EntityKind } from './contracts.generated';
  let { kind, payload, disabled = false, onchange }: { kind: EntityKind; payload: Record<string, unknown>; disabled?: boolean; onchange: () => void } = $props();
  const allowed: Record<EntityKind, string[]> = { note: ['inbox'], task: ['state', 'priority', 'inbox', 'planned_date', 'due_date'], project: ['state', 'purpose', 'start_date', 'end_date'], tag: ['classification'], person: [] };
  const unrepresented = $derived(Object.keys(payload).filter(k => !allowed[kind].includes(k)));
  function set(key: string, value: unknown) { payload[key] = value; onchange(); }
</script>

<fieldset class="entity-fields" disabled={disabled}>
  <legend>Propiedades</legend>
  {#if kind === 'note' || kind === 'task'}<label class="checkbox-label"><input aria-label="En Inbox" type="checkbox" checked={payload.inbox !== false} onchange={(e) => set('inbox', e.currentTarget.checked)} /> En Inbox (salida explícita)</label>{/if}
  {#if kind === 'task'}
    <label>Estado<select aria-label="Estado de tarea" value={String(payload.state ?? 'todo')} onchange={(e) => set('state', e.currentTarget.value)}><option value="todo">Pendiente</option><option value="done">Completada</option></select></label>
    <label>Prioridad<select aria-label="Prioridad" value={String(payload.priority ?? 'normal')} onchange={(e) => set('priority', e.currentTarget.value)}><option value="low">Baja</option><option value="normal">Normal</option><option value="high">Alta</option></select></label>
    <label>Planificada<input aria-label="Fecha planificada" type="date" value={String(payload.planned_date ?? '')} onchange={(e) => set('planned_date', e.currentTarget.value || null)} /></label>
    <label>Vencimiento<input aria-label="Vencimiento" type="date" value={String(payload.due_date ?? '')} onchange={(e) => set('due_date', e.currentTarget.value || null)} /></label>
  {:else if kind === 'project'}
    <label>Estado<select aria-label="Estado de proyecto" value={String(payload.state ?? 'planned')} onchange={(e) => set('state', e.currentTarget.value)}><option value="planned">Planificado</option><option value="doing">En curso</option><option value="ongoing">Continuo</option><option value="on_hold">En espera</option><option value="done">Completado</option></select></label>
    <label>Resultado o propósito<textarea aria-label="Propósito" value={String(payload.purpose ?? '')} onchange={(e) => set('purpose', e.currentTarget.value)}></textarea></label>
    <label>Inicio<input aria-label="Inicio de proyecto" type="date" value={String(payload.start_date ?? '')} onchange={(e) => set('start_date', e.currentTarget.value || null)} /></label>
    <label>Fin<input aria-label="Fin de proyecto" type="date" value={String(payload.end_date ?? '')} onchange={(e) => set('end_date', e.currentTarget.value || null)} /></label>
  {:else if kind === 'tag'}
    <label>Clasificación<select aria-label="Clasificación" value={String(payload.classification ?? 'topic')} onchange={(e) => set('classification', e.currentTarget.value)}><option value="area">Área</option><option value="resource">Recurso</option><option value="topic">Tema</option></select></label>
  {/if}
  {#if unrepresented.length}<p class="conflict">Campos no representados conservados: {unrepresented.join(', ')}. No se convierten ni eliminan; sólo podés cambiar el título sin reemplazar este payload.</p>{/if}
  {#if kind === 'person'}<p class="metadata">Identidad para relaciones. Propiedades personalizadas llegarán con colecciones.</p>{/if}
</fieldset>
