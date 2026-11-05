## ADDED Requirements

### Requirement: Proyecciones civiles para calendario de Tasks

El núcleo SHALL ofrecer consultas paginadas de calendario semanal y mensual que incluyan tareas de todos los estados del período y tareas sin fecha, preservando `tasks_week` y `tasks_month`.

#### Scenario: Período seleccionado

- **WHEN** se consulta un calendario con fecha de referencia
- **THEN** el núcleo selecciona fechas civiles del período, sin fecha y fechas no interpretables antes de paginar, sin cambiar las consultas anteriores
