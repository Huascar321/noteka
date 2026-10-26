# Verificación de navegación, pestañas y detalle

Fecha: 23/10/2026. Cambio: `refine-desktop-navigation-and-tabs`.

## Alcance y preservación

Se reutilizó la sesión Notion del perfil autorizado para inspeccionar barra,
buscador, portada y referencias visuales. Se recuperó sólo la imagen decorativa
de Home para empaquetarla localmente. Las capturas de páginas, videos, sesión,
URLs firmadas y datos personales no forman parte de Git ni de las fixtures.
Las capturas de aceptación usan únicamente workspaces sintéticos.

Antes de modificar la UI se conservó un baseline fuera del checkout, incluido
el App anterior con todos sus cambios locales. Los componentes adicionales se
respaldaron antes de ampliarlos. Se separó la shell del panel de edición y se
conservaron operaciones, validaciones, contratos y persistencia Rust existentes.
No se ejecutaron migraciones ni se modificó el workspace personal del usuario.

## Resultados

| Recorrido | Resultado |
| --- | --- |
| Contratos, Svelte/TypeScript y build | Aprobados; cero errores/advertencias de Svelte. Vite conserva la advertencia de bundle mayor a 500 kB. |
| Controladores, editor, lecturas y proyecciones | 96 tests aprobados. |
| Navegación y presentación integrada | 102 checks aprobados en claro/oscuro, con CLI y SQLite reales. |
| Búsqueda de evidencia existente | 1.248 checks aprobados en claro/oscuro; cero requests externos. |
| Regresión general de editor, colecciones y Home | 592 checks compartidos + 14 de teclado aprobados en seis sesiones headless, con cero errores de ejecución del navegador. |

Los recorridos finales comparten el bundle `index-Dg7B3ABh.js`. Logs y resultados
locales se conservan en `output/playwright/` (ignorado), con copia de aceptación
fuera del repositorio. No se sustituyen resultados fallidos anteriores por éxitos.

La primera regresión funcional completa detectó dos errores de Svelte durante
el blur del editor: una notificación de transacción modificaba estado mientras
se desmontaba su vista. Se difirió sólo esa notificación a una microtarea, sin
diferir la escritura del documento. Se añadió una comprobación de errores del
navegador y se repitieron los tres recorridos finales sobre el mismo bundle.
Los logs aceptados son `navigation-clean.log`, `evidence-clean.log` y
`regression-clean.log`; los 96 tests puros están en `ui-acceptance.log`.

OpenSpec cerró 8/8 tareas. Se sincronizaron `desktop-shell` (un requisito
modificado y siete añadidos) y `local-text-search` (uno añadido), preservando
los otros 42 requisitos de esas dos specs y los archivos de las otras quince
capabilities. La validación estricta final aprobó las 17 specs (265 requisitos).
El cambio está en `archive/2026-10-23-refine-desktop-navigation-and-tabs` y no
quedan cambios activos.

## Casos nuevos cubiertos

- Seis entradas exactas y en orden; Collections fuera de la lista; vistas
  secundarias y búsqueda avanzada siguen disponibles.
- Un diálogo para Search y `+`; Escape no crea pestaña y devuelve foco.
  Flechas/Enter, Ctrl Tab y selección actual/nueva utilizan destinos reales.
- Dos borradores independientes, undo/redo al alternar, apertura retenida que
  termina en otra pestaña, guardias, fallo de almacenamiento, reintento confirmado
  y conflicto con una revisión real de CLI.
- Cierre de pestaña, cambio de workspace y evento de cierre del host simulado
  consideran el buffer inactivo; cancelar impide el cierre.
- El detalle no monta un listado, tampoco durante su carga. Tasks, Projects,
  Tags y People se abren desde listados centrales sin agregar una columna.
- Portada ASCII inicial, foto original local alternativa, cambio de tema y
  movimiento reducido. Cuadros cambian al animar y se detienen fuera de la
  vista o en una pestaña inactiva. Foto desmonta el canvas.
- Vista estrecha de 420 px sin overflow del documento; modal y navegación
  accesibles por teclado; cero errores de ejecución del navegador.
- Propiedades importadas separadas del cuerpo; abrir no muta. Editar/guardar
  recompone sus bloques e IDs exactamente como fueron persistidos.
- Una tabla Markdown se presenta como tabla, conserva bloque/ID al guardar otra
  parte y puede editarse explícitamente como Markdown mediante una transacción
  real del editor. Los valores se insertan como texto, sin evaluar HTML.
- Preparación y lector de adjuntos conservan acceso mediante Más opciones;
  abrir sus desplegables no altera el documento ni ejecuta preparación.

Las pruebas puras también cubren generaciones/cancelación de consultas y preview,
cursores/deduplicación, filtros reales, storage inválido, geometría finita,
recomposición exacta de prefijos, estructuras ambiguas y tablas con texto HTML
literal o pipes escapados. La inspección de código confirma limpieza de observers,
listeners y RAF al desmontar y la suspensión por `document.hidden`.

## Adaptación de regresiones

Los antiguos accesos Search/Archive/My Day se recorren mediante el diálogo real.
Los selectores de recuperación y shell usan sus nuevos nombres/alcance. Al abrir
una página se vuelve al listado mediante el menú cuando hace falta; la evidencia
de adjuntos y un buffer editado se mantienen en pestañas distintas. Se conservan
las aserciones de datos, guardias, cursores y fuentes. Los campos conservan el
mínimo de contraste 3:1 mediante `--control-border`, separado de los separadores
visuales sutiles.

## Límites

Estos recorridos usan Chrome headless y un puente de pruebas hacia el CLI Rust
real y SQLite sintético. El transporte Tauri y el cierre efectivo de una ventana
nativa no se ejecutaron en esta entrega; el evento y su confirmación se simulan.
Las verificaciones nativas históricas de Home conservan su alcance histórico.
No se acreditan paridad funcional completa con Notion, recuperación de borradores
entre reinicios, editor nativo de tablas, permisos/Share de Notion ni calidad
idéntica de la animación del video. Los tiburones son una composición procedural
propia observada desde arriba, con cola, giros, sombreado por celdas y trayectorias.
