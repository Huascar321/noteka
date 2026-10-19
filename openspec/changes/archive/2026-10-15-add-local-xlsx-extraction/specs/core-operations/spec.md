# Spec Delta

## MODIFIED Requirements

### Requirement: Contrato compartido y versionado

El sistema SHALL exponer un catálogo versionado con esquemas de entrada, salida y errores para todas las operaciones implementadas, incluidas colecciones, propiedades, fórmulas, vistas, plantillas, consultas, lotes y extracción local. Interfaz, CLI y herramientas MCP habilitadas MUST aplicar las mismas reglas; el catálogo MUST identificar operaciones compatibles con lotes, referencias admitidas y límites, sin anunciar funciones futuras como disponibles.

#### Scenario: Clientes equivalentes
- **WHEN** la interfaz y el CLI solicitan la misma actualización válida en workspaces equivalentes
- **THEN** obtienen estados y resultados de dominio equivalentes

#### Scenario: Descubrimiento sin interfaz
- **WHEN** se solicita el catálogo de operaciones desde el CLI
- **THEN** se obtienen versión y esquemas sin abrir la ventana de escritorio

#### Scenario: Catálogo y transportes completos
- **WHEN** se compara el catálogo con las operaciones públicas y los subcomandos CLI
- **THEN** cada operación tiene entrada, salida y errores documentados; changes.preview/apply y su alcance limitado son descubribles sin alterar solicitudes anteriores

#### Scenario: Paridad del lote
- **WHEN** núcleo y CLI ejecutan el mismo plan y recibo sobre copias equivalentes con el mismo scope
- **THEN** aplican iguales validaciones y producen estados, IDs y cálculos equivalentes, salvo origen, identificador de solicitud y timestamps de ejecución documentados

#### Scenario: Catálogo de extracción acotado
- **WHEN** se describe el núcleo con extracción local implementada
- **THEN** las doce operaciones de extracción publican esquemas/límites y distinguen utf8_text, pdf_text y xlsx_cells acotados, sin compatibilidad como paso/check de lote ni anuncio de PDF/Excel genérico, OCR, embeddings, LLM o ejecución residente

#### Scenario: Paridad de herramientas MCP habilitadas
- **WHEN** CLI y MCP autorizado ejecutan entradas equivalentes de su superficie común sobre workspaces actuales equivalentes, dentro de los límites de entrega MCP
- **THEN** conservan validaciones, cálculos exactos, CAS, estados y motivos de error de dominio, salvo scope, IDs aleatorios y metadatos de ejecución documentados; el transporte no añade reglas de negocio propias

#### Scenario: Catálogo completo y disponibilidad por transporte
- **WHEN** se compara el catálogo compartido con tools/list y el recurso noteka://catalog
- **THEN** existen 52 operaciones conservando los 51 contratos previos, diferenciadas de las 24/44/45 herramientas y acciones habilitadas; las siete operaciones excluidas de MCP no desaparecen del CLI ni se anuncian como invocables por ese transporte

### Requirement: Operaciones aditivas de indexación y búsqueda de adjuntos

El núcleo SHALL publicar `extraction.index` y `extraction.search` en API v1, catálogo y CLI JSON, con DTOs estrictos compartidos. Index distingue get paginado, put con trabajo/generación/revisión esperados y reset con revisión esperada; search admite sólo consulta/filtros/límite/cursor documentados. MUST conservar sin cambios los 49 contratos anteriores y no añadir rutas, comandos o activación de proveedores en estas entradas.

#### Scenario: Catálogo y compatibilidad
- **WHEN** un cliente inspecciona el catálogo y ejecuta las operaciones por núcleo o CLI
- **THEN** encuentra 52 operaciones con esquemas de entrada/salida y errores compartidos; los 49 contratos anteriores al índice, lectores separados y search.query/rebuild mantienen shapes y significado

#### Scenario: Entrada estricta por acción
- **WHEN** se omiten revisiones/generación requeridas, se mezclan campos de acciones o se envían campos desconocidos, IDs/cursor/límites inválidos, shell, ruta de helper o proveedor
- **THEN** se devuelve validation_error y salida JSON estable, sin ejecutar comandos, escribir selecciones ni reinterpretar el input

#### Scenario: Estado paginado
- **WHEN** se pide index get con filtros admitidos y límite válido
- **THEN** devuelve revisión/perfil/cobertura, selecciones ordenadas y cursor acotado ligado a su operación/filtros/revisión; get no usa los cursores de search ni modifica datos

### Requirement: Ejecución scoped sobre workspace actual

El núcleo SHALL ofrecer ejecución scoped aditiva con ruta canónica, identidad y política fijadas por el host, sin admitir override del cliente. MUST comprobar versión e identidad en la conexión/snapshot realmente usados y rechazar un workspace no actual sin crear, migrar, restaurar ni reparar. Esta entrada conserva execute/execute_json, contratos API v1 y apertura histórica de CLI/UI.

#### Scenario: Scope actual y versión incompatible
- **WHEN** se ejecuta la entrada scoped sobre v11 válido y sobre fixtures ausente, anterior, futura o inconsistente
- **THEN** sólo el actual se admite; los demás conservan sus datos y devuelven not_found, unsupported_workspace_version o storage_error según corresponda, sin backup de migración ni cambios de formato

#### Scenario: Identidad comprobada en la operación
- **WHEN** se reemplaza el workspace por otro entre admisión inicial y apertura de una operación genérica o especializada
- **THEN** se rechaza la identidad distinta antes de efectos usando la conexión/snapshot efectivo, sin confiar únicamente en el check realizado al arrancar

#### Scenario: Clientes anteriores conservados
- **WHEN** se comparan los 51 inputs/outputs, definiciones, errores y operaciones de lote previos y se ejecutan CLI/UI sin la entrada scoped
- **THEN** no cambian sus shapes, whitelist o validaciones; la única evolución de formato añadida es la migración v11 explícita documentada, sin ejecución implícita

## ADDED Requirements

### Requirement: Contrato compartido de lectura XLSX

El núcleo SHALL añadir sólo `extraction.xlsx_cells` a API v1 (52 operaciones), con input/output tipados, validación Rust y CLI `extraction xlsx-cells` equivalente a invoke. MUST preservar los 51 contratos anteriores, errores/salida JSON y exclusión de operaciones de extracción de los nueve steps batchables; los bindings frontend se generan desde el mismo catálogo, sin nuevas pantallas.

#### Scenario: CLI y core equivalentes
- **WHEN** se consulta la misma generación/paginación con invoke y el subcomando XLSX
- **THEN** devuelven el mismo resultado tipado y los esquemas publicados describen valores léxicos, fórmulas, metadatos y localizadores sin tipos any abiertos

#### Scenario: Conservación y ámbito limitado
- **WHEN** se comparan catálogos con baseline v1/51 y se intenta insertar extracción XLSX en un lote
- **THEN** sólo se añade un contrato y un extractor al descriptor; los previos son idénticos y el lote rechaza la operación sin mutaciones parciales
