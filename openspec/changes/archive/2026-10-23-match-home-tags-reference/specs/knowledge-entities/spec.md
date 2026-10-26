# Spec Delta

## ADDED Requirements

### Requirement: Entity y metadatos opcionales de tags

TagFields SHALL admitir clasificación Entity además de Area/Resource/Topic, favorito y procedencia opcionales y actividad de origen opcional RFC3339 acotada. MUST validar en Rust, aceptar payloads previos, conservar campos no representados al editar y mantener Topic independiente de Entity, sin migración SQL.

#### Scenario: Compatibilidad y validación
- **WHEN** se recibe un payload previo o uno con actividad/procedencia inválidas
- **THEN** el primero conserva su significado y el segundo se rechaza sin modificar datos; el filtro general de entidades admite Entity y el detalle preserva los metadatos enriquecidos
