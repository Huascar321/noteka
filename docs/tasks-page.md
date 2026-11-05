# Página Tasks

El breadcrumb compartido muestra Home → Tasks y mantiene Home navegable.
Tasks del menú lateral abre una página amplia con icono sobre el título,
navegación Inbox/Today/Week/Month, el botón New y las vistas locales de Home.
Today es la vista inicial y agrupa las tareas pendientes de proyectos
Doing/Ongoing no archivados; All Active Projects agrega Planned. La vista core
`tasks_today` conserva su filtro por vencimiento para CLI/MCP. Las primeras diez
tareas se acumulan con Load more,
sin duplicar IDs; cambiar vista o actualizar reinicia la página. Apertura y
captura pasan por las guardias de buffers existentes y crear no asigna fechas.
El checkbox pendiente es un cuadrado vacío con borde neutro; el completado es un
check verde, compartido con Home y contenidos de Project.

Recurring y Active Content aparecen deshabilitadas como pendientes. No hay
acciones ficticias de colaboración, backlinks o portada de Notion.

## Projects

Debajo aparece Projects, inicialmente Active con grupos Doing/Ongoing y filas
compactas. Planned selecciona planificados; Completed selecciona Done. Board
reutiliza las cinco columnas y el movimiento atómico/CAS existentes. Contexto,
pendientes, vencidas y progreso se calculan a partir de relaciones locales.
Los grupos cubren los proyectos cargados y los originales siguen siendo la
fuente de verdad. Los [Goals](goals.md) referidos conservan sus páginas originales
y sus subtipos tipados abren la sección Goals.

Deadlines muestra un calendario mensual por `end_date` con proyectos de todos
los estados, incluidos Done y sin fecha. Los días permiten preparar un proyecto
Planned con ese plazo; guardar sigue siendo explícito. La proyección aditiva
`projects_calendar` excluye archivo local y de origen antes de paginar; se avisa
si hay más páginas. El calendario separa fechas inválidas y no inventa plazos.
La consulta previa `projects_deadlines` conserva su selección de proyectos no
Done con fecha; los cursores siguen ligados a su consulta. Ver
[verificación](verification-project-deadlines-calendar.md).

## Lectura y teclado

Las dos secciones tienen estado, error/reintento, vista y cursor propios. Se
comparten dos cupos de lectura y se descartan resultados de consultas abandonadas.
⌘R en macOS, Ctrl+R en Windows/Linux o F5 refrescan la página Tasks activa sin
recargar el WebView; diálogos y bloqueos impiden ejecutar el refresco. Se conserva
la vista elegida. Ambos temas y la ventana estrecha mantienen controles alcanzables.

Home conserva sus vistas iniciales anteriores. Este ajuste no modifica el detalle
individual de una tarea: las capturas suministradas muestran la página Tasks.


Los selectores existentes de regreso a Home en los recorridos históricos se
adaptaron al breadcrumb, sin agregar escenarios ni ejecutar esos recorridos.


## Pies y acciones de fila

Las cinco secciones omiten los contadores y fechas de consulta de sus pies;
la cobertura queda en Opciones de vista. El aviso técnico final de Home se retiró.
Load more aparece sólo cuando hay otra página y acumula IDs únicos. Todo el ancho
de esa fila activa el mismo botón. También ocupan todo su contenedor las filas
New page/New task sin borde; las capturas y tarjetas con borde conservan su estilo.
No se guarda un borrador al crearlo. Home usa la misma acumulación para sus cinco
lectores, con límites, cursores y guardias anteriores.


## Contenido del detalle

Abrir una Task muestra su cuerpo enriquecido propio bajo Status/Project/Due.
Enter desde el título permite comenzar, `/` agrega bloques y Guardar/⌘/Ctrl+S
confirma. View details conserva los campos secundarios. Las Tasks sin cuerpo
siguen sin documento persistido hasta editarlo y guardarlo.
[Uso, compatibilidad y límites](task-page-content.md).
