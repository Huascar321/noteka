# Spec Delta

## ADDED Requirements

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
