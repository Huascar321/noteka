# 0059: Inbox derivado de relaciones Project

Fecha: 5 de noviembre de 2026. Estado: aceptada para el cambio
`derive-inbox-from-projects`. Sustituye la decisión del 5/11/2026 registrada en
`verification-unclassified-views.md` y en la revisión PARA del plan.

## Contexto

Inbox era una bandera manual (`payload.inbox`, default `true`) con checkbox en el
detalle de tarea y nota. Unclassified, agregada más tarde, se derivaba de
relaciones: tareas pendientes sin Project; notas de biblioteca sin
Tag/Project/People. Mantener ambos conceptos producía dos formas de "sin
clasificar" y un checkbox que no describía la realidad: una tarea creada desde un
proyecto seguía en Inbox hasta desmarcarla, y una tarea sin proyecto quedaba
fuera de Inbox si alguien desmarcaba la bandera.

El usuario pidió que Inbox sea el depósito automático de lo que todavía no
pertenece a un proyecto: al vincularlo, abandona Inbox; al quitar el vínculo,
vuelve. Sin marca manual que mantener.

## Decisión

1. Inbox se deriva en lectura de la ausencia de una relación `project` con
   destino Project. Tasks usa `todo`/`doing`; Notes acepta todas las notas
   activas (sin restricción de biblioteca). Tag y People no excluyen una nota de
   Inbox.
2. Rige la misma dirección de relación que Unclassified: la entidad es
   `relations.source_id` y el destino es un Project. Una relación a un proyecto
   archivado sigue contando.
3. La bandera `payload.inbox` se conserva en payload y metadata, pero deja de
   consultarse. No se migra, no se borra y no hay cambio de formato SQLite.
4. Se retira el checkbox "Inbox" del detalle y la tab Unclassified de Tasks: el
   núcleo mantiene `tasks_unclassified` como alias compatible del mismo conjunto,
   pero la interfaz no muestra dos tabs idénticas.
5. Notes conserva Unclassified, que sigue siendo más estricta: biblioteca sin
   Tag/Project/People.
6. `workflow.query` Inbox (destino lateral/buscador, notas + tareas pendientes)
   usa la misma derivación.
7. Seeds que enviaban `inbox:false` (goals, milestones, plantillas) siguen
   guardando el dato heredado; no alteran la pertenencia.

## Consecuencias

- El comportamiento cambia de inmediato con datos existentes: tareas con
  `inbox:false` sin proyecto aparecen en Inbox; notas con `inbox:true` vinculadas
  a un proyecto dejan de aparecer. Ningún dato se modifica.
- Clientes CLI/MCP reciben otro conjunto bajo las mismas vistas, sin cambio de
  forma ni de esquema.
- La UI deja de prometer un estado que no controla; la vista se explica con una
  línea y en las opciones de vista.
- `tasks_inbox` y `tasks_unclassified` devuelven lo mismo para tareas. Se acepta
  como compatibilidad; la consolidación futura puede retirar el alias.
- No se amplía Inbox a proyectos, tags, people ni goals, ni se agregan reglas de
  Content (no modelado).

## Alternativas rechazadas

- **Conservar la bandera como pin adicional** (`Inbox = sin Project OR flagged`):
  contradice "abandona la categoría al vincular" y mantiene el modo de falla.
- **Autoclasificar por Tag/People en Inbox de notas**: el pedido nombra Project;
  Unclassified ya cubre esas relaciones y queda como vista más estricta.
- **Reemplazar Unclassified de Notes por Inbox**: perdería la distinción de
  biblioteca y la exclusión por Tag/People que el usuario eligió conservar.
- **Migrar o borrar `payload.inbox`**: descarta datos sin necesidad y complica
  backups/futuros usos; el campo es inerte y barato de conservar.
