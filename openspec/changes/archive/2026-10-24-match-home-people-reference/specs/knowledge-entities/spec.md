## ADDED Requirements

### Requirement: Metadatos opcionales de personas

PersonFields SHALL admitir relaciones textuales acotadas, empresa, foto local UUID con posición vertical entre 0 y 100, cumpleaños/check-in/último check-in civiles, pipeline Prospect/Contacted/Negotiating/Rejected/Closed, procedencia y orden de galería opcionales. MUST validar en Rust, aceptar payloads anteriores, rechazar valores inválidos sin escrituras y conservar propiedades no representadas, documentos e IDs sin migración SQL.

#### Scenario: Compatibilidad y validación
- **WHEN** se recibe un payload previo o metadata de persona inválida
- **THEN** el primero sigue siendo válido y la segunda se rechaza antes de persistir; el detalle permite editar los campos representados y conserva la procedencia

