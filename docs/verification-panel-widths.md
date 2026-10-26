# Verificación de anchos y transiciones de paneles

Fecha: 26/10/2026. Cambio: `resize-workspace-panels`. Implementado y verificado
localmente; deltas pendientes de sincronización/archivo.

## Method

App compilada en Chromium headless aislado, puente al CLI Rust y SQLite sintético.
Los eventos de cierre del host usan un shim de prueba. No se inspeccionó la
pantalla nativa ni se abrió el workspace personal. Evidencia privada/ignorada:
`output/playwright/panel-resize-20261026/`.

## Results

- Cinco pruebas puras de `tests/panel-widths.mjs`: caps, umbral exacto, recuperación
  tras achicar ventana, validación independiente de valores y fallos de storage.
- 31 comprobaciones de `core.cjs`: arrastre real, ambos bordes, memoria después de
  cerrar/reabrir y recargar, teclado, Escape, máximos reales y alternancia con el
  mismo editor. A 1440px se observaron 504px/936px como máximos; 504px permite
  coexistir y 505px contrae el menú. Suprimir/revelar conservó texto, identidad,
  undo/redo y revisión; otro destino/cierre conservó la guardia.
- 22 comprobaciones de `integration.cjs`: pane oculto protegido ante cierre de
  host, workspace y dueño; Cancel y fallo inyectado de SQLite conservan buffer.
  Error de preferencia visible/recuperable; cancelación al cruzar el umbral
  restaura ancho y menú. A 420px los máximos son 147px/273px, sin overflow global;
  agrandar recupera el ancho preferido. Rail de 24px/paso de 14px conserva teclado.
  Se observaron posiciones intermedias del Side y expansión del origen, inert
  inmediato, inversión rápida, editor estable y salida antes de desmontar.
  Movimiento reducido aplica el destino sin interpolación.
- 18 comprobaciones compartidas de `final.cjs`: portada activa/pausada, resize,
  Photo/ASCII, coexistencia con Side pequeño, menú de modo dentro de viewport,
  Center y New tab, sin writes de entidades, errores de navegador o red externa.
  Estas 18 también se citan en la revisión ASCII; no son otra suite distinta.

Las 71 comprobaciones headless aprobaron. Se revisaron capturas claro/oscuro y
420px. `npm run build` pasó con iconos/contratos vigentes y cero errores o
advertencias Svelte; permanece el aviso previo de tamaño del bundle. OpenSpec
estricto se validó por separado de las pruebas del producto.

Durante implementación se corrigieron el redondeo de `1440 * .35` mediante
porcentajes enteros y el revelado de un pane recién insertado usando el proxy
reactivo de Svelte. Un reintento del recorrido final encontró una preferencia
pequeña dejada por la ejecución anterior; se fijó el ancho inicial de la fixture
para comprobar la rama de supresión ancha de manera reproducible.

## Limits

Recargar verifica la preferencia local del cliente; no acredita reapertura física
de la ventana de macOS. El cierre simulado verifica guardias y confirmación del
shell, no IPC/host nativo. No se ejecutó la suite histórica completa. No se
modificaron Rust, contratos, dependencias, formatos o migraciones. Ver
[guía](panel-widths.md) y [ADR0051](decisions/0051-panel-widths-and-suppressed-peeks.md).
