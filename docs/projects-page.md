# Página Projects

El registro individual usa ahora el [detalle de Project](project-detail.md), con
propiedades compactas, progreso del núcleo y secciones Tasks/Notes relacionadas.

Projects del menú lateral abre una página amplia con breadcrumb Home → Projects,
icono propio sobre título, Nav, By Tag/By People y New Project. Active es la vista
inicial, con grupos Doing y Ongoing y filas con contexto, pendientes, vencidas y
progreso calculados desde originales locales.

## Vistas

| Vista | Alcance local |
| --- | --- |
| Active | Doing y Ongoing, agrupados por estado. |
| Planned | Proyectos Planned. |
| Board | Cinco columnas y movimiento atómico existente. |
| Deadlines | Lista de proyectos no Done con fecha Fin de proyecto válida, ascendente. |
| Completed | Proyectos Done. |
| On Hold | Proyectos on_hold. |

By Tag y By People cambian a una agrupación local dentro de Projects. Incluyen
los cinco estados y usan vínculos explícitos. El encabezado de cada grupo abre
el tag o la persona original; los proyectos sin relación quedan en Sin tag o
Sin persona. Un proyecto puede aparecer en varios grupos y sigue siendo una
única entidad. Cada fila muestra su estado. Las seis pestañas permiten volver
a una vista por estado; Nav conserva los mismos accesos.

Todas las vistas excluyen archivo local/de origen. La búsqueda general conserva
acceso a proyectos importados archivados sólo en origen. Los iconos son locales;
no se recupera arte de URLs remotas durante la navegación. Los [Goals](goals.md)
referidos conservan sus páginas originales y los tipados abren su sección canónica.

## Captura y lectura

New Project/New preparan Planned. New page en Doing/Ongoing/Done/On Hold/Planned
prepara ese estado. En By Tag/People prepara Planned con el vínculo al grupo;
Sin tag/persona no añade una relación. Las guardias existentes protegen el
borrador y no hay escritura hasta guardar.

Load more y New page textuales responden en todo el ancho de su fila. La captura
con borde conserva su tamaño. El lector carga hasta cien proyectos por página,
acumula IDs únicos y mantiene grupos de los proyectos cargados; la cobertura
está en Opciones de vista. Sólo Board completo permite mover tarjetas, con los
[límites y guardias existentes](project-board-movement.md).

⌘R en macOS, Ctrl+R en Windows/Linux o F5 vuelve a leer la vista elegida sin
recargar la aplicación. Diálogos/bloqueos, composición y repetición no inician
la actualización. Las consultas anteriores se descartan al cambiar vista o
salir; cada instancia mantiene su propio lector y retira el listener al desmontar.

## Proyecciones y alcance

home.query agrega projects_on_hold/projects_by_tag/projects_by_people y el campo
opcional related_entity de los grupos. Rust conserva filtros, orden, identidad,
resúmenes, límites y cursor. Las agrupaciones se construyen antes de truncar el
contexto de una fila. No cambia SQLite, payloads, operaciones ni documentos.
Se conserva la composición de Projects en Home/Tasks y el editor individual.

Deadlines conserva la lista por fecha existente. Las agrupaciones auxiliares usan
listas plegables locales; no implementan arrastre para cambiar relaciones ni
reproducen todas las vistas auxiliares de Notion. No hay sincronización automática.

Ver [ADR0044](decisions/0044-projects-page-and-relation-projections.md) y
[revisión](verification-projects-page.md).
