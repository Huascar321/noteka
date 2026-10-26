# Corrección del parpadeo ASCII al plegar el lateral

Fecha: 25/10/2026. Cambio `fix-ascii-resize-flicker`, sin deltas de specs.

## Causa y corrección

La transición del lateral cambia el tamaño del canvas en cada cuadro.
ResizeObserver limpiaba su bitmap al asignar width/height después del RAF,
aplazando el repintado al RAF siguiente. Ese bitmap vacío podía presentarse.
Además, cada medición suspendía el reloj y cancelaba el cuadro pendiente.

El renderer conserva el último tiempo de escena y repinta dentro de resize,
antes de devolver control al navegador. measure conserva el RAF pendiente y
sólo despierta el bucle cuando falta. El atlas se reutiliza cuando no cambia
DPR/tema/color; capas e historial de sombreado se reconstruyen como antes.
No se modifica la duración/curva del lateral ni las trayectorias de los animales.

## Evidencia

Bundle compilado → Chromium headless → puente CLI existente → workspace sintético.
Una segunda ResizeObserver lee los píxeles después del observador de producto,
durante la apertura y cierre reales del lateral:

| Recorrido | Muestras con tamaño positivo | Bitmaps completamente vacíos |
|---|---:|---:|
| Antes, abrir/cerrar en oscuro | 33 | 32 |
| Corregido, abrir/cerrar en oscuro | 33 | 0 |
| Corregido, inversión rápida en oscuro | 11 | 0 |
| Corregido, abrir/cerrar en claro | 33 | 0 |
| Corregido, movimiento reducido en claro | 3 | 0 |
| Corregido, panel estrecho con movimiento reducido | 1 | 0 |

La superposición estrecha no modifica el ancho del canvas, por lo que genera
una sola muestra inicial. Doce comprobaciones del recorrido aprobadas: píxeles,
RAF activo durante resize, composición estática reducida, 420 px sin overflow,
pausa/reanudación por documento oculto y pestaña Home inactiva, sin errores JS.

`npm run build` aprobado, sin errores/advertencias Svelte y con el aviso previo
de tamaño de bundle. Las siete pruebas existentes de `tests/ascii-ocean.mjs`
aprobaron con Node, incluyendo reloj/cadencia/pausas y geometrías finitas.
No se añaden suites permanentes ni se repiten suites ajenas al cambio.

Evidencia temporal fuera de Git:
`/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/noteka-headless-yDJ3cG/`.
Incluye fuentes anteriores, controlador, before.json y after-dark.json; los
resultados de las doce comprobaciones están en dark/keyboard-results.json.

## Límites

No se usó pantalla ni IPC nativo, ni se midió la cadencia física de Mac/Windows.
No cambia datos, contratos, preferencias, dependencias, reglas Rust o Meeting
Notes. Corrige el cumplimiento de la estabilidad temporal/redimensionado ya
vigentes en desktop-shell; se archiva con skip_specs y 71 requisitos intactos.

Cierre: 2/2 tareas verificadas, archivado como
`2026-10-25-fix-ascii-resize-flicker`; specs y archivos de Meeting Notes
contrastados por hash sin cambios.
