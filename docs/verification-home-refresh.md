# Encabezado y actualización Home — 24/10/2026

Se retiraron el desplegable Navegación y el botón Actualizar Home. Los encabezados
de sección y el lateral conservan sus accesos.

Con Home activo, ⌘R en macOS, Ctrl+R en Windows/Linux o F5 sin modificadores
vuelven a consultar las secciones mediante refreshPanel. Se conservan las vistas
seleccionadas y no se recarga el WebView. No se modifican originales ni buffers.
Shift/Alt no forman parte de estos atajos.

HomePanel registra el listener en captura y lo retira al desmontar. Una instancia
inactiva no lo consume. Home activo previene la recarga nativa, pero no consulta
durante composición IME, repetición, un diálogo abierto o el bloqueo que recibe
explícitamente de WorkspacePane. Se conservan cancelación, límites y descarte
tardío de los lectores existentes.

Compilación `npm run build` aprobada: cero errores y advertencias Svelte;
persiste la advertencia previa Vite por bundle mayor a 500 kB. Revisión del diff,
flujo de props, guardias y limpieza del listener. No se agregaron ni ejecutaron
suites ni una nueva aceptación nativa de teclado en macOS/Windows/Linux.

Sin cambios Rust, API, migraciones, dependencias ni escrituras sobre el workspace.
Baseline/evidencia privados fuera de Git en
`~/.local/share/noteka-verification/2026-10-24-home-refresh-shortcuts`.
Log ignorado: `output/playwright/home-refresh-build.log`.

Cierre OpenSpec: 2/2 tareas, archivo `2026-10-24-simplify-home-header-and-refresh`.
Desktop-shell 41→42 requisitos, uno previo modificado explícitamente y los otros
cuarenta preservados. Las otras dieciséis specs no se modificaron en la
sincronización; validación estricta conjunta aprobada (18 items, cero fallos).

Las cuatro llamadas existentes al botón retirado en `tests/native/home.js` se
adaptaron al evento F5, conservando sus escenarios y aserciones. No se agregaron
casos ni se ejecutó ese recorrido; el resto de su compatibilidad histórica no se
revalida en este cambio.
