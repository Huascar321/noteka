# Verificación: Board de tareas en el detalle de Project

Fecha: 4 de noviembre de 2026. Cambio `add-project-task-board`.

## Referencia observada

Se abrió Notion con el perfil persistente indicado por el usuario
(`$HOME/.config/playwright-mcp/whatsapp`) y se navegó, en modo lectura, al
detalle de un Project. La referencia muestra:

- propiedades Status/Target Deadline/Progress y Relations;
- Tasks con List/Board/Calendar;
- Board con columnas To Do/Doing/Done, conteo por columna y New task;
- tarjetas con título y Due;
- Status de tarea con tres valores;
- un proyecto Doing con tareas distribuidas entre Doing y Done, y progreso
  calculado por completadas/total.

No se editaron páginas, propiedades, filtros ni registros de Notion. La sesión se
cerró al terminar.

## Implementación

- `TaskState` agrega `doing`; `home.query` y `workflow.query` lo tratan como
  pendiente en Today, Inbox, Week, Month, Scheduled, No Due, Active Projects,
  My Day, Pending y Overdue. Done sigue seleccionando sólo `done`; All incluye
  los tres estados.
- ProjectContents agrega List/Board, grupos To Do/Doing/Done, checkbox real en
  List, columnas con conteo, tarjetas de título/vencimiento, New task por columna,
  selector de estado y arrastre con CAS.
- El detalle de tarea y el formulario secundario ofrecen To Do/Doing/Done.
- No hay migración SQLite ni operación nueva. Payloads existentes se conservan.

## Comprobaciones ejecutadas

- `npm run check`: 0 errores y 0 advertencias Svelte.
- `npm run build`: iconos/contratos/Svelte aprobados; 304 módulos; persiste el
  aviso conocido de chunk > 500 kB (994,56 kB).
- `cargo test -p noteka-core --test home --locked`: 9/9, incluida
  `doing_tasks_are_pending_for_views_workflows_and_project_progress`.
- `cargo test -p noteka-core --test productivity --locked`: 7/7.
- `cargo test -p noteka-core --test local_core --locked`: 11/11.
- `npx @fission-ai/openspec@1.14.1 validate add-project-task-board --strict`:
  válido.
- `npm run test:headless-project-board`: recorrido headless sobre workspace
  sintético, UI compilada y puente CLI. Aprobó seis checks: columnas/tarjetas,
  selector de estado, arrastre, grupos de List, checkbox de List y New task por
  columna con estado/Project preseleccionados. Captura:
  `output/playwright/project-task-board-20261104.png` (ignorado).

## Sincronización puntual de estados (4/11/2026)

Se exportó la base Tasks de Notion en modo lectura (CSV completo, 578 tareas) y
se comparó con las 578 tareas del workspace local por título normalizado,
prefijos truncados y proyecto/vencimiento para duplicados. Resultado:

- Antes: 51 todo, 527 done.
- Cambios aplicados: 24 (21 todo→doing, 2 done→doing, 1 done→todo).
- Después: 31 todo, 23 doing, 524 done, idéntico al estado de Notion.
- Cada cambio se confirmó por `entity.update` con revisión esperada; 24
  operaciones `entity.update` de origen `cli`.
- Backup canónico previo en
  `$HOME/Library/Application Support/Noteka/backups/task-status-sync-20261104`,
  fuera de Git. El CSV/export temporal se eliminó.

## Límites

No se ejecutó la aceptación nativa ni la suite headless histórica completa. No se
implementaron Calendar, comentarios, filtros, orden manual, automatizaciones ni
subtipos de tarea. El Board muestra las relaciones cargadas y conserva el cursor
de Load more; no promete un total global. La captura y las pruebas usan un
workspace sintético, no datos personales.
