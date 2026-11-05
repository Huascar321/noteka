# Spec Delta

## MODIFIED Requirements

### Requirement: Página Tasks del menú lateral

Tasks SHALL abrir una página amplia con icono sobre título, navegación Inbox/Today/Week/Month, New Task y filas compactas con relaciones/fechas locales. MUST iniciar Today, que muestra tareas pendientes de proyectos Doing/Ongoing no archivados agrupadas por proyecto, acumular páginas únicas y mantener apertura/captura protegidas. All Active Projects SHALL incluir también proyectos Planned. Recurring y Active Content se identifican como pendientes deshabilitadas.

#### Scenario: Apertura y páginas independientes
- **WHEN** se navega a Tasks, se cambia de vista o se solicita Load more
- **THEN** aparece Today inicialmente con el conjunto agrupado de proyectos Doing/Ongoing, cada vista conserva su identidad de consulta y las páginas siguientes acumulan tareas únicas; una respuesta vieja no reaparece tras cambiar vista o abandonar la página

#### Scenario: Error y captura
- **WHEN** falla la lectura o se abre un original o New Task
- **THEN** el error permite reintentar sin fingir ausencia, y apertura/captura usan guardias y originales existentes; crear no asigna fechas ni guarda automáticamente

## ADDED Requirements

### Requirement: Tabs de Tasks consolidadas

La sección Tasks de Home y la página Tasks SHALL compartir las tabs Today, Inbox, Week, Month, Scheduled, No Due, All Active Projects y All. La tab Today MUST consultar la vista de tareas pendientes en proyectos Doing/Ongoing no archivados y agruparlas por proyecto; la tab duplicada Active Projects (In Progress) MUST NOT mostrarse. La tab All Active Projects MUST incluir Planned, Doing y Ongoing. El núcleo SHALL conservar `home.query` con `tasks_today` y su filtro por vencimiento para CLI/MCP.

#### Scenario: Today y proyectos en curso
- **WHEN** se elige Today en Home o en la página Tasks
- **THEN** se muestran las mismas tareas pendientes agrupadas por proyecto Doing/Ongoing que devuelve `tasks_projects_in_progress`, sin una tab separada Active Projects (In Progress)

#### Scenario: All Active Projects
- **WHEN** se elige All Active Projects
- **THEN** se incluyen tareas pendientes de proyectos Planned, Doing y Ongoing, y el rótulo no se confunde con el destino lateral Active Projects

#### Scenario: Núcleo sin cambios
- **WHEN** CLI o MCP consultan `home.query` con `tasks_today`
- **THEN** la vista conserva el filtro `todo` por vencimiento hasta el día civil, independiente del mapeo visual de la tab Today
