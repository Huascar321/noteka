## ADDED Requirements

### Requirement: Vistas automáticas separadas de Inbox

Home y las páginas Tasks y Notes SHALL ofrecer una vista Unclassified separada de Inbox y explicar que sólo comprueba relaciones locales, no Content.

#### Scenario: Usar ambas vistas

- **WHEN** una persona abre Inbox o Unclassified y navega a un elemento
- **THEN** cada vista consulta su regla propia y abrirla no modifica marcas ni relaciones
