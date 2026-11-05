# Verificación: calendarios Week y Month de Tasks

El 05/11/2026 se inspeccionaron en lectura las vistas Week y Month de Tasks en
Notion: ambas usan Due, escalas Week/Month y acceso a tareas sin fecha; no se
observó filtro de estado en esas vistas. No se cambiaron registros remotos.

Las variantes aditivas `tasks_calendar_week` y `tasks_calendar_month` reciben
`as_of` y seleccionan la semana ISO o mes civil antes de paginar, junto con
las tareas sin fecha o con fecha no interpretable. Incluyen completadas. El
cursor queda ligado a vista, fecha y tamaño; `tasks_week` y `tasks_month`
conservan sus reglas anteriores. La interfaz vuelve a consultar al navegar y
anuncia cuando sólo hay páginas cargadas. Crear desde un día prepara Due y no
persiste hasta Save; New general no asigna una fecha.

- Rust `home`: 12 pruebas aprobadas; la nueva cubre cruce de año ISO, mes,
  Done, sin fecha, fecha inválida, filtro previo y cursores incompatibles.
- Dos pruebas puras de calendario cubren huso horario, bisiestos y límites
  civiles. Chromium y WebKit: 19 checks por motor en Home y Tasks, cambio de
  período, Today, captura/Save/Discard, 420 px, tema oscuro y carga parcial.
- Build y OpenSpec estricto aprobados. Vite mantiene su aviso de chunk grande.
  Fixtures sintéticas sin solicitudes externas; no se verificaron IPC ni
  composición nativa Tauri/macOS. El cambio no se sincronizó ni archivó.
