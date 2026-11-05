## ADDED Requirements

### Requirement: Calendario de tareas del proyecto

El detalle de Project SHALL ofrecer un calendario mensual de tareas relacionadas cargadas por su fecha civil Due, con navegación entre meses, acceso a las tareas sin fecha y advertencia de carga parcial.

#### Scenario: Navegar y abrir

- **WHEN** la persona cambia de mes o activa una tarea del calendario
- **THEN** se presenta el mes solicitado o se abre la entidad original con las guardias existentes

#### Scenario: Preparar una tarea en un día

- **WHEN** se elige crear desde un día del calendario
- **THEN** se prepara un borrador con Due y vínculo al Project, sin persistir hasta confirmar
