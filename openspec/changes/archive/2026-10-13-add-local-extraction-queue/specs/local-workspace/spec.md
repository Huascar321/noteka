# Spec Delta

## ADDED Requirements

### Requirement: Evolución recuperable a cola local

El sistema MUST migrar al esquema de extracción local con backup previo y transacción atómica, conservando originales, IDs, revisiones y formatos desconocidos existentes. SHALL inicializar una cola pausada y vacía, sin encolado o ejecución implícitos. Una migración fallida conserva el formato anterior; una versión futura se rechaza sin reparación ni descarte.

#### Scenario: Migración desde SQLite v7

- **WHEN** se abre explícitamente un workspace v7 con documentos v1/v2/v3/futuros, payloads y ASTs desconocidos, adjuntos y colecciones
- **THEN** se crea backup previo, se migra a v8 y permanecen exactos el contenido previo y hashes originales; la cola nueva está pausada sin trabajos

#### Scenario: Falla y recuperación desde versión previa

- **WHEN** falla la migración antes del commit o se restaura su backup previo en otra ruta
- **THEN** la falla conserva v7 íntegro y la restauración recupera sus datos; sólo una apertura compatible migra, sin iniciar extracción

#### Scenario: Versiones anteriores y futuras

- **WHEN** se abre una versión anterior soportada o una versión posterior a v8
- **THEN** la anterior recorre migraciones con backup y conserva datos; la futura devuelve unsupported_workspace_version antes de modificar archivos

### Requirement: Backup y restauración de procesamiento

El backup SHALL conservar configuración, trabajos y generaciones confirmados junto con SQLite consistente y originales verificables, sin resultados en preparación. Restaurar MUST preservar identidad/revisiones y resultados, sin reconocer ownership activo de otra ruta, ejecutar trabajos ni aceptar sus cursores. Abrir o leer no recupera mutando; la recuperación operativa requiere una acción explícita.

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
