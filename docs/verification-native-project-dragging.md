# Verificación del arrastre de Projects

28/10/2026. `fix-native-project-board-dragging` desactiva el manejador de
arrastre nativo de Tauri para entregar los eventos HTML al WebView. La versión
local de Wry delega al WebView cuando no hay manejador y el manejador de Tauri
aceptaba los eventos; esto identifica una causa probable de la diferencia entre
la aplicación y la prueba en navegador. Se previene el drop externo de archivos
y URLs para evitar navegación o importación implícita.

La compilación pasó con cero errores y advertencias de Svelte; Vite informó el
chunk grande habitual. Una prueba en Chromium y WebKit aprobó nueve comprobaciones
por motor, con UI compilada, CLI real y SQLite sintético: arrastre entre estados,
conservación de propiedades, recarga, posición sin cambio, alternativa de diálogo
y prevención de drops externos. El arrastre HTML funcionaba en esos navegadores
incluso antes del cambio de configuración, por lo que no demuestra por sí solo la
corrección nativa.

En la ventana Tauri reiniciada se abrió un workspace temporal y la automatización
inició el arrastre de su tarjeta, pero no completó el gesto de forma verificable;
el proyecto sintético permaneció en su estado anterior. No se acredita aún el
drop nativo completo. Se restauró el workspace habitual del usuario y no se
movieron sus proyectos.

## Seguimiento 29/10/2026

El usuario informó que el problema persistía. Se reemplazó el arrastre HTML de
las tarjetas por un gesto de puntero con umbral de seis píxeles, tarjeta flotante,
columna resaltada y marca de inserción. El guardado mantiene el lote y CAS
existentes. Escape y soltar fuera del tablero cancelan; los controles de relación
y el diálogo Move conservan su acción propia.

Build aprobado (cero errores y advertencias Svelte; aviso habitual de tamaño de
chunk). Chromium y WebKit aprobaron 14 comprobaciones cada uno con UI compilada,
CLI real y SQLite sintético: indicador, movimiento, propiedades, persistencia,
cancelación, alternativa de diálogo y drops externos. En Tauri/macOS se creó un
workspace temporal y se comprobó el clic de la tarjeta, pero dos intentos de
arrastre con la automatización de escritorio dejaron el proyecto en Doing; no se
estableció si el gesto llegó al WebView. Por ello el movimiento nativo físico
sigue sin verificarse y la tarea de aceptación permanece abierta. Se restauró el
workspace original sin mover proyectos personales.

Ante el seguimiento visual del usuario se quitó el fondo parcial de hover y se
extendió el inicio del gesto a toda la tarjeta, incluidas relaciones y Move.
Build y 17 comprobaciones en cada motor (Chromium/WebKit) pasaron con datos
sintéticos, incluido arrastrar desde una relación y abrirla con clic. Esto no
añade una prueba de arrastre físico nativo.

Seguimiento de recorte: el tablero reserva cuatro píxeles laterales y superiores
para el contorno exterior de destino. Build y 18 comprobaciones WebKit pasaron;
la prueba confirma durante el arrastre que el contorno de Planned cabe dentro
del contenedor desplazable. No se hizo una nueva prueba física nativa.
