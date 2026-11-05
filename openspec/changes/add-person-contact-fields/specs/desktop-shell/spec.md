## ADDED Requirements

### Requirement: Edición local de contacto de Person

El detalle de Person SHALL permitir editar campos locales de contacto e intereses con guardado explícito, sin copiar datos externos ni activar proveedores.

#### Scenario: Descartar una edición

- **WHEN** una persona modifica un dato de contacto y descarta el borrador
- **THEN** el valor persistido permanece intacto
