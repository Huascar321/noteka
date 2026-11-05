# Verificación del vidrio líquido lateral

Aceptación posterior del mismo 04/11/2026: el usuario aprobó el acabado y
autorizó extenderlo a pestaña, breadcrumb, vista activa y New. Ver
[extensión verificada](verification-liquid-glass-controls.md). El registro
original siguiente conserva el alcance y los resultados de la primera prueba.

04/11/2026. Cambio `refine-sidebar-liquid-glass`. El commit local `affca8a`
preserva el estado completo anterior al experimento. La prueba posterior queda
como diff local para su aceptación visual; no se hizo push, sync ni archivo.

## Implementación

La cápsula activa existente usa tinte gris translúcido por tema, sheen estático,
un borde luminoso direccional de 1 px mediante máscara y sombras interiores.
La sombra exterior pasa de `0 4px 14px #0002` a una sombra corta de 2/5 px.
Se conservan 34 px de alto, 4 px entre filas, iconos, Jelly y desplazamiento
elástico de 280 ms. Las filas inactivas conservan fondo transparente.

El material no usa `backdrop-filter`, filtros SVG, dependencias ni animación
decorativa. Con transparencia reducida o contraste aumentado, el indicador
adopta `--hover` sólido, borde `--border` y elimina las dos capas de reflejo.
El fallback se coloca después de la regla base para que no quede sobrescrito.

## Evidencia

- `npm run build`: aprobado; 0 errores/advertencias Svelte. Vite conserva su
  aviso de bundle mayor a 500 kB; no se modificaron dependencias ni JavaScript
  de producto.
- `node scripts/headless-system-theme-smoke.mjs` y la misma ejecución con
  `NOTEKA_TEST_BROWSER=webkit`: aprobadas. Cubren tema claro/oscuro/en vivo,
  acentos, borrador conservado, preferencia fija/System, almacenamiento y
  reapertura. El texto activo supera 4,5:1 en todas las muestras interiores
  compuestas comprobadas.
- `node scripts/headless-sidebar-threshold-smoke.mjs` y su variante WebKit:
  12 checks por navegador aprobados; sidebar/peek, teclado y borrador conservados.
- Recorrido focal temporal con Playwright CLI, CSS de producción y SQLite
  sintético, DPR 2: 16 checks Chromium y 15 WebKit aprobados. Verifica borde
  enmascarado, neutralidad claro/oscuro, filas inactivas transparentes, ausencia
  de filtros, fallback de alto contraste, movimiento reducido, navegación/foco
  de teclado y cápsula contenida en ventana de 600 px (189 px de ancho de menú),
  sin overflow global, errores de página ni solicitudes remotas. Chromium
  verifica además `prefers-reduced-transparency` mediante CDP; WebKit no
  simula esa preferencia. Las primeras versiones de este recorrido se ajustaron
  al formato de máscara de cada motor y al colapso/animación del sidebar
  compacto; esos fallos de aserción no requirieron cambios en producto.
- Capturas aisladas de ambas variantes en
  `output/playwright/sidebar-liquid-glass-{chromium,webkit}-{light,dark}.png`
  y logs focales en `output/playwright/sidebar-liquid-glass-*.log` (artefactos
  locales ignorados por Git).
- Ventana Tauri/macOS con Vite HMR: observados el material y el borde en claro
  y oscuro, navegación Home → Projects → Home y apertura/cierre de Settings.
  El tema original System quedó restituido y la ventana quedó en Home.

El primer check de tema falló con contraste aparente 3,84:1 porque medía el RGB
del tinte `rgba(115,115,115,0.3)` como si fuera opaco. Se corrigió el check
existente: captura el indicador con etiquetas y diálogo temporalmente ocultos,
lee píxeles compuestos del interior y calcula el mínimo, excluyendo el borde
decorativo. Después pasaron ambos navegadores. La captura temporal no modifica
el contenido ni la preferencia de tema.

`openspec validate refine-sidebar-liquid-glass --strict` y `git diff --check`
aprobados. La validación de OpenSpec acredita los documentos, no reemplaza las
comprobaciones de UI anteriores.

## Límites

El fondo lateral y la ventana son opacos; el efecto aproxima volumen y luz,
sin refracción del escritorio ni paridad con Liquid Glass nativo. No se midió
GPU, energía ni 120 Hz físicos, y no se repitió la suite completa del núcleo.
La preferencia de transparencia reducida de macOS no se cambió globalmente.
No hay cambios de persistencia, Rust, contratos, contenido o configuración de
ventana. La aceptación estética corresponde al usuario.
