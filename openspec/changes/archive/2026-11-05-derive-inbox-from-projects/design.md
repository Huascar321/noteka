# Design

## Context

`home.query` resuelve Inbox con `TasksInbox => m.inbox` y `NotesInbox => m.inbox`
(`crates/noteka-core/src/home.rs`). Unclassified usa un bloque que recolecta
relaciones `project` (tareas) o `project`/`tag`/`person` (notas) y excluye los
ids clasificados. `workflow.query` Inbox usa
`coalesce(json_extract(payload,'$.inbox'),1)=1` sobre notas y tareas pendientes.
La bandera se guarda con default `true` en `TaskFields`/`NoteFields` y se edita
con un checkbox en `EntityFields.svelte`.

La revisión PARA del 5/11/2026 documentó la decisión anterior: conservar Inbox
manual y sumar Unclassified separado. Este cambio la sustituye.

## Goals / Non-Goals

**Goals:** que Inbox sea un conjunto derivado y sin mantenimiento; que vincular
a un proyecto saque el elemento y desvincularlo lo devuelva; quitar la bandera
como criterio y su control; conservar Notes Unclassified; mantener compatibilidad
de operaciones y datos.

**Non-Goals:** migrar o borrar `payload.inbox`, eliminar la operación
`tasks_unclassified` del núcleo, cambiar el formato SQLite, la paginación o los
caps, e incluir otros tipos de entidad (proyectos, tags, people) en Inbox.

## Decisions

1. **Derivación en lectura.** Inbox se calcula por ausencia de relación `project`
   con destino Project, igual que Unclassified para tareas. No escribe nada:
   crear o quitar la relación ya cambia la vista.
2. **Misma dirección de relación que Unclassified.** Se evalúa
   `relations.source_id = entidad` con `kind='project'` y destino Project, que es
   la forma en que la interfaz y el contexto de creación guardan los vínculos.
   Una relación a un proyecto archivado sigue contando.
3. **Alcance.** Tasks: sólo `todo`/`doing`. Notes: todas las notas activas, sin
   restricción de biblioteca; Tag/People no excluyen. Workflow Inbox: notas
   activas y tareas pendientes sin Project.
4. **Unclassified de Tasks se retira de la interfaz.** Su resultado queda
   idéntico a Inbox; dos tabs con las mismas filas sólo confunden. El núcleo
   conserva `tasks_unclassified` para clientes existentes, documentado como
   alias. Notes Unclassified se conserva porque añade biblioteca + Tag/People.
5. **Bandera heredada.** `payload.inbox` no se borra ni se migra; sigue en
   payload y en `HomeMetadata`, pero ninguna vista la consulta. El checkbox se
   elimina para no prometer un efecto inexistente.
6. **Sin token/estado nuevo.** Los cursores existentes siguen valiendo; cambia
   el conjunto seleccionado, no la forma de paginar.

## Risks / Trade-offs

- **Cambio inmediato de pertenencia con datos existentes.** Tareas con
  `inbox:false` y sin proyecto pasan a mostrarse en Inbox; notas con
  `inbox:true` vinculadas a un proyecto dejan de mostrarse. Es el comportamiento
  pedido y no toca datos.
- **Clientes CLI/MCP** que esperaban la bandera reciben otro conjunto bajo los
  mismos nombres de vista. Se documenta en specs y guías; no hay bump de
  contrato porque la forma no cambia.
- **Inbox de Tasks y `tasks_unclassified` idénticos en el núcleo.** Se acepta
  como compatibilidad; la UI muestra una sola tab.
- **Notas importadas:** Inbox deja de ser "biblioteca"; notas de colecciones
  importadas sin Project aparecen. Se avisa en la guía de Notes.
