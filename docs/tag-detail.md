# Detalle de un Tag

El detalle sigue la composición del Tag vinculado a la misma Note y Project de
referencia observados en Notion: portada, icono/título, propiedades compactas,
relaciones, Nav y Notes/Clips. La portada es decorativa local y el icono procede
del original administrado; no se descarga una portada de Notion.

Type permite Area, Resource, Entity y Topic, conservando su significado. Favorite
es la marca local editable. View details conserva actividad/procedencia de lectura
y revisión. Las modificaciones preparan un borrador y se guardan explícitamente
con Guardar o ⌘S/Ctrl+S; navegar mantiene Guardar/Descartar/Cancelar.

Note Count usa el resumen Rust de tags_alphabetical, en una página de hasta cien
tags con la cuota existente. Un Tag fuera de esa página o excluido de la consulta
muestra No disponible; un fallo permite reintentar. El botón lleva el foco a Notes,
sin guardar ni abandonar el borrador. No se copia un contador de la fuente.

## Contenidos y navegación

Nav abre Notes o Tags locales. Notes y Clips presentan los originales de
workflow.query/context, con List y su orden existente. Clips sólo separa las
notas cuyo tipo local es web_clip; no descarga páginas ni extrae texto.
No se copian pestañas de Notion que necesitan otras consultas de dominio.

Las tareas relacionadas y su captura siguen disponibles en Tasks, desplegable
cuando está vacío. Proyectos, personas, tags asociados y Goals/Milestones tipados
conservan sus vínculos de cabecera y el editor completo de relaciones. No se
inventan People, Goals, Parent Tag o Sub-Tags por aparecer en la referencia.

Las secciones comparten el cursor anterior y acumulan IDs únicos. Con cursor,
los vacíos describen sólo las relaciones cargadas. El contador sigue las reglas
de Home sobre archivo; las filas conservan el contexto anterior, y las páginas
Goals/Milestones se presentan como contexto en vez de repetirse en Notes.

New note/New clip/Capturar tarea acá preparan una entidad vinculada al Tag,
sin crearla hasta guardar. Clip preselecciona Web Clip; las otras capturas mantienen
sus defaults. Los cambios pendientes del Tag bloquean la captura y las filas
abren sus originales mediante las guardias compartidas.

El grupo [Recursos externos](page-tools.md) conserva panel, condiciones y teclado;
Tag no incorpora Adjuntos de Notes ni exportación sin documento guardado.
Claro/oscuro y ventanas estrechas conservan el mismo flujo.

Sin cambios Rust, contratos, SQLite, documentos o datos personales. Ver
[revisión y límites](verification-tag-detail.md).
