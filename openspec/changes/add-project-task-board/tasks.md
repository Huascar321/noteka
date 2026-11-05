# Tasks

## 1. Dominio y consultas

- [x] 1.1 Agregar `TaskState::Doing`, validarlo en payloads y hacer que Today/Inbox/Week/Month/Scheduled/No Due/Active Projects y los workflows pending/my_day/overdue incluyan doing como pendiente; agregar pruebas de Home, workflow y progreso.
- [x] 1.2 Regenerar contratos y verificar schema/TS sin cambios de migración.

## 2. Detalle de Project

- [x] 2.1 Implementar vistas List/Board en ProjectContents, grupos To Do/Doing/Done y checkbox real en List.
- [x] 2.2 Implementar Board con columnas, tarjetas título/vencimiento, New task por columna, selector de estado y arrastre con CAS/refresh.
- [x] 2.3 Exponer To Do/Doing/Done en el detalle de tarea y en el formulario secundario; conservar estilos y accesibilidad.

## 3. Verificación y documentación

- [x] 3.1 Actualizar guías, plan, ADR y spec; agregar recorrido headless del Project Board.
- [x] 3.2 Ejecutar build/check, pruebas Rust focales, contrato y recorrido headless; registrar evidencia y límites.

## Verification

Build, check, contratos, OpenSpec estricto, pruebas Rust de Home/Productividad/
Local Core y seis checks headless del Project Board aprobados. La aceptación
nativa y la suite histórica completa no se ejecutaron. Calendar, comentarios,
filtros, orden manual y automatizaciones quedan fuera de alcance. Ver
`docs/verification-project-task-board.md`.
