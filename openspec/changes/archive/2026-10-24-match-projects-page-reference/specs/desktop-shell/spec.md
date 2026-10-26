# Spec Delta

## ADDED Requirements

### Requirement: Página Projects del menú lateral

Projects SHALL abrir una página amplia con icono propio sobre título, Nav, By Tag/By People, New Project y vistas Active/Planned/Board/Deadlines/Completed/On Hold. MUST iniciar Active con grupos Doing/Ongoing, filas con referencias y progreso locales, breadcrumb Home → Projects, apertura/captura protegidas y acciones textuales de ancho completo. Board conserva movimiento atómico existente; actualizar conserva vista sin recargar la aplicación.

#### Scenario: Composición y vistas reales
- **WHEN** se abre Projects y se alternan sus vistas en cualquiera de los temas
- **THEN** Active aparece inicialmente con los estados Doing/Ongoing, Planned/Completed/On Hold muestran sus estados, Deadlines conserva la lista local por fecha y Board sus cinco columnas; iconos, relaciones y porcentajes provienen de los originales

#### Scenario: Captura y ancho estrecho
- **WHEN** se activa New page en un grupo o se estrecha la página
- **THEN** se prepara un borrador del estado o relación elegidos sin guardarlo, el breadcrumb y controles siguen accesibles y no hay overflow global; las filas textuales responden en todo el ancho

#### Scenario: Lectura y salida protegidas
- **WHEN** falla una lectura, llega una respuesta después de salir o se actualiza con un diálogo/bloqueo
- **THEN** se ofrece reintento sin fingir ausencia, se descartan respuestas anteriores y el bloqueo no inicia actualización ni pierde borradores; el listener se retira al desmontar

### Requirement: Consultas de proyectos por pausa y relación

Rust SHALL ofrecer On Hold y proyectos agrupados por tag o persona en home.query. MUST excluir archivo local/de origen antes de paginar, conservar límites/cursores/snapshots y resúmenes vigentes, agrupar relaciones reales antes de truncar contexto y entregar identidad navegable de cada grupo. Un proyecto aparece una vez como entidad y una vez en cada grupo relacionado; los proyectos sin relación tienen un grupo explícito. Leer no modifica entidades, documentos ni vínculos.

#### Scenario: Estados y pertenencia
- **WHEN** se consulta On Hold, By Tag o By People
- **THEN** On Hold selecciona sólo on_hold y las agrupaciones admiten los cinco estados válidos, conservan relaciones múltiples y orden estable por grupo/estado/proyecto sin inferir vínculos por título ni limitar People a proyectos activos

#### Scenario: Página y metadata inválida
- **WHEN** se pagina, existe metadata no interpretable o se entrega un cursor de otra vista
- **THEN** los grupos cubren sólo los proyectos de la página con IDs únicos, el dato no interpretable se informa o el cursor se rechaza; una referencia ausente no se sustituye por otro registro y los límites existentes se conservan
