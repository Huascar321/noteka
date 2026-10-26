# Spec Delta

## ADDED Requirements

### Requirement: Buscador contextual compartido del escritorio

La barra de búsqueda y el botón de nueva pestaña SHALL abrir el mismo diálogo contextual con foco inicial, consulta local de notas/entidades, filtros compatibles, resultados navegables y vista previa de sólo lectura. MUST conservar la búsqueda avanzada de adjuntos y grupos mediante un acceso real, sin presentar búsqueda semántica o respuestas de IA que no se ejecutan.

#### Scenario: Barra y nueva pestaña
- **WHEN** se abre el buscador desde la barra o `+` y se elige una sección o entidad
- **THEN** ambos usan el mismo diálogo y datos; desde la barra se abre en la pestaña actual con su guardia de edición, mientras desde `+` se abre en una pestaña nueva sin modificar la anterior

#### Scenario: Consulta y vista previa vigentes
- **WHEN** cambian consulta, filtros, resultado seleccionado o workspace antes de recibir una respuesta
- **THEN** sólo se publican resultados y vista previa de la identidad vigente; el texto se muestra escapado, el fallo es explícito y ninguna lectura modifica datos ni índices

#### Scenario: Teclado y recuperación de foco
- **WHEN** se usa el atajo de búsqueda, flechas, Enter, modificador de nueva pestaña y Escape
- **THEN** se puede seleccionar/abrir el resultado y cerrar el diálogo, el foco vuelve al control de origen y los buffers de las páginas permanecen intactos

#### Scenario: Buscar en adjuntos y vistas secundarias
- **WHEN** se solicita búsqueda avanzada o se localiza Archive, Inbox, My Day, Pending, Overdue, Active Projects, Areas, Resources o Collections
- **THEN** se accede a la pantalla existente con sus operaciones, filtros y guardias, sin crear copias ni cambiar la semántica de fechas o estados
