# local-workspace Specification

## Purpose
Conservar el contenido de un espacio de trabajo personal en el equipo y permitir su apertura, evolución de formato y recuperación sin depender de servicios externos.

## Requirements

### Requirement: Workspace local explícito

El sistema SHALL permitir crear y abrir un workspace identificado en una ruta de datos explícita o en el directorio de datos de la aplicación, sin guardar datos de usuario en el checkout por defecto.

#### Scenario: Creación fuera del checkout

- **WHEN** se crea un workspace sin indicar una ruta personalizada
- **THEN** se devuelve su identificador y ruta en el directorio de datos de la aplicación
- **AND** el checkout no contiene la base de datos ni contenido personal

#### Scenario: Selección de workspace

- **WHEN** se abre un workspace existente por su ruta
- **THEN** las lecturas y escrituras operan sobre ese workspace y no sobre otro seleccionado anteriormente

### Requirement: Uso offline sin cuentas

El sistema SHALL permitir abrir el workspace y operar sus entidades locales sin conexión de red, cuenta externa, credencial o modelo de IA.

#### Scenario: Operaciones sin red

- **WHEN** se crea una nota, se modifica y se vuelve a leer con la red deshabilitada y sin modelos configurados
- **THEN** las operaciones se completan usando únicamente almacenamiento local

### Requirement: Persistencia después de reiniciar

El sistema SHALL conservar identificadores, documentos, relaciones, estados y revisiones confirmados cuando se cierren y reabran sus clientes.

#### Scenario: Reapertura

- **WHEN** una nota y su relación a un proyecto se guardan correctamente, se cierran todos los clientes y se reabre el workspace
- **THEN** se leen el mismo contenido, identificadores, relación y revisión confirmada

### Requirement: Migración controlada de formato

El sistema MUST versionar el formato persistente, crear un backup consistente antes de migrar datos existentes y aplicar cada migración de forma atómica, sin pérdida de contenido confirmado.

#### Scenario: Migración compatible

- **WHEN** se abre un workspace de una versión anterior soportada
- **THEN** se crea el backup previo, se aplica la migración y se conservan las entidades y relaciones

#### Scenario: Migración fallida

- **WHEN** una migración falla antes de completar su transacción
- **THEN** permanece disponible el estado anterior completo y se informa el error y la ubicación del backup

### Requirement: Rechazo de formato futuro

El sistema MUST rechazar escrituras y migraciones automáticas sobre un workspace cuya versión persistente sea posterior a la soportada, conservando sus archivos sin cambios.

#### Scenario: Apertura con aplicación antigua

- **WHEN** se intenta abrir un workspace de formato futuro
- **THEN** se devuelve `unsupported_workspace_version` antes de modificar cualquier archivo del workspace

### Requirement: Backup consistente

El sistema SHALL generar un backup con todos los datos confirmados de un mismo estado lógico, originales administrados y un manifiesto de versión e integridad, incluso si otro cliente tiene abierto el workspace.

#### Scenario: Backup mientras el workspace está abierto
- **WHEN** se solicita backup con un cliente abierto y escrituras confirmadas
- **THEN** el backup pasa su validación y contiene un estado consistente que puede restaurarse
- **AND** no incluye una escritura parcial

#### Scenario: Originales de adjuntos
- **WHEN** el estado confirmado referencia adjuntos originales
- **THEN** el backup incluye esos originales con hashes y la restauración rechaza faltantes o alterados

### Requirement: Restauración sin sobrescritura

El sistema SHALL validar el backup y restaurarlo en una ruta nueva, preservando identificadores y revisiones. MUST rechazar un destino no vacío y un backup inválido sin alterar el workspace activo.

#### Scenario: Restauración válida

- **WHEN** se restaura un backup válido en un destino nuevo
- **THEN** se puede abrir el workspace restaurado y leer sus entidades con los mismos IDs y revisiones

#### Scenario: Backup corrupto o destino ocupado

- **WHEN** el backup no supera la validación o la ruta destino contiene datos
- **THEN** se devuelve un error identificable sin sobrescribir el destino ni el workspace activo

### Requirement: Evolución recuperable a cola local

El sistema MUST migrar al esquema vigente de extracción local/PDF con backup previo y transacción atómica, conservando originales, IDs, revisiones y formatos desconocidos existentes. SHALL inicializar una cola nueva pausada/vacía y preservar configuración, jobs, ownership y resultados de una cola existente sin ejecución implícita. Una migración fallida conserva el formato anterior; una versión futura se rechaza sin reparación ni descarte.

#### Scenario: Migración desde SQLite v7

- **WHEN** se abre explícitamente un workspace v7 con documentos v1/v2/v3/futuros, payloads y ASTs desconocidos, adjuntos y colecciones
- **THEN** se crea backup previo y se recorren v8/v9 hasta la versión actual conservando exactamente contenido previo y hashes originales; la cola nueva está pausada sin trabajos ni helper iniciado y el índice nuevo de adjuntos está vacío

#### Scenario: Falla y recuperación desde versión previa

- **WHEN** falla la migración antes del commit o se restaura su backup previo en otra ruta
- **THEN** la falla conserva íntegra la versión de origen y la restauración recupera sus datos; sólo una apertura compatible migra, sin iniciar extracción

#### Scenario: Versiones anteriores y futuras

- **WHEN** se abre una versión anterior soportada o una versión posterior a la actual
- **THEN** la anterior recorre migraciones con backup y conserva datos; la futura devuelve unsupported_workspace_version antes de modificar archivos

#### Scenario: Migración desde SQLite v8 con procesamiento
- **WHEN** se abre explícitamente v8 con control pausado o habilitado, trabajos en estados conocidos con configuraciones futuras conservables, running heredado y generaciones UTF-8
- **THEN** se crea backup previo y migra a la versión actual sin reescribir filas existentes, recuperar leases, resetear control o ejecutar jobs; las estructuras PDF y el índice de adjuntos nuevos están vacíos y sólo cambian los metadatos de versión necesarios

#### Scenario: Rollback de software sin pérdida posterior
- **WHEN** se vuelve a un cliente que sólo soporta v8 luego de confirmar datos en v9
- **THEN** se rechaza el workspace v9 sin cambiarlo; el backup previo puede restaurarse en otra ruta, sin sobrescribir ni descartar escrituras posteriores del original

### Requirement: Backup y restauración de procesamiento

El backup SHALL conservar configuración, trabajos y generaciones confirmados, incluidos manifiestos/fragmentos PDF, junto con SQLite consistente y originales verificables, sin resultados en preparación. Restaurar MUST preservar identidad/revisiones y resultados, sin reconocer ownership activo de otra ruta, ejecutar trabajos ni aceptar sus cursores. Abrir o leer no recupera mutando; la recuperación operativa requiere una acción explícita.

#### Scenario: Backup durante extracción

- **WHEN** se realiza backup con un running y una generación previa mientras un worker prepara otra en memoria
- **THEN** contiene un estado confirmado completo de la cola y sólo la generación publicada en su snapshot, nunca los fragmentos en preparación

#### Scenario: Restauración con ownership ajeno

- **WHEN** se restaura el backup en otra ruta y se leen trabajos y fragmentos
- **THEN** se conservan IDs, revisiones y resultados verificables, el running heredado se identifica como no activo/recuperable y un token o cursor de la ruta original no puede publicar ni continuar lectura en la restaurada

#### Scenario: Recuperación explícita sin afectar origen

- **WHEN** se solicita ejecución en la copia restaurada con un running heredado
- **THEN** se recupera como fallo identificable sin reintento automático de ese trabajo; el workspace origen y sus workers siguen independientes y retry requiere revisión actual

#### Scenario: Datos futuros o backup inválido

- **WHEN** la cola contiene configuración futura conservable, o un backup tiene hash incorrecto, originales alterados o tablas obligatorias ausentes aunque su hash coincida
- **THEN** los datos futuros válidos se preservan y se identifican como incompatibles para ejecución; un backup inválido se rechaza sin sobrescribir el origen ni publicar un destino parcial

#### Scenario: PDF previo durante parsing y publicación
- **WHEN** se respalda durante parsing o publicación de una regeneración PDF y se restaura en otra ruta sin helper instalado
- **THEN** se recupera una generación completa del snapshot con su manifiesto, hashes/localizadores y todos sus fragmentos; leer resultados no necesita parser ni restaura un proceso activo

#### Scenario: Integridad PDF inválida en backup
- **WHEN** un backup v9 tiene manifiesto/rangos/hash de texto inconsistentes, fragmentos PDF huérfanos o falta alguna estructura requerida
- **THEN** se rechaza antes de publicar el destino, aun si se recalculó su hash de base; no se purgan ni reparan derivados para hacer pasar la validación

### Requirement: Migración recuperable para índice de adjuntos

El sistema SHALL migrar v9→v10 aditivamente con backup previo y rollback íntegro ante falla. MUST conservar IDs, documentos, originales, configuraciones desconocidas, cola/ownership, fragmentos UTF-8/PDF y búsqueda de notas; el índice de adjuntos inicia vacío. Un cliente v9 rechaza v10 antes de escribir. No se reextrae, indexa ni recupera trabajo como parte de migrar.

#### Scenario: Migración de estados reales v9
- **WHEN** se abre explícitamente un workspace v9 con generaciones UTF-8/PDF y otro con trabajo running/resultado previo/configuración futura
- **THEN** el backup es restaurable y las filas existentes permanecen iguales salvo la versión del workspace; v10 añade el índice vacío sin cambiar pausa, intentos, generación ni ownership

#### Scenario: Falla y cliente anterior
- **WHEN** la migración falla antes de confirmar o un CLI v9 intenta abrir un workspace v10
- **THEN** la falla conserva íntegramente v9 y su backup, o el cliente anterior rechaza la versión futura sin modificar archivos ni datos

### Requirement: Backup y restauración con selecciones confirmadas

El sistema MUST incluir selecciones, revisión/perfil, copias y postings en backup/restauración consistentes con las generaciones. La restauración SHALL validar su integridad en destino nuevo y conservar datos originales y configuraciones desconocidas, sin indexar, reextraer ni recuperar jobs. No requiere helper PDF, red o modelos; un cursor de la ruta anterior no sirve en la restaurada.

#### Scenario: Restaurar y buscar offline
- **WHEN** se restaura un backup con selecciones UTF-8/PDF y un PDF sin texto, sin helper disponible
- **THEN** get/search conserva cobertura, ranking y localizadores con revisión/perfil intactos; un cursor del origen se rechaza por ruta distinta y los jobs conservan su estado

#### Scenario: Backup concurrente o dañado
- **WHEN** un backup coincide con put/reset/publicación, o una copia de backup contiene índice inconsistente
- **THEN** captura un estado confirmado completo anterior o posterior, nunca una mezcla; el backup dañado se rechaza sin publicar destino parcial ni reparar o modificar el origen

### Requirement: Snapshots y huellas preservan el índice sin volverlo original

El sistema SHALL incluir todo el esquema/estado nuevo en snapshots y registry sin añadirlo a la huella base.v2 de datos originales. MUST conservar el cap de 64 MiB del snapshot y rechazar puts que lo excedan antes de commit, sin eviction. Cambiar sólo el índice no altera la huella original; migrar versión invalida recibos previos. Restaurar el índice no modifica la autoridad de SQLite y adjuntos originales.

#### Scenario: Huella y preservación en lotes
- **WHEN** se compara base.v2 antes/después de put/reset y se crean snapshots/preview/apply con el índice presente
- **THEN** la huella original no cambia por la indexación, el registry acepta el esquema completo y los lotes conservan todas sus tablas y filas, incluyendo shadows y revisión

#### Scenario: Presupuesto real y recibo viejo
- **WHEN** put haría superar el snapshot de 64 MiB o se presenta en v10 un recibo calculado sobre v9
- **THEN** la indexación revierte sin cambiar otras selecciones/originales o el recibo se rechaza sin aplicar mutaciones; no se aumenta el cap ni se elimina texto para forzar éxito

### Requirement: Migración aditiva y recuperación de derivados XLSX

El sistema SHALL recorrer v10→v11 con manifiesto/chunks XLSX y v11→v12 con barrera de compatibilidad y llegar al esquema vigente. MUST respaldar primero, migrar transaccionalmente, validar integridad y conservar originales, cola/pausa, generaciones, índice FTS, IDs y documentos desconocidos; migrar no ejecuta ni selecciona. Backup/restore verifican representaciones e índice mixto con originales en destino nuevo sin helpers. El paso v12 conserva DDL, perfil, revisión, hashes y filas UTF-8/PDF.

#### Scenario: Migración de baseline v10 real
- **WHEN** se abre con el núcleo nuevo un workspace creado por el binario v10 independiente, con datos y generaciones UTF-8/PDF indexadas
- **THEN** recorre v11/v12 hasta el esquema vigente con backup v10 restaurable y almacenamiento XLSX inicialmente vacío, sin alterar datos/hash/resultados anteriores ni arrancar helpers

#### Scenario: Falla y rollback de migración
- **WHEN** falla la migración de v10 o v11 antes del commit, incluso tras actualizar las versiones
- **THEN** el workspace conserva íntegramente su versión y datos de origen, sin estado parcial; se informa el backup utilizable

#### Scenario: Restore actual y original alterado
- **WHEN** se respalda/restaura el formato actual con XLSX seleccionado y faltan helpers o se altera representación/posting/original del backup
- **THEN** el caso íntegro se restaura, lee y busca sin parsing; el alterado se rechaza sin sobrescribir un destino existente, aun con hash externo de SQLite reparado

#### Scenario: Versiones futuras
- **WHEN** se abre un workspace v14 o se encuentran jobs/configuraciones futuros o un índice de versión/perfil futuros en v12
- **THEN** v14 se rechaza sin modificar archivos; los jobs/configuraciones se conservan no ejecutables y un índice futuro no se reinterpreta, migra a perfil conocido ni elimina mediante reset

### Requirement: Compatibilidad recuperable del índice XLSX v12

El sistema MUST incrementar la versión de workspace a v12 antes de admitir selecciones XLSX, aunque las tablas FTS sean genéricas. SHALL rechazar acceso a v12 desde un cliente v11 sin escrituras, y acceso index/search/scoped a v11 desde el cliente nuevo sin migración implícita. La apertura explícita conserva el índice literal_unicode61_v1 y permite restaurar el backup v11 con el cliente anterior en otra ruta; no hay downgrade in-place, reset ni reindexación como sustituto de migración.

#### Scenario: Baseline v11 poblado
- **WHEN** se migra un workspace creado por un binario v11 independiente con UTF-8/PDF seleccionados, XLSX extraído sin seleccionar, jobs running y configuraciones futuras
- **THEN** el paso v11→v12 sólo cambia versiones y la continuación aditiva a la versión vigente conserva contenido/filas/hashes/revisiones/ownership y XLSX sigue sin seleccionar, compatible para un put posterior dentro de cuotas

#### Scenario: Cliente anterior y backup previo
- **WHEN** un CLI/MCP v11 intenta usar el workspace v12 y luego su CLI restaura el backup previo en otra ruta
- **THEN** el acceso al nuevo formato falla antes de escrituras y el backup recupera el formato/resultados v11 completos sin borrar escrituras posteriores del workspace v12

#### Scenario: Cursores y recibos al migrar
- **WHEN** se conserva ruta/identidad/perfil/revisión del índice durante v11→v12 y luego se agrega una selección XLSX
- **THEN** el cursor previo del índice conserva validez antes de ese put y vence por revisión después; los recibos de lote v11 se rechazan en v12 y requieren preview nuevo

#### Scenario: Snapshot con índice mixto
- **WHEN** se indexa XLSX entre preview y apply de un lote válido en v12 o un put excede el snapshot de 64 MiB
- **THEN** apply conserva el índice actual sin ampliación de whitelist ni invalidación por cambio puramente derivado; el exceso revierte completo y no elimina selecciones para caber

### Requirement: Evolución recuperable a OCR persistido v13

El sistema MUST añadir el almacenamiento OCR mediante migración 013 con backup previo consistente y transacción, sin modificar migraciones 001–012, originales, documentos, cola/ownership o selecciones previas. SHALL rechazar clientes antiguos/futuros incompatibles y conservar configuraciones/datos desconocidos sin OCR, selección, reparación o purga implícitos al migrar.

#### Scenario: Baseline v12 real preservado
- **WHEN** se abre una copia poblada creada por el binario v12 durable, con originales y generaciones/selecciones UTF-8/PDF/XLSX, jobs y datos futuros
- **THEN** el backup previo es v12 restaurable y v13 conserva exactamente filas/hashes/revisiones/control/ownership previos; sólo se añaden tablas OCR vacías y cambian versiones

#### Scenario: Migración fallida y cliente anterior
- **WHEN** falla 013 antes del commit o el cliente v12 intenta usar v13
- **THEN** el fallo conserva versión/datos anteriores y backup indicado, o el cliente antiguo rechaza antes de escrituras; el backup previo se restaura en otra ruta sin mezclar escrituras posteriores

#### Scenario: Versión futura sin reparación
- **WHEN** un cliente v13 encuentra un workspace posterior o configuración/representación no soportada
- **THEN** conserva archivos/datos y devuelve incompatibilidad sin convertir, ejecutar, borrar o reparar automáticamente

### Requirement: Recuperación y snapshots de OCR íntegro

Backup/restore y snapshots SHALL conservar representación, manifiesto, índice/huellas y originales OCR como parte del estado completo, validándolos en destino nuevo sin helper. MUST sumar OCR al cap retenido y mantener snapshot <=64 MiB, excluir derivados de base.v2 y rechazar corrupción/exceso sin purga o sobreescritura; una migración invalida recibos previos conforme al contrato existente.

#### Scenario: Restore OCR sin motor
- **WHEN** se restaura un backup con OCR seleccionado, vacío, intento anterior y metadata futura, sin helper disponible
- **THEN** el estado íntegro se conserva y sus lectores/búsqueda funcionan sin inferencia; la representación/posting/original alterados se rechazan sin destino parcial o modificación del origen

#### Scenario: Snapshot lote y cuota
- **WHEN** se crean previews/applies con OCR y cambios derivados concurrentes o una operación excede snapshot/retención
- **THEN** el lote conserva todo el esquema/estado OCR y las nueve mutaciones, base.v2 no cambia sólo por derivados y el exceso revierte completo sin aumentar caps ni eliminar datos
