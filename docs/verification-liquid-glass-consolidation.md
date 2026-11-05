# Consolidación de Liquid Glass — 04/11/2026

El usuario pidió guardar toda la serie aceptada en un commit local y ampliar el
documento de diseño para que otra IA reproduzca los resultados, con screenshots.
Esta autorización posterior sustituye el «sin commit adicional» de los registros
de verificación anteriores. No autoriza push, sync o archivo automático.

## Alcance entregado

- [Guía obligatoria](design-system.md): criterios de superficies, matriz de
  componentes, tokens y medidas vigentes, intensidad del borde/reflejo,
  anatomía de pseudoelementos, reutilización, estados, movimiento, accesibilidad,
  contraste compuesto, rendimiento y errores conocidos.
- [Atlas](design/liquid-glass/README.md): 29 PNG sintéticos, ambos temas,
  navegación/herramientas, People, Note editable/bloqueada/pendiente,
  relaciones/menús, Settings/Search, guardia, acento alternativo y fallback.
- `npm run design:screenshots`: generador versionado, Playwright CLI 0.1.22,
  Chromium 156.0.8078.80, frontend de producción y puente Rust/SQLite aislado.
  Viewport 1440×900, DPR 2; guardia estrecha 420×740; movimiento reducido.
- Manifest con SHA-256 de 68 fuentes, build y cada PNG, motor/versión y estado.
  PNG totales: 3.738.349 bytes. No contiene rutas del corpus temporal ni datos
  del usuario. Los nombres y empresas son fixtures; la foto es el icono local.

## Verificación de esta consolidación

- `npm run build`: catálogo de 85 iconos y contratos correctos; Svelte 0 errores
  y 0 advertencias. Vite conserva el aviso de chunk mayor a 500 kB.
- `npm run design:screenshots`: 29 capturas completas, 0 errores de página y
  0 solicitudes externas del frontend. Se verificaron carga de relaciones,
  Type bloqueado, placeholder Search y fallback sin blur antes de capturar.
- Inspección visual de los encuadres y estados. Se corrigió sólo el encuadre
  documental de Note para incluir la cabecera completa, sin cambiar la UI.
- `node scripts/headless-system-theme-smoke.mjs`: aprobó opción/default de
  System, cambio de tema en vivo, borrador, acento, tema fijo, error de storage,
  recarga y contraste compuesto del sidebar. Las advertencias del navegador
  de ese recorrido no son advertencias de compilación Svelte.
- Sintaxis de ambos scripts correcta; enlaces de guía/atlas y hashes de
  fuentes/build/29 PNG contrastados con los archivos reales.
- Validación estricta de los siete cambios visuales y del cambio documental;
  `git diff --check` y revisión de alcance antes del commit.

Los siete cambios visuales son `refine-sidebar-liquid-glass`,
`extend-liquid-glass-controls`, `refine-sidebar-search-capsule`,
`unify-liquid-glass-tools-and-popovers`, `refine-people-and-note-glass`,
`refine-settings-and-search-glass` y `refine-save-guard-glass`.
Conservan sus tareas verificadas y evidencia histórica enlazada en la guía.
`document-liquid-glass-system` usa `skip_specs: true`: sólo documentación y
herramientas de captura, sin una nueva modificación de comportamiento.

## Límites

La galería nueva corresponde a Chromium headless; no es una captura nativa ni
prueba IPC/Tauri, refracción Apple o rendimiento. No se repitieron las suites
funcionales completas ni la regresión Rust: este pedido no modifica lógica de
producto. La evidencia previa de Chromium/WebKit y los fallos conocidos de
suites históricas se conserva en los documentos de cada área. El generador
ofrece WebKit como recorrido separado, sin prometer el mismo blur entre motores.

Las primeras ejecuciones del generador fallaron por la selección explícita de
un browser no instalado, el uso de `require` no disponible en run-code y un
selector de toolbar no único. Se corrigió la herramienta para usar el Chromium
headless predeterminado, el endpoint de resultados existente y el toolbar
contextual; ninguna falla implicó modificar el producto. El comando final
completo pasó y los assets/manifest corresponden a esa ejecución.
