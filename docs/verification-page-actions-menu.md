# Verificación del menú de acciones de página

Fecha: 27/10/2026. Cambio: `unify-page-actions-menu`.

El componente de cabecera de Note se reutiliza para Task, Project, Tag, Person,
Goal y Milestone, incluso al crear borradores. Sus grupos conservan la capacidad
original por tipo; Archive/Restore pasa al menú y Lock page sigue limitado a
Note normal. La apertura del menú o sus paneles no ejecuta acciones.

## Comprobaciones

- `npm run build`: compilación correcta y cero diagnósticos de Svelte; Vite
  conserva el aviso de tamaño del bundle.
- `node scripts/headless-page-actions-smoke.mjs`: 103 comprobaciones con tema
  oscuro y 103 con tema claro en Chromium; la misma prueba con
  `NOTEKA_TEST_BROWSER=webkit` aprobó 103 en tema claro. Se utilizó un workspace
  sintético con el núcleo local, sin solicitudes externas.
- La prueba recorre siete tipos de página, disponibilidad de herramientas,
  ausencia de duplicados y escrituras al abrir, navegación y foco con teclado,
  guardia de borrador al archivar, restauración, creación de tarea, Side/Center/
  Full y geometría de menú y panel a 420 px.

Las pruebas son headless; no sustituyen una inspección visual ni una aceptación
con IPC de la ventana Tauri nativa.
