# Navegación y pestañas de escritorio

## Arranque y workspace

Al iniciar, la aplicación abre sin pedir nada el último workspace usado. Si no
hay preferencia o esa ruta ya no contiene un workspace, intenta el directorio de
datos predeterminado de Noteka. La ruta se guarda en la preferencia local
`noteka.workspace-path.v1` cada vez que se crea o abre un workspace con éxito.

La pantalla de selección aparece sólo si no existe ningún workspace, si la
apertura falla —con el error visible y sin presentar datos de otro workspace— o
si se elige **Change workspace** en Settings. El primer arranque no crea datos
por su cuenta: crear un workspace sigue siendo una acción explícita.

El lateral muestra, en orden, **Home, Tasks, Notes, Projects, Tags, Goals y People**.
El botón de cuadrícula junto a Search abre Collections. Las vistas Inbox,
My Day, Pending, Overdue, Active Projects, Areas, Resources y Archive se abren
buscándolas por nombre; mantienen las mismas reglas del núcleo.

## Buscar y abrir

- **Search** o **⌘/Ctrl K** abre el buscador contextual en la pestaña actual.
- **+** o **⌘/Ctrl T** abre el mismo diálogo para una pestaña nueva. Cancelar
  no crea una pestaña vacía.
- Flechas recorren resultados; Enter abre. **⌘/Ctrl Enter** abre en otra pestaña.
- Escape cierra el diálogo y devuelve el foco al control que lo abrió.
- **Advanced search** conserva notas/entidades, adjuntos y consulta agrupada.

El diálogo lee entidades y el índice textual local. Admite tipo, sólo archivadas
y contexto local; la consulta de contexto requiere texto. La previsualización
es texto escapado, sin ejecutar HTML ni escribir originales o índices.
No ejecuta embeddings, LLM, extracción o reparación automática.

## Pestañas y edición

Las secciones muestran sus listados en el área principal. Al abrir un elemento
o crear un borrador, el editor ocupa el ancho disponible. Los listados sólo aparecen
al entrar a una sección desde el menú o elegir esa vista en el buscador.
El detalle no monta una columna de registros ni un listado encima del contenido.

Tasks, [Notes](notes-page.md), [Projects](projects-page.md), [Tags](tags-page.md) y [Goals](goals.md) usan páginas compuestas con las vistas del núcleo.
Notes abre su biblioteca con ocho pestañas y Tags/Recent debajo; All mantiene
acceso a todas las notas activas. Los registros siguen abriendo su detalle original.
Tags lateral inicia Recent y añade Table con Tag/Last Note/Type; Nav abre
Tasks/Notes/Projects y la última nota conserva su ruta canónica en Notes.
Goals inicia By Activity y presenta estados, progreso por hitos y Milestones.
People lateral inicia Contacts con galería, fotos originales y siete vistas;
Nav abre Contacts/Pipeline/Cal, siendo Cal el calendario local de cumpleaños.
New Person, lectura acumulativa y actualización activa conservan las guardias.
Goals/Milestones conservan sus páginas note y abren Home → Goals → título desde
cualquier contexto; crear en un grupo prepara su estado y mantiene las guardias.

Se puede alternar con clic, **Ctrl Tab / Ctrl Shift Tab**, o flechas cuando el
foco está en la tira de pestañas. **⌘/Ctrl W** cierra la pestaña actual.
Cada pestaña mantiene editor, historial de deshacer, borrador, revisión y scroll.
Cambiar de pestaña no guarda ni descarta. El contenido inactivo queda oculto e
inerte; sólo el panel activo recibe atajos y actualización por activación.

Cerrar una pestaña editada ofrece guardar, descartar o cancelar. El cierre de
ventana y cambio de workspace comprueban todas las pestañas. Un error de guardado
o conflicto conserva el borrador. Cerrar la última pestaña abre Home.
Las pestañas y los borradores siguen siendo de sesión: no se recuperan después
de cerrar o reiniciar la aplicación. SQLite conserva los guardados confirmados.

## Portada y movimiento

ASCII es la portada inicial. El control ASCII/Foto aparece al pasar el puntero
o enfocar la portada; en pantallas táctiles está visible. Foto usa una imagen
local incluida con la aplicación. La preferencia sólo guarda `ascii` o `photo`
en almacenamiento de presentación, separado del workspace.

La escena combina tiburones y peces compactos/alargados con mallas y nado propios,
vista cenital, celdas sombreadas, letras contrastantes y grilla/partículas tenues.
Canvas tiene DPR máximo 2 y se programa hasta 120 cuadros/s, sujeto a pantalla
y presupuesto de render. Ver [océano ASCII](ascii-ocean-cover.md) y sus
[mediciones](verification-ascii-ocean-cover.md).
Se suspende fuera de la vista, con la pestaña inactiva o el documento oculto;
movimiento reducido muestra una composición estática. Las transiciones de interfaz
usan 120–180 ms y respetan la misma preferencia, sin demorar operaciones.

La imagen decorativa se recuperó del Home de Notion por pedido del usuario.
No se incluyen URLs firmadas, identificadores de cuenta, páginas ni retratos
personales. Los videos se usaron como referencia y no forman parte del producto.

Ver [decisión de estado](decisions/0035-session-tabs-and-context-search.md) y
[evidencia](verification-desktop-navigation.md).

## Detalles de entidades e importaciones

Notas, tareas, proyectos, tags y personas usan la misma composición de página:
ícono sobre el título, propiedades compactas, relaciones reales y contenido
centrado. View details muestra revisión/procedencia; los [grupos de herramientas](page-tools.md)
abren paneles de exportación, recursos o adjuntos según el contexto. Formato y bloques aparece al acercar el
puntero o enfocar el editor, y conserva acceso por teclado y pantalla táctil.

Las propiedades que el importador guardó como encabezado y lista se presentan
como valores de origen de **sólo lectura**. No se convierten en propiedades
editables del núcleo. Al editar el cuerpo se recomponen exactamente sus bloques
e IDs originales, incluidos párrafos duplicados inmediatos reconocidos. Una
estructura ambigua permanece como contenido. No se cambia el importador ni se
reescriben documentos al abrirlos.

Una tabla válida dentro de un bloque Markdown importado tiene una vista tabular
con texto escapado. Editar tabla como Markdown abre su texto original; el bloque
sigue siendo `code_block`, con el mismo ID. No implica un nuevo formato persistido
ni un editor de tablas equivalente al de Notion.

El ajuste posterior de [Tasks y arte local](tasks-reference.md) incorpora iconos
originales de proyecto, grupos más cercanos a la referencia y vistas Rust
adicionales, con sus propios límites de revisión.

## Actualizar Home

Home omite el desplegable Navegación y el botón Actualizar Home. Con Home activo,
**⌘R** en macOS, **Ctrl+R** en Windows/Linux o **F5** actualizan sus secciones
conservando las vistas. Los diálogos/bloqueos impiden la consulta y el atajo no
recarga el WebView ni descarta borradores de otras pestañas.
[Revisión del cambio](verification-home-refresh.md).


## Breadcrumb de páginas y detalles

Las páginas del menú muestran Home → sección; Home muestra su propio tramo.
Los detalles muestran Home → sección canónica del tipo → título actual. Por
ejemplo, Home → Projects → Proyecto de ejemplo. No existe el nivel
Databases en esta jerarquía. Los iconos reutilizan el catálogo y el arte local de
cada registro; el título del borrador refleja su edición actual.

Home y la sección son botones que pasan por las guardias existentes de
Guardar/Descartar/Cancelar. El tramo actual se identifica con aria-current y
conserva su texto completo aunque visualmente se trunque en un ancho estrecho.
El breadcrumb reemplaza Volver a Home y el rótulo plano del detalle; no modifica
contenido, propiedades ni relaciones. Tasks comparte ese componente con las
otras secciones. [Composición Tasks](tasks-page.md).


## Barra superior y lateral plegable

El control del lateral, las pestañas y `+` comparten una fila sobre la shell.
En Mac esa fila comparte el espacio de la barra nativa y conserva sus tres
botones; Windows dispone controles de ventana a la derecha, con ejecución real
pendiente. Web omite los controles nativos. Plegar el lateral en escritorio no
reconstruye paneles ni altera borradores. Hasta 760 px se usa un panel superpuesto;
Escape lo cierra y devuelve el foco al control.
La decoración requiere un nuevo inicio del host; recargar sólo el frontend no
la cambia. [Guía y límites](window-titlebar.md).
