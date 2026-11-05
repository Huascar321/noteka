# Verificación: adopción de Liquid Glass

Cambio `complete-liquid-glass-adoption`, sobre el checkout inicial `744a49a`.
La evidencia usa workspaces sintéticos y un puente headless hacia CLI Rust y
SQLite; no equivale a IPC ni composición nativa de Tauri/macOS. No se usaron
workspaces privados ni proveedores externos.

## Construcción y recorridos

- `npm run build`: aprobado, sin errores ni advertencias de Svelte. Vite advierte
  que el chunk principal supera 500 kB.
- `headless-system-theme-smoke`: aprobado en Chromium y WebKit; cambio de tema y
  acento, borrador, persistencia y restauración de la preferencia.
- `headless-liquid-adoption-smoke`: 569 checks en Chromium y 568 en WebKit.
  Incluye 1440, 900 y 420 px, claro/oscuro y Blue/Violet; barras, nombres
  accesibles e iconos de creación, overflow, creación y cambio de colección,
  calendario, Move/Cancel/Escape, foco, relaciones y guardado. Comprueba menús
  y su foco en side/center/full con scroll y Cancel/Discard/Save de borradores.
  El core admite un solo tag directo en Goal; se verifica la validación y la
  conservación del vínculo, y el wrapping múltiple se comprueba en Task.
- `headless-liquid-insert-smoke`: 10 checks por motor para selector, inserción,
  cancelación, Escape, restauración de foco y variante integrada sin blur.
- `headless-liquid-focus-smoke`: 21 checks por motor para puntero y teclado en
  pestañas, campos, título y diálogo, selección de color, movimiento reducido y
  mayor contraste. El anillo de teclado permanece visible; los indicadores de
  selección conservan su significado.
- `headless-liquid-tools-smoke`: 13 checks por motor para búsquedas, herramientas,
  exportación sintética, Settings, color y workspace, sin solicitudes externas.
- Project board: cinco checks por motor. Goal detail: 28 por motor. Relation
  picker: 33 por motor.

## Auditoría visual y contraste

En cada motor se midieron 532 muestras con Blue y 550 con Violet, entre claro y
oscuro: 2.164 en total, sin fallos y con mínimo 4,503:1. Los informes de
[Chromium Blue](design/liquid-glass/contrast/liquid-contrast-chromium-blue.json),
[Chromium Violet](design/liquid-glass/contrast/liquid-contrast-chromium-violet.json),
[WebKit Blue](design/liquid-glass/contrast/liquid-contrast-webkit-blue.json) y
[WebKit Violet](design/liquid-glass/contrast/liquid-contrast-webkit-violet.json)
registran cada muestra. Por cada combinación se omitieron 72 controles ocultos
y 14 deshabilitados. Se aprobaron 476 comprobaciones de fallback por motor con
Blue y 492 con Violet, sin fallos.

La medición captura los píxeles de fondo reales del control, temporalmente sin
su tinta, y calcula contraste sRGB sobre las zonas de texto o icono en reposo y
hover. No usa el alfa de un token como sustituto de composición. Se revisan
controles visibles y habilitados de las escenas instrumentadas: los ocultos y
deshabilitados quedan registrados como omitidos, y sus estados funcionales se
comprueban por separado. La muestra no cubre todos los fondos o contenidos
posibles. Los informes incluyen hashes de fuentes y build, muestras y omisiones.

Mayor contraste se emuló en ambos motores. Playwright no expone la preferencia
menor transparencia para WebKit: allí se activaron en su lugar las mismas reglas
CSS de esa media query. En Chromium también se comprobó la emulación de la
preferencia por CDP. No se afirma una prueba de la preferencia nativa de macOS.

El [atlas](design/liquid-glass/README.md) contiene 71 capturas sintéticas por
motor, sin solicitudes externas. Ambos manifiestos registran los tres ejes de
contexto, con `null` cuando no se presentan en la escena; se verificaron los
hashes de las imágenes, fuentes y build. La captura espera operaciones ociosas
y una ventana sin actividad. La revisión visual incluye detalles, campos,
foco y búsqueda estrecha. Las imágenes no acreditan composición nativa.

## Fallos observados durante la verificación

La suite histórica `headless-page-actions` se detuvo antes de los recorridos en
ambos motores al estimar contraste con `backgroundColor` y alfa sin componer las
capas. Esa fórmula ya estaba en `744a49a` y no mide los píxeles compuestos. La
suite histórica `headless-grouped` esperaba `.home-coverage` sin abrir opciones;
ese elemento ya era condicional en `744a49a`. No se modificó el producto para
satisfacer esas expectativas obsoletas.

El primer runner de Project board no generaba `results.json`; se corrigió el
guion y se repitió. Una primera prueba de relation picker en WebKit falló en una
aserción de duplicación y la repetición aislada pasó, sin causa determinada.
Los recorridos nuevos necesitaron esperas explícitas de carga y guardado; una
primera comprobación de foco después de clic se reemplazó por la interacción de
teclado que se quería verificar. Los intentos del atlas corrigieron selectores
para un `<option>`, TipTap y un fieldset sin dimensiones. Una primera versión
del medidor leyó mal el placeholder e incluyó bordes; se corrigió antes de las
mediciones finales. La auditoría también detectó una sombra conservada por
especificidad en el fallback del menú de herramientas: se corrigió el selector y
se repitieron las comprobaciones.

`openspec validate complete-liquid-glass-adoption --strict` y `git diff --check`
pasaron. Las specs principales no se sincronizaron y el cambio no se archivó.
