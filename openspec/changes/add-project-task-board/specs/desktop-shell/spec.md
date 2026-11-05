# Spec Delta

## MODIFIED Requirements

### Requirement: Contenidos relacionados del Project separados por tipo

Project SHALL presentar Nav a Projects/Dashboard y secciones Tasks/Notes de los originales obtenidos por workflow/context. Tasks SHALL ofrecer List y Board; List agrupará To Do/Doing/Done y Board mostrará esas tres columnas con conteo, tarjetas de título/vencimiento y New task por columna. MUST permitir mover una tarjeta con arrastre o selector de estado, releyendo la revisión y actualizando `state` con CAS; conservar paginación acumulativa con cobertura explícita, carga/error/reintento, apertura protegida y captura contextual existente sin writes al preparar. Notes conservará títulos/tipos reales. Relaciones de cabecera no duplicarán todas las Tasks/Notes y el editor de relaciones completo seguirá accesible. Temas y 420 px SHALL evitar overflow global.

#### Scenario: Páginas y captura contextual
- **WHEN** se continúa la lectura o se prepara una Task/Note desde el Project
- **THEN** se acumulan originales únicos con el cursor del núcleo o se prepara la captura vinculada sin guardarla; una página parcial no afirma ausencia total de un tipo

#### Scenario: Board y movimiento de tareas
- **WHEN** se abre Board con tareas To Do/Doing/Done y se arrastra una tarjeta o se cambia su selector
- **THEN** la tarea actualiza `state` con revisión esperada, el tablero y el progreso se refrescan y un conflicto se informa sin reintento automático

#### Scenario: Cambios pendientes y respuesta vieja
- **WHEN** se intenta abrir un registro con cambios pendientes o llega una lectura después de abandonar el Project
- **THEN** se conserva Guardar/Descartar/Cancelar o se descarta la respuesta anterior sin reemplazar el buffer de otra página
