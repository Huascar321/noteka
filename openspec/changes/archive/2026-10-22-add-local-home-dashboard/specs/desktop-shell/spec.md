# Spec Delta

## ADDED Requirements

### Requirement: Home local con cobertura visible

El escritorio SHALL ofrecer Home de ancho amplio con portada local, icono de casa, navegación compacta y secciones Tasks, Notes, Projects, Tags, Goals y People en ese orden. MUST conservar franjas de clima/Goals como pendientes explícitos sin cifras o acciones ficticias, recursos remotos ni bloqueo de las secciones implementadas.

#### Scenario: Workspace vacío y estructura offline
- **WHEN** se abre Home en un workspace vacío sin red ni modelos
- **THEN** aparecen composición, secciones, estados vacíos y creación local, mientras clima y Goals indican su falta de implementación y no presentan datos simulados

#### Scenario: Navegación compacta y funciones diferidas
- **WHEN** se usa la navegación de Home y se consulta la cobertura de las pestañas de referencia
- **THEN** Tasks/Notes/Projects/Tags llevan a secciones reales y Content, Goals, fotos y auxiliares pendientes se identifican sin controles que aparenten ejecutar esas funciones

### Requirement: Presentación Home de vistas originales

Home SHALL mostrar las vistas locales habilitadas como listas compactas de tareas/notas/tags, grupos plegables, listas/tablero de proyectos y galería textual de personas. MUST presentar contexto, errores, resúmenes y cobertura del núcleo, conservar IDs al cambiar de vista y ofrecer paginación y apertura por teclado sin recalcular reglas de negocio en la interfaz.

#### Scenario: Cambio de vista y páginas
- **WHEN** se alternan vistas de las cinco secciones, se pliega un grupo y se recorren 51 elementos coincidentes
- **THEN** cada vista conserva el orden, IDs y alcance de su consulta, el grupo no borra datos y la cobertura distingue filas únicas de apariciones; abrir una fila o tarjeta conduce al original

#### Scenario: Progreso y metadatos no disponibles
- **WHEN** un proyecto carece de tareas, un resumen no es interpretable o una persona no tiene foto/organización soportadas
- **THEN** la presentación distingue sin tareas y dato no disponible, usa tarjeta textual y no sustituye desconocimiento por cero, porcentajes inventados o atributos de otra entidad

### Requirement: Navegación y captura Home protegidas

La interfaz SHALL abrir originales y comenzar creación desde Home con las guardias existentes de guardar/descartar/cancelar. MUST conservar buffers de entidades y colecciones ante cancelación, conflicto o fallo; crear tareas en un grupo de proyecto con relación atómica y proyectos en una columna con ese estado, sin writes hasta confirmar ni defaults ocultos de fechas.

#### Scenario: Llegar a Home con buffer pendiente
- **WHEN** desde una nota o colección editada se intenta navegar a Home y se cancela o falla guardar
- **THEN** permanecen buffer, IDs y revisiones capturadas, no se monta Home como sustituto del contenido pendiente y la versión del CLI no se sobrescribe

#### Scenario: Crear en proyecto o estado
- **WHEN** se crea y guarda una tarea desde un grupo de proyecto o un proyecto desde una columna del tablero
- **THEN** CLI y Home leen una única entidad con la relación o estado elegidos; descartar antes de guardar no crea entidad y capturar desde Today no asigna automáticamente due_date ni planned_date

#### Scenario: Confirmación seguida de fallo de lectura
- **WHEN** una creación se confirma pero falla el refresco posterior de Home
- **THEN** se distingue guardado confirmado de lectura fallida y se ofrece reintentar la consulta, no repetir una creación ni reponer un buffer ya confirmado

### Requirement: Lecturas Home vigentes y aisladas

Home SHALL mantener selección, página, estado de carga/error y actualización por sección. MUST descartar respuestas de vistas, días, workspaces o componentes anteriores, limpiar datos al cambiar su identidad de consulta y no etiquetar resultados viejos como actuales después de un fallo. Una lectura o refresh MUST dejar intactos originales y buffers pendientes.

#### Scenario: Respuesta tardía al cambiar vista o workspace
- **WHEN** una consulta termina después de cambiar la vista, día, workspace o desmontar Home
- **THEN** no publica sus filas, contexto, cantidades ni error en la nueva pantalla y no modifica el buffer activo

#### Scenario: Fallo y reintento independiente
- **WHEN** falla una sección y se solicita reintentar mientras otras secciones tienen resultados válidos
- **THEN** la sección afectada muestra el fallo sin filas viejas como respuesta vigente; el reintento consulta los mismos originales y no borra las otras selecciones ni escribe datos

#### Scenario: Actualizar después del CLI
- **WHEN** el CLI cambia una entidad o relación y se actualiza Home o se vuelve a activar su ventana
- **THEN** las proyecciones reflejan los originales confirmados con día/zona visibles, sin polling, rebase de revisiones capturadas ni incremento de revisiones por leer

### Requirement: Temas y foco verificables en Home

El escritorio SHALL ofrecer temas claro y oscuro con preferencia local, oscuro inicial, contraste legible y foco visible; navegación, vistas, grupos, paginación y creación Home SHALL funcionar por teclado y offline. MUST conservar datos/buffers al cambiar tema y tratar fallo de preferencia como recuperable, sin cambiar el formato del workspace.

#### Scenario: Tema persistido y buffer intacto
- **WHEN** se cambia tema con edición pendiente y se reabre la aplicación
- **THEN** la edición no se escribe ni descarta por el cambio y la preferencia reaparece; una preferencia inválida o inaccesible usa oscuro y no impide operar el workspace

#### Scenario: Foco y layouts en ambos temas
- **WHEN** se recorren Home, pestañas, grupos, tablero, tarjetas y creación por teclado en temas claro/oscuro
- **THEN** el foco es identificable, los controles tienen nombres accesibles y el contenido es legible; en viewport más estrecho no desaparecen las acciones esenciales ni se fuerza un listado lateral vacío de Home
