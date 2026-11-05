# Goals y Milestones

El [detalle de Goal](goal-person-detail.md) usa Status/Tag/Progress, fechas en
View details y tabla de hitos, conservando el editor y las operaciones actuales.

Goals aparece entre Tags y People y también en búsqueda. La página presenta
trofeo propio sobre el título, breadcrumb Home → Goals, Nav a Tasks/Notes/Projects/
Tags, cuatro vistas y grupos Active/Dream/Achieved. Home muestra la misma sección
con datos locales. New/New page preparan una captura; no guardan automáticamente.

## Vistas

| Vista | Regla |
| --- | --- |
| By Activity | Goals sin archivo local/de origen, por actividad descendente; grupos por estado. |
| By Deadline | Los mismos Goals, por Target Deadline ascendente, sin plazo al final; muestra Latest Activity. |
| By Tag | Goals Active por su tag, en columnas de estado. Los otros estados permanecen vacíos por el filtro. |
| Achieved | Estado Achieved, por fecha de logro descendente. |
| Milestones → List | Hitos con Date Completed, por fecha completada descendente. |

Los grupos y conteos cubren la página cargada. Load more acumula identidades con
el cursor; al actualizar se conserva la vista y vuelve la página inicial. Las
acciones textuales ocupan toda su fila y las tablas/columnas usan scroll interno
en ventanas estrechas. ⌘R/Ctrl+R/F5 actualiza sólo la página activa, con las guardias
de diálogos y edición existentes, sin recargar el WebView.

## Datos y edición

Goal y Milestone son páginas `note` tipadas, con sus IDs y documentos anteriores.
No se crean copias de las páginas importadas. Goal admite Active/Dream/Achieved,
Dream por defecto, Goal Set, Target Deadline y Achieved opcionales. Un Goal admite
un tag directo y proyectos mediante relaciones existentes. El detalle muestra sus
hitos, permite prepararlos con ese vínculo y conserva relaciones a originales.

Milestone admite un Goal opcional, plazo y Date Completed. El selector de Goal
usa títulos, continuación y actualización de opciones; la referencia actual se
conserva aunque quede fuera de la página de opciones. Rust comprueba tipos,
workspace, fechas y revisiones. No se asigna un hito nuevo a un Goal archivado
ni se quita el tipo de un Goal que aún tenga hitos vinculados.

Progreso = hitos activos completados/total, redondeado a una décima porcentual.
Sin hitos se muestra **Sin hitos**, no cero inventado. Un hito no interpretable
invalida el resumen afectado. Completar hitos no cambia automáticamente estado o
fecha de logro del Goal. Un Goal archivado conserva documentos y vínculos.

Note Library excluye los subtipos nuevos; Notes → All y búsqueda conservan acceso
a sus páginas. Desde un proyecto o hito, un Goal tipado abre su ruta canónica
Home → Goals → título. Metadata duplicada del export se conserva en View details;
el contenido original y los bloques no interpretados siguen disponibles.

## Actividad y fuente

La actividad de origen se conserva con revisiones de Goal, hitos, proyectos y sus
notas/tareas. Una edición posterior o fuente nueva aporta su updated_at local;
recuperar metadata no adelanta actividad a hoy. El snapshot observado tiene
precisión de minuto y zona explícita. Las fórmulas remotas pueden diferir tras
cambios no importados; no hay sincronización automática ni paridad universal.

La recuperación del 24/10/2026 acreditó tres Goals reales, dieciséis Milestones,
ocho hitos vinculados, cinco relaciones a proyectos y tres tags. La plantilla
Goal conserva su condición de página original. Backup/CAS/informes y fuentes
quedan fuera de Git; los 1.802 IDs y documentos se conservaron. Los tres Goals
estaban Active con 0%; ningún hito tenía Date Completed. Por eso List estaba vacío.

## Contrato y límites

Operaciones CRUD/CAS existentes y home.query compartidos por CLI/MCP/escritorio.
Vistas `goals_activity`, `goals_deadline`, `goals_by_tag`, `goals_achieved`,
`milestones_completed`, `milestones_all`; `goal_id` sólo en la última. Páginas 1–100,
10.000 candidatos/fuentes, 50.000 vínculos y 200 grupos, sin expandir documentos.
Cursores ligados a workspace, vista, día, límite y Goal; no se mezclan contextos.

Campos opcionales nuevos, sin migración SQL ni nueva operación pública. Clientes
de payload cerrado deben actualizarse antes de editar los enriquecidos. Recuperar
no es atómico globalmente ni una restauración automática sobre cambios posteriores.
Ver [ADR0045](decisions/0045-goals-as-typed-pages.md) y
[revisión acotada](verification-goals.md).

## Overview, Journal y proyectos

El detalle ofrece Goal Overview y Additional Details editables, Journal desplegable con New entry fechada y una tabla de proyectos relacionados. Los cambios se guardan con el borrador del Goal; abrirlo no modifica datos. Los bloques anteriores e importados continúan en Previous content y no se convierten automáticamente. Ver [detalle](goal-person-detail.md), [ADR0056](decisions/0056-goal-document-sections.md) y [verificación](verification-goal-detail-enhancement.md).
