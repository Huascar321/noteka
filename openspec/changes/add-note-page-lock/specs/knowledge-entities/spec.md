# Spec Delta

## ADDED Requirements

### Requirement: Preferencia persistente de bloqueo de Note

NoteFields SHALL admitir `page_locked` booleano opcional. Ausencia o null SHALL
equivaler a no bloqueada y las Notes anteriores MUST conservar su payload sin
reescritura al leer. El núcleo SHALL validar el tipo y compartir lectura/escritura
CAS, exportación y backup existentes, sin modificar documento, identidad,
relaciones o metadatos omitidos. Esta preferencia MUST NOT introducir un permiso
de acceso: clientes explícitos conservan las operaciones compartidas.

#### Scenario: Guardar y recuperar el estado
- **WHEN** se activa o desactiva el bloqueo con revisión vigente y se reabre, exporta o restaura el workspace
- **THEN** se recupera el booleano confirmado con el mismo documento/IDs/relaciones, y cada escritura usa su revisión normal sin migración SQL

#### Scenario: Datos anteriores y entrada inválida
- **WHEN** se lee una Note sin el campo o se intenta guardar un valor no booleano o una revisión vencida
- **THEN** la lectura no normaliza datos anteriores y la escritura inválida/conflictiva se rechaza sin alterar contenido ni estado previo

#### Scenario: Propiedades desconocidas y reversión
- **WHEN** existen propiedades futuras en un payload o se usa un cliente anterior que no conoce `page_locked`
- **THEN** el contenido sigue recuperable por lectura/exportación/backup; los parches de payload incompatibles se rechazan íntegros y las actualizaciones que lo omiten no borran metadatos
