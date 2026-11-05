# Verificación de la cápsula Search del lateral

04/11/2026. Cambio `refine-sidebar-search-capsule`, después de la aceptación
del sidebar y de los cuatro controles de navegación/creación. Queda local,
sin commit adicional, push, sincronización ni archivo.

## Implementación

Search conserva el botón existente y sus operaciones, con cápsula de 32 px,
borde de 1 px, gris translúcido, luces interiores superior/inferior y sin
elevación difusa ni blur. Los tokens `--glass-field-*` controlan tinta, tinte,
borde, sheen, hover y presión. El atajo sigue visible en ancho normal, con tinta
opaca; en sidebar estrecho se oculta como antes. El foco usa el anillo existente.

El texto oscuro se mantiene gris; en tema oscuro se aclaró para sostener su
contraste sobre el brillo del sidebar. Transparencia reducida/contraste alto
retiran gradientes y sombras, con superficie sólida y borde definido.

## Evidencia

- `npm run build`: aprobado, 0 errores/advertencias Svelte. Vite conserva el
  aviso de bundle mayor a 500 kB. No se modificaron JavaScript, dependencias,
  operaciones ni datos en este cambio.
- Recorrido focal temporal con Playwright CLI, UI compilada, CLI real y SQLite
  sintético, DPR 2: **26 checks Chromium / 25 WebKit**, aprobados.
- Claro/oscuro con Blue/Violet: material neutro, radio completo, 32 px y sin
  filtro. Contraste de etiqueta sobre píxeles compuestos en reposo/hover:
  mínimo **4,68:1 Chromium / 4,77:1 WebKit**. El primer intento identificó
  contraste insuficiente en oscuro; se corrigió la tinta. La medición obtiene
  el color después de finalizar las transiciones de la captura para evitar
  mezclar tinta transitoria y fondo final.
- Foco visible, apertura con clic, Enter y Ctrl+K, y cierre con Escape aprobados
  en ambos motores; en ventana de 600 px, la cápsula queda contenida, el atajo
  se oculta y no hay overflow horizontal global.
- Alto contraste y movimiento reducido aprobados en ambos motores.
  Transparencia reducida simulada con CDP en Chromium, aprobada; no se emuló
  esa preferencia en WebKit. Sin errores de página ni solicitudes externas.
- Tauri/macOS: forma, borde y reflejos observados en el sidebar claro, con Home
  y el resto del acabado previo conservados. El tema oscuro se acredita con
  las capturas de WebKit, sin cambiar la preferencia nativa del usuario.
- `openspec validate refine-sidebar-search-capsule --strict` y
  `git diff --check` aprobados. Sistema de diseño, plan y workflow actualizados.

Capturas/logs locales ignorados por Git:
`output/playwright/sidebar-search-capsule-{chromium,webkit}-{light,dark}.png`,
`output/playwright/sidebar-search-capsule-{chromium,webkit}.log` y
`output/playwright/sidebar-search-capsule-build.log`.
Scripts temporales: `/tmp/noteka-search-capsule-check.cjs` y
`/tmp/noteka-search-capsule-runner.mjs`.

## Límites

No se convierte Search en un input editable del sidebar ni se cambia el popup
de búsqueda. La ventana es opaca y el acabado es CSS; no se midió GPU/energía
ni se repitió la suite amplia del núcleo. Las otras propuestas de Liquid Glass
no se implementaron en este pedido.
