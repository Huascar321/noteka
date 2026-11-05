# Verificación del vidrio en guardado y cambios pendientes

04/11/2026. Cambio `refine-save-guard-glass`, autorizado para los componentes
de las dos imágenes del usuario. Queda local sin commit adicional, push, sync
ni archivo.

## Implementación

- La guardia usa `.glass-popup` con `--glass-confirm-surface` al 69% en claro
  y 76% en oscuro, borde direccional y reflejos compartidos. Tiene un único
  blur local de 16 px. Los modales que adoptan el material se excluyen de la
  receta genérica anterior para que su especificidad no tape el nuevo acabado.
- Cancel/Discard usan cápsulas neutras `.glass-secondary`; Save and continue y
  Save/Retry de WorkspacePane comparten `.glass-action.primary` con los tokens
  de New. Texto/atajo blancos, borde al 75%, reflejo al 50%, hover y presión.
- La cabecera conserva 28 px mínimos para el botón y la guardia 36 px. A menos
  de 480 px, los secundarios comparten fila y la confirmación ocupa la siguiente.
  Unsaved changes mantiene su señal semántica y Save conserva su atajo.
- Contraste aumentado/transparencia reducida retiran filtros, sombras y
  reflejos, con fondos sólidos y foco visible. No cambian handlers, lógica de
  guardado, guardias, textos, contratos, dependencias o formatos persistentes.

## Evidencia

- `npm run build`: aprobado, cero errores/advertencias Svelte y checks de
  contratos/iconos aprobados. Sigue el aviso previo de Vite por bundle >500 kB.
- Recorrido focal Playwright CLI, UI compilada, CLI Rust y SQLite sintético,
  DPR 2: **63 comprobaciones Chromium / 62 WebKit**, aprobadas.
- Claro/oscuro y Blue/Violet: Save de cabecera con marco, tinte y atajo blanco,
  contraste compuesto en reposo/hover. La guardia conserva orden y etiquetas,
  Cancel como foco inicial, botones de 36 px y capas decorativas sin eventos.
- Contraste de texto medido sobre píxeles compuestos ocultando la tinta:
  mínimo **5,10:1 Chromium / 5,31:1 WebKit** en las muestras de primarios,
  secundario y descripción; los primarios de cabecera también se verificaron
  en hover. Se aíslan las transiciones al obtener tinta y fondo.
- Cancel y Escape preservan el borrador. Clic en el backdrop deja la decisión
  pendiente. Discard conserva el original persistido. Save and continue guarda
  antes de cerrar; los tres botones se deshabilitan durante un guardado retenido
  por una barrera temporal del transporte de prueba.
- Una falla real de SQLite mediante el trigger sintético existente conserva
  buffer y registro previo en cabecera y guardia. Retry y Save and continue
  completan después de retirar la falla. Ctrl+S guarda mediante el atajo
  existente; el documento original permanece idéntico durante las pruebas de
  edición de título. Sin errores de página ni solicitudes externas.
- Contraste aumentado y movimiento reducido: marcos/acciones sólidos sin
  reflejos o sombras en ambos motores. Transparencia reducida emulada por CDP
  en Chromium, no en WebKit. A 420 × 740, cabecera, diálogo y botones quedan
  alcanzables; la confirmación usa la segunda fila.
- `openspec validate refine-save-guard-glass --strict` y `git diff --check`:
  aprobados. Diff propio revisado contra la copia previa al pedido, con los
  cambios locales anteriores conservados.

## Límites

Pruebas sobre datos sintéticos y puente CLI, sin aceptación de IPC, foco físico
o composición nativa Tauri/macOS. No se accedió a la pantalla nativa en este
cambio. WebKit headless conserva más detalle del fondo que Chromium pese al
filtro calculado; se mantiene el límite documentado del popup anterior.
La ventana sigue opaca y el material es CSS sobre contenido de la aplicación,
sin refracción nativa ni medición GPU/energía. No se repitieron suites amplias
del núcleo ni se modificaron pruebas existentes.

Capturas/logs locales ignorados por Git:
`output/playwright/save-guard-{dialog,header}-{chromium,webkit}-{light,dark}.png`,
`save-guard-narrow-{chromium,webkit}.png`,
`save-guard-glass-{chromium,webkit}.log`, `save-guard-glass-build.log`.
Scripts temporales: `/tmp/noteka-save-guard-glass-check.cjs` y
`/tmp/noteka-save-guard-glass-runner.mjs`.

Sistema de diseño, plan y workflow actualizados; el delta permanece activo sin
publicar comportamiento propuesto en las specs principales.
