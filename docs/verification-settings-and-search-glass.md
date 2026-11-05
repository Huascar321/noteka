# Verificación del vidrio de Settings y Search

04/11/2026. Cambio `refine-settings-and-search-glass`, autorizado por el usuario
al revisar ambos popups. Queda local sin commit adicional, push, sync ni archivo.

## Implementación

Settings y Search comparten `.glass-popup`: tinte claro al 56% y oscuro al 72%,
sheen tenue, borde direccional al 80% y reflejo interior al 45%. Se retira el
radial blanco denso de Search que se sumaba al tinte previo al 72% en claro.
El blur de 16 px/saturación 1,35 se limita al diálogo montado; el backdrop no
lleva filtro. El velo de oscurecimiento exterior también es más tenue.

Resultados, preview, pie y navegación tienen un velo ligero de lectura.
Settings usa un velo algo más denso en su contenido, selección lateral en
cápsula y cierre circular. Ninguno de esos hijos agrega filtros. Los documentos,
tarjetas y controles previamente aceptados mantienen su tratamiento.

Los dos campos pasan a 32 px con fuente de 13 px/peso 400, padding único y
placeholder Search. Comparten borde y reflejos con Search lateral. Conservan
los nombres accesibles Search workspace/Search settings, semántica y funciones;
los controles de cierre/limpieza tienen un área de 24 px. Se normaliza la
apariencia de input search en WebKit y se evita duplicar su limpieza nativa.

Transparencia reducida/contraste aumentado retiran gradientes, sombras,
reflejos y filtros, con fondos sólidos y foco visible. No se agregan operaciones,
dependencias, formatos persistentes ni cambios en la ventana nativa.

## Evidencia

- `npm run build`: aprobado, cero errores/advertencias Svelte y checks de
  contratos/iconos aprobados. Vite conserva el aviso de bundle mayor a 500 kB.
- Recorrido focal Playwright CLI, UI compilada, CLI Rust y SQLite sintético,
  DPR 2: **62 comprobaciones Chromium / 61 WebKit**, aprobadas.
- Ambos temas: los campos miden 32 px y usan fuente de 13 px y placeholder
  Search; foco inicial del buscador, búsqueda de Settings, navegación a General,
  ruta de lectura, vacío y limpieza, Blue/Violet y ajuste de scroll operables.
- Search: resultados/preview, filtro de tipo, cierre con Escape y clic externo.
  Settings: entrada por teclado, contención de foco y Escape con retorno al
  disparador. Los cambios de popup no modifican la revisión de la nota sintética.
- Contraste medido sobre píxeles compuestos con fondos negro/blanco: mínimo
  **5,05:1 en ambos motores** para las muestras de campo/texto de Settings y
  campo/etiqueta de Search. Se oculta la tinta durante la captura y se aísla
  su transición al obtener el color.
- Transmisión comprobada sustituyendo un fondo de prueba azul por uno naranja:
  en un punto del contenido, la máxima diferencia de canal RGB fue
  **56/67 en claro y 37/40 en oscuro** para Settings/Search en Chromium;
  **45/54 y 30/32** respectivamente en WebKit. La superficie transmite cambios
  del fondo; esto no acredita refracción nativa ni mide un alfa óptico uniforme.
  El producto no incorpora ese fondo de prueba.
- Contraste aumentado/movimiento reducido, marcos y campos sólidos en ambos
  motores. Transparencia reducida emulada con CDP en Chromium; no emulada en
  WebKit. Ventana de 420 × 740 con popups contenidos y sin overflow global.
  Sin errores de página ni solicitudes externas en el recorrido focal.
- `headless-system-theme-smoke.mjs`: aprobado en Chromium. Verifica System,
  cambios en vivo claro/oscuro, conservación de borrador, acento, elección fija,
  error de almacenamiento y restauración al recargar.
- `openspec validate refine-settings-and-search-glass --strict` y
  `git diff --check`: aprobados. Diff propio revisado contra la copia previa al
  pedido; los cambios locales anteriores permanecen conservados.

## Límites

Las capturas Chromium muestran el fondo desenfocado. En WebKit headless el
contenido detrás conserva más detalle pese al filtro calculado de 16 px; la
transmisión y legibilidad se midieron también en ese caso. Una prueba temporal
con plano de material hijo no corrigió esa diferencia y no se incorporó al
producto. No se afirma paridad visual entre motores ni aceptación nativa
Tauri/macOS; no se accedió a la pantalla nativa en este cambio.

El acabado es CSS sobre contenido de la aplicación, con ventana opaca; no se
reproduce la refracción nativa de Apple ni se midió GPU/energía. Los primeros
intentos del recorrido corrigieron sincronización con la animación de entrada,
selectores ambiguos del test y apertura de sidebar en ancho estrecho. Las
pruebas existentes no se alteraron en este pedido.

Capturas y logs ignorados por Git:
`output/playwright/popup-glass-{settings,search}-{chromium,webkit}-{light,dark}.png`,
`popup-glass-{chromium,webkit}.log`, `popup-glass-system-theme.log`,
`popup-glass-build.log`. Scripts focales temporales:
`/tmp/noteka-popup-glass-check.cjs` y `/tmp/noteka-popup-glass-runner.mjs`.

Sistema de diseño, plan y workflow actualizados. El delta permanece activo;
las specs principales no publican comportamiento propuesto.
