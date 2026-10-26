# Revisión acotada — AnimateIcons

Fecha: 23/10/2026. Cambio `adopt-static-animateicons`.

## Compilación

`npm run build` aprobado: catálogo generado vigente, contratos vigentes y
svelte-check con cero errores/advertencias. Bundle principal `index-B5tEFoG_.js`,
739,14 kB (227,75 kB gzip); player separado `sidebar-icon-animation-BWDgbt-h.js`,
74,69 kB (25,92 kB gzip). Persiste el aviso previo de Vite sobre chunks >500 kB.
Los SVG elegidos ocupan 55.243 bytes de fuente generada; no se incluye el catálogo
completo. `@animateicons/vanilla` fijado 0.1.0; dependencia Svelte anterior retirada.

## Inspección manual

UI de producción en Chromium headless → puente confinado → CLI/SQLite de copia
real. Cero errores de página y cero peticiones externas. Sin chunk de animación
antes del clic; aparece únicamente al pulsar Home. Hover conserva el SVG exacto;
un icono activo durante el ciclo y retorno exacto después. Movimiento reducido
mantiene reposo; clics repetidos conservan un único ciclo y reposo exacto final.

Se revisó el ciclo en los ocho botones, activación con Enter y cancelación al
cambiar a movimiento reducido durante la reproducción; sin alertas.

La pantalla conserva seis entradas primarias y los dos botones Search/Collections.
Tags: 13 filas y 21 imágenes locales, temas claro/oscuro; ancho 420 sin overflow
horizontal global. Se inspeccionaron las capturas `icons-sidebar-light/dark.png`,
`tags-reference-final-light.png` e `icons-narrow.png`. El selector de imágenes,
los payloads y el núcleo no tienen nuevas escrituras de este cambio.

## Límites

No se agregaron ni ejecutaron suites. El resultado es compilación e inspección
manual de navegador; no nueva aceptación de IPC/WebKit/cierre Tauri. Cancelación
al desmontar/ocultar se contrastó en código; no se simuló cierre del proceso nativo.
Las mediciones describen este build, no un benchmark general ni una reducción del
bundle total. Licencias completas en `third-party/animateicons/` y distribuidas como assets
offline desde `public/licenses/animateicons/`.
