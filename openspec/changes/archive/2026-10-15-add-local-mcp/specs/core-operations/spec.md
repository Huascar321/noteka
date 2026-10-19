# Spec Delta

## ADDED Requirements

### Requirement: Ejecución scoped sobre workspace actual

El núcleo SHALL ofrecer ejecución scoped aditiva con ruta canónica, identidad y política fijadas por el host, sin admitir override del cliente. MUST comprobar versión e identidad en la conexión/snapshot realmente usados y rechazar un workspace no actual sin crear, migrar, restaurar ni reparar. Esta entrada conserva execute/execute_json, contratos API v1 y apertura histórica de CLI/UI.

#### Scenario: Scope actual y versión incompatible
- **WHEN** se ejecuta la entrada scoped sobre v10 válido y sobre fixtures ausente, anterior, futura o inconsistente
- **THEN** sólo el actual se admite; los demás conservan sus datos y devuelven not_found, unsupported_workspace_version o storage_error según corresponda, sin backup de migración ni cambios de formato

#### Scenario: Identidad comprobada en la operación
- **WHEN** se reemplaza el workspace por otro entre admisión inicial y apertura de una operación genérica o especializada
- **THEN** se rechaza la identidad distinta antes de efectos usando la conexión/snapshot efectivo, sin confiar únicamente en el check realizado al arrancar

#### Scenario: Clientes anteriores conservados
- **WHEN** se comparan los 51 inputs/outputs, definiciones, errores y operaciones de lote previos y se ejecutan CLI/UI sin la nueva entrada scoped
- **THEN** no cambian sus shapes, whitelist, formatos persistentes, validaciones ni comportamiento de apertura/migración ya documentado

### Requirement: Lecturas scoped sin mutaciones persistentes

Las lecturas scoped SHALL consultar SQLite principal sin permiso de escritura y sin migración, reparación, reindexación, recuperación de jobs o cambios de dominio/derivados. MUST conservar originales y formatos desconocidos no modificados, incluir commits visibles en WAL y permitir temporales privados cuando la operación los necesite. Preview mantiene su evaluación aislada y los límites existentes.

#### Scenario: Lecturas generales y acciones get
- **WHEN** se consultan entidades, colecciones, registros, fórmulas derivadas, vistas, templates, Home/workflows, historial, búsqueda y metadatos/fragmentos de extracción, incluido control/index get
- **THEN** se obtienen los mismos resultados que el contrato existente con main sólo legible, sin incrementar revisiones ni modificar dominio, FTS, control, ownership, intentos o generaciones

#### Scenario: WAL y estado operativo pendientes
- **WHEN** hay commits en WAL, trabajos queued/running vencidos y contenido de formatos futuros que la consulta no edita
- **THEN** se leen datos confirmados sin usar immutable para ignorar WAL, recuperar trabajos, ejecutar helpers, descartar bytes desconocidos o presentar una reparación como lectura

#### Scenario: Daño o lectura incompatible
- **WHEN** una lectura detecta índice dañado, estructura incompatible o falta de acceso a los temporales necesarios
- **THEN** devuelve el error recuperable existente sin resetear, reconstruir ni abrir main con permisos superiores para completar la consulta

### Requirement: Política scoped validada antes de efectos

El núcleo MUST clasificar explícitamente cada operación y acción como lectura, escritura, ejecución o excluida para la entrada scoped, denegando desconocidas por defecto. SHALL validar DTOs, permisos y límites antes de efectos; ejecución exige escritura. La política no deriva de nombres, compatibilidad con lotes, annotations o recibos. El host fija origen y el núcleo genera request_id; ninguno procede del input de herramientas.

#### Scenario: Acción mixta y operación excluida
- **WHEN** se llama control set/index put/reset sin escritura, run sin ambos permisos o administración/import/read/export por la entrada scoped
- **THEN** se rechaza con validation_error y reason permission_denied sin abrir una conexión de escritura, recuperar trabajos ni acceder a rutas de la operación excluida

#### Scenario: DTO inválido y lote permitido
- **WHEN** una operación permitida contiene campos desconocidos, una revisión obsoleta o un lote con un paso no admitido
- **THEN** se mantienen errores y atomicidad de dominio existentes; un preview válido no otorga permiso para apply y un recibo ajeno/obsoleto no se acepta

#### Scenario: Origen compartido sin falsificación
- **WHEN** una mutación scoped válida confirma con origen mcp y el cliente intenta aportar otro origen o request_id dentro de los argumentos
- **THEN** los campos ajenos se rechazan y sólo la ejecución válida registra el origen confiable y su identificador de solicitud generado por el núcleo

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
- **THEN** las nueve operaciones de extracción publican esquemas/límites y distinguen utf8_text de pdf_text acotado, sin compatibilidad como paso/check de lote ni anuncio de PDF genérico, OCR, Excel, embeddings, LLM o ejecución residente

#### Scenario: Paridad de herramientas MCP habilitadas
- **WHEN** CLI y MCP autorizado ejecutan entradas equivalentes de su superficie común sobre workspaces actuales equivalentes, dentro de los límites de entrega MCP
- **THEN** conservan validaciones, cálculos exactos, CAS, estados y motivos de error de dominio, salvo scope, IDs aleatorios y metadatos de ejecución documentados; el transporte no añade reglas de negocio propias

#### Scenario: Catálogo completo y disponibilidad por transporte
- **WHEN** se compara el catálogo compartido con tools/list y el recurso noteka://catalog
- **THEN** siguen existiendo 51 operaciones con sus contratos previos, diferenciadas de las 23/43/44 herramientas y acciones habilitadas; las siete operaciones excluidas de MCP no desaparecen del CLI ni se anuncian como invocables por ese transporte
