# Spec Delta

## MODIFIED Requirements

### Requirement: Metadatos de trabajo validados

El núcleo SHALL validar estado, prioridad y fechas civiles opcionales de tareas, propósito y fechas de proyectos y clasificación de tags. MUST rechazar datos inválidos sin cambios parciales, conservar campos omitidos al actualizar y conservar payloads antiguos sin reescribirlos al abrir.

#### Scenario: Planificar y completar
- **WHEN** una tarea se planifica para una fecha válida, se prioriza y se completa con revisión vigente
- **THEN** conserva ID, fechas y relaciones y aparece como completada en lectura estructurada

#### Scenario: Tarea en curso
- **WHEN** se guarda una tarea con `state` doing y revisión vigente
- **THEN** Rust acepta el valor, conserva los demás campos y la tarea sigue siendo pendiente para filtros y progreso

#### Scenario: Fecha o prioridad inválida
- **WHEN** se envía una fecha imposible, un instante en lugar de fecha civil o prioridad desconocida
- **THEN** devuelve validation_error sin incrementar revisión ni historial

### Requirement: Inbox y vistas deterministas

El sistema SHALL consultar Inbox de notas/tareas con bandera inbox, My Day de tareas planificadas para el día civil del workspace, pendientes, vencidas y proyectos activos. MUST excluir archivadas y tareas completadas de las vistas de trabajo; las consultas SHALL ser paginadas y no depender de modelos. `doing` SHALL contar como pendiente y sólo `done` como completada.

#### Scenario: Día civil y vencimiento
- **WHEN** se consultan tareas para un día civil explícito o el día actual en la zona del workspace
- **THEN** My Day usa planned_date igual al día y vencidas usa due_date menor al día, incluyendo tareas todo/doing

#### Scenario: Salida de Inbox y archivo
- **WHEN** se desmarca inbox o se archiva una captura
- **THEN** deja de aparecer en Inbox sin perder identidad, contenido ni relaciones

### Requirement: Vistas locales Home explícitas

El sistema SHALL ofrecer Tasks Today/Inbox/All/Done, Notes Notes/Inbox, Projects Active/Planned/Board, Tags Recent/A–Z/Types y People Contacts/New/Updated con filtros y órdenes documentados. MUST excluir entidades archivadas, conservar referencias a contextos archivados y distinguir sus reglas propias de filtros de Notion no inspeccionados. `doing` SHALL incluirse en las vistas de pendientes.

#### Scenario: Tareas y notas por estado e Inbox
- **WHEN** se consultan las vistas con tareas todo/doing/done y notas dentro/fuera de Inbox
- **THEN** Tasks Inbox devuelve todo/doing con inbox, All incluye todo/doing/done y Done sólo done; Notes incluye todas las notas activas e Inbox sólo las marcadas, sin inferir clasificación desde relaciones

#### Scenario: Proyectos y clasificación conservada
- **WHEN** se consultan proyectos en los cinco estados y tags area/resource/topic
- **THEN** Home Active incluye doing/ongoing, Planned sólo planned y Board los cinco estados, mientras el workflow Active Projects previo sigue incluyendo planned/doing/ongoing; Types conserva las tres clasificaciones sin convertir topic en otro tipo ni duplicar entidades

#### Scenario: Orden documentado de tags y personas
- **WHEN** se consultan Tags Recent/A–Z y People Contacts/New/Updated con timestamps y títulos empatados
- **THEN** se aplican los órdenes documentados con desempate por ID estable; no se interpreta New como un período relativo ni se inventan favoritos, organización o atributos de contacto
