# Verificación del vidrio en navegación y creación

04/11/2026. Cambio `extend-liquid-glass-controls`, sobre el acabado del sidebar
aceptado por el usuario. El checkpoint anterior continúa en `affca8a`; la
extensión queda local, sin commit adicional, push, sync ni archivo.

## Resultado

- Pestaña superior activa: cápsula neutra de 32 px, borde direccional al 65%,
  reflejo interior al 60% y sombra corta. El contenedor reserva 4 px para el
  acabado; +/× conservan sus áreas y muestran vidrio circular en hover/foco.
- Breadcrumb: borde al 25%, reflejo interior al 30% y sombra de 1/4 px; mantiene
  semántica de miga actual y filtro local existente de 3 px.
- Vista activa: material neutro compartido con la pestaña, sobre el indicador
  deslizante existente. Mantiene su movimiento y Jelly.
- New: borde al 75%, reflejo interior al 50%, tinte derivado del primario,
  texto blanco y relieve interior al presionar. La paleta deriva sólo
  `--glass-action-tint`, con contraste base mínimo de 7:1 para dejar margen al
  sheen; iconos y otros botones conservan su acento anterior.

El borde y relieve reutilizan la receta del sidebar. Sus capas decorativas no
reciben eventos. Los fallbacks sólidos se definen después de las variantes y
eliminan reflejos/filtros; no se incorporan dependencias, filtros grandes ni
relojes de animación. Rust, operaciones y persistencia no cambian.

## Comprobaciones

- `npm run build`: aprobado, 0 errores y 0 advertencias Svelte. Vite conserva
  el aviso de chunk mayor a 500 kB. Log local:
  `output/playwright/liquid-glass-controls-build.log`.
- `node scripts/headless-system-theme-smoke.mjs` y variante
  `NOTEKA_TEST_BROWSER=webkit`: aprobadas. Conservan temas/acento, borrador,
  preferencias fijas/System, errores de almacenamiento y reapertura.
- Recorrido temporal de Playwright CLI contra UI de producción/CLI Rust/SQLite
  sintético: **48 checks Chromium y 47 WebKit**, DPR 2. Material y borde en los
  cuatro controles, área de pestaña, neutralidad de navegación, seis acentos,
  estados +/×, foco, cambio de vista, breadcrumb, creación/cierre/navegación de
  tabs, creación de Task y ventana de 600 px sin overflow global aprobados.
- New se midió sobre píxeles compuestos de su interior en seis colores por dos
  temas, en reposo y hover: mínimo **5,16:1 Chromium / 5,19:1 WebKit**. Para
  capturar el fondo se ocultan el texto y su sombra y se desactiva temporalmente
  su transición de color. Los primeros intentos de WebKit todavía capturaban
  letras mientras se volvían transparentes; el fallo aparente de contraste se
  resolvió en el instrumento de captura, sin cambiar el material.
- Alto contraste y movimiento reducido aprobados en ambos motores.
  Transparencia reducida simulada con CDP en Chromium, aprobada; no se emuló
  esa preferencia en WebKit. El recorrido no registró errores de página ni
  solicitudes externas.
- Las primeras versiones del recorrido corrigieron un selector ambiguo de
  Inbox y activaron modalidad de teclado antes de observar el foco en WebKit;
  los resultados anteriores no se presentan como aceptación de producto.
- Ventana Tauri/macOS: material de los cuatro controles observado en Home en
  claro y oscuro. El tema original System quedó restituido y la ventana quedó
  en Home. Los recorridos completos de acciones se acreditan con las pruebas
  anteriores: dos intentos adicionales de + nativo se interrumpieron por una
  detección de cambio externo de la app en la herramienta de UI.
- OpenSpec estricto y `git diff --check` aprobados. El diseño compartido y el
  plan se actualizaron; las specs principales no se sincronizaron.

Las capturas y logs locales están en
`output/playwright/liquid-glass-controls-{chromium,webkit}-{light,dark}.png`,
`output/playwright/liquid-glass-controls-{chromium,webkit}.log` y
`output/playwright/liquid-glass-controls-system-*.log`; Git los ignora.
Los scripts temporales usados están en `/tmp/noteka-controls-glass-check.cjs`
y `/tmp/noteka-controls-glass-runner.mjs`.

## Límites

La ventana es opaca: es una aproximación CSS al material, sin refracción del
escritorio. El breadcrumb sobre contenido variable conserva el límite de
contraste documentado y su fallback sólido. No se midieron GPU, energía ni
120 Hz físicos, no se cambió la accesibilidad global de macOS y no se repitió
la suite amplia del núcleo ni se hizo aceptación Windows.
