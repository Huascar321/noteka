# Spec Delta

## ADDED Requirements

### Requirement: Contratos PDF aditivos sin reinterpretar texto literal

El núcleo SHALL añadir extraction.pdf_fragments con entrada estricta y salida PDF dedicada, conservando API v1 y los shapes previos. MUST mantener extraction.fragments como lector literal, sin offsets PDF ficticios; cada lector rechaza generaciones del otro tipo. La selección pdf_text amplía enqueue, no introduce ejecución automática ni shell, rutas de helper o contraseñas en solicitudes públicas.

#### Scenario: Lectores separados y DTOs anteriores
- **WHEN** se comparan esquemas previos o se solicita fragments sobre PDF y pdf_fragments sobre utf8_text
- **THEN** los inputs/outputs anteriores permanecen iguales y el tipo ajeno devuelve unsupported_generation_config, no un resultado con campos nulos o semántica de offsets alterada

#### Scenario: Entrada PDF estricta
- **WHEN** se envían campos desconocidos, límite/cursor inválidos, contraseña, comando, ruta de helper o versión/configuración no soportada
- **THEN** se rechaza con validation_error sin ejecución ni escritura y los datos futuros persistidos se conservan crudos para consultas de metadatos

#### Scenario: Helper no disponible sólo afecta ejecución PDF
- **WHEN** el helper local está ausente o es incompatible y se describen operaciones, leen resultados previos o se ejecuta texto literal
- **THEN** esas operaciones no necesitan el helper; sólo un run PDF reclamado falla de forma recuperable con motivo explícito sin fallback en proceso, red o descarga automática

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
- **THEN** las nueve operaciones de extracción publican esquemas/límites y distinguen utf8_text de pdf_text acotado, sin compatibilidad como paso/check de lote ni anuncio de PDF genérico, OCR, Excel, embeddings, LLM o ejecución residente

### Requirement: CLI operativo con ventana cerrada

El CLI SHALL permitir crear/abrir workspace, consultar catálogo, operar entidades/documentos/relaciones, colecciones, propiedades, registros, fórmulas, vistas, plantillas y consultas, archivar/restaurar entidades y realizar backup/restauración sin iniciar la interfaz gráfica. SHALL ofrecer changes preview/apply y extraction enqueue/get/list/run/cancel/retry/control/fragments/pdf-fragments con entrada/salida JSON y las mismas validaciones del núcleo.

#### Scenario: Nota creada por CLI
- **WHEN** se crea una nota desde el CLI con la ventana cerrada y luego se abre ese workspace en escritorio
- **THEN** la interfaz puede consultar la misma nota y revisión

#### Scenario: Lote sin escritorio ni red
- **WHEN** se previsualiza y aplica un plan válido por CLI sin red, LLM ni ventana de escritorio
- **THEN** stdout contiene un único resultado JSON por llamada y los errores conservan los códigos de salida documentados, sin iniciar procesos gráficos

#### Scenario: Extracción sin escritorio ni red
- **WHEN** se encola, habilita, ejecuta y consulta un texto sintético desde el CLI con red denegada y ventana cerrada
- **THEN** se obtienen estados y fragmentos verificables; stdout sigue siendo un único JSON por llamada y fallos/conflictos usan los códigos documentados

#### Scenario: PDF con helper y errores compartidos
- **WHEN** se realiza el recorrido PDF por CLI y por el núcleo con un helper real, incluidos fallo de parsing, interrupción y revisión obsoleta
- **THEN** estados, manifiestos, fragmentos y errores son equivalentes salvo metadatos de ejecución; stdout público conserva un único envelope JSON y los logs del parser no se mezclan con canales públicos

### Requirement: Operaciones compartidas de extracción local

El núcleo SHALL exponer extraction.enqueue/get/list/run/cancel/retry/control/fragments/pdf_fragments con DTOs estrictos, revisión esperada para control/cancel/retry y lecturas paginadas. MUST compartir validación y errores entre clientes, distinguir ejecución de consulta y excluir las nueve operaciones de los planes de cambios. Un fallo de procesamiento SHALL identificar el intento confirmado sin devolverlo como éxito de extracción.

#### Scenario: Paridad de estado y error
- **WHEN** núcleo y CLI realizan el mismo recorrido de extracción o modificación con revisión obsoleta sobre workspaces equivalentes
- **THEN** producen iguales fragmentos, transiciones, errores y motivos, salvo IDs aleatorios, scope y metadatos de ejecución documentados

#### Scenario: Entrada estricta y control de lectura
- **WHEN** se solicita control de sólo lectura, o se envían campos desconocidos, SQL, configuración no admitida o una modificación sin revisión
- **THEN** la lectura no modifica datos y la entrada inválida se rechaza sin encolar, reclamar o ejecutar contenido

#### Scenario: Operación excluida del lote
- **WHEN** un plan intenta ejecutar o consultar extracción como paso o check, incluida pdf_fragments
- **THEN** se rechaza completo antes de confirmar efectos, sin crear trabajos, ejecutar workers ni editar originales

### Requirement: Recibos independientes de derivados de extracción

Los lotes SHALL incluir todo SQLite en el presupuesto/snapshot y reconocer el esquema de extracción soportado sin ejecutarla. MUST vincular recibos a configuración original de la cola y excluir contenido operativo/derivado de extracción, incluido PDF, de la huella base. Cambios de fuente, configuración original o estructura SQL siguen invalidando el recibo; apply conserva trabajos/resultados ajenos en la base original.

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

#### Scenario: PDF confirmado entre preview y apply
- **WHEN** se publica o regenera PDF después de un preview válido en el mismo esquema, sin editar originales/control ni exceder presupuesto
- **THEN** el recibo sigue vinculando originales con namespace base v2 y apply conserva el manifiesto/fragmentos actuales; la whitelist de nueve mutaciones y database.query no se amplía

#### Scenario: Recibo anterior a migración PDF
- **WHEN** se intenta aplicar en v9 un recibo preparado antes de migrar v8
- **THEN** se rechaza por estado base incompatible, sin reinterpretar la huella ni migrar automáticamente; se requiere preview nuevo
