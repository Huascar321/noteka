## ADDED Requirements

### Requirement: Selector local de relaciones por nombre

El escritorio SHALL permitir seleccionar proyectos, etiquetas y personas compatibles mediante un menú buscable por nombre, sin requerir IDs. MUST identificar vínculos existentes, mostrar resultados progresivos y estados de carga, vacío y error, y permitir continuar la búsqueda sin afirmar que se agotó si quedan páginas. MUST conservar el acceso a las páginas originales y permitir desvincular relaciones existentes, incluidas las entrantes, sin eliminar entidades. Los vínculos archivados SHALL identificarse. Los controles ya representados como propiedades no se duplicarán en Relations.

#### Scenario: Agregar y quitar por nombre
- **WHEN** se busca un destino compatible y se selecciona o deselecciona
- **THEN** se aplica la operación local correspondiente, se actualiza la página y se preservan las entidades

#### Scenario: Búsqueda y cobertura
- **WHEN** hay más páginas, un resultado tardío o un error de lectura
- **THEN** el menú muestra resultados y cobertura progresivos, permite continuar o reintentar y no presenta una ausencia parcial como definitiva

#### Scenario: Borrador y bloqueo
- **WHEN** hay edición pendiente o la página está bloqueada
- **THEN** no se modifica un vínculo silenciosamente; el borrador puede guardarse explícitamente o conservarse, y el bloqueo conserva su protección

#### Scenario: Teclado, conflicto y preservación
- **WHEN** se usa teclado, se cierra el selector, hay cambio de revisión o un vínculo entrante/archivado
- **THEN** el foco y los mensajes permiten operar o recuperarse, se respeta el control de revisión y no se descartan buffers ni se elimina la entidad vinculada
