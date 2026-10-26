# Spec Delta

## ADDED Requirements

### Requirement: Detalle de Task con propiedades y cuerpo editable

El escritorio SHALL presentar una Task como página con título editable, Status, Project y Due principales, relaciones originales y cuerpo enriquecido debajo. Inbox, prioridad y planificación SHALL seguir editables en View details. MUST conservar breadcrumb canónico, icono personal opcional, temas, ancho estrecho, apertura de proyectos protegida y acceso a metadatos/opciones existentes. Las acciones de subtareas, Content, comentarios, recurrencia o seguimiento de tiempo sin capacidad local no se presentan como funciones disponibles por copiar la referencia.

#### Scenario: Composición de la tarea original
- **WHEN** se abre una Task desde Home, Tasks o búsqueda
- **THEN** su detalle muestra propiedades reales y una entrada visible al cuerpo sobre la misma identidad; el proyecto abre su original mediante las guardias y los campos secundarios conservan sus valores en View details

#### Scenario: Formato no reconocido y ventana estrecha
- **WHEN** una Task tiene un cuerpo desconocido o se reduce el ancho en cualquier tema
- **THEN** el original desconocido se ofrece como JSON recuperable sin editor que lo reemplace; con cuerpo compatible, título, propiedades y editor permanecen alcanzables sin overflow horizontal global
