# Vidrio transparente compartido en popups

Fecha: 10/11/2026. Cambio: `share-clear-popup-glass`. Continúa el ajuste
`clarify-search-glass` por pedido explícito del usuario.

**Refinamiento posterior:** el usuario pide blancos más luminosos. El tema
claro pasa a `#eeeeeead` con el mismo alfa, blur y sheen. La verificación y
capturas nuevas están en la sección «Ajuste de luminosidad» al final; las
galerías anteriores conservan el tinte gris inicial como antecedente.

## Resultado e inventario

La receta aprobada en Search pasa a `--glass-overlay-*`: tinte gris
`#c2c2c2ad`/`#202020a8`, sheen tenue, `blur(6px) saturate(1.15)` y tinta
principal/secundaria adaptada. No quedan recetas de popup con blur 10/24 px,
tinte de confirmación independiente ni velos blancos de Settings.

| Familia anterior | Superficies actualizadas |
| --- | --- |
| `.glass-popup` | Search, Settings, guardia de cambios pendientes y Move project |
| `.glass-menu` | Page actions, Peek mode, IconPicker, RelationPicker, PageTools flotante y More views |
| `.editor-popover` | Formato de selección, comandos `/`, acciones de bloque, link/attachment e Insert local view |
| `.date-picker-material` | Calendario Due; el padre `.date-picker` comparte tinta con su contenido hermano |
| `.page-tool-tooltip` | Ayudas de herramientas que ya usaban el filtro anterior |
| Modal genérico | La receta de reserva existente también usa el material compartido |

Los cuerpos internos de Search y Settings quedan transparentes. Menús y
diálogos conservan sus radios, bordes y sombras respectivos. La capa decorativa
de transición copia el material calculado; no se agrega filtro a su padre.
Los documentos, tarjetas, Side/Full/Center peek y navegación persistente
conservan su categoría de superficie. Nav y disclosures en el flujo no tenían
ese material y no se convierten en popups de vidrio. Los controles del sistema
siguen siendo nativos.

La tinta de placeholders y atajos no se atenúa por reglas genéricas. Done y
otras acciones primarias planas de overlays usan el tinte con contraste que ya
comparten Save/New. Hoy sin seleccionar en Due usa tinta legible y borde de
acento; la selección conserva su color. Load more de relaciones conserva acento
en el subrayado. Errores y avisos mantienen fondos semánticos sólidos locales.

## Evidencia visual

Frontend de producción con CLI Rust real y SQLite temporal. Datos ficticios,
sin notas/contactos del usuario ni solicitudes externas del frontend.
Capturas originales, sin edición; cada motor conserva sus diferencias reales.

- [Galería Chromium](design/clear-popups/chromium/manifest.json) y
  [galería WebKit](design/clear-popups/webkit/manifest.json): 28 PNG por motor,
  ambos temas, material de Settings, Search, guardias, Move project, PageTools,
  menús y relaciones; incluye 420 px y alternativas sólidas.
- [Settings claro](design/clear-popups/chromium/settings-light.png) y
  [oscuro](design/clear-popups/chromium/settings-dark.png): material nuevo en
  todo el diálogo, sin velos blancos en sidebar/main.
- [Menú](design/clear-popups/chromium/page-menu-light.png),
  [relaciones](design/clear-popups/chromium/relation-picker-light.png) y
  [herramientas](design/clear-popups/chromium/export-tools-light.png): cuerpos
  flotantes con la misma receta.
- [Guardia](design/clear-popups/chromium/save-guard-light.png),
  [Move project](design/clear-popups/chromium/move-project-light.png) y
  [guardia a 420 px](design/clear-popups/chromium/save-guard-narrow-dark.png):
  decisiones y controles conservados sobre material transparente.
- [Search sólido](design/clear-popups/chromium/search-solid-light.png):
  contraste aumentado retira el filtro y los reflejos.

Comparación capturada antes de modificar el material compartido:
[Settings anterior](design/clear-popups/before/settings.png) /
[nuevo](design/clear-popups/chromium/settings-dark.png),
[IconPicker anterior](design/clear-popups/before/icon-picker.png) /
[nuevo](design/clear-popups/details/icon-picker.png),
[guardia anterior](design/clear-popups/before/save-guard.png) /
[nueva](design/clear-popups/chromium/save-guard-dark.png).
Los corpus/escenas pueden variar entre recorridos; no son golden tests de fauna.

## Verificación

- Build: cero errores/advertencias Svelte; se conserva el aviso de Vite por
  bundle JavaScript mayor a 500 kB.
- Overlays: 77 checks Chromium y 77 WebKit de transiciones, limpieza, foco del
  destino, buffers, apertura/cierre, temas, acentos y ancho estrecho.
- Due: 56 checks Chromium y 56 WebKit de selección, teclado, anclaje, resize,
  movimiento reducido, alternativas sólidas y neutralidad con otro acento.
- Adopción existente: 569 checks Chromium, incluidas operaciones de Move
  project/Cancel/persistencia y controles compartidos.
- Search: 23 checks focales Chromium, incluidos extremos compuestos,
  resultados/filtros, preview, Enter, cierre y preferencias.
- Auditoría focal del material: superficies asentadas, texto real, ambos temas
  y acentos en Settings, extremos negro/blanco en diálogos y fondo real en menús.
  Comprueba blur compartido, contraste de texto, opacidad de atajos/placeholders
  y fallback sólido; excluye controles deshabilitados de la exigencia textual.
  Resultados por región en los JSON locales de auditoría.
- OpenSpec estricto y `git diff --check` verifican documentos y whitespace;
  no sustituyen los recorridos de aplicación.

La auditoría conserva la tinta calculada antes de ocultarla sólo durante la
captura de medición, compone su opacidad real y calcula luminancia sRGB/WCAG
en el PNG. Los fondos sintéticos extremos se usan sólo en mediciones y se
retiran antes de capturar evidencia visual de producto.

Auditoría de la extensión inicial: 27 grupos y 517 regiones de texto por motor. Chromium aprobó
110 comprobaciones y WebKit 111; la diferencia corresponde a un frame de
transición observable opcional del harness de medición, no a requisitos
distintos. La suite independiente de transiciones aprobó 77 en cada motor.

| Motor | Mínimo claro | Mínimo oscuro |
| --- | ---: | ---: |
| Chromium | 4,61:1 | 4,61:1 |
| WebKit | 4,54:1 | 4,91:1 |

Los manifests de ambas galerías se contrastaron con el hash de
`src/style.css` de la extensión inicial; los 56 PNG corresponden a esa etapa,
previa al ajuste de luminosidad. Capturas adicionales
de IconPicker y Due están en `design/clear-popups/details/`.

Artefactos locales: `output/playwright/clear-popups/`, incluidas trazas de
antes/después, recorridos, logs de build/pruebas y auditorías por motor. La
galería se obtiene con el recorrido existente
`scripts/capture-liquid-glass-guide.cjs`, filtrando únicamente estados de
popups al copiar; los manifests conservan hashes y estado de cada captura.

## Límites y hallazgos

- WebKit headless muestra el fondo más nítido que Chromium aunque compute el
  mismo filtro. No se verificó esta extensión en Tauri/WKWebView nativo ni se
  afirma refracción de Apple, transparencia al escritorio o mejora de CPU/GPU.
- Se conserva la limitación existente de devolución de foco a `body` en
  WebKit al cerrar Search con Close; ver el
  [ajuste anterior](verification-search-clear-glass.md). La propagación no
  modifica handlers de foco o navegación.
- La auditoría encontró contraste insuficiente en Done (4,26:1), placeholder
  de relaciones (3,62:1) y atajos atenuados de Search (3,58:1). Se corrigieron
  usando tinta/tinte compartidos y opacidad completa. No se aceptaron como
  excepciones a 4,5:1.
- El harness inicial heredó movimiento reducido para una prueba de morph;
  se corrigió su configuración. La auditoría de píxeles espera superficies
  asentadas: no exige observar un frame de 400 ms después de hacer capturas.
  Los tiempos se verifican por separado con la suite existente.
- Sin operaciones, Rust, SQLite, dependencias, commit, push, sync o archivo
  nuevos. Los cambios anteriores del checkout se conservan.

## Ajuste de luminosidad

El pedido posterior del usuario aclara sólo el tinte del tema claro, de
`#c2c2c2ad` a `#eeeeeead`. Conserva alfa `ad`, sheen, blur 6 px, saturación y
texto. El tono oscuro sigue siendo `#202020a8`. Un único token compartido
propaga el blanco más luminoso a todos los popups del inventario.

Comparación en Chromium:
[Settings antes](design/bright-glass/before-settings-light.png) /
[después](design/bright-glass/chromium-settings-light.png),
[Search antes](design/bright-glass/before-search-light.png) /
[después](design/bright-glass/chromium-search-light.png).
El [menú de página](design/bright-glass/chromium-menu-light.png) verifica la
misma receta en una superficie pequeña. Cada motor conserva capturas de
Settings, Search y menú en ambos temas en `docs/design/bright-glass/`.

Verificación focal sobre build de producción, datos ficticios y CLI/SQLite
temporal: estilos calculados del tinte, búsqueda/apertura de página y menús,
contraste de texto compuesto con extremos negro/blanco en diálogos, fondo
real en menú y fallbacks de contraste aumentado. Se conservan trazas y JSON
en `output/playwright/bright-glass/`. Build y OpenSpec estricto correctos;
permanece el aviso Vite del bundle mayor a 500 kB. No se repiten las suites
de operaciones/transiciones porque sólo cambia un color, no su comportamiento.
La composición nativa Tauri sigue sin verificarse.

El recorrido focal aprobó 22 checks por motor. Se midieron 14 grupos por motor
con opacidad real: todos superan 4,5:1 (mínimos 4,77:1 en Chromium y 4,91:1 en
WebKit); los resultados y hashes están en
el [manifest](design/bright-glass/manifest.json). El harness enfoca el menú antes
de probar Escape, tras alternar las preferencias de contraste; no se modificó
el producto para ajustar ese control de la prueba.
