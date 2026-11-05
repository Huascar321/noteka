# Verificación: All de Tasks

El 05/11/2026 se inspeccionaron en lectura las reglas de las vistas All y Done
reales de Notion desde el perfil autorizado. All aplica `Status is not Done` en
Home y en la página Tasks; Done aplica `Status is Done`. No se modificaron
páginas ni registros remotos ni se copiaron contenidos privados como fixtures.

`tasks_pending` es una variante aditiva de `home.query`: incluye todo/doing,
excluye done antes de ordenar y paginar, conserva los grupos y cuenta estados
no interpretables como no disponibles. La interfaz usa esa variante para All.
`tasks_all` sigue devolviendo todas las tareas y la búsqueda sigue encontrando
las completadas. La pestaña Today conserva la decisión local previa; Inbox se
revisa por separado. No hay migración ni cambio de datos persistentes.

- `cargo test -p noteka-core --test home`: 10 pruebas aprobadas, incluida la
  nueva prueba de filtro previo a paginación, grupos, cursor incompatible,
  estados desconocidos y transición All → Done, además de `tasks_all` anterior.
- Chromium y WebKit headless: nueve checks por motor en Home y Tasks, con
  completar/reabrir, All/Done, búsqueda y CLI/SQLite sintéticos; sin llamadas
  externas. No es aceptación nativa Tauri/macOS.
- `npm run contracts:generate` y `npm run build`: aprobados; sin errores ni
  advertencias Svelte. Vite conserva su advertencia de chunk mayor a 500 kB.
- `openspec validate align-pending-tasks-view --strict`: aprobado. Specs sin
  sincronizar y cambio sin archivar.

El esquema generado incorpora un valor de enum: clientes anteriores conservan
sus consultas, pero necesitan el contrato actualizado para enviar la variante
nueva. La revisión de las otras secciones y capacidades pendientes continúa.
