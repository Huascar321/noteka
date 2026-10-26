# Spec Delta

## ADDED Requirements

### Requirement: Goals y Milestones sobre páginas de identidad estable

El núcleo SHALL admitir extensiones opcionales y mutuamente excluyentes Goal y
Milestone en páginas note, conservando ID, documento, archivo y revisión. Goal
SHALL validar Active/Dream/Achieved, Dream por defecto, fechas civiles opcionales
de inicio, plazo y logro y snapshot de actividad acotado. Milestone SHALL validar
plazo, fecha completada y un Goal opcional del mismo workspace. MUST compartir
CRUD/CAS con CLI, MCP y escritorio sin migración SQL ni conversión de originales.

#### Scenario: Página anterior y captura nueva
- **WHEN** se enriquece una página importada o se crea un Goal/Milestone
- **THEN** se conserva el documento e identidad anteriores o se crea una página
  nueva normal, con validaciones Rust y campos representables sin guardar porcentajes

#### Scenario: Campos o vínculo inválidos
- **WHEN** se envía estado/fecha/snapshot inválido, ambas extensiones, un Goal
  inexistente/incompatible o se elimina un subtipo Goal aún referenciado por hitos
- **THEN** la transacción devuelve un error estructurado sin escritura parcial

### Requirement: Relaciones y recuperación de objetivos acreditadas

Goals SHALL reutilizar sus relaciones Note→Project y Note→Tag, limitando su tag
directo a uno y permitiendo un Goal por Milestone. La recuperación MUST usar IDs
originales inequívocos y metadata observada, backup, CAS e informe incremental
privado; conservar IDs, documentos, estados elegidos y datos desconocidos, y no
convertir la plantilla Goal en un objetivo real.

#### Scenario: Recuperación e identidad
- **WHEN** se recuperan Goals, hitos y relaciones desde metadata de origen
- **THEN** se enriquece sólo cada identidad acreditada y el progreso se deriva de
  hitos únicos; notas no vinculadas conservan su identidad y el template queda fuera

#### Scenario: Conflicto y archivo
- **WHEN** cambia una revisión durante la recuperación o se archiva un objetivo
- **THEN** el conflicto rechaza esa escritura sin forzarla y el archivo conserva
  los vínculos/documentos para lectura o restauración posterior
