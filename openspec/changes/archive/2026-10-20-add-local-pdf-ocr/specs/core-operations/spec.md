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
- **THEN** las quince operaciones de extracción distinguen utf8_text, pdf_text, xlsx_cells, image_ocr pdfkit_text y pdf_ocr acotados y sus proyecciones explícitas, sin compatibilidad de extracción como paso/check ni anuncio de PDF/Excel genérico, embeddings, LLM o ejecución residente

#### Scenario: Paridad de herramientas MCP habilitadas
- **WHEN** CLI y MCP autorizado ejecutan entradas equivalentes de su superficie común sobre workspaces actuales equivalentes, dentro de los límites de entrega MCP
- **THEN** conservan validaciones, cálculos exactos, CAS, estados y motivos de error de dominio, salvo scope, IDs aleatorios y metadatos de ejecución documentados; el transporte no añade reglas de negocio propias

#### Scenario: Catálogo completo y disponibilidad por transporte
- **WHEN** se compara el catálogo compartido con tools/list y el recurso noteka://catalog
- **THEN** existen55 operaciones y27/47/48 herramientas; respecto del baseline14/54 sólo index/search amplían enums/localizadores PDFKit y su metadata, mientras los52 contratos restantes, errores, nueve mutaciones batchables y siete exclusiones MCP se conservan

### Requirement: Operaciones aditivas de indexación y búsqueda de adjuntos

El núcleo SHALL publicar `extraction.index` y `extraction.search` en API v1, catálogo y CLI JSON, con DTOs estrictos compartidos. Index distingue get paginado, put con trabajo/generación/revisión esperados y reset con revisión esperada; search admite sólo consulta/filtros/límite/cursor documentados. MUST conservar sin cambios los 49 contratos anteriores y no añadir rutas, comandos o activación de proveedores en estas entradas.

#### Scenario: Catálogo y compatibilidad
- **WHEN** un cliente inspecciona el catálogo y ejecuta las operaciones por núcleo o CLI
- **THEN** encuentra55 operaciones con esquemas de entrada/salida y errores compartidos; los 49 contratos anteriores al índice, lectores separados y search.query/rebuild mantienen shapes y significado

#### Scenario: Entrada estricta por acción
- **WHEN** se omiten revisiones/generación requeridas, se mezclan campos de acciones o se envían campos desconocidos, IDs/cursor/límites inválidos, shell, ruta de helper o proveedor
- **THEN** se devuelve validation_error y salida JSON estable, sin ejecutar comandos, escribir selecciones ni reinterpretar el input

#### Scenario: Estado paginado
- **WHEN** se pide index get con filtros admitidos y límite válido
- **THEN** devuelve revisión/perfil/cobertura, selecciones ordenadas y cursor acotado ligado a su operación/filtros/revisión; get no usa los cursores de search ni modifica datos

### Requirement: Contrato OCR compartido y aditivo

El catálogo/API v1 SHALL añadir extraction.ocr_regions y exponer image_ocr/proyección de regiones en index/search sin modificar los otros cincuenta contratos previos, errores, nueve mutaciones batchables o siete exclusiones MCP. MUST compartir validaciones entre Rust/CLI/MCP/UI, rechazar entradas ajenas y usar campos decimales explícitos sin convertir OCR en UTF-8 original o resultado XLSX.

#### Scenario: Clientes y catálogo OCR
- **WHEN** se describe el catálogo vigente de55 operaciones y se solicitan regiones equivalentes por CLI y MCP autorizado
- **THEN** los inputs/outputs/errores son completos y equivalentes, con fuente/manifest/localizadores y límites reales, salvo metadatos de ejecución documentados

#### Scenario: Entradas lectores y versiones
- **WHEN** se envían campos desconocidos, límite/cursor/ID inválidos, versión futura o se usa un lector de otro formato
- **THEN** se rechaza sin coerción/fallback, ejecución o escritura; los lectores anteriores conservan sus shapes y semántica

#### Scenario: Compatibilidad contractual completa
- **WHEN** se comparan catálogos y DTOs con el baseline v12 durable quitando sólo adiciones y ampliaciones identificadas
- **THEN** los cincuenta contratos no afectados, las nueve mutaciones, límites y formatos de documento se conservan exactamente, sin habilitar funciones futuras

### Requirement: Extensión contractual acotada de búsqueda PDFKit

El catálogo SHALL identificar pdfkit_text y pdfkit_page_text en los DTOs de index/search, con perfil/proyección, página/rango y motor/limitaciones reales. MUST conservar52 contratos restantes del baseline14/54, errores, batch9/MCP27/47/48 y equivalencia Rust/CLI/MCP/UI; no añade operaciones, helper/ruta o permisos al input.

#### Scenario: Contratos y paridad
- **WHEN** se comparan catálogos completos con el cliente14 durable y se busca/selecciona PDFKit por CLI y MCP autorizado
- **THEN** sólo los dos contratos nombrados amplían sus tipos y los resultados son equivalentes con las mismas cuotas/cercas/errores; los lectores anteriores son idénticos

#### Scenario: Input estricto y permisos conservados
- **WHEN** un cliente envía campos/locators no admitidos, indexa sin escritura o intenta usar extracción/indexación como step batchable
- **THEN** se rechaza sin preparar fuentes, ampliar whitelist, recibir comandos/rutas de helper o cambiar la política de transporte

## ADDED Requirements

### Requirement: Contrato compartido de regiones PDF OCR

El catálogo SHALL añadir extraction.pdf_ocr_regions en APIv1/CLI/MCP con DTOs estrictos y validar pdf_ocr/localizadores en index/search. MUST conservar52 operaciones no afectadas del baseline15/54, errores, batch9 y siete exclusiones; el lector no acepta helpers, rutas, comandos, credenciales o activación remota.

#### Scenario: Paridad y catálogo completo
- **WHEN** se comparan catálogos15/16 y se consulta la misma generación por núcleo/CLI/MCP autorizado
- **THEN** sólo index/search amplían tipos y se añade el lector; API55/MCP27/47/48 comparten datos/validaciones salvo metadatos de ejecución documentados

#### Scenario: Entradas y pasos ajenos
- **WHEN** se aportan campos desconocidos, cursor/límite/tipo ajenos o la lectura/extracción/indexación como step batchable
- **THEN** rechaza sin writes/procesamiento ni ampliación de whitelist/permisos
