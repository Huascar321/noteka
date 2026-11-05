## ADDED Requirements

### Requirement: Elementos sin relaciones locales de clasificación

El núcleo SHALL ofrecer consultas separadas que incluyan Tasks pendientes sin vínculo Project y Notes de biblioteca sin vínculos Tag, Project ni People, independientemente de la marca Inbox. MUST filtrar antes de paginar y conservar vínculos a destinos archivados como vínculos existentes.

#### Scenario: Clasificar y desclasificar

- **WHEN** se agrega o quita un vínculo de clasificación a un elemento
- **THEN** la consulta siguiente refleja su pertenencia sin modificar la marca manual Inbox

#### Scenario: Datos incompatibles

- **WHEN** una relación de clasificación tiene un destino de tipo incompatible
- **THEN** la consulta falla en vez de tratar la relación como ausente
