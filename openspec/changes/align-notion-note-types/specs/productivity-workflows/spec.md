## ADDED Requirements

### Requirement: Proyección conjunta de diarios

El núcleo SHALL ofrecer una proyección paginada de notas Daily y Journal, excluyendo archivo y filtrando antes de paginar, sin cambiar la consulta `notes_journal` anterior.

#### Scenario: Diario y nota general

- **WHEN** se consultan las vistas de diarios y notas generales con notas Daily, Journal y Book
- **THEN** los diarios incluyen Daily y Journal, las generales incluyen Book y excluyen ambos diarios
