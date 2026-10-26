# ADR0045: Goals e hitos sobre páginas de identidad estable

Fecha: 24 de octubre de 2026. Estado: adoptada.

## Contexto

El usuario pidió Goals entre Tags y People y la composición de su página Notion.
La importación contiene tres Goals reales, su plantilla y dieciséis Milestones
como páginas note. Cinco relaciones a proyectos ya existían; ocho hitos apuntan
a esos Goals y ocho no tienen Goal. Reemplazar entidades por tipos nuevos rompería
IDs y referencias o exigiría migrar documentos sin necesidad para este alcance.

## Decisión

Goal y Milestone son subtipos de página mediante NoteFields.goal/milestone
opcionales y excluyentes. EntityKind sigue siendo note: SQLite/documentos/IDs y
operaciones CRUD/CAS/lotes existentes son la fuente de verdad. Esta decisión
resuelve el dominio pendiente de ADR0038 conservando identidades, sin una tabla
o tipo persistido nuevos y sin migración SQL. SQLite permanece en versión 20.

Rust valida Active/Dream/Achieved, Dream por defecto, fechas civiles y referencias
Milestone→Goal del mismo workspace. Observa esa dependencia en MutationContext,
rechaza asignaciones nuevas a Goals archivados, conserva las referencias previas
y no permite quitar el subtipo de un Goal aún referenciado. Goal reutiliza
Note→Project/Tag y admite un tag directo. El estado de Goal, su fecha de logro
y finalización de hitos son decisiones separadas, sin transiciones automáticas.

home.query añade seis vistas y goal_id sólo para milestones_all. Proyecciones
sin documentos, snapshot por lectura, caps previos y cursor ligado a contexto.
Actividad/plazo agrupan tres estados; By Tag filtra Active, Achieved filtra
logrados y List de Milestones filtra fecha completada. El detalle de Goal consulta
todos sus hitos. Progreso = completados/total de hitos activos, redondeado a una
décima porcentual; sin hitos es ausencia y metadatos inválidos invalidan el resumen.

La actividad conserva un snapshot acreditado `{at, revisions}`. La recuperación
captura Goal, hitos, proyectos y sus notas/tareas ordinarias con sus revisiones;
una revisión nueva/posterior participa con updated_at local. Así recuperar
metadata no se convierte en una edición ficticia de hoy. El timestamp de origen
se observó con precisión de minuto y zona America/La_Paz. No es sincronización ni
una fórmula universal de actividad. El mapa de revisiones no se expande en filas Home.

La sección lógica goals reconoce payload tipado para breadcrumb y apertura desde
cualquier contexto. UI reutiliza lectores, cola, guardias, filas y originales;
Goals real reemplaza el aviso de Home. El selector de Goal usa títulos/paginación.
La metadata de origen duplicada se presenta en Details, conservando bloques e IDs
del prefijo. No se interpretan como funcionales los bloques remotos desconocidos.

## Alternativas

- Nuevos EntityKind y tablas Goal/Milestone: requerirían migración y continuidad
  de documentos, referencias y clientes para el mismo resultado observable.
- Agrupar notas por título o sólo colección de origen: no distingue plantillas,
  estados editables, identidades ni hitos sin Goal; se requiere subtipo explícito.
- Copiar porcentaje o usar tareas de proyecto: cambiaría la fórmula acreditada.
- Usar updated_at de recuperación como actividad: mostraría fechas artificiales.

## Recuperación y compatibilidad

Helper genérico por IDs originales, fuente privada, backup, CAS e informe por
mutación. Recupera tres subtipos Goal, dieciséis Milestones y tres tags; conserva
cinco relaciones a proyectos y ocho vínculos a hitos. No convierte la plantilla,
sobrescribe una clasificación local ya tipada ni envía documentos en updates.
Los 1.802 IDs y digest de documentos anteriores/posteriores coinciden.

No hay atomicidad global: el informe conserva trabajo parcial y una reversión
debe considerar las revisiones posteriores. Clientes antiguos de payload cerrado
necesitan actualizarse para editar los campos nuevos. Campos/documentos desconocidos
permanecen recuperables. Compilación e inspección manual no sustituyen suites,
aceptación nativa, carreras o mediciones de rendimiento no ejecutadas.
