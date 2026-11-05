## ADDED Requirements

### Requirement: Proyección de tareas pendientes

El núcleo SHALL ofrecer una proyección paginada y agrupada de tareas no completadas con estados interpretables y SHALL conservar el comportamiento de `tasks_all` para clientes existentes.

#### Scenario: Filtrar antes de paginar

- **WHEN** se consultan tareas pendientes con tareas To Do, Doing y Done
- **THEN** sólo To Do y Doing participan en el total, los grupos y las páginas, mientras `tasks_all` mantiene todos los estados
