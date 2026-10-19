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
- **THEN** existen54 operaciones y26/46/47 herramientas; extraction.pdfkit_pages es aditiva respecto del baseline13/53, cuyos53 inputs/outputs, errores, nueve mutaciones batchables y siete exclusiones MCP se conservan salvo metadatos de capacidad identificados

## ADDED Requirements

### Requirement: Contrato compartido de páginas PDFKit

El catálogo/API v1 SHALL añadir extraction.pdfkit_pages y CLI extraction pdfkit-pages con DTOs estrictos y validación Rust compartida. MUST conservar los53 inputs/outputs previos, errores, whitelist batch9 y siete exclusiones MCP; sólo añade definiciones propias y metadatos de capacidad identificados. No acepta rutas de helper, comandos, contraseñas o selección remota en solicitudes.

#### Scenario: Paridad y contratos anteriores
- **WHEN** se compara el catálogo con baseline13 durable y se leen páginas equivalentes mediante núcleo/CLI/MCP autorizado
- **THEN** los53 contratos anteriores permanecen iguales y la nueva lectura devuelve los mismos datos/cobertura, sin convertirse en step batchable

#### Scenario: Entradas estrictas
- **WHEN** se envían campos desconocidos, límites/cursor/tipo ajenos o se intenta insertar la lectura en un batch
- **THEN** se rechaza antes de writes/procesamiento, sin ampliar permisos o whitelist
