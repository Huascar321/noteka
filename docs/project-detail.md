# Detalle de Project

El detalle usa la composición del Project vinculado a la Note de referencia
observada en Notion: portada, icono/título, propiedades compactas, relaciones,
Nav y secciones Tasks/Notes. La portada es un gradiente decorativo local; no
representa una descarga ni un campo persistente de la portada de Notion.

Status y Target Deadline editan estado y fecha de fin locales. View details
conserva inicio, propósito y procedencia. Progress presenta la proyección Rust
con precisión de décimas y tareas completas/total; no copia porcentajes del
origen ni deriva nuevas reglas en la interfaz.

El resumen se busca en la página de hasta cien proyectos de projects_board,
respetando su cuota existente. Un proyecto excluido de esa vista o fuera de esa
página muestra No disponible; un fallo permite reintentar y Sin tareas sólo se
usa cuando el resumen canónico acredita total cero.

## Registros y navegación

La cabecera muestra relaciones de contexto reales. Tasks y Notes se presentan
aparte, mediante workflow.query/context y el cursor compartido anterior. Tasks
se agrupa por sus estados locales To Do/Done; un estado no reconocido permanece
visible como no disponible. Done empieza plegado. Notes muestra sus títulos,
iconos y tipos existentes; Goals/Milestones continúan en relaciones de contexto.

Cargar más relaciones acumula IDs únicos para ambas secciones. Mientras existe
cursor, los mensajes de vacío hablan sólo de las relaciones cargadas. Los
contadores de grupo pertenecen a esa página acumulada; el progreso es el resumen
independiente del núcleo. Los errores no se presentan como ausencia de registros.

New/New task/New note preparan captura vinculada al mismo proyecto sin guardarla.
Exigen resolver cambios del proyecto antes de capturar; las filas abren sus
originales con Guardar/Descartar/Cancelar. Nav abre Projects o Dashboard local.
No se agregan vistas Board/Calendar de Tasks ni un editor de documento de Project.

El Project utiliza el grupo Recursos externos de las [herramientas compartidas](page-tools.md).
Sin cambios Rust, contratos, SQLite, documentos ni datos personales.

Ver [revisión y límites](verification-project-detail-tools.md).
