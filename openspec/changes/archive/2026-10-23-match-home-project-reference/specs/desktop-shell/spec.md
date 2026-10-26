# Spec Delta

## ADDED Requirements

### Requirement: Projects con tablero compacto de referencia

Home Projects SHALL abrir Board y presentar Active, Planned y Board, encabezado, New Project y cinco columnas Planned/Doing/Ongoing/Done/On Hold. MUST mostrar tarjetas compactas con iconos originales locales, título, progreso disponible y contexto navegable, conservar captura protegida, teclado, temas y desplazamiento horizontal contenido.

#### Scenario: Tablero con columnas vacías
- **WHEN** se abre Projects con datos locales
- **THEN** aparecen cinco columnas en el orden indicado, incluso vacías, y New page prepara un proyecto del estado de esa columna sin guardarlo automáticamente

#### Scenario: Pantalla estrecha o error de lectura
- **WHEN** el viewport es estrecho o falla una consulta
- **THEN** el tablero permite desplazamiento interno sin overflow global y la sección conserva error, reintento y cobertura accesibles

### Requirement: Proyección real de proyectos y contexto importado

Rust SHALL filtrar procedencia archivada antes de paginar Projects y calcular total, completas, pendientes, vencidas y progreso en décimas desde tareas locales relacionadas únicas. MUST conservar caps, cursores y snapshots; proyectos sin tareas no equivalen a completos y datos inválidos producen resumen no disponible. Las páginas importadas explícitamente de Goals MAY aparecer como referencias de contexto, sin atribuirles un dominio Goals implementado.

#### Scenario: Relaciones completas y estados locales
- **WHEN** un proyecto tiene tareas relacionadas activas con estados válidos
- **THEN** su progreso es completas/total redondeado a una décima, las vencidas son pendientes con fecha anterior al día civil del workspace y los enlaces duplicados no incrementan los contadores

#### Scenario: Fuente archivada y consulta de tareas por proyecto
- **WHEN** un proyecto está archivado sólo en la procedencia
- **THEN** se excluye de las vistas Projects y de la pertenencia a proyectos activos de Home, conservando la entidad local y sus documentos

### Requirement: Recuperación acreditada de relaciones de proyectos

La recuperación SHALL vincular identidades mediante manifiesto, secuencia completa verificada y propiedades originales inequívocas o referencias explícitas autorizadas. MUST usar backup, CAS e informe privado incremental, conservar documentos, IDs, estados y relaciones existentes, omitir ambigüedades y recuperar sólo arte observado sin solicitudes remotas en Home.

#### Scenario: Recuperación con edición concurrente
- **WHEN** se aplica metadata acreditada o relaciones y cambia una revisión local
- **THEN** se usa la revisión vigente esperada y se rechaza un conflicto, dejando informe de lo completado y sin reemplazar documentos ni forzar escrituras
