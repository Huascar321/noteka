# ADR 0035: paneles montados por pestaña y buscador contextual compartido

- Estado: adoptado
- Fecha: 23/10/2026

## Contexto

El menú de escritorio debe reducirse a seis rutas; el resto sigue siendo
accesible. Search y `+` comparten un diálogo y los originales deben poder
abrirse en pestañas independientes sin perder edición, conflictos ni undo.
La instancia única de App anterior mezclaba navegación y estado del editor.

## Decisión

App conserva workspace, tema, tira de pestañas, buscador y una sola autoridad
para `pending_edits` y cierre del host. `WorkspacePane` conserva la lógica previa
del editor y colecciones por instancia. Se monta al elegir un destino y se
oculta con `hidden`/`inert` al desactivar; no se serializa ni reconstruye el
editor al alternar. Un catálogo de rutas completo se separa de las seis entradas
visibles. El buscador sólo usa `entity.list`, `search.query` y `entity.get`.

Cada panel informa título, icono, edición, operaciones protegidas y guardia.
La confirmación de salida devuelve un resultado; la shell recorre todas las
pestañas antes del cierre de ventana/workspace. Las escrituras de protección
nativa se serializan para que una respuesta vieja no desactive una posterior.
Cancelar o un guardado fallido impide el cierre y conserva el panel pendiente.

La búsqueda invalida generaciones de resultados y preview por separado. Un
cupo compartido de dos lecturas mantiene ocupados los IPC realmente iniciados
hasta que terminen; descartar un resultado no finge cancelar el trabajo nativo.
Los controles no ejecutan HTML ni activan modelos o proveedores.

## Consecuencias

- Editor, undo, selección, revisión y scroll permanecen en su instancia.
- Cada pestaña abierta consume memoria; se crean al elegir un resultado y se
  liberan al cerrar. No hay promesa de pestañas ilimitadas ni recuperación de
  borradores después de terminar la sesión.
- Los paneles inactivos no reciben atajos ni polling. Lecturas iniciadas pueden
  terminar en su propia instancia; al cerrarla, sus resultados no se presentan
  en otra pestaña.
- Una sola preferencia de portada local y tema no altera SQLite. La portada
  procedural se suspende por visibilidad y movimiento reducido; no usa videos.
- No cambian migraciones, contratos, reglas Rust ni persistencia canónica.
- Separadores sutiles y bordes de campos usan tokens distintos; los campos
  conservan contraste de borde de 3:1 contra el control en ambos temas.

## Alternativas consideradas

Serializar y desmontar el panel inactivo reduciría memoria, pero requeriría
reconstruir selección, undo, formatos desconocidos y buffers inválidos. Un solo
editor compartido trasladaría esos riesgos a cada cambio de pestaña. Se eligen
instancias montadas mientras la sesión permanezca abierta.

## Ampliaciones de presentación solicitadas durante la implementación

El listado de una sección se monta sólo en modo de exploración. Abrir entidad,
resultado o captura cambia explícitamente al modo detalle antes de leer, para
que una carga lenta no muestre un listado encima del documento. No se ofrece
una segunda columna automática ni se infiere el modo de vista de un buffer nulo.

El detalle comparte ritmo de documento y controles auxiliares desplegables.
El prefijo exacto de propiedades del importador se proyecta en el encabezado y se
recompone sin cambios al guardar el cuerpo. Se conserva una separación explícita
entre metadatos de origen y propiedades editables del núcleo. El reconocimiento
conservador falla sin agrupar contenido ambiguo. Un NodeView de tablas Markdown
mantiene el bloque de código original y acceso a su texto, sin ampliar el
formato persistido. Ambos caminos se verifican con corpus sintético y lectura
real de SQLite, incluidas revisiones e IDs; no se usan páginas personales como
fixtures.
