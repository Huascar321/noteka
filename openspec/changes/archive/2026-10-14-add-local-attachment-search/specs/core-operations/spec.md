# Spec Delta

## ADDED Requirements

### Requirement: Operaciones aditivas de indexación y búsqueda de adjuntos

El núcleo SHALL publicar `extraction.index` y `extraction.search` en API v1, catálogo y CLI JSON, con DTOs estrictos compartidos. Index distingue get paginado, put con trabajo/generación/revisión esperados y reset con revisión esperada; search admite sólo consulta/filtros/límite/cursor documentados. MUST conservar sin cambios los 49 contratos anteriores y no añadir rutas, comandos o activación de proveedores en estas entradas.

#### Scenario: Catálogo y compatibilidad
- **WHEN** un cliente inspecciona el catálogo y ejecuta las operaciones por núcleo o CLI
- **THEN** encuentra 51 operaciones con esquemas de entrada/salida y errores compartidos; los 49 contratos previos, lectores separados y search.query/rebuild mantienen shapes y significado

#### Scenario: Entrada estricta por acción
- **WHEN** se omiten revisiones/generación requeridas, se mezclan campos de acciones o se envían campos desconocidos, IDs/cursor/límites inválidos, shell, ruta de helper o proveedor
- **THEN** se devuelve validation_error y salida JSON estable, sin ejecutar comandos, escribir selecciones ni reinterpretar el input

#### Scenario: Estado paginado
- **WHEN** se pide index get con filtros admitidos y límite válido
- **THEN** devuelve revisión/perfil/cobertura, selecciones ordenadas y cursor acotado ligado a su operación/filtros/revisión; get no usa los cursores de search ni modifica datos

### Requirement: Admisión y errores sin ejecución implícita

Las operaciones de adjuntos SHALL exigir un workspace de la versión actual, sin migración implícita, y abrir get/search sin permiso de escritura a la base principal. MUST distinguir validación, recurso ausente, conflicto, contención/deadline e integridad/fuente mediante códigos existentes y razones estables. No recuperan trabajos, ejecutan helpers, abren ventanas ni activan modelos/red.

#### Scenario: Workspace que requiere migración
- **WHEN** cualquiera de las dos operaciones recibe un workspace v9 o de una versión futura
- **THEN** devuelve unsupported_workspace_version antes de escribir en el workspace; el cliente debe abrir/migrar explícitamente mediante el flujo existente

#### Scenario: Errores diferenciables
- **WHEN** se solicita un ID ausente, se usa revisión vieja, se encuentra fuente alterada o se agota la espera/presupuesto
- **THEN** se conserva el sobre API v1 y código de salida CLI correspondiente con una razón estable; conflicto e índice dañado incluyen la revisión actual cuando puede leerse con seguridad

#### Scenario: Lectura sin permisos de escritura
- **WHEN** get/search accede a un workspace actual con el archivo SQLite principal sólo legible
- **THEN** puede consultar los datos confirmados y validar el índice sin modificar filas persistentes ni ejecutar escrituras de reparación; temporales privados no se publican como datos del usuario

### Requirement: Indexación fuera de la whitelist de lotes

El sistema MUST mantener la whitelist actual de nueve mutaciones de changes.preview/apply y sus checks. Las nuevas operaciones no son pasos ejecutables de lote; preview SHALL rechazarlas sin indexar ni recuperar jobs. Un apply válido conserva el índice actual al mutar dominio, sin reemplazarlo por el snapshot del preview ni convertir su revisión en revisión de entidades.

#### Scenario: Intento de indexar desde preview
- **WHEN** un lote incluye extraction.index o extraction.search como mutación o check no admitido
- **THEN** preview devuelve validación sin ejecutar la operación, crear selección, modificar ownership o activar el helper

#### Scenario: Escritura derivada entre preview y apply
- **WHEN** después de un preview válido se confirma put/reset y luego se aplica el lote original sin cambios de dominio conflictivos
- **THEN** el apply conserva exactamente el índice derivado actual y sus revisiones, sin restaurar los valores del snapshot anterior ni invalidar el recibo sólo por ese cambio derivado
