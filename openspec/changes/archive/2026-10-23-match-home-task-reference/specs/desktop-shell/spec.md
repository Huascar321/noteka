# Spec Delta

## ADDED Requirements

### Requirement: Tasks con composición de la referencia

Tasks SHALL presentar encabezado, acción New Task, pestañas con iconos, filas compactas y grupos plegables por proyecto con icono real, nombre y conteo de página. MUST alinear relaciones y fechas, conservar apertura/creación protegidas y ofrecer cobertura/reglas accesibles sin convertir cada fila en una tabla técnica.

#### Scenario: Vista agrupada y recursos locales
- **WHEN** se elige una vista agrupada de Tasks en claro u oscuro
- **THEN** los grupos y filas comparten iconos de sus proyectos, las fechas mantienen su valor civil y abrir/crear usa los originales y guardias existentes sin recursos remotos

#### Scenario: Cobertura error y ancho estrecho
- **WHEN** hay paginación, error o viewport estrecho
- **THEN** la cobertura distingue página de total, el error no simula ausencia y títulos/controles permanecen alcanzables con teclado sin desbordar la página

### Requirement: Vistas adicionales de tareas locales

El núcleo SHALL ofrecer Week, Month, Scheduled, No Due y tareas pendientes de proyectos activos/en progreso en home.query. MUST filtrar antes de paginar, conservar cursores/caps/snapshot y distinguir metadatos inválidos; Today, Inbox, All y Done conservan sus reglas. Las vistas sin soporte no se presentan como acciones habilitadas.

#### Scenario: Fechas y proyectos
- **WHEN** se consulta una semana ISO, mes civil, fechas asignadas/ausentes o proyectos activos
- **THEN** Rust devuelve sólo tareas pendientes admisibles del alcance solicitado, con grupos de proyectos elegibles y conteos anteriores a la página; no asigna fechas ni cambia estados

#### Scenario: Cursor o metadatos inválidos
- **WHEN** se reutiliza un cursor de otra vista o hay metadatos no interpretables
- **THEN** se rechaza el cursor ajeno y se identifica la cobertura no disponible, sin reparar originales ni atribuir filas viejas a la vista nueva

### Requirement: Arte de entidad como adjunto local

La interfaz SHALL presentar iconos personalizados de imagen desde adjuntos del workspace, además de Lucide. MUST usar referencias locales explícitas, no inferir arte por títulos, limitar cache/lecturas y mostrar fallback si el adjunto falta o no es una imagen compatible. El cambio de icono conserva contenido, relaciones y propiedades ajenas.

#### Scenario: Importación explícita de arte
- **WHEN** se recupera un icono autorizado y se asocia a una entidad con revisión esperada
- **THEN** se guarda el original como adjunto, la entidad conserva una referencia local y el mismo arte aparece en Home, relaciones, detalle, búsqueda y pestañas sin URL remota en runtime

#### Scenario: Falta de fuente o cambio de workspace
- **WHEN** el adjunto no está disponible, es excesivo/incompatible o termina una lectura de otro workspace
- **THEN** aparece un icono alternativo, no se ejecuta SVG/HTML y la respuesta anterior no muestra imágenes ajenas ni modifica originales

### Requirement: Recuperación explícita de vínculos importados

La recuperación de relaciones omitidas SHALL usar referencias explícitas de la fuente autorizada y revisiones esperadas. MUST conservar originales y relaciones existentes, respaldar antes de aplicar y omitir coincidencias ambiguas sin elegir por posición aislada o inventar proyectos.

#### Scenario: Vínculo confirmado y título duplicado
- **WHEN** se recuperan relaciones de la referencia y una tarea tiene más de una coincidencia local con el mismo título/fecha sin evidencia que resuelva su identidad
- **THEN** las coincidencias únicas se vinculan a sus proyectos reales y el duplicado queda pendiente en un informe externo, sin modificar fechas, estados, documentos o entidades ajenas
