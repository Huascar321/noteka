# Design

## Context

`ProjectContents.svelte` recibe las tareas del Project desde
`workflow.query`/`context` y las agrupa localmente por `payload.state`. El modelo
`TaskFields.state` sólo permitía `todo`/`done`. La referencia de Notion agrupa
las tareas por un Status de tres valores y permite arrastrarlas entre columnas;
la tarjeta muestra título y Due. El Project ya calcula progreso en Rust y ya
tiene captura contextual con `ContextLink`.

## Goals / Non-Goals

**Goals:** agregar `doing` de forma aditiva; reflejar pendientes en las vistas
existentes; ofrecer List/Board en el detalle de Project; mover tareas con
arrastre o selector; conservar CAS, paginación, captura y progreso.

**Non-Goals:** Calendar, comentarios, filtros, orden, automatizaciones, subtareas,
recurrencia, orden manual persistido, cambios de portada o de la base SQLite.

## Decisions

1. `TaskState` suma `Doing`; el payload sigue siendo JSON y no requiere migración.
   Rust valida el valor y `home`/`workflow` tratan `todo` y `doing` como
   pendientes; `done` sigue siendo el único completado. All incluye los tres.
2. El Board no crea otra consulta: usa las mismas tareas cargadas por
   `workflow.query`/`context` y el cursor de “Load more relations”. No inventa un
   total global.
3. Las columnas se ordenan To Do/Doing/Done; dentro de cada una, por `due_date`
   ascendente (sin fecha al final), título e ID. No se persiste orden manual.
4. Cada tarjeta abre el original; un selector de estado permite moverla con
   teclado. El arrastre por puntero usa un umbral de 6 px y resalta la columna
   destino. Ambos caminos releen la entidad y escriben `{state}` con
   `expected_revision`; un conflicto no se reintenta solo y se muestra el error.
5. `New task` por columna pasa `{state}` como seed y el `ContextLink` del Project;
   no guarda hasta que el usuario confirme.
6. List reutiliza el patrón de checkbox de Home/Tasks: pendiente hueco, completado
   verde; `doing` es un pendiente. Done empieza plegado.
7. Calendar queda fuera: la referencia lo muestra, pero la solicitud concreta es
   el Board y no se presenta una vista vacía o ficticia.

## Risks / Trade-offs

- Un cliente anterior que no conozca `doing` podría rechazar ese valor; el
  contrato generado y los consumidores locales se actualizan juntos.
- El Board muestra sólo las relaciones cargadas; una página parcial no se
  presenta como total. Load more conserva la cobertura existente.
- Mover una tarea con una revisión vieja devuelve conflicto; no se reintenta.
- Un estado desconocido se conserva visible en List y en una columna Unavailable
  del Board, sin convertirlo a `todo`.

## Migration

No hay DDL ni migración: `state` vive en el JSON del payload. Payloads `todo` y
`done` existentes se leen igual; `doing` es un valor nuevo. Backup/restore y
exportación no cambian de formato.
