## ADDED Requirements

### Requirement: Datos de contacto locales opcionales

El núcleo SHALL aceptar, validar y preservar campos opcionales de contacto e intereses de Person mediante las operaciones compartidas y MUST rechazar valores inválidos sin descartar datos previos.

#### Scenario: Editar contacto

- **WHEN** se actualiza una persona con datos válidos y revisión vigente
- **THEN** los campos se conservan localmente y permanecen disponibles en lecturas posteriores

#### Scenario: Metadata futura o inválida

- **WHEN** una actualización contiene valores incompatibles o el payload original no puede interpretarse
- **THEN** se rechaza sin reemplazar el original
