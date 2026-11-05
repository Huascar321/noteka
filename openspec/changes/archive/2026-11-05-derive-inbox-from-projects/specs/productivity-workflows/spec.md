# Spec Delta

## MODIFIED Requirements

### Requirement: Inbox y vistas deterministas

El sistema SHALL derivar Inbox de notas activas y tareas pendientes (`todo`/`doing`) sin relación local Project: vincularlas a un proyecto las excluye y quitar el vínculo las devuelve; un vínculo Tag o People no excluye una nota y un destino archivado sigue contando. MUST NOT usar `payload.inbox` como criterio. MUST consultar My Day, pendientes, vencidas y proyectos activos, excluir archivadas y tareas completadas, y paginar sin depender de modelos.

#### Scenario: Día civil y vencimiento

- **WHEN** se consultan tareas para un día civil explícito o el día actual en la zona del workspace
- **THEN** My Day usa planned_date igual al día y vencidas usa due_date menor al día

#### Scenario: Salida de Inbox y archivo

- **WHEN** una nota o una tarea pendiente sin relación Project se vincula a un proyecto, o se archiva una captura, o se completa una tarea
- **THEN** deja de aparecer en Inbox o en las vistas de trabajo sin perder identidad, contenido, bandera heredada ni relaciones
