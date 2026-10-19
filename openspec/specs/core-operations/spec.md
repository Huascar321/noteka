# core-operations Specification

## Purpose
Ofrecer operaciones estructuradas y consistentes para la interfaz y el CLI, de modo que clientes humanos o agentes operen el mismo conocimiento sin saltarse validaciones ni sobrescribir cambios concurrentes.

## Requirements

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

### Requirement: Concurrencia con revisión esperada

El sistema MUST exigir la revisión esperada en modificaciones de entidades existentes y rechazar una revisión obsoleta con `revision_conflict`, informando la revisión actual sin sobrescribirla.

#### Scenario: Dos clientes editan una nota

- **WHEN** dos clientes leen la revisión 1 y el primero confirma su actualización
- **THEN** la actualización del segundo con revisión 1 se rechaza y permanece el contenido del primero

#### Scenario: Cliente omite revisión

- **WHEN** se solicita modificar o archivar una entidad existente sin revisión esperada
- **THEN** se devuelve `validation_error` sin modificar datos

### Requirement: Contención acotada del almacenamiento

El sistema SHALL limitar la espera ante otro escritor y devolver un error recuperable `workspace_busy` al agotarla, sin bloquear indefinidamente ni reejecutar automáticamente una mutación ya confirmada.

#### Scenario: Escritor mantiene bloqueo

- **WHEN** otro escritor mantiene el bloqueo más allá del timeout documentado
- **THEN** la operación termina con `workspace_busy` y no modifica el workspace

### Requirement: Resultados y errores estructurados

El sistema SHALL devolver resultados con versión de contrato, identificador de solicitud y datos, o errores con código, mensaje y detalles útiles, sin incluir secretos o trazas internas por defecto.

#### Scenario: Entidad no encontrada

- **WHEN** se solicita un ID que no existe en el workspace
- **THEN** se devuelve `not_found` con un identificador de solicitud y sin traza interna

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

### Requirement: Canales de salida y códigos del CLI

El CLI SHALL producir JSON válido exclusivamente en stdout en modo estructurado y enviar logs a stderr. SHALL documentar códigos diferentes para éxito, validación, no encontrado, conflicto y almacenamiento.

#### Scenario: Operación estructurada exitosa

- **WHEN** una operación en modo JSON termina correctamente
- **THEN** stdout contiene un único resultado JSON y el código de salida indica éxito

#### Scenario: Entrada inválida

- **WHEN** el CLI recibe JSON inválido o argumentos incompatibles
- **THEN** devuelve un error JSON, un código de validación y no escribe datos

### Requirement: Historial local de mutaciones

El sistema SHALL registrar en cada mutación confirmada de una entidad la entidad afectada, revisión anterior/nueva, operación y origen del cliente, permitiendo consultar esos registros sin un servicio externo. Los trabajos de extracción MUST conservar su trazabilidad operativa sin inventar entidades ni incrementos del historial de notas.

#### Scenario: Mutación rechazada y aceptada

- **WHEN** primero se rechaza una edición por conflicto y después se confirma una edición válida
- **THEN** el historial contiene la mutación confirmada y no presenta la rechazada como aplicada

#### Scenario: Procesamiento sin historial ficticio

- **WHEN** se encola, ejecuta, pausa, cancela o reintenta extracción sin editar entidades
- **THEN** cambian solamente configuración o datos de procesamiento pertinentes; el historial y las revisiones de entidades permanecen iguales

### Requirement: Planes de cambios acotados y referencias explícitas

El núcleo SHALL aceptar planes versionados de hasta 100 mutaciones permitidas y 10 comprobaciones finales, dentro de 2 MiB y 512 bindings. MUST resolver únicamente referencias explícitas a exports de pasos anteriores en slots de ID/revisión admitidos y claves de propiedades, sin sustituir texto literal ni relajar los DTOs finales. MUST rechazar campos, versiones y operaciones desconocidos o anidación de lotes sin efectos confirmados.

#### Scenario: Esquema y registros encadenados

- **WHEN** un plan referencia colección, propiedad, revisión, registro y vista producidos por pasos anteriores
- **THEN** se resuelven los IDs/revisiones y las claves de valores sin exigir IDs preexistentes ni omitir validaciones de dominio

#### Scenario: Referencia inválida o clave duplicada

- **WHEN** una referencia apunta al futuro, a un paso/export inexistente, a un slot no permitido o renombra dos claves al mismo ID
- **THEN** se devuelve validation_error con paso y binding identificables y ningún efecto confirmado

#### Scenario: Límites y operaciones excluidas

- **WHEN** el plan excede un límite, contiene un lote anidado, una operación no admitida o una acción de filesystem como attachment.import
- **THEN** se rechaza completo sin ejecutar parcialmente sus pasos, migrar ni copiar originales

#### Scenario: Texto que parece referencia

- **WHEN** títulos, texto de bloques o valores textuales contienen cadenas parecidas a aliases, IDs o referencias
- **THEN** permanecen literales; sólo los bindings declarados en slots admitidos pueden cambiar entradas del plan

### Requirement: Preview aislado del workspace

changes.preview SHALL evaluar el plan y sus comprobaciones sobre un snapshot consistente, sin confirmar contenido, revisiones, relaciones, valores, FTS ni historial en el original. MUST conservar adjuntos y no crear/migrar/restaurar workspaces, generar backups ni exportar archivos. SHALL rechazar de forma recuperable workspaces no actuales, esquemas SQL no reconocidos para lotes y snapshots fuera de los límites publicados.

#### Scenario: Preview exitoso y fallido sin mutaciones

- **WHEN** se previsualizan un plan válido y otro que falla después de varias mutaciones simuladas
- **THEN** entidades, documentos, colecciones, valores, revisiones, FTS, historial y hashes de originales permanecen iguales en el workspace

#### Scenario: Snapshot con WAL y escritor concurrente

- **WHEN** hay datos confirmados en WAL y un segundo cliente confirma una escritura durante la obtención del preview
- **THEN** el resultado corresponde íntegramente a un único estado confirmado, incluye su huella y no mezcla lecturas de estados distintos ni ignora el WAL

#### Scenario: Versión o estructura no admitida

- **WHEN** se solicita preview sobre un workspace ausente, anterior, futuro, con extensión SQL no admitida o por encima del límite de snapshot
- **THEN** se identifica el motivo sin actualizar formato, descartar datos, crear workspace o publicar un preview incompleto

### Requirement: Diff de dominio e identidades estables del plan

El preview SHALL devolver resultados por paso, comprobaciones, diff antes/después y un recibo versionado que vincule scope, plan, estado base y efecto esperado. MUST conservar IDs de dominio entre preview y apply del mismo plan, incluidos desempates de cálculos por ID; distinguir timestamps simulados de los de confirmación y fallar antes del commit si la respuesta supera el límite publicado, sin truncar un diff como completo.

#### Scenario: Crear y editar con diff verificable

- **WHEN** un plan crea una colección y modifica una nota existente
- **THEN** se distinguen altas de cambios, se identifican originales/revisiones afectados y el estado confirmado coincide con el efecto aceptado, excepto metadatos de ejecución explícitamente excluidos

#### Scenario: Reparto empatado y tamaño excesivo

- **WHEN** se repite preview del mismo plan sobre el mismo estado con un reparto empatado o un diff supera el límite
- **THEN** los IDs y el desempate son reproducibles; el exceso produce un error explícito y no una confirmación o aprobación de contenido omitido

### Requirement: Comprobaciones finales sobre el estado propuesto

Los planes SHALL permitir consultas database.query después de todas sus mutaciones usando los mismos filtros, fórmulas y snapshot. Las comprobaciones MUST rechazar errores de cálculo y cobertura incompleta por defecto; sólo flags explícitos pueden permitirlos, conservando errores/cursores visibles. Un fallo de comprobación MUST impedir la aplicación completa, sin convertir vacío o error en cero válido.

#### Scenario: Cálculo inválido al final

- **WHEN** una comprobación final detecta división por cero u otro error derivado, aunque los pasos de escritura sean válidos
- **THEN** preview/apply rechazan el plan con código de cálculo identificable y no dejan mutaciones confirmadas

#### Scenario: Página parcial declarada

- **WHEN** una comprobación tiene más resultados que su página
- **THEN** el modo completo predeterminado rechaza el plan; una comprobación explícitamente parcial conserva cursor y cobertura sin afirmar que verificó todos los registros

### Requirement: Aplicación atómica de planes validados

changes.apply MUST revalidar y ejecutar todas las mutaciones y comprobaciones en una única transacción sobre el workspace original. SHALL confirmar juntos contenido, esquema, membresías, valores, referencias, revisiones, FTS e historial aplicables, o revertir todo ante fallo de paso, comprobación, comparación, tamaño o persistencia. Las mutaciones de entidad SHALL compartir el request_id y origen de apply en el historial existente.

#### Scenario: Fallo intermedio y de persistencia

- **WHEN** un paso posterior es inválido o falla almacenamiento antes del commit después de crear colecciones, páginas y valores
- **THEN** no queda ninguno de esos efectos, incremento de revisión, fila FTS ni cambio de historial confirmado y se identifica el fallo sin presentar resultados parciales como aplicados

#### Scenario: Commit visible y agrupado

- **WHEN** apply confirma un plan válido con varias mutaciones de entidad y esquema
- **THEN** otra conexión observa el estado anterior o el completo confirmado; las mutaciones de entidad tienen el request_id/origen compartido y las de esquema están identificadas en el resultado, sin inventar historial persistente de esquema

#### Scenario: Misma validación que operaciones individuales

- **WHEN** se ejecutan valores, documentos, fórmulas, vistas o revisiones inválidos por operación individual y por lote
- **THEN** el lote conserva los errores/reglas existentes y agrega contexto de paso, sin aceptar tipos o referencias que la operación individual rechaza

### Requirement: Confirmación vinculada al preview y concurrencia segura

apply MUST exigir el plan y recibo esperado, comprobar identidad/ruta, estado base y efecto antes del commit y mantener revisiones esperadas de cada operación. SHALL rechazar preview obsoleto con revision_conflict y contención con workspace_busy dentro del timeout publicado. MUST rechazar recibos incompatibles, planes alterados y divergencias, sin migrar, rebasar revisiones, confirmar parcialmente ni reintentar escrituras automáticamente.

#### Scenario: Escritura entre preview y apply

- **WHEN** otro cliente cambia datos originales después del preview, incluso fuera de los objetos escritos por el lote
- **THEN** apply rechaza el recibo obsoleto, conserva el cambio ajeno y no confirma pasos del lote; un preview nuevo es necesario

#### Scenario: Revisión vieja o escritor ocupado

- **WHEN** un plan incluye una revisión esperada obsoleta o otro escritor mantiene el bloqueo más allá del timeout
- **THEN** termina con revision_conflict o workspace_busy, respectivamente, sin sobrescritura ni reejecución automática

#### Scenario: Recibo ajeno o efecto alterado

- **WHEN** se omite el recibo, pertenece a otra ruta/workspace, cambia el plan, usa versión inválida o no coincide el efecto esperado
- **THEN** la aplicación completa es rechazada con detalles recuperables y ningún efecto confirmado; la huella no se presenta como token de permiso o autenticación

#### Scenario: Respuesta perdida después del commit

- **WHEN** el cliente pierde la respuesta de un apply confirmado e intenta usar el mismo recibo para repetir una mutación efectiva
- **THEN** el estado base ya no coincide y el núcleo rechaza la repetición; el cliente puede consultar IDs originales para reconciliar, sin promesa de idempotencia persistente

### Requirement: Conservación y recuperación al agrupar operaciones

Los lotes MUST conservar campos, documentos, ASTs y valores futuros no modificados, y rechazar reemplazos de formatos que el núcleo no pueda editar. SHALL operar sólo metadatos/referencias de adjuntos ya existentes, sin modificar originales ni prometer atomicidad del filesystem. Backup/restauración SHALL conservar todo efecto confirmado sin introducir un nuevo formato persistente.

#### Scenario: Formatos futuros y referencia existente

- **WHEN** el workspace incluye documentos v1/v2/v3/futuros, payloads o ASTs desconocidos y un original referenciado
- **THEN** el preview y los planes rechazados conservan bytes/IDs; un lote válido no reescribe datos omitidos ni el original y un reemplazo no soportado aborta completo

#### Scenario: Reapertura y recuperación del lote

- **WHEN** se aplica un plan, se reabre y se restaura su backup en una nueva ruta
- **THEN** IDs, valores exactos, documentos, cálculos, vistas e historial confirmados se conservan; el recibo de la ruta original no autoriza apply en la restaurada

### Requirement: Construcción determinista del sistema de suscripción por CLI

El recorrido sintético SHALL construir mediante operaciones públicas y un lote previsualizado dos colecciones relacionadas, costo BOB 145,69, seis asientos, cinco miembros con pesos 2/1/1/1/1, pago, fórmula exacta y vista Pendientes incrustada en una página. MUST comprobar resultados antes de apply, conservar el total de 14.569 centavos y recalcular cambios sin LLM, red, SQL del cliente ni una pantalla financiera específica.

#### Scenario: Preview y construcción completa

- **WHEN** se ejecuta el recorrido en un workspace sintético actual con la ventana cerrada
- **THEN** preview muestra 48,57 y cuatro 24,28 sin persistir colecciones; apply confirma una sola estructura con los mismos IDs, seis asientos, total 145,69 y referencia de página a la vista original

#### Scenario: Fallo sin estructura parcial

- **WHEN** una variante del lote de suscripción contiene una relación o fórmula inválida en un paso posterior
- **THEN** no queda colección, propiedad, miembro, vista o página parcialmente creado

#### Scenario: Costo y pagos actualizados

- **WHEN** se cambia el costo por operación pública y se marca un miembro como pagado con revisión vigente
- **THEN** la consulta recalcula importes exactos, Pendientes excluye al pagado y la página sigue apuntando a la misma colección/vista sin copiar registros ni inventar reglas de vacantes o períodos

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

### Requirement: Contrato compartido de lectura XLSX

El núcleo SHALL conservar `extraction.xlsx_cells` y CLI `extraction xlsx-cells` con input/output tipados, validación Rust y equivalencia con invoke en API v1/52. MUST mantener su lectura de celdas completas independiente de los fragmentos de búsqueda, errores/salida JSON y exclusión de extracción de los nueve steps batchables; bindings frontend se generan desde el catálogo sin nuevas pantallas. La ampliación de index/search no cambia este reader.

#### Scenario: CLI y core equivalentes
- **WHEN** se consulta la misma generación/paginación con invoke y el subcomando XLSX
- **THEN** devuelven el mismo resultado tipado y los esquemas publicados describen valores léxicos, fórmulas, metadatos y localizadores sin tipos any abiertos

#### Scenario: Conservación y ámbito limitado
- **WHEN** se comparan catálogos con baseline v11/52 y se intenta insertar extracción/indexación/búsqueda XLSX en un lote
- **THEN** sólo los DTOs y descriptor de index/search tienen las extensiones nombradas; extraction.xlsx_cells y los otros contratos son idénticos y el lote rechaza la operación sin mutaciones parciales

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
