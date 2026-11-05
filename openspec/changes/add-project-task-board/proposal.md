# Proposal

## Why

El detalle de Project sólo muestra Tasks como una lista agrupada To Do/Done. La
referencia de Notion que pidió el usuario muestra también un Board con columnas
To Do/Doing/Done, tarjetas con vencimiento y New task por columna. El modelo local
no tenía `doing`, por lo que el Board no podía representar trabajo en curso.

## What Changes

- Tareas admiten `state` todo/doing/done. `doing` cuenta como pendiente; sólo
  `done` es completada.
- Today, Inbox, Week, Month, Scheduled, No Due, Active Projects y los resúmenes
  de proyectos incluyen doing como pendiente. Done y All conservan su alcance.
- El detalle de Project ofrece vistas List y Board. Board tiene columnas
  To Do/Doing/Done, conteo, tarjetas con título/vencimiento y New task por columna.
- Mover una tarjeta por arrastre o por el selector de estado actualiza `state`
  con revisión esperada y refresca el tablero y el progreso.
- List usa un checkbox real para completar/reabrir y agrupa To Do/Doing/Done.
- El selector de estado de la tarea en su detalle ofrece To Do/Doing/Done.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `productivity-workflows`: estado doing y filtros de pendientes.
- `desktop-shell`: List/Board de Tasks dentro del Project.

## Impact

Enum/validación Rust, contratos generados, consultas Home/workflow, componentes
Svelte del detalle y CSS. Sin migración SQLite, operación nueva, dependencia,
comentarios, Calendar, filtros, automatizaciones ni orden manual persistido.
