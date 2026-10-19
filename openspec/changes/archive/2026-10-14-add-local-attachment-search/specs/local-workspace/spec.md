# Spec Delta

## ADDED Requirements

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

## MODIFIED Requirements

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
