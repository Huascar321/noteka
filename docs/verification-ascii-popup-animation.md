# Verificación: animación ASCII bajo popups

Fecha: 05/11/2026. Cambio: `keep-ascii-animation-behind-popups`.

La portada ASCII de Home conserva su animación cuando Settings, Search o Center
peek están delante, siempre que Home y su portada sigan visibles. `blocked` y el
foco de edición deshabilitan controles, atajos y edición, pero no suspenden el
canvas. Full page que cubre Home, la pestaña inactiva, la portada fuera de vista,
el documento oculto y `prefers-reduced-motion` mantienen sus pausas.

## Causa corregida

`HomePanel` decidía la animación con `visible && (!blocked || searchOpen) &&
(active || animateAlongsideSidePeek)`. Settings fijaba `blocked` y Center peek
movía el foco a otro panel, aunque el canvas permaneciera en pantalla. Ahora la
única señal es `visible` (pestaña mostrada y no cubierta por Full page); el
`IntersectionObserver`, la visibilidad del documento y el movimiento reducido
siguen dentro de `AsciiShark`. Se retiraron `searchOpen` y
`animateAlongsideSidePeek` porque duplicaban esa señal.

## Auditoría de popups y overlays

| Superficie | Comportamiento anterior | Decisión |
| --- | --- | --- |
| Settings | Pausaba por `blocked`. | Corregido: anima mientras Home siga visible. |
| Search | Ya eximía la pausa desde `improve-search-popup-interaction`. | Verificado: anima y conserva los controles del fondo bloqueados. |
| Center peek | Pausaba porque el foco pasaba al detalle. | Corregido: anima detrás del backdrop mientras Home siga visible. |
| Full page | Pausaba porque cubre Home por completo. | Conservado: la portada no es visible y no debe consumir un bucle. |
| Guardia de cambios pendientes | No cambiaba `visible` ni el reloj. | Sin cambios; no es un popup con pausa de portada. |
| Diálogo de mover Project | No cambiaba `visible` ni el reloj. | Sin cambios; la portada sólo pausa si queda fuera de vista. |
| Menús, pickers y paneles plegables | No cambiaban `visible` ni el reloj. | Sin cambios. |
| Pestaña inactiva, portada fuera de vista, documento oculto, movimiento reducido | Pausas existentes. | Conservadas. |

## Comprobaciones

- `npm run build`: marca, iconos y contratos verificados; `svelte-check` con
  **0 errores y 0 advertencias**; Vite compiló el cliente (aviso habitual de
  tamaño de bundle).
- `node --experimental-strip-types --test tests/ascii-ocean.mjs
  tests/ascii-cover-scenes.mjs tests/ascii-koi-palette.mjs`: **18 pruebas
  aprobadas**, 0 fallos. No se modificó el renderer.
- `npm run test:headless-ascii-popups` en Chromium y
  `NOTEKA_TEST_BROWSER=webkit npm run test:headless-ascii-popups` en WebKit:
  **10 comprobaciones aprobadas por motor**, con workspace sintético vía bridge
  CLI Rust/SQLite, sin solicitudes externas ni errores de página. Se comprobó:
  - ASCII avanza con Settings abierto y sigue al cerrarlo.
  - ASCII avanza con Search abierto y sigue al cerrarlo.
  - ASCII avanza con Side peek enfocado.
  - ASCII avanza detrás de Center peek con Home visible.
  - Full page mantiene la portada suspendida mientras cubre Home.
  - Al volver de Full page a Center peek, la portada reanuda.
  - El mismo canvas de Home permanece montado durante toda la secuencia.
- `npx --yes @fission-ai/openspec@1.14.1 validate
  keep-ascii-animation-behind-popups --strict`: **válido**.

La suite histórica `headless-ascii-side-peek` no se usó como evidencia verde:
conserva expectativas sustituidas (blur exacto de 8 px y pausas de Search/Center
peek) y falla antes de llegar a esas comprobaciones. La suite nueva
`headless-ascii-popups` cubre la decisión vigente.

## Límites

La verificación es headless en Chromium y WebKit; no sustituye una aceptación en
la ventana Tauri nativa ni una medición en hardware de 120 Hz. No se midió
CPU/GPU ni se afirma una mejora de rendimiento: se reutiliza el único
`requestAnimationFrame` existente, sin timers, listeners, datos ni preferencias
nuevas. Sin cambios en Rust, SQLite, contratos, dependencias o red.
