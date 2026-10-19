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
- **THEN** las trece operaciones de extracción distinguen utf8_text, pdf_text, xlsx_cells e image_ocr acotados y sus proyecciones explícitas, sin compatibilidad de extracción como paso/check ni anuncio de PDF/Excel genérico, embeddings, LLM o ejecución residente

#### Scenario: Paridad de herramientas MCP habilitadas
- **WHEN** CLI y MCP autorizado ejecutan entradas equivalentes de su superficie común sobre workspaces actuales equivalentes, dentro de los límites de entrega MCP
- **THEN** conservan validaciones, cálculos exactos, CAS, estados y motivos de error de dominio, salvo scope, IDs aleatorios y metadatos de ejecución documentados; el transporte no añade reglas de negocio propias

#### Scenario: Catálogo completo y disponibilidad por transporte
- **WHEN** se compara el catálogo compartido con tools/list y el recurso noteka://catalog
- **THEN** existen 53 operaciones y 25/45/46 herramientas; extraction.ocr_regions es aditiva y sólo index/search amplían sus DTOs de extractor/localizador OCR, los otros 50 contratos y las siete exclusiones MCP se conservan

## ADDED Requirements

### Requirement: Contrato OCR compartido y aditivo

El catálogo/API v1 SHALL añadir extraction.ocr_regions y exponer image_ocr/proyección de regiones en index/search sin modificar los otros cincuenta contratos previos, errores, nueve mutaciones batchables o siete exclusiones MCP. MUST compartir validaciones entre Rust/CLI/MCP/UI, rechazar entradas ajenas y usar campos decimales explícitos sin convertir OCR en UTF-8 original o resultado XLSX.

#### Scenario: Clientes y catálogo OCR
- **WHEN** se describen las 53 operaciones y se solicitan regiones equivalentes por CLI y MCP autorizado
- **THEN** los inputs/outputs/errores son completos y equivalentes, con fuente/manifest/localizadores y límites reales, salvo metadatos de ejecución documentados

#### Scenario: Entradas lectores y versiones
- **WHEN** se envían campos desconocidos, límite/cursor/ID inválidos, versión futura o se usa un lector de otro formato
- **THEN** se rechaza sin coerción/fallback, ejecución o escritura; los lectores anteriores conservan sus shapes y semántica

#### Scenario: Compatibilidad contractual completa
- **WHEN** se comparan catálogos y DTOs con el baseline v12 durable quitando sólo adiciones y ampliaciones identificadas
- **THEN** los cincuenta contratos no afectados, las nueve mutaciones, límites y formatos de documento se conservan exactamente, sin habilitar funciones futuras
