## MODIFIED Requirements

### Requirement: Selector local de relaciones por nombre

El escritorio SHALL permitir seleccionar proyectos, etiquetas y personas compatibles mediante un menú buscable por nombre, sin requerir IDs. MUST identificar vínculos existentes, mostrar resultados progresivos y estados de carga, vacío y error, y permitir continuar la búsqueda sin afirmar que se agotó si quedan páginas. MUST conservar el acceso a las páginas originales y permitir desvincular relaciones existentes, incluidas las entrantes, sin eliminar entidades. Los vínculos archivados SHALL identificarse. Los controles ya representados como propiedades no se duplicarán en Relations.

#### Scenario: Agregar y quitar por nombre
- **WHEN** se busca un destino compatible y se selecciona o deselecciona
- **THEN** se aplica la operación local correspondiente, se actualiza la página y se preservan las entidades

#### Scenario: Búsqueda y cobertura
- **WHEN** hay más páginas, un resultado tardío o un error de lectura
- **THEN** el menú muestra resultados y cobertura progresivos, permite continuar o reintentar y no presenta una ausencia parcial como definitiva

#### Scenario: Borrador y bloqueo
- **WHEN** una página existente tiene edición pendiente y se agrega o quita una relación, o la página está bloqueada
- **THEN** la relación se guarda independientemente sin guardar ni descartar el borrador; la página bloqueada conserva su protección

#### Scenario: Guardado posterior y conflicto de revisión
- **WHEN** se cambia una relación con un borrador pendiente y luego se guarda el contenido, o una revisión externa impide modificar la relación
- **THEN** el guardado posterior conserva la relación usando la revisión confirmada; ante conflicto se conserva el borrador y se informa el error sin sobrescribir cambios externos

#### Scenario: Teclado, conflicto y preservación
- **WHEN** se usa teclado, se cierra el selector, hay cambio de revisión o un vínculo entrante/archivado
- **THEN** el foco y los mensajes permiten operar o recuperarse, se respeta el control de revisión y no se descartan buffers ni se elimina la entidad vinculada
