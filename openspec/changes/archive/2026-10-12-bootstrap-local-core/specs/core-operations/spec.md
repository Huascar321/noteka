# Spec Delta

## Purpose

Ofrecer operaciones estructuradas y consistentes para la interfaz y el CLI, de modo que clientes humanos o agentes operen el mismo conocimiento sin saltarse validaciones ni sobrescribir cambios concurrentes.

## ADDED Requirements

### Requirement: Contrato compartido y versionado

El sistema SHALL exponer un catálogo versionado con esquemas de entrada, salida y errores para las operaciones de workspace, entidades, documentos y relaciones. Interfaz y CLI MUST aplicar las mismas reglas.

#### Scenario: Clientes equivalentes

- **WHEN** la interfaz y el CLI solicitan la misma actualización válida en workspaces equivalentes
- **THEN** obtienen estados y resultados de dominio equivalentes

#### Scenario: Descubrimiento sin interfaz

- **WHEN** se solicita el catálogo de operaciones desde el CLI
- **THEN** se obtienen versión y esquemas sin abrir la ventana de escritorio

### Requirement: Escritura atómica validada

El sistema MUST validar cada operación antes de confirmarla y guardar conjuntamente contenido, revisión y registro de cambio, sin estados parciales si falla una validación o la persistencia.

#### Scenario: Documento inválido durante actualización

- **WHEN** una actualización incluye título válido y documento inválido
- **THEN** se rechaza la operación completa y se conservan título, documento y revisión anteriores

#### Scenario: Falla de persistencia

- **WHEN** ocurre un fallo de almacenamiento antes del commit
- **THEN** no aparece ningún cambio parcial confirmado y se devuelve `storage_error`

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

El CLI SHALL permitir crear/abrir workspace, consultar catálogo, operar entidades/documentos/relaciones, archivar/restaurar entidades y realizar backup/restauración de workspace sin iniciar la interfaz gráfica.

#### Scenario: Nota creada por CLI

- **WHEN** se crea una nota desde el CLI con la ventana cerrada y luego se abre ese workspace en escritorio
- **THEN** la interfaz puede consultar la misma nota y revisión

### Requirement: Canales de salida y códigos del CLI

El CLI SHALL producir JSON válido exclusivamente en stdout en modo estructurado y enviar logs a stderr. SHALL documentar códigos diferentes para éxito, validación, no encontrado, conflicto y almacenamiento.

#### Scenario: Operación estructurada exitosa

- **WHEN** una operación en modo JSON termina correctamente
- **THEN** stdout contiene un único resultado JSON y el código de salida indica éxito

#### Scenario: Entrada inválida

- **WHEN** el CLI recibe JSON inválido o argumentos incompatibles
- **THEN** devuelve un error JSON, un código de validación y no escribe datos

### Requirement: Historial local de mutaciones

El sistema SHALL registrar en cada mutación confirmada la entidad afectada, revisión anterior/nueva, operación y origen del cliente, permitiendo consultar esos registros sin un servicio externo.

#### Scenario: Mutación rechazada y aceptada

- **WHEN** primero se rechaza una edición por conflicto y después se confirma una edición válida
- **THEN** el historial contiene la mutación confirmada y no presenta la rechazada como aplicada
