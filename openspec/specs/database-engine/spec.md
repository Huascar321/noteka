# database-engine Specification

## Purpose
Construir sistemas locales de registros con propiedades tipadas sin duplicar identidades, contenido ni reglas de entidades existentes.

## Requirements

### Requirement: Colecciones y registros con identidad compartida

El sistema SHALL crear colecciones y registros que referencian entidades existentes o nuevas páginas con bloques, conservando su ID, revisión, contenido, relaciones y archivo. MUST confirmar creación y pertenencia con valores juntos o revertir todo.

#### Scenario: Incorporar entidad y editar página
- **WHEN** una tarea o nota existente se incorpora a una colección y se abre como página
- **THEN** conserva el mismo ID y los cambios se leen desde colección, entity.get y CLI

#### Scenario: Creación inválida
- **WHEN** se crea un registro con colección o valores inválidos
- **THEN** no queda entidad, pertenencia ni valor parcialmente confirmado

### Requirement: Esquema y valores tipados conservadores

El núcleo SHALL validar título, texto, número, decimal/moneda exacta, checkbox, fecha/rango, selección/multiselección, URL/email, archivos, relaciones y timestamps. MUST conservar valores omitidos y rechazar tipos, referencias y cambios de esquema incompatibles sin descartar datos desconocidos.

#### Scenario: Tipos y vacíos
- **WHEN** se guardan valores compatibles, nulls explícitos y un patch parcial
- **THEN** se conservan tipos, precisión decimal y campos omitidos; título y timestamps reflejan la entidad original

#### Scenario: Referencias y tipos inválidos
- **WHEN** un valor no corresponde al tipo, moneda, opción o destino declarado
- **THEN** se devuelve validation_error sin incrementar revisión ni modificar otros valores

### Requirement: Relaciones de propiedad sin duplicación

El sistema SHALL validar membresía en la colección destino y permitir consultar la proyección inversa desde los mismos valores lógicos. MUST conservar vínculos a archivadas y rechazar IDs ajenos sin crear una segunda relación inversa mutable.

#### Scenario: Relación compartida e inversa
- **WHEN** se vincula un miembro a un registro destino y se consulta desde ambos extremos
- **THEN** se observan los mismos IDs y quitar el valor elimina la proyección sin borrar entidades

### Requirement: Concurrencia y recuperación de colecciones

El sistema MUST usar revisión esperada para valores/esquema, rechazar escritor antiguo y migrar SQLite con backup previo; backup/restauración SHALL conservar colecciones, valores, vistas y originales íntegros.

#### Scenario: Conflicto y restauración
- **WHEN** el CLI confirma valores mientras escritorio conserva un buffer y se restaura un backup consistente
- **THEN** el escritor antiguo recibe conflicto y el restaurado conserva IDs y valores del snapshot

### Requirement: Plantillas de registro versionadas

El sistema SHALL guardar, leer y listar plantillas por colección con título inicial, documento y defaults editables tipados. MUST exigir revisión vigente al actualizar, validar esquema/referencias y conservar definiciones futuras desconocidas sin reemplazarlas. Backup/restauración MUST conservar plantillas y originales referenciados.

#### Scenario: Reapertura y edición concurrente
- **WHEN** se guarda una plantilla, se reabre y el CLI confirma otra revisión mientras escritorio mantiene una edición pendiente
- **THEN** la definición confirmada se conserva y el escritor antiguo recibe conflicto sin sobrescribirla ni perder su buffer

#### Scenario: Definición futura y restauración
- **WHEN** una plantilla almacenada contiene una versión o campo desconocido y se restaura un backup
- **THEN** la definición cruda permanece íntegra, no se instancia ni convierte silenciosamente y los originales locales siguen recuperables

### Requirement: Instanciación atómica con defaults

El núcleo SHALL crear una página de registro desde una plantilla vigente y un override parcial de título/valores, con entidad y bloques nuevos. MUST conservar defaults omitidos, limpiar valores con null, compartir sólo referencias explícitas y validar todo antes del commit. Plantilla, instancias anteriores y originales referenciados MUST permanecer sin modificación.

#### Scenario: Dos instancias independientes
- **WHEN** se instancia dos veces la misma plantilla con valores monetarios exactos, bloques anidados y referencias locales, omitiendo un default y limpiando otro con null
- **THEN** se obtienen entidades y bloques con IDs distintos, defaults/overrides compatibles y las mismas referencias a originales; editar una instancia no modifica la otra ni la plantilla

#### Scenario: Entrada inválida o revisión antigua
- **WHEN** la revisión de plantilla/esquema es antigua, un default/override tiene tipo o referencia inválida, se intenta escribir un cálculo o falla la persistencia
- **THEN** se devuelve error identificable y no queda entidad, documento, pertenencia, valores, cambios ni índice parcialmente confirmados
