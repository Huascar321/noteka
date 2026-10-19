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
- **THEN** las doce operaciones de extracción distinguen utf8_text, pdf_text y xlsx_cells acotados y anuncian xlsx_cell_fields_v1 sólo para index/search, sin compatibilidad de extracción como paso/check ni anuncio de PDF/Excel genérico, OCR, embeddings, LLM o ejecución residente

#### Scenario: Paridad de herramientas MCP habilitadas
- **WHEN** CLI y MCP autorizado ejecutan entradas equivalentes de su superficie común sobre workspaces actuales equivalentes, dentro de los límites de entrega MCP
- **THEN** conservan validaciones, cálculos exactos, CAS, estados y motivos de error de dominio, salvo scope, IDs aleatorios y metadatos de ejecución documentados; el transporte no añade reglas de negocio propias

#### Scenario: Catálogo completo y disponibilidad por transporte
- **WHEN** se compara el catálogo compartido con tools/list y el recurso noteka://catalog
- **THEN** existen 52 operaciones y 24/44/45 herramientas; sólo index/search amplían sus DTOs de extractor/localizador XLSX, los otros 50 contratos y las siete exclusiones MCP se conservan

### Requirement: Ejecución scoped sobre workspace actual

El núcleo SHALL ofrecer ejecución scoped con ruta canónica, identidad y política fijadas por el host, sin admitir override del cliente. MUST comprobar versión e identidad en la conexión/snapshot realmente usados y rechazar un workspace no actual sin crear, migrar, restaurar ni reparar. Esta entrada conserva execute/execute_json y apertura explícita CLI/UI; la evolución de index/search y la migración v12 son las excepciones documentadas a la igualdad con v11.

#### Scenario: Scope actual y versión incompatible
- **WHEN** se ejecuta la entrada scoped sobre v12 válido y sobre fixtures ausente, anterior, futura o inconsistente
- **THEN** sólo el actual se admite; los demás conservan sus datos y devuelven not_found, unsupported_workspace_version o storage_error según corresponda, sin backup de migración ni cambios de formato

#### Scenario: Identidad comprobada en la operación
- **WHEN** se reemplaza el workspace por otro entre admisión inicial y apertura de una operación genérica o especializada
- **THEN** se rechaza la identidad distinta antes de efectos usando la conexión/snapshot efectivo, sin confiar únicamente en el check realizado al arrancar

#### Scenario: Clientes anteriores conservados
- **WHEN** se contrastan inputs/outputs/definiciones/errores/lotes con baseline v11/52 y se ejecutan CLI/UI sin la entrada scoped
- **THEN** sólo evolucionan los enums/DTOs y descriptor de index/search previstos; los otros 50 contratos, lectores y whitelist se conservan, con apertura explícita v12 sin ejecución implícita

### Requirement: Contrato compartido de lectura XLSX

El núcleo SHALL conservar `extraction.xlsx_cells` y CLI `extraction xlsx-cells` con input/output tipados, validación Rust y equivalencia con invoke en API v1/52. MUST mantener su lectura de celdas completas independiente de los fragmentos de búsqueda, errores/salida JSON y exclusión de extracción de los nueve steps batchables; bindings frontend se generan desde el catálogo sin nuevas pantallas. La ampliación de index/search no cambia este reader.

#### Scenario: CLI y core equivalentes
- **WHEN** se consulta la misma generación/paginación con invoke y el subcomando XLSX
- **THEN** devuelven el mismo resultado tipado y los esquemas publicados describen valores léxicos, fórmulas, metadatos y localizadores sin tipos any abiertos

#### Scenario: Conservación y ámbito limitado
- **WHEN** se comparan catálogos con baseline v11/52 y se intenta insertar extracción/indexación/búsqueda XLSX en un lote
- **THEN** sólo los DTOs y descriptor de index/search tienen las extensiones nombradas; extraction.xlsx_cells y los otros contratos son idénticos y el lote rechaza la operación sin mutaciones parciales
