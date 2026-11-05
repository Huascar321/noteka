# Editor contextual de Notes

Las Notes normales se editan directamente sobre una columna de página. Título,
propiedades y relaciones conservan sus operaciones; el cuerpo usa los mismos
documentos v1/v2/v3 y el guardado explícito de Noteka.

## Escribir y dar formato

- Hacé clic en el cuerpo o presioná Enter desde el título.
- Hacé clic en el espacio vacío debajo del último bloque para continuar en un
  párrafo nuevo con el cursor listo para escribir. Si ya hay un párrafo vacío al
  final, se reutiliza; los clics repetidos no acumulan bloques vacíos. También
  funciona después de código, listas y encabezados. El cambio queda en el
  borrador, con undo/redo y guardado explícito.
- Seleccioná texto para abrir la barra de negrita, cursiva, tachado, código y enlace.
  Sólo se ofrecen las marcas que admite el documento local.
- Escribí `/` al inicio de un bloque o después de un espacio para filtrar comandos.
  Usá ↑/↓ y Enter para elegir, Escape para seguir con la barra escrita. `/h1`,
  `/h2` y `/h3` eligen niveles de encabezado. URLs y bloques de código no activan
  el menú por sus barras.
- Adjuntos y vistas se eligen explícitamente. La imagen o archivo se importa desde
  una ruta local; la vista guarda su referencia al original. Cancelar el selector
  conserva el texto del comando.

## Trabajar con bloques

Al pasar sobre un bloque aparecen `+` y el control de puntos. `+` inserta debajo
y abre los tipos disponibles. Los puntos permiten duplicar, mover arriba/abajo,
insertar debajo o eliminar; párrafos, encabezados y código también permiten
cambiar de tipo. Una lista se mueve como unidad. Arrastrá los puntos para mover
un bloque dentro de la página, o usá los botones de movimiento.

Los controles siguen visibles al mover el puntero hacia cualquiera de los dos
márgenes, mientras siga a la altura del bloque dentro del panel de Note. También
aparecen al entrar directamente por un margen y cambian al pasar a otra fila.
Al salir de esa franja o del panel se ocultan, aunque el editor siga enfocado.
Los huecos verticales, el espacio vacío debajo del párrafo y los popovers no
cuentan como hover. Escribir o entrar desde el título no los activa. El foco
explícito por teclado y el arrastre conservan el acceso a las acciones.

Mover conserva IDs y marcas. Duplicar genera nuevas identidades para el bloque y
sus hijos. Undo/redo siguen disponibles con ⌘Z/Ctrl+Z y su combinación de rehacer.
Alt+Shift+B abre acciones del bloque desde el teclado; Alt+Shift+F lleva a las
herramientas de una selección de texto.

## Herramientas de página y guardado

El botón `…` de la barra superior agrupa Export, External resources, Attachments
y Archive (Restore si la Note está archivada). Cada opción muestra su icono y
nombre; las acciones no disponibles conservan su condición. Usá flechas y
Home/End para recorrerlas, Enter para elegir y Escape para cerrar. Los paneles
se ajustan al viewport y conservan campos/resultados al cerrar. Escape devuelve
el foco a `…`. Abrir el menú o un panel no exporta, activa proveedores ni ejecuta
extracción. Archive/Restore conserva la guardia Guardar/Descartar/Cancelar cuando
hay cambios pendientes.

## Bloquear una Note

Activá **Lock page** desde `…` para guardar el bloqueo. Aparece el candado
**Locked** en la cabecera y quedan protegidos el título, cuerpo, icono,
propiedades, relaciones y asociaciones externas. Podés seguir leyendo,
seleccionando texto, abriendo vínculos y exportando. Para archivar o restaurar,
desbloqueá primero. Una Note nueva debe guardarse antes de poder bloquearla.

Hacé clic en **Locked** para editar temporalmente: el indicador cambia a
**Re-lock** y el switch Lock page sigue activo. Guardar conserva esa apertura
temporal; pulsar Re-lock vuelve a bloquear. Al cerrar la página o navegar a otra
entidad, la próxima apertura vuelve a Locked. Cambiar entre Side/Center/Full/New
tab conserva el modo de la misma instancia abierta.

Para quitar el bloqueo de forma permanente, apagá **Lock page** en `…`; el
indicador desaparece tras confirmar el cambio. Si hay un borrador pendiente al
bloquear o re-bloquear, elegí Guardar/Descartar/Cancelar. Cancelar conserva el
borrador y el modo actual; un error conserva el estado anterior y muestra el
problema. El bloqueo protege la edición en la interfaz.

Guardar o ⌘S/Ctrl+S persiste mediante Rust/SQLite. Errores, conflictos y
Guardar/Descartar/Cancelar conservan el buffer existente. No hay autoguardado.
Los prefijos importados se recomponen íntegros al editar. Los formatos desconocidos
siguen recuperables como JSON.

El diálogo de salida presenta Cancel, Discard y Save and continue en ese orden,
con iconos a la izquierda y separación sobre los botones. Cancel conserva el foco
inicial; Tab recorre las acciones de izquierda a derecha. El cuerpo de Note no
repite la ayuda de atajos ni un botón Add paragraph al pie.

Desde el 28/10/2026 Task comparte la interacción de escritura en bloques de
Note. Goal y Milestone conservan la presentación anterior de su contenido; todos
comparten el menú de acciones superior. No se incluyen marcas,
tipos de bloque o funciones de colaboración nuevas. Referencia de interacción:
[Notion: writing and editing basics](https://www.notion.com/help/writing-and-editing-basics).

Ver [revisión y límites](verification-note-editor.md).
