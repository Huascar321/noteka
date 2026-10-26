# Spec Delta

## MODIFIED Requirements

### Requirement: Contrato compartido y versionado

El sistema SHALL exponer un catálogo versionado con esquemas de entrada, salida y errores para todas las operaciones implementadas, incluidas colecciones, propiedades, fórmulas, vistas, plantillas, consultas, lotes y extracción local. Interfaz, CLI y herramientas MCP habilitadas MUST aplicar las mismas reglas; el catálogo MUST identificar operaciones compatibles con lotes, referencias admitidas, capacidades y límites, sin anunciar funciones futuras como disponibles.

#### Scenario: Clientes equivalentes
- **WHEN** la interfaz y el CLI solicitan la misma actualización válida en workspaces equivalentes
- **THEN** obtienen estados y resultados de dominio equivalentes

#### Scenario: Descubrimiento sin interfaz
- **WHEN** se solicita el catálogo de operaciones desde el CLI
- **THEN** se obtienen versión y esquemas sin abrir la ventana de escritorio

#### Scenario: Catálogo y transportes completos
- **WHEN** se compara el catálogo con las operaciones públicas y los subcomandos CLI
- **THEN** cada operación tiene entrada, salida y errores documentados; changes.preview/apply y su alcance limitado son descubribles sin invalidar solicitudes anteriores admitidas

#### Scenario: Paridad del lote
- **WHEN** núcleo y CLI ejecutan el mismo plan y recibo sobre copias equivalentes con el mismo scope
- **THEN** aplican iguales validaciones y producen estados, IDs y cálculos equivalentes, salvo origen, identificador de solicitud y timestamps de ejecución documentados

#### Scenario: Catálogo de extracción acotado
- **WHEN** se describe el núcleo con extracción e índice mixto implementados
- **THEN** las catorce operaciones de extracción distinguen utf8_text, pdf_text, xlsx_cells, image_ocr y pdfkit_text acotados y sus proyecciones explícitas, sin compatibilidad de extracción como paso/check ni anuncio de PDF/Excel genérico, embeddings, LLM o ejecución residente

#### Scenario: Paridad de herramientas MCP habilitadas
- **WHEN** CLI y MCP autorizado ejecutan entradas equivalentes de su superficie común sobre workspaces actuales equivalentes, dentro de los límites de entrega MCP
- **THEN** conservan validaciones, cálculos exactos, CAS, estados y motivos de error de dominio, salvo scope, IDs aleatorios y metadatos de ejecución documentados; el transporte no añade reglas de negocio propias

#### Scenario: Catálogo completo y disponibilidad por transporte
- **WHEN** se compara el catálogo compartido con tools/list y el recurso noteka://catalog
- **THEN** existen54 operaciones y26/46/47 herramientas; respecto del baseline14/54 sólo index/search amplían enums/localizadores PDFKit y su metadata, mientras los52 contratos restantes, errores, nueve mutaciones batchables y siete exclusiones MCP se conservan

### Requirement: Contrato compartido de páginas PDFKit

El catálogo/API v1 SHALL añadir extraction.pdfkit_pages y CLI extraction pdfkit-pages con DTOs estrictos y validación Rust compartida. MUST conservar lectores previos, errores, whitelist batch9 y siete exclusiones MCP; las ampliaciones identificadas de index/search no cambian este lector. No acepta rutas de helper, comandos, contraseñas o selección remota en solicitudes.

#### Scenario: Paridad y contratos anteriores
- **WHEN** se compara el catálogo con baseline13 durable y se leen páginas equivalentes mediante núcleo/CLI/MCP autorizado
- **THEN** los51 contratos del baseline13 ajenos a index/search permanecen iguales y la nueva lectura devuelve los mismos datos/cobertura, sin convertirse en step batchable

#### Scenario: Entradas estrictas
- **WHEN** se envían campos desconocidos, límites/cursor/tipo ajenos o se intenta insertar la lectura en un batch
- **THEN** se rechaza antes de writes/procesamiento, sin ampliar permisos o whitelist

### Requirement: Operaciones aditivas de indexación y búsqueda de adjuntos

El núcleo SHALL publicar `extraction.index` y `extraction.search` en API v1, catálogo y CLI JSON, con DTOs estrictos compartidos. Index distingue get paginado, put con trabajo/generación/revisión esperados y reset con revisión esperada; search admite sólo consulta/filtros/límite/cursor documentados. MUST conservar sin cambios los 49 contratos anteriores y no añadir rutas, comandos o activación de proveedores en estas entradas.

#### Scenario: Catálogo y compatibilidad
- **WHEN** un cliente inspecciona el catálogo y ejecuta las operaciones por núcleo o CLI
- **THEN** encuentra54 operaciones con esquemas de entrada/salida y errores compartidos; los 49 contratos anteriores al índice, lectores separados y search.query/rebuild mantienen shapes y significado

#### Scenario: Entrada estricta por acción
- **WHEN** se omiten revisiones/generación requeridas, se mezclan campos de acciones o se envían campos desconocidos, IDs/cursor/límites inválidos, shell, ruta de helper o proveedor
- **THEN** se devuelve validation_error y salida JSON estable, sin ejecutar comandos, escribir selecciones ni reinterpretar el input

#### Scenario: Estado paginado
- **WHEN** se pide index get con filtros admitidos y límite válido
- **THEN** devuelve revisión/perfil/cobertura, selecciones ordenadas y cursor acotado ligado a su operación/filtros/revisión; get no usa los cursores de search ni modifica datos

### Requirement: Ejecución scoped sobre workspace actual

El núcleo SHALL ofrecer ejecución scoped con ruta canónica, identidad y política fijadas por el host, sin admitir override del cliente. MUST comprobar versión e identidad en la conexión/snapshot realmente usados y rechazar un workspace no actual sin crear, migrar, restaurar ni reparar. Esta entrada conserva execute/execute_json y apertura explícita CLI/UI; la evolución de index/search y la migración v12 son las excepciones documentadas a la igualdad con v11.

#### Scenario: Scope actual y versión incompatible
- **WHEN** se ejecuta la entrada scoped sobre el esquema vigente válido y sobre fixtures ausente, anterior, futura o inconsistente
- **THEN** sólo el actual se admite; los demás conservan sus datos y devuelven not_found, unsupported_workspace_version o storage_error según corresponda, sin backup de migración ni cambios de formato

#### Scenario: Identidad comprobada en la operación
- **WHEN** se reemplaza el workspace por otro entre admisión inicial y apertura de una operación genérica o especializada
- **THEN** se rechaza la identidad distinta antes de efectos usando la conexión/snapshot efectivo, sin confiar únicamente en el check realizado al arrancar

#### Scenario: Clientes anteriores conservados
- **WHEN** se contrastan inputs/outputs/definiciones/errores/lotes con baseline v11/52 y se ejecutan CLI/UI sin la entrada scoped
- **THEN** sólo evolucionan los enums/DTOs y descriptor de index/search previstos; los otros 50 contratos, lectores y whitelist se conservan, con apertura explícita al esquema vigente sin ejecución implícita

### Requirement: Contrato OCR compartido y aditivo

El catálogo/API v1 SHALL añadir extraction.ocr_regions y exponer image_ocr/proyección de regiones en index/search sin modificar los otros cincuenta contratos previos, errores, nueve mutaciones batchables o siete exclusiones MCP. MUST compartir validaciones entre Rust/CLI/MCP/UI, rechazar entradas ajenas y usar campos decimales explícitos sin convertir OCR en UTF-8 original o resultado XLSX.

#### Scenario: Clientes y catálogo OCR
- **WHEN** se describe el catálogo vigente de54 operaciones y se solicitan regiones equivalentes por CLI y MCP autorizado
- **THEN** los inputs/outputs/errores son completos y equivalentes, con fuente/manifest/localizadores y límites reales, salvo metadatos de ejecución documentados

#### Scenario: Entradas lectores y versiones
- **WHEN** se envían campos desconocidos, límite/cursor/ID inválidos, versión futura o se usa un lector de otro formato
- **THEN** se rechaza sin coerción/fallback, ejecución o escritura; los lectores anteriores conservan sus shapes y semántica

#### Scenario: Compatibilidad contractual completa
- **WHEN** se comparan catálogos y DTOs con el baseline v12 durable quitando sólo adiciones y ampliaciones identificadas
- **THEN** los cincuenta contratos no afectados, las nueve mutaciones, límites y formatos de documento se conservan exactamente, sin habilitar funciones futuras

## ADDED Requirements

### Requirement: Extensión contractual acotada de búsqueda PDFKit

El catálogo SHALL identificar pdfkit_text y pdfkit_page_text en los DTOs de index/search, con perfil/proyección, página/rango y motor/limitaciones reales. MUST conservar52 contratos restantes del baseline14/54, errores, batch9/MCP26/46/47 y equivalencia Rust/CLI/MCP/UI; no añade operaciones, helper/ruta o permisos al input.

#### Scenario: Contratos y paridad
- **WHEN** se comparan catálogos completos con el cliente14 durable y se busca/selecciona PDFKit por CLI y MCP autorizado
- **THEN** sólo los dos contratos nombrados amplían sus tipos y los resultados son equivalentes con las mismas cuotas/cercas/errores; los lectores anteriores son idénticos

#### Scenario: Input estricto y permisos conservados
- **WHEN** un cliente envía campos/locators no admitidos, indexa sin escritura o intenta usar extracción/indexación como step batchable
- **THEN** se rechaza sin preparar fuentes, ampliar whitelist, recibir comandos/rutas de helper o cambiar la política de transporte
