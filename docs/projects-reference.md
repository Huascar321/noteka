# Projects en Home

La sección abre **Board**, con Active/Planned/Board y cinco columnas compactas:
Planned, Doing, Ongoing, Done y On Hold. Active reúne Doing/Ongoing; Planned
muestra los planificados. El tablero se desplaza horizontalmente dentro de su
sección en ventanas estrechas.

New Project y New preparan un proyecto Planned. New page en una columna prepara
su estado correspondiente. La captura mantiene la protección de buffers y no
guarda hasta la confirmación habitual. El encabezado abre la página Projects;
las tarjetas y sus referencias abren registros locales.

## Avance y contexto

Rust calcula el resumen con las tareas locales relacionadas únicas y no
archivadas. Avance = completas/total, redondeado a una décima; una décima cero se
presenta sin decimal. Active cuenta pendientes y Overdue cuenta pendientes con
vencimiento anterior al día civil del workspace. Un proyecto sin tareas muestra
Sin tareas; metadata inválida muestra resumen no disponible.

Las referencias muestran tags y páginas de Goals acreditadas. [Goals](goals.md)
añade subtipos/estados/progreso sobre esas mismas notas originales, conservando
identidad y documentos; los Goals tipados abren su sección canónica.
Los iconos son adjuntos originales locales; Home no descarga recursos remotos.

## Procedencia y recuperación

ProjectFields admite `source` y `board_order` opcionales. Rust valida procedencia,
timestamps y orden 0–10000. Los payloads anteriores siguen siendo válidos y no
hay migración SQLite. El archivo de origen excluye el proyecto de Home Projects
y de pertenencia a proyectos activos de Home Tasks; la entidad local sigue
accesible en listados generales y no se archiva automáticamente.

La recuperación del 23/10/2026 vinculó los 578 registros Tasks y 20 Projects
tipados con sus IDs originales. Se verificó toda la secuencia CSV/manifiesto y
Created/Edited/Status del bloque inicial de propiedades; los títulos largos se
corroboraron por prefijo y propiedades, con coincidencia única. No se leyeron
cuerpos. No se escoge una entidad por posición aislada ni por su icono.

Se conservaron 514 vínculos Task→Project acreditados: 14 ya existían y 500 se
añadieron. Se añadieron 17 vínculos Project→Tag, cinco GoalNote→Project y 12
asignaciones de icono observadas. Siete referencias del origen no tienen una
tarea disponible en el export y se omitieron con informe; no se crean tareas
parciales para completarlas. Los doce porcentajes visibles coinciden con la
referencia en el snapshot revisado, calculados desde las tareas locales.

El helper `scripts/recover-notion-project-metadata.py` hace dry-run por defecto.
Aplicar exige un backup nuevo; las escrituras usan CAS y un informe incremental
privado con payload/revisión anteriores. Los 1.802 IDs y todos los documentos
permanecieron intactos. Las fuentes, arte, informes y backups quedan fuera de
Git. No es una transacción global ni una sincronización continua: una reversión
debe considerar las revisiones actuales y los cambios posteriores.

Un cliente anterior con validación cerrada de ProjectFields debe actualizarse
antes de editar proyectos enriquecidos. El detalle conserva la procedencia de
lectura y el orden aunque no represente un control para cambiarlo.

Ver [ADR0038](decisions/0038-project-board-and-source-relations.md) y
[revisión acotada](verification-projects-reference.md).

La entrada lateral usa ahora la [página Projects](projects-page.md), con Active
inicial, seis pestañas y agrupaciones por relación. Home conserva Board inicial
y sus tres vistas; los cálculos de progreso y contexto se comparten.

El [detalle de Project](project-detail.md) toma su composición de una captura
real autorizada de Notion y presenta propiedades y registros locales. Sus
herramientas usan [paneles por grupo](page-tools.md), sin operaciones al abrirlos.
