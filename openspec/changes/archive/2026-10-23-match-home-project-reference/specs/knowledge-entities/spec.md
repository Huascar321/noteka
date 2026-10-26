# Spec Delta

## ADDED Requirements

### Requirement: Procedencia y orden opcionales de proyectos

ProjectFields SHALL aceptar procedencia explícita opcional y orden de tablero opcional entre 0 y 10000. MUST validar límites y timestamps en Rust, aceptar payloads anteriores y conservar campos no representados al editar. La procedencia archivada no equivale a archivar la entidad local.

#### Scenario: Compatibilidad y campos inválidos
- **WHEN** se recibe un payload antiguo sin procedencia/orden o uno con metadata inválida
- **THEN** el primero conserva su comportamiento y el segundo se rechaza sin modificar la entidad; no se cambia el esquema persistente
