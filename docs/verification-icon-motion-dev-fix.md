# Corrección de la animación lateral en desarrollo

Fecha: 23/10/2026. Cambio `fix-sidebar-animation-dev-loading`.

## Causa observada

La sesión Tauri usa Vite en `127.0.0.1:1420`. El import diferido de AnimateIcons
se reescribía a `node_modules/.vite/deps/@animateicons_vanilla.js` con una versión
que respondía **504 Outdated Optimize Dep**. Recargar la página de diagnóstico no
lo resolvía. El catch decorativo ocultaba la carga fallida y el icono permanecía
estático. La misma falla se reprodujo en Chromium y WebKit, con movimiento reducido
desactivado. No se atribuye el problema a WebKit ni a los SVG.

La revisión anterior correspondía al build de producción y no cubría el Vite de
larga duración donde se agregó/cambió la dependencia. Además, observar el indicador
de reproducción no demuestra por sí solo un cambio visible del dibujo.

## Corrección

`optimizeDeps.exclude` excluye sólo `@animateicons/vanilla` del prebundler en dev.
Es ESM autocontenido; se sirve directamente conservando el import al clic. La
[opción oficial de Vite](https://vite.dev/config/dep-optimization-options#optimizedeps-exclude)
no cambia el bundling de producción. No se relaja CSP ni se borran cachés globales.
Una excepción de carga ahora deja un diagnóstico en consola y conserva navegación.

## Evidencia acotada

- `npm run build` aprobado, catálogo/contratos vigentes, Svelte cero errores y
  advertencias. Bundle `index-aDzt6584.js`, 739,22 kB (227,79 gzip); player sigue
  separado y sin cambios, 74,69 kB (25,92 gzip). Persiste el aviso Vite >500 kB.
- Página local temporal con **la acción, el catálogo y el import reales** del
  producto, servida por el mismo Vite activo. Sin entidades ni puente de datos.
  Antes: una sola geometría, cero frames de reproducción y HTTP 504.
- Después: Home produce 32 matrices distintas en Chromium y 17 en WebKit;
  desplazamiento/escala y reposo exacto comprobados mediante rAF y bounding boxes.
- Revisión de los ocho iconos: 24–32 estados geométricos distintos en Chromium,
  12–17 en WebKit; todos vuelven al SVG original. Los siete adicionales se
  revisaron a 18 px. Sin errores de página ni fallos de carga de módulos; reducción de
  movimiento mantiene reposo.
- El proceso Tauri abierto se conservó; Vite reinició por configuración. No se
  enviaron escrituras de datos. La revisión no inspeccionó la pantalla nativa
  sin la autorización específica pendiente y no acredita IPC/cierre de ventana.

No se agregaron ni ejecutaron suites. Los logs de antes/después se conservan en
`output/playwright/icon-motion-{diagnosis,fix}-*.log`; fuentes y evidencia aceptada
se copiaron fuera del repositorio. Es una corrección del requisito vigente: no
modifica main specs, API, migraciones, imágenes, duración ni campos persistidos.
