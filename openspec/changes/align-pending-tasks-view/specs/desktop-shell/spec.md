## ADDED Requirements

### Requirement: All y Done separan el estado de las tareas

El escritorio SHALL presentar en All las tareas pendientes interpretables y en Done las completadas, tanto en Home como en Tasks, sin eliminar ni modificar las entidades.

#### Scenario: Completar una tarea

- **WHEN** una tarea pendiente se completa y se actualiza la vista All
- **THEN** deja de figurar en All y queda disponible en Done
