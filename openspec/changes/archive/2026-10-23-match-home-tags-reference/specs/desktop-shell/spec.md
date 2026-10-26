# Spec Delta

## ADDED Requirements

### Requirement: Tags con grupos y filas de referencia

Home Tags SHALL abrir Types y presentar Recent/Fav./A-Z/Types, encabezado, controles reales y grupos Area/Resource/Entity con colores, conteos y filas compactas. MUST mostrar iconos locales, última nota navegable y contador, conservar teclado, temas, errores, cobertura y captura con clasificación contextual. Topic conserva un grupo independiente cuando hay registros de ese tipo.

#### Scenario: Grupos e iconos locales
- **WHEN** se abre Tags en Types
- **THEN** aparecen los tres grupos de referencia incluso vacíos, los registros se ordenan por título en su tipo y las filas muestran la última nota y cantidad calculadas con recursos locales

#### Scenario: Ventana estrecha o fallo
- **WHEN** se reduce el viewport o falla la lectura
- **THEN** las filas se adaptan sin overflow global y el error permite reintentar sin confundirse con una lista vacía

### Requirement: Vistas y resúmenes acreditados de tags

Rust SHALL filtrar archivo de procedencia y favoritos antes de paginar y conservar A-Z/Types. MUST contar notas relacionadas activas únicas, excluir archivo de origen y elegir la última por modificación de fuente acreditada o local si falta, con desempate estable. Recent SHALL combinar actividad importada y fechas de notas relacionadas con fallback local, conservando caps, snapshots y cursores.

#### Scenario: Contador y última nota
- **WHEN** un tag tiene notas relacionadas válidas
- **THEN** el contador refleja esas notas únicas y la última nota se determina con timestamps interpretados como instantes, sin que el orden de recuperación de iconos altere el resultado

#### Scenario: Actividad reciente acreditada
- **WHEN** se consulta Recent con actividad de fuente y notas relacionadas
- **THEN** se combina su actividad explícita con fechas acreditadas de notas, usando modificación local si falta; la procedencia conserva su carácter de snapshot sin prometer sincronización continua

#### Scenario: Favoritos, archivo y datos inválidos
- **WHEN** hay favoritos, tags archivados en origen o metadatos inválidos
- **THEN** Fav. filtra la marca explícita, Home excluye los archivados sin archivar sus entidades locales y los datos no interpretables se informan; se mantienen caps, snapshots y cursores por vista

### Requirement: Recuperación explícita de tags y notas relacionadas

La recuperación SHALL acreditar tags con manifiesto/CSV y propiedades originales y notas por su ID original importado. MUST conservar documentos, IDs, metadatos elegidos y relaciones existentes, respaldar antes de aplicar, usar CAS e informe privado incremental y omitir identidades ambiguas. El arte sólo se recupera de referencias observadas y Home no realiza solicitudes remotas.

#### Scenario: Datos y revisión concurrente
- **WHEN** se recuperan relaciones y arte acreditados
- **THEN** sólo se agregan vínculos inequívocos, se registra la revisión vigente y un conflicto rechaza la escritura sin reemplazar cuerpos ni forzar el guardado
