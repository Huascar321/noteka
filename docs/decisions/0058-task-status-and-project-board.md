# 0058: Estado de tarea Doing y Board del Project

Fecha: 4 de noviembre de 2026. Estado: aceptada para el cambio
`add-project-task-board`.

## Contexto

La referencia de Notion del detalle de Project muestra Tasks como una base
vinculada con vistas List/Board/Calendar. El Board agrupa por Status en To Do,
Doing y Done; las tarjetas muestran título y Due, y se mueven entre columnas. El
modelo local sólo tenía `todo`/`done`, suficiente para List pero incapaz de
representar trabajo en curso ni de sostener un Board de tres columnas.

## Decisión

1. `TaskState` suma `Doing` además de `Todo` y `Done`. El valor se guarda en el
   JSON existente; no hay migración SQLite. `doing` es pendiente para filtros,
   workflows y resumen de proyecto; sólo `done` es completado.
2. El detalle de Project ofrece List y Board. List agrupa por estado y usa un
   checkbox real; Board usa tres columnas fijas y sólo las relaciones cargadas
   por `workflow.query`/`context`.
3. Mover una tarjeta puede hacerse con arrastre por puntero o con el selector de
   estado de la tarjeta. Ambos caminos releen la tarea, escriben `{state}` con
   `expected_revision` y refrescan tablero/progreso. No hay reintento automático.
4. New task por columna pasa `{state}` como seed y conserva el ContextLink del
   Project; la creación sigue siendo explícita.
5. No se persiste un orden manual dentro de la columna: el orden es determinista
   por vencimiento, título e ID. Calendar, comentarios, filtros, orden,
   automatizaciones y subtareas quedan fuera de este cambio.

## Consecuencias

- Los clientes antiguos que no conozcan `doing` podrían rechazar ese valor; los
  contratos y consumidores locales se actualizan juntos.
- El Board no promete un total global si la lectura está paginada; Load more
  conserva la cobertura existente.
- Un estado desconocido se conserva visible y no se convierte silenciosamente a
  `todo`.
- Debido a que el estado vive en el payload, backup/restore y exportación no
  cambian de formato ni necesitan migración.
