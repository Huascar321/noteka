## ADDED Requirements

### Requirement: Recuperación del tipo de voz exportado

La recuperación de metadata de una nota SHALL reconocer `Voice Note` como tipo local `voice` sólo cuando la fila de origen sea inequívoca y el tipo local esté ausente. MUST conservar un tipo existente y las guardias de respaldo y revisión.

#### Scenario: Etiqueta de voz explícita

- **WHEN** una fila inequívoca del export tiene tipo `Voice Note` y la nota no tiene tipo local
- **THEN** el dry-run propone `voice` y la aplicación explícita lo guarda con revisión esperada

#### Scenario: Tipo existente o fuente ambigua

- **WHEN** la nota tiene tipo local o la fila del export no se identifica de forma inequívoca
- **THEN** la recuperación no reemplaza ni inventa un tipo
