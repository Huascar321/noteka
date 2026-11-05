# Spec Delta

## MODIFIED Requirements

### Requirement: Página Tasks del menú lateral

Tasks SHALL abrir una página amplia con icono sobre título, navegación Inbox/Today/Week/Month, New Task y filas compactas con relaciones/fechas locales. Inbox MUST listar tareas pendientes sin relación Project local. MUST iniciar Today, acumular páginas únicas y mantener apertura/captura protegidas. MUST NOT ofrecer la vista Unclassified ni un control manual de Inbox. Recurring y Active Content se identifican como pendientes deshabilitadas.

#### Scenario: Apertura y páginas independientes

- **WHEN** se navega a Tasks, se cambia de vista o se solicita Load more
- **THEN** aparece Today inicialmente, cada vista conserva su identidad de consulta y las páginas siguientes acumulan tareas únicas; una respuesta vieja no reaparece tras cambiar vista o abandonar la página

#### Scenario: Error y captura

- **WHEN** falla la lectura o se abre un original o New Task
- **THEN** el error permite reintentar sin fingir ausencia, y apertura/captura usan guardias y originales existentes; crear no asigna fechas ni guarda automáticamente

### Requirement: Vistas adicionales de tareas locales

El núcleo SHALL ofrecer Week, Month, Scheduled, No Due y tareas pendientes de proyectos activos/en progreso en home.query. MUST filtrar antes de paginar, conservar cursores/caps/snapshot y distinguir metadatos inválidos; Today, All y Done conservan sus reglas e Inbox se deriva de la ausencia de relación Project local. Las vistas sin soporte no se presentan como acciones habilitadas.

#### Scenario: Fechas y proyectos

- **WHEN** se consulta una semana ISO, mes civil, fechas asignadas/ausentes o proyectos activos
- **THEN** Rust devuelve sólo tareas pendientes admisibles del alcance solicitado, con grupos de proyectos elegibles y conteos anteriores a la página; no asigna fechas ni cambia estados

#### Scenario: Cursor o metadatos inválidos

- **WHEN** se reutiliza un cursor de otra vista o hay metadatos no interpretables
- **THEN** se rechaza el cursor ajeno y se identifica la cobertura no disponible, sin reparar originales ni atribuir filas viejas a la vista nueva

### Requirement: Página Notes del menú lateral

Notes SHALL abrir una página amplia con icono sobre el título, navegación Inbox/Fav./All/Tags, New Note y vistas Notes/Inbox/Fav./Clips/Voice/Journal/Meetings/All. Inbox MUST listar notas activas sin Project local; Unclassified conserva biblioteca sin Tag/Project/People. MUST iniciar Notes, mostrar filas con relaciones, tipo y antigüedad reales y mantener breadcrumb Home → Notes, captura/apertura protegidas y acciones textuales de ancho completo. All conserva acceso a todas las notas activas.

#### Scenario: Composición y vistas locales

- **WHEN** se abre Notes desde el menú lateral y se alternan sus vistas
- **THEN** aparece Notes inicialmente y se muestran sólo resultados de la consulta vigente, con iconos y relaciones locales, sin listado técnico paralelo ni recursos remotos

#### Scenario: Paginación y captura

- **WHEN** se activa Load more o New page desde el extremo derecho de su fila
- **THEN** se acumulan notas únicas con el cursor existente o se prepara un borrador protegido sin guardarlo; las categorías de creación conservan su tipo y favorito, e Inbox deja de seleccionarse con una marca manual

#### Scenario: Actualización y salida

- **WHEN** se actualiza mediante ⌘R/Ctrl+R/F5 o se abandona la página durante una lectura
- **THEN** se conservan las vistas sin recargar el WebView, un diálogo/bloqueo impide iniciar la actualización y las respuestas de una instancia anterior no reemplazan la pantalla actual ni sus borradores
