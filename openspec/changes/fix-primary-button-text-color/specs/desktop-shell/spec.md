## ADDED Requirements

### Requirement: Texto blanco de botones primarios

Los botones primarios SHALL mostrar texto blanco, incluidos sus atajos de teclado,
en ambos temas, independientemente de la tinta automática del acento. MUST
conservar fondo, hover, foco, estados y eventos existentes.

#### Scenario: New y Save
- **WHEN** se muestran acciones primarias como New o Save
- **THEN** su texto y atajo son blancos, también al hacer hover, y los controles deshabilitados conservan su atenuación y bloqueo existentes
