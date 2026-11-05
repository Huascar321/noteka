## ADDED Requirements

### Requirement: Proyección general de Notes con Voice

El núcleo SHALL ofrecer una proyección paginada de notas generales que incluya Voice sin modificar la consulta `notes_library` existente.

#### Scenario: Filtrar antes de paginar

- **WHEN** se consulta la proyección general con notas Reference, Voice y Web Clip
- **THEN** se incluyen Reference y Voice y se excluye Web Clip antes de calcular las páginas
