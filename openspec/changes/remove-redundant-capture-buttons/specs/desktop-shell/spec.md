## ADDED Requirements

### Requirement: Acción única de creación por sección

El shell SHALL ofrecer una sola acción de creación contextual en la barra de vistas de Tasks, Notes y Projects. MUST conservar las acciones `New page`/`New task` dentro de grupos, no duplicar un botón de captura sobre la lista y presentar `New` con un vidrio teñido con el color primario (`--accent`, azul por defecto) y texto blanco.

#### Scenario: Sección de listas

- **WHEN** la persona abre Tasks, Notes o Projects
- **THEN** ve una sola acción `New` en la barra de vistas y ningún botón de captura duplicado sobre la lista

#### Scenario: Crear desde un grupo

- **WHEN** la persona usa `New page`/`New task` dentro de un grupo
- **THEN** se conserva la captura contextual existente con sus guardias

### Requirement: Alineación de barras de vistas

Las páginas de sección y las secciones de Home SHALL compartir el ritmo de barra de Tags: `gap: 8px` y 13px debajo de la fila de vistas. En Home, la fila MUST quedar 10px debajo de la tarjeta de sección; en las páginas de sección sin bloque de captura directo, MUST reservarse `padding-top: 44px` para alinearse con Tags/People.

#### Scenario: Recorrido de páginas de sección

- **WHEN** la persona recorre Tasks, Notes, Projects, Goals, Tags o People
- **THEN** la fila de vistas mantiene la misma altura vertical, aunque la sección no tenga bloque de captura

#### Scenario: Recorrido de Home

- **WHEN** la persona recorre las secciones de Home
- **THEN** la fila de vistas queda a la misma distancia de la tarjeta de sección en todas
