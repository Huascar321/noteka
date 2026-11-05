## ADDED Requirements

### Requirement: Tipos explícitos adicionales de Notes

El núcleo SHALL admitir Daily, Book, Recipe, Outlier Content, Creator Tips y Tools como valores distintos y persistentes. La recuperación SHALL reconocer los valores explícitos de una fila inequívoca sin reemplazar un tipo existente.

#### Scenario: Conservar tipo distinto

- **WHEN** se guarda y vuelve a abrir una nota Daily
- **THEN** conserva Daily sin convertirse en Journal y su contenido y relaciones permanecen intactos
