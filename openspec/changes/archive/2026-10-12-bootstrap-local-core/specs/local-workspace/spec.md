# Spec Delta

## Purpose

Conservar el contenido de un espacio de trabajo personal en el equipo y permitir su apertura, evolución de formato y recuperación sin depender de servicios externos.

## ADDED Requirements

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

El sistema SHALL generar un backup con todos los datos confirmados de un mismo estado lógico y un manifiesto de versión e integridad, incluso si otro cliente tiene abierto el workspace.

#### Scenario: Backup mientras el workspace está abierto

- **WHEN** se solicita backup con un cliente abierto y escrituras confirmadas
- **THEN** el backup pasa su validación y contiene un estado consistente que puede restaurarse
- **AND** no incluye una escritura parcial

### Requirement: Restauración sin sobrescritura

El sistema SHALL validar el backup y restaurarlo en una ruta nueva, preservando identificadores y revisiones. MUST rechazar un destino no vacío y un backup inválido sin alterar el workspace activo.

#### Scenario: Restauración válida

- **WHEN** se restaura un backup válido en un destino nuevo
- **THEN** se puede abrir el workspace restaurado y leer sus entidades con los mismos IDs y revisiones

#### Scenario: Backup corrupto o destino ocupado

- **WHEN** el backup no supera la validación o la ruta destino contiene datos
- **THEN** se devuelve un error identificable sin sobrescribir el destino ni el workspace activo
