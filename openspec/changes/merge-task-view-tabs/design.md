# Design

## Context

`HomeTaskSection.svelte` comparte la barra de tabs entre Home y la página Tasks.
Hoy arma dos listas distintas: Home ofrece Today (`tasks_today`) y Active
Projects (In Progress) (`tasks_projects_in_progress`) por separado; la página
Tasks reemplaza la segunda por una tab rotulada Active Projects y oculta
`tasks_active_projects`. `HomePanel.svelte` sólo usa su lista de vistas para la
lectura inicial, mientras que la barra real vive en `HomeTaskSection`.

En Rust, `tasks_today` filtra `todo` con `due_date <= día`; en cambio
`tasks_projects_in_progress` filtra `todo` con al menos un proyecto Doing/Ongoing
no archivado y agrupa por proyecto. `tasks_active_projects` agrega Planned.

## Goals / Non-Goals

**Goals:** una sola tab Today con el conjunto y la agrupación de proyectos en
curso; eliminar la tab duplicada; rotular All Active Projects; mantener Home y
Tasks alineados; conservar los placeholders Recurring/Active Content.

**Non-Goals:** cambiar el núcleo `home.query`, `tasks_today`, la semántica de
Planned/Doing/Ongoing, los checkboxes, los estilos o la persistencia. Tampoco se
renombra el destino lateral Active Projects ni la tab homónima de People.

## Decisions

1. El mapeo se resuelve en la UI: la tab Today usa
   `tasks_projects_in_progress`. Así el resultado es idéntico al de la vista
   eliminada sin alterar contratos ni consumidores CLI/MCP de `tasks_today`.
2. `HomeTaskSection` pasa a tener una única lista de vistas para ambos modos y
   agrega Done sólo en la página Tasks. La lectura inicial de Home y Tasks usa
   `tasks_projects_in_progress`, de modo que Today queda activa al abrir.
3. Recurring y Active Content se mantienen como botones deshabilitados en la
   página Tasks, independientes de las vistas reales.
4. All Active Projects conserva `tasks_active_projects`, cuyo filtro ya incluye
   proyectos Planned además de Doing/Ongoing.

## Risks / Trade-offs

- El rótulo Today deja de coincidir con la vista `tasks_today` del núcleo. Es
  deliberado: la UI prioriza el flujo de trabajo en curso y el contrato sigue
  disponible para CLI/MCP y tests.
- All y Today pueden solaparse en tareas pendientes de proyectos Doing/Ongoing.
  All sigue siendo el conjunto completo, incluido Done y Sin proyecto.
- Reversión: restaurar el mapeo de Today a `tasks_today` y la lista anterior de
  tabs; sin migraciones ni datos que deshacer.
