## ADDED Requirements

### Requirement: Proyección de proyectos para calendario

El núcleo SHALL ofrecer una proyección paginada de proyectos no archivados para calendario, incluyendo estados completados y ausencias de fecha, sin alterar `projects_deadlines`.

#### Scenario: Conservar consulta previa

- **WHEN** un cliente solicita la consulta anterior de deadlines
- **THEN** recibe la selección anterior de proyectos no completados con fecha
