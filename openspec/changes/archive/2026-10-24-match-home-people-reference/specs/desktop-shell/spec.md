## ADDED Requirements

### Requirement: People con galería de referencia

Home People SHALL abrir Contacts y ofrecer Contacts/New/Updated/Check-Ins/Birthdays/Pipeline/Active Projects, encabezado y acciones reales. MUST presentar tarjetas compactas con foto original local y recorte explícito, icono, nombre, relaciones y empresa; conservar captura/apertura protegidas, teclado, temas, errores y cobertura. Load more SHALL acumular contactos únicos; una foto ausente usa espacio neutro sin atributos inventados ni solicitudes remotas.

#### Scenario: Galería offline y página siguiente
- **WHEN** se abre Contacts y se solicita otra página en claro u oscuro
- **THEN** las tarjetas mantienen identidad, orden acreditado y fotos locales, se añaden contactos únicos y la cobertura distingue los cargados del total

#### Scenario: Ancho estrecho o lectura fallida
- **WHEN** se estrecha el viewport o falla la consulta/lectura de foto
- **THEN** controles y tarjetas siguen accesibles sin overflow global, el error de consulta ofrece reintento y una imagen ausente no sustituye la identidad

### Requirement: Vistas locales diferenciadas de People

Rust SHALL ofrecer Contacts/New/Updated/Check-Ins/Birthdays/Pipeline/Active Projects con procedencia explícita, períodos civiles y relaciones locales. MUST filtrar antes de paginar, conservar caps/snapshots/cursores y distinguir metadata inválida. El listado general conserva personas activas de todas las colecciones.

#### Scenario: Biblioteca de contactos
- **WHEN** hay personas locales, contactos People y miembros WorkspacePeople importados
- **THEN** las vistas de contactos admiten personas sin procedencia o de People no archivadas en origen, y el listado general conserva también los miembros

#### Scenario: Períodos y calendario
- **WHEN** se consulta New/Updated, check-ins o un mes de cumpleaños
- **THEN** New/Updated usa timestamps de fuente o locales desde un mes civil antes inclusivo; Check-Ins admite fechas hasta un mes después, incluyendo atrasados; Birthdays muestra ocurrencias del mes elegido, con 29/2 el 28 en años no bisiestos sin modificar el original

#### Scenario: Proyectos y estados
- **WHEN** un proyecto Doing/Ongoing no archivado localmente ni en origen tiene vínculos a varias personas admisibles
- **THEN** aparece una vez como entidad y en cada grupo de persona, con resumen local y cobertura de apariciones; Pipeline agrupa por estado explícito y conserva grupos vacíos, sin inventar avance

#### Scenario: Metadata o cursor inválidos
- **WHEN** hay metadata no interpretable o un cursor de otra vista/día/workspace
- **THEN** se informa cobertura no disponible o se rechaza el cursor ajeno, sin reparar originales ni sustituir la vista actual con respuestas anteriores


### Requirement: Recuperación explícita de contactos

La recuperación SHALL usar el export/manifiesto y metadata/fotos observadas autorizadas, con backup nuevo, CAS e informe privado incremental. MUST conservar documentos, IDs, valores existentes y relaciones, separar las dos colecciones People, omitir identidades ambiguas y mantener datos personales fuera de Git.

#### Scenario: Identidad o revisión incierta
- **WHEN** una coincidencia no es única o una revisión cambia antes de escribir
- **THEN** se omite la ambigüedad o se rechaza el conflicto y queda informe de lo completado, sin reemplazar cuerpos ni forzar el guardado


## MODIFIED Requirements

### Requirement: Home local con cobertura visible

El escritorio SHALL ofrecer Home de ancho amplio con portada local, icono de casa, navegación compacta y secciones Tasks, Notes, Projects, Tags, Goals y People en ese orden. MUST conservar franjas de clima/Goals como pendientes explícitos sin cifras o acciones ficticias, recursos remotos ni bloqueo de las secciones implementadas.

#### Scenario: Workspace vacío y estructura offline
- **WHEN** se abre Home en un workspace vacío sin red ni modelos
- **THEN** aparecen composición, secciones, estados vacíos y creación local, mientras clima y Goals indican su falta de implementación y no presentan datos simulados

#### Scenario: Navegación compacta y funciones diferidas
- **WHEN** se usa la navegación de Home y se consulta la cobertura de las pestañas de referencia
- **THEN** Tasks/Notes/Projects/Tags llevan a secciones reales y Content, Goals y auxiliares pendientes se identifican sin controles que aparenten ejecutar esas funciones

### Requirement: Presentación Home de vistas originales

Home SHALL mostrar las vistas locales habilitadas como listas compactas de tareas/notas/tags, grupos plegables, listas/tablero de proyectos y galería de personas con fotos locales opcionales. MUST presentar contexto, errores, resúmenes y cobertura del núcleo, conservar IDs al cambiar de vista y ofrecer paginación y apertura por teclado sin recalcular reglas de negocio en la interfaz.

#### Scenario: Cambio de vista y páginas
- **WHEN** se alternan vistas de las cinco secciones, se pliega un grupo y se recorren 51 elementos coincidentes
- **THEN** cada vista conserva el orden, IDs y alcance de su consulta, el grupo no borra datos y la cobertura distingue filas únicas de apariciones; abrir una fila o tarjeta conduce al original

#### Scenario: Progreso y metadatos no disponibles
- **WHEN** un proyecto carece de tareas, un resumen no es interpretable o una persona carece de foto u organización
- **THEN** la presentación distingue sin tareas y dato no disponible, usa una tarjeta con fallback textual y no sustituye desconocimiento por cero, porcentajes inventados o atributos de otra entidad
