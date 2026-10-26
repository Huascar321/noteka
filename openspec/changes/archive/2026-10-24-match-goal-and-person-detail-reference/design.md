# Design

## Context

Las dos referencias se buscaron por títulos y se verificaron contra source.record_id local. Goal no tiene portada: Status/Tag/Progress, Nav, overview, tabla de hitos y proyectos. Person tiene foto como portada, propiedades en View details y vistas de relaciones. Notion incluye campos personales y contenido libre que no forman parte de Person local; no se copian. El usuario aclaró conservar acciones propias sin extender cuerpo libre core y preservar el documento existente de Goal.

## Goals / Non-Goals

Ver proposal.md. Reutilizar ADR0045/0049 y operaciones actuales. No ampliar esquema, importador, reglas de cuerpo, acciones de dominio ni Meeting Notes. Datos personales, referencias y backups permanecen fuera de Git.

## Decisions

1. GoalPageProperties usa payload.goal, tags reales y HomeSummary/goal desde goals_activity limit100, con guardias generación/workspace/ID. Status queda principal; fechas y fuente se mantienen editables/de lectura respectivamente en View details. No derivar progreso de filas parciales. Ausencia/resumen no interpretable/sin hitos son estados distintos.
2. MilestonesPanel conserva lector, orden, cursor, errores, opciones, actualización, apertura y creación; sólo el detalle Goal usa tabla con ambas fechas. El listado de la biblioteca se conserva. Proyectos relacionados abren sus originales y el editor completo de relaciones permanece accesible.
3. El BlockEditor de Goal conserva exactamente su configuración, documento, prefijo y recomposición. No se ocultan ni eliminan bloques de plantilla para hacerlos pasar por controles vivos. Cambia sólo composición alrededor del editor; Person no recibe editor porque su modelo no admite documentos.
4. Person usa PersonPhoto existente con foto/posición originales y estilo de portada, con fallback neutro. EntityFields se presenta dentro de View details; las acciones siguen siendo editables. PersonContents separa Notes/Projects/Tasks del workflow/context ya existente; captura sólo Note/Task, con ContextLink. Se conserva orden/cursor/deduplicación y se añaden carga/error/reintento visibles al detalle. No simular Notes heredadas de proyectos ni reuniones de audio.
5. PageTools y sus condiciones no cambian: Goal es Note y conserva tres grupos; Person conserva Recursos externos. Navegación, guardias, revisión CAS y operaciones siguen compartidas. CSS se añade acotado a los nuevos detalles.

## Risks / Trade-offs

- Goal fuera de los primeros100 resultados o excluido de Home → No disponible, sin aumentar cuotas.
- Plantillas importadas del Goal pueden contener etiquetas de vistas; siguen siendo texto del documento original y no consultas nuevas. Preservarlas prevalece sobre paridad visual perfecta.
- Person puede tener menos relaciones locales que Notion; se muestra únicamente el contexto local cargado. No se recuperan Phone/Email ni cuerpos desde la referencia.
- La revisión sobre copia con escrituras bloqueadas verifica borradores y navegación; no demuestra persistencia, proveedores o IPC nativo.

## Migration Plan

Sin cambios persistentes/Rust/contratos/dependencias. Baseline de 53 archivos y backup consistente antes de editar, fuera de Git. Preservar Meeting Notes0/19, trabajo previo y otros proyectos. Revertir sólo presentación conserva los datos originales; no restaurar ciegamente sobre revisiones posteriores.
