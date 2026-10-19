# Spec Delta

## Purpose

Organizar captura y tareas mediante estados, fechas civiles y relaciones compartidas con consultas deterministas que funcionan offline.

## ADDED Requirements

### Requirement: Metadatos de trabajo validados

El núcleo SHALL validar estado, prioridad y fechas civiles opcionales de tareas, propósito y fechas de proyectos y clasificación de tags. MUST rechazar datos inválidos sin cambios parciales, conservar campos omitidos al actualizar y conservar payloads antiguos sin reescribirlos al abrir.

#### Scenario: Planificar y completar
- **WHEN** una tarea se planifica para una fecha válida, se prioriza y se completa con revisión vigente
- **THEN** conserva ID, fechas y relaciones y aparece como completada en lectura estructurada

#### Scenario: Fecha o prioridad inválida
- **WHEN** se envía una fecha imposible, un instante en lugar de fecha civil o prioridad desconocida
- **THEN** devuelve validation_error sin incrementar revisión ni historial

### Requirement: Inbox y vistas deterministas

El sistema SHALL consultar Inbox de notas/tareas con bandera inbox, My Day de tareas planificadas para el día civil del workspace, pendientes, vencidas y proyectos activos. MUST excluir archivadas y tareas completadas de las vistas de trabajo; las consultas SHALL ser paginadas y no depender de modelos.

#### Scenario: Día civil y vencimiento
- **WHEN** se consultan tareas para un día civil explícito o el día actual en la zona del workspace
- **THEN** My Day usa planned_date igual al día y vencidas usa due_date menor al día

#### Scenario: Salida de Inbox y archivo
- **WHEN** se desmarca inbox o se archiva una captura
- **THEN** deja de aparecer en Inbox sin perder identidad, contenido ni relaciones

### Requirement: Dashboards y captura contextual

El sistema SHALL listar entidades relacionadas a proyecto, tag o persona sin duplicarlas y crear notas/tareas en ese contexto con su relación en la misma transacción. MUST rechazar contextos inexistentes/incompatibles sin dejar una captura parcial.

#### Scenario: Crear desde proyecto
- **WHEN** se crea una nota o tarea desde un proyecto
- **THEN** se confirma su relación y el dashboard y CLI leen el mismo ID

#### Scenario: Contexto inválido
- **WHEN** el contexto pertenece a otro workspace o no admite la relación solicitada
- **THEN** ninguna entidad ni relación queda creada

### Requirement: Edición y relaciones desde escritorio

La interfaz SHALL crear y editar los cinco tipos, planificar/completar tareas, clasificar tags y agregar/quitar relaciones explícitas. MUST conservar buffers ante conflicto/fallo, proteger navegación/cierre y permitir archivo reversible sin red.

#### Scenario: Recorrido de productividad offline
- **WHEN** se crea proyecto, captura nota y tarea vinculadas, se planifica/completa la tarea y se abre My Day/Inbox/contexto sin red
- **THEN** las vistas reflejan reglas del núcleo y el CLI lee las mismas entidades/revisiones

#### Scenario: Conflicto de tarea
- **WHEN** el CLI modifica una tarea con buffer pendiente en escritorio
- **THEN** el guardado devuelve conflicto y conserva buffer y estado externo sin sobrescribirlo
