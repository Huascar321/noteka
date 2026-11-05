# Verificación del detalle de Goal

Fecha: 27/10/2026. Cambio: `enhance-goal-detail`.

La referencia visual fue la captura local de la plantilla Notion revisada el
24/10/2026 y su estructura consultada en la conversación. La interfaz incorpora
tarjeta Goal Overview, Additional Details desplegable, Milestones, Journal
desplegable y tabla Goal Projects, en ese orden. No se afirma paridad exacta.

## Verificación

- `npm run build`: compilación aprobada y cero diagnósticos Svelte; Vite mantiene
  su aviso previo de tamaño del bundle.
- `node --experimental-strip-types --test tests/goal-document.mjs tests/editor-adapter.mjs`:
   10 pruebas aprobadas, incluyendo documentos anteriores, identificadores,
  delimitadores alterados, compatibilidad entre variantes y recomposición.
- `node scripts/headless-goal-detail-smoke.mjs`: 28 comprobaciones con Chromium
  y tema claro; con `NOTEKA_TEST_BROWSER=webkit NOTEKA_TEST_THEME=dark`, 28 con
  WebKit y tema oscuro. Workspace y registros sintéticos con el núcleo local,
  sin solicitudes externas. Se ejercitaron creación, varias entradas fechadas,
  despliegues, edición de las tres secciones, guardado y reapertura, relación con
  cambios pendientes, tabla, Side/Full, 420 px, preservación exacta de bloques
  importados y metadatos, edición del contenido previo, conflicto/Cancel y un
  documento de formato futuro que permanece en modo de recuperación.
- La regresión `headless-page-actions-smoke.mjs` aprobó 103 comprobaciones antes
  de los últimos ajustes visuales de Goal.
- Capturas sintéticas de Overview y Journal/tabla en claro y oscuro revisadas
  frente a la referencia; no incluyen datos personales del usuario.

Abrir no modifica el documento. Los Goals anteriores conservan su contenido en
Previous content; no se intenta clasificar automáticamente una plantilla
importada ambigua. La validación headless no sustituye una aceptación visual ni
IPC de la ventana nativa Tauri.
