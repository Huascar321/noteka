# Verificación: tabs de Tasks consolidadas

Fecha: 3 de noviembre de 2026. Cambio `merge-task-view-tabs`.

## Alcance

- La tab Today de Home/Tasks consulta `tasks_projects_in_progress` y muestra
  tareas pendientes de proyectos Doing/Ongoing no archivados, agrupadas.
- Se retiró la tab duplicada Active Projects (In Progress).
- Active Projects pasó a llamarse All Active Projects y conserva
  Planned/Doing/Ongoing.
- La página Tasks conserva Recurring y Active Content deshabilitados.
- `home.query` y `tasks_today` no cambian; CLI/MCP mantienen el filtro por
  vencimiento.
- No se modificaron checkboxes, estilos, datos, contratos ni migraciones.

## Comprobaciones ejecutadas

- `npm run check`: 0 errores y 0 advertencias Svelte.
- `npm run build`: iconos 85/71, contratos y Svelte aprobados; Vite compiló 301
  módulos. Persiste el aviso conocido de chunk mayor de 500 kB (986,93 kB).
- `npx --yes @fission-ai/openspec@1.14.1 validate merge-task-view-tabs --strict`:
  cambio válido.
- `node --check scripts/headless-bridge.mjs` y `node --check tests/native/home.js`:
  sintaxis válida.
- Inspección de fuentes: no queda `tasks_today` en `src/`; Today apunta a
  `tasks_projects_in_progress`; no queda el rótulo Active Projects (In Progress);
  All Active Projects usa `tasks_active_projects`.

## Evidencia no ejecutada

No se ejecutó el recorrido headless completo ni la aceptación nativa. El test
`tests/native/home.js` se actualizó para esperar `tasks_projects_in_progress` al
elegir Today, verificar la ausencia de la tab duplicada y recorrer All Active
Projects; el fixture `home_seed` ahora vincula las tareas pendientes con proyectos
Doing para que esa vista tenga páginas reales. Queda pendiente ejecutar esa suite
y revisar visualmente ambos temas.

## Filtro de All y checkboxes

All no aplica un filtro de estado, fecha o proyecto: incluye tareas pendientes y
completadas, con o sin vencimiento, dentro y fuera de proyectos, agrupadas por
proyecto y ordenadas por creación ascendente. Por eso muestra checkboxes verdes
llenos junto a cuadrados vacíos: el componente es el mismo que en Today; el
checkbox pendiente es un cuadrado vacío con borde neutro y el completado un check
verde. En Today, Inbox, Week, Month, Scheduled, No Due y All Active Projects el
núcleo selecciona tareas `todo` y `doing` como pendientes, de modo que todos los
checkboxes visibles son vacíos; Done muestra sólo completadas.
