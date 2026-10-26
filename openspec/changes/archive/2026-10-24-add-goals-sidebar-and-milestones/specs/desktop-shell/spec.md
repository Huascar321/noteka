# Spec Delta

## ADDED Requirements

### Requirement: Goals lateral y página de referencia

El escritorio SHALL ubicar Goals entre Tags y People, con icono propio, entrada
de búsqueda y breadcrumb Home → Goals → registro. La página SHALL presentar
icono sobre título, Nav a Tasks/Notes/Projects/Tags, cuatro vistas, By Activity
inicial y grupos Active/Dream/Achieved con contexto, progreso y fecha disponibles.
By Tag SHALL presentar agrupación por tags y estados; Achieved su fecha de logro.
MUST reutilizar guardias, temas, actualización activa y filas de creación/paginación
de ancho completo, y reemplazar el aviso pendiente de Goals en Home con datos reales.

#### Scenario: Menú y página
- **WHEN** se abre Goals desde el lateral, buscador o Home
- **THEN** aparece en la posición solicitada, se consultan sus originales y las
  cuatro vistas reflejan reglas Rust con contextos navegables y vacíos explícitos

#### Scenario: Ventana estrecha y cambio de página
- **WHEN** se reduce el ancho en cualquiera de los temas o se navega con un borrador
- **THEN** no hay overflow global, se mantienen títulos/controles accesibles y
  Guardar/Descartar/Cancelar protege los cambios sin guardar automáticamente

### Requirement: Captura y detalle de Goal y Milestone

El escritorio SHALL preparar Goals con estado contextual o Dream por defecto,
editar estado/fechas y presentar sus hitos originales con creación vinculada.
Milestones SHALL aparecer debajo de Goals con List de completados, captura y
apertura originales; su detalle SHALL editar fecha completada, plazo y selección
de Goal por título. MUST mantener progreso derivado y la navegación canónica de
los contextos; ninguna lectura o selección de vista modifica entidades.

#### Scenario: Goal e hitos vinculados
- **WHEN** se abre un Goal y se solicita un hito nuevo
- **THEN** se muestran sus hitos cargados y se prepara una página Milestone con
  el vínculo al Goal, protegida por las guardias y validaciones existentes

#### Scenario: Edición de fechas y conservación
- **WHEN** se prepara o edita un Goal/Milestone y se descarta o falla su guardado
- **THEN** no se pierde el original ni un buffer por reemplazo silencioso;
  las fechas/progreso disponibles se presentan sin inventar metadatos ausentes

## MODIFIED Requirements

### Requirement: Apertura y navegación local

La interfaz SHALL permitir seleccionar o crear un workspace. El menú lateral SHALL mostrar únicamente Home, Tasks, Notes, Projects, Tags, Goals y People, en ese orden, con búsqueda contextual fuera de esa lista y Collections como botón de icono accesible. MUST conservar los destinos secundarios, incluido Archive, mediante accesos de búsqueda y mostrar estados explícitos para listas vacías y fallos de apertura.

#### Scenario: Primer workspace
- **WHEN** se inicia la aplicación sin un workspace seleccionado
- **THEN** ofrece crear o abrir uno y muestra su ruta después de seleccionarlo

#### Scenario: Workspace no disponible
- **WHEN** la ruta seleccionada no se puede abrir
- **THEN** muestra el error y permite elegir otra ruta sin presentar datos de otro workspace como propios

#### Scenario: Navegación primaria reducida
- **WHEN** se abre un workspace y se recorre el lateral por teclado
- **THEN** la lista contiene exactamente los siete destinos definidos, Collections es un botón de icono con nombre accesible y Archive, Inbox, My Day, Pending, Overdue, Active Projects, Areas y Resources siguen disponibles desde el buscador sin cambiar sus reglas

### Requirement: Home local con cobertura visible

El escritorio SHALL ofrecer Home de ancho amplio con portada local, icono de casa y secciones Tasks, Notes, Projects, Tags, Goals y People en ese orden. MUST presentar Goals desde sus originales y conservar la franja de clima como pendiente explícito sin cifras o acciones ficticias, recursos remotos ni bloqueo de las secciones implementadas.

#### Scenario: Workspace vacío y estructura offline
- **WHEN** se abre Home en un workspace vacío sin red ni modelos
- **THEN** aparecen composición, secciones, estados vacíos y creación local, mientras Goals muestra sus grupos vacíos y clima indica su falta de implementación sin datos simulados

#### Scenario: Navegación compacta y funciones diferidas
- **WHEN** se usan los encabezados de las secciones de Home y se consulta la cobertura de las pestañas de referencia
- **THEN** Tasks/Notes/Projects/Tags/Goals/People llevan a secciones reales y Content y auxiliares pendientes se identifican sin controles que aparenten ejecutar esas funciones

### Requirement: Proyección real de proyectos y contexto importado

Rust SHALL filtrar procedencia archivada antes de paginar Projects y calcular total, completas, pendientes, vencidas y progreso en décimas desde tareas locales relacionadas únicas. MUST conservar caps, cursores y snapshots; proyectos sin tareas no equivalen a completos y datos inválidos producen resumen no disponible. Goals tipados y páginas importadas explícitamente de Goals MAY aparecer como referencias de contexto, conservando su página original; los Goals tipados abren su sección canónica.

#### Scenario: Relaciones completas y estados locales
- **WHEN** un proyecto tiene tareas relacionadas activas con estados válidos
- **THEN** su progreso es completas/total redondeado a una décima, las vencidas son pendientes con fecha anterior al día civil del workspace y los enlaces duplicados no incrementan los contadores

#### Scenario: Fuente archivada y consulta de tareas por proyecto
- **WHEN** un proyecto está archivado sólo en la procedencia
- **THEN** se excluye de las vistas Projects y de la pertenencia a proyectos activos de Home, conservando la entidad local y sus documentos
