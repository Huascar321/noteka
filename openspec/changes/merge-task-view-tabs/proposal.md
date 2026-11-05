# Proposal

## Why

Tasks muestra dos tabs que abren casi el mismo conjunto: Today y Active
Projects (In Progress). El usuario quiere quedarse con una sola, que Today sea
literalmente la vista de proyectos en curso, y que la tab de proyectos activos
diga All Active Projects para distinguir que también incluye Planned.

## What Changes

- La tab Today de Tasks (Home y página Tasks) consulta
  `tasks_projects_in_progress`: tareas pendientes en proyectos Doing/Ongoing no
  archivados, agrupadas por proyecto.
- Se retira la tab duplicada Active Projects (In Progress).
- La tab `tasks_active_projects` pasa a llamarse All Active Projects y conserva
  Planned/Doing/Ongoing.
- La página Tasks conserva los pendientes deshabilitados Recurring y Active
  Content, y ahora muestra All Active Projects en su barra.
- El núcleo no cambia: `home.query` sigue ofreciendo `tasks_today` con su
  semántica de vencimiento para CLI/MCP y consumidores existentes.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: tabs de Tasks consolidadas y página Tasks inicial.

## Impact

Sólo presentación y mapeo de vistas en Svelte, documentación y un test nativo de
selección de tabs. Sin cambios Rust, SQLite, contratos, operaciones, dependencias
ni migraciones. Los checkboxes y sus estilos no se modifican.
