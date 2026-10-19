# Spec Delta

## MODIFIED Requirements

### Requirement: Contrato compartido y versionado

El sistema SHALL exponer un catálogo versionado con esquemas de entrada, salida y errores para todas las operaciones implementadas, incluidas colecciones, propiedades, fórmulas, vistas, plantillas, consultas, lotes y extracción local. Interfaz y CLI MUST aplicar las mismas reglas; el catálogo MUST identificar operaciones compatibles con lotes, referencias admitidas y límites, sin anunciar funciones futuras como disponibles.

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
- **THEN** las ocho operaciones de extracción publican esquemas y límites, sin compatibilidad como paso/check de lote ni anuncio de PDF, OCR, Excel, embeddings, LLM o ejecución residente

### Requirement: Escritura atómica validada

El sistema MUST validar cada operación antes de confirmarla y guardar conjuntamente contenido, revisión y registro de cambio aplicables, sin estados parciales si falla una validación o la persistencia. Las transiciones de trabajos y la publicación de derivados SHALL ser atómicas sin crear cambios de entidades que no fueron modificadas.

#### Scenario: Documento inválido durante actualización

- **WHEN** una actualización incluye título válido y documento inválido
- **THEN** se rechaza la operación completa y se conservan título, documento y revisión anteriores

#### Scenario: Falla de persistencia

- **WHEN** ocurre un fallo de almacenamiento antes del commit
- **THEN** no aparece ningún cambio parcial confirmado y se devuelve `storage_error`

#### Scenario: Claim separado de publicación

- **WHEN** un trabajo reclama ownership correctamente y falla después al procesar o publicar
- **THEN** el claim confirmado puede quedar como intento recuperable, pero no aparecen una generación parcial ni modificaciones de notas; el error identifica trabajo y estado sin prometer rollback del intento

### Requirement: CLI operativo con ventana cerrada

El CLI SHALL permitir crear/abrir workspace, consultar catálogo, operar entidades/documentos/relaciones, colecciones, propiedades, registros, fórmulas, vistas, plantillas y consultas, archivar/restaurar entidades y realizar backup/restauración sin iniciar la interfaz gráfica. SHALL ofrecer changes preview/apply y extraction enqueue/get/list/run/cancel/retry/control/fragments con entrada/salida JSON y las mismas validaciones del núcleo.

#### Scenario: Nota creada por CLI

- **WHEN** se crea una nota desde el CLI con la ventana cerrada y luego se abre ese workspace en escritorio
- **THEN** la interfaz puede consultar la misma nota y revisión

#### Scenario: Lote sin escritorio ni red

- **WHEN** se previsualiza y aplica un plan válido por CLI sin red, LLM ni ventana de escritorio
- **THEN** stdout contiene un único resultado JSON por llamada y los errores conservan los códigos de salida documentados, sin iniciar procesos gráficos

#### Scenario: Extracción sin escritorio ni red

- **WHEN** se encola, habilita, ejecuta y consulta un texto sintético desde el CLI con red denegada y ventana cerrada
- **THEN** se obtienen estados y fragmentos verificables; stdout sigue siendo un único JSON por llamada y fallos/conflictos usan los códigos documentados

### Requirement: Historial local de mutaciones

El sistema SHALL registrar en cada mutación confirmada de una entidad la entidad afectada, revisión anterior/nueva, operación y origen del cliente, permitiendo consultar esos registros sin un servicio externo. Los trabajos de extracción MUST conservar su trazabilidad operativa sin inventar entidades ni incrementos del historial de notas.

#### Scenario: Mutación rechazada y aceptada

- **WHEN** primero se rechaza una edición por conflicto y después se confirma una edición válida
- **THEN** el historial contiene la mutación confirmada y no presenta la rechazada como aplicada

#### Scenario: Procesamiento sin historial ficticio

- **WHEN** se encola, ejecuta, pausa, cancela o reintenta extracción sin editar entidades
- **THEN** cambian solamente configuración o datos de procesamiento pertinentes; el historial y las revisiones de entidades permanecen iguales

## ADDED Requirements

### Requirement: Operaciones compartidas de extracción local

El núcleo SHALL exponer extraction.enqueue/get/list/run/cancel/retry/control/fragments con DTOs estrictos, revisión esperada para control/cancel/retry y lecturas paginadas. MUST compartir validación y errores entre clientes, distinguir ejecución de consulta y excluir las ocho operaciones de los planes de cambios. Un fallo de procesamiento SHALL identificar el intento confirmado sin devolverlo como éxito de extracción.

#### Scenario: Paridad de estado y error

- **WHEN** núcleo y CLI realizan el mismo recorrido de extracción o modificación con revisión obsoleta sobre workspaces equivalentes
- **THEN** producen iguales fragmentos, transiciones, errores y motivos, salvo IDs aleatorios, scope y metadatos de ejecución documentados

#### Scenario: Entrada estricta y control de lectura

- **WHEN** se solicita control de sólo lectura, o se envían campos desconocidos, SQL, configuración no admitida o una modificación sin revisión
- **THEN** la lectura no modifica datos y la entrada inválida se rechaza sin encolar, reclamar o ejecutar contenido

#### Scenario: Operación excluida del lote

- **WHEN** un plan intenta ejecutar o consultar extracción como paso o check
- **THEN** se rechaza completo antes de confirmar efectos, sin crear trabajos, ejecutar workers ni editar originales

### Requirement: Recibos independientes de derivados de extracción

Los lotes SHALL incluir todo SQLite en el presupuesto/snapshot y reconocer el esquema de extracción soportado sin ejecutarla. MUST vincular recibos a configuración original de la cola y excluir contenido operativo/derivado de extracción de la huella base. Cambios de fuente, configuración original o estructura SQL siguen invalidando el recibo; apply conserva trabajos/resultados ajenos en la base original.

#### Scenario: Procesamiento entre preview y apply

- **WHEN** se encola, ejecuta, cancela o reintenta extracción entre preview y apply sin cambiar datos originales ni superar límites
- **THEN** ese procesamiento no invalida por sí solo el recibo y apply conserva la cola y generaciones actuales, sin reemplazarlas por la copia del preview

#### Scenario: Configuración original entre preview y apply

- **WHEN** se pausa o reanuda la cola después de un preview, incluso si luego se vuelve al valor anterior
- **THEN** cambia su revisión original y apply devuelve revision_conflict; se necesita otro preview

#### Scenario: Esquema y tamaño siguen limitados

- **WHEN** derivados hacen superar 64 MiB de snapshot, falta una tabla requerida o aparece una extensión SQL no admitida
- **THEN** preview/apply rechazan de forma recuperable sin borrar derivados, migrar, aumentar el límite ni reparar datos

#### Scenario: Preview no reclama ni recupera

- **WHEN** se previsualiza un lote en presencia de queued, running vencidos o trabajos de configuración futura
- **THEN** no cambian sus estados, revisiones, intentos, ownership ni resultados y no se ejecuta extracción en la copia ni en el original
