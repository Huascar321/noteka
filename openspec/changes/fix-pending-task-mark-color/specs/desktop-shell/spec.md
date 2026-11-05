# Spec Delta

## ADDED Requirements

### Requirement: Glifo blanco de tarea pendiente

Los marcadores compartidos de tareas pendientes SHALL mostrar el guion blanco
sobre su fondo de acento, en ambos temas. MUST conservar tamaño, fondo, estado
y eventos existentes, sin modificar el glifo de tareas completadas.

#### Scenario: Pendientes en listas
- **WHEN** se muestran tareas pendientes en Home, Tasks o contenidos de Project
- **THEN** su guion es blanco y su fondo y comportamiento previos se conservan
