# Editor contextual de Notes

Las Notes normales se editan directamente sobre una columna de página. Título,
propiedades y relaciones conservan sus operaciones; el cuerpo usa los mismos
documentos v1/v2/v3 y el guardado explícito de Noteka.

## Escribir y dar formato

- Hacé clic en el cuerpo o presioná Enter desde el título.
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

Al salir del bloque se ocultan los controles, aunque el editor siga enfocado.
Escribir o entrar desde el título no los activa. El espacio vacío debajo del
párrafo tampoco cuenta como hover. El foco explícito por teclado y el arrastre
conservan el acceso a las acciones.

Mover conserva IDs y marcas. Duplicar genera nuevas identidades para el bloque y
sus hijos. Undo/redo siguen disponibles con ⌘Z/Ctrl+Z y su combinación de rehacer.
Alt+Shift+B abre acciones del bloque desde el teclado; Alt+Shift+F lleva a las
herramientas de una selección de texto.

## Herramientas de página y guardado

Export, External resources y Attachments están en la barra superior junto al
estado de guardado. Sus paneles se abren desde los iconos y se ajustan al viewport;
Escape devuelve el foco al disparador. Cerrar conserva campos y resultados.
Abrir un panel no exporta, activa proveedores ni ejecuta extracción.

Guardar o ⌘S/Ctrl+S persiste mediante Rust/SQLite. Errores, conflictos y
Guardar/Descartar/Cancelar conservan el buffer existente. No hay autoguardado.
Los prefijos importados se recomponen íntegros al editar. Los formatos desconocidos
siguen recuperables como JSON.

El diálogo de salida presenta Cancel, Discard y Save and continue en ese orden,
con iconos a la izquierda y separación sobre los botones. Cancel conserva el foco
inicial; Tab recorre las acciones de izquierda a derecha. El cuerpo de Note no
repite la ayuda de atajos ni un botón Add paragraph al pie.

Task, Goal y Milestone conservan su presentación anterior. No se incluyen marcas,
tipos de bloque o funciones de colaboración nuevas. Referencia de interacción:
[Notion: writing and editing basics](https://www.notion.com/help/writing-and-editing-basics).

Ver [revisión y límites](verification-note-editor.md).
