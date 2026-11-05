# Tasks y arte local de la referencia

Tasks usa encabezado de32px, New Task, pestañas con iconos, filas compactas sin
separadores repetidos y grupos por proyecto con icono, título y conteo de página.
La relación y la fecha civil se alinean a la derecha. El área de Home tiene
ancho disponible mayor; en ventana estrecha las filas distribuyen sus metadatos
en otra línea. Las herramientas son actualizar, abrir listado, opciones y crear.

## Vistas

La vista inicial es **Today**: tareas pendientes de proyectos locales
Doing/Ongoing, no archivados, agrupadas por proyecto. La tab **All Active
Projects** también admite Planned. Estas definiciones locales no acreditan
paridad con todos los filtros internos de Notion. El modelo actual de tarea
conserva todo/doing/done; `doing` cuenta como pendiente y `done` como completada.

Week usa la semana ISO de vencimiento; Month su año/mes civil. Scheduled tiene
vencimiento asignado y No Due carece de él. No se derivan estas vistas de
planned_date ni se asignan fechas al crear. Inbox, All y Done mantienen sus
reglas anteriores. El núcleo conserva `tasks_today` con vencimiento hasta el día
civil para CLI/MCP; la tab visual Today ya no consulta esa vista.

Recurring y Active Content siguen pendientes. Cobertura, contexto de consulta y
reglas se consultan en Opciones de vista y en el footer; los conteos de grupo son
apariciones de esta página. No hay un total global inventado entre grupos.

## Imágenes originales

Los iconos personalizados se guardan como adjuntos del workspace. El campo
`payload.icon` puede contener `image-` más el UUID local sin guiones (38 bytes).
La interfaz sólo interpreta ese patrón completo; las claves lógicas existentes
mantienen su comportamiento. Un recurso ausente/incompatible usa un icono
alternativo. Se admiten PNG/JPEG/GIF/WebP de hasta1MiB para presentación; no SVG
ni HTML ejecutables. La lectura conserva el límite nativo de attachment.read;
la cache de imágenes tiene máximo32 entradas y2 lecturas físicas simultáneas.

El arte se comparte entre Home, contexto de tareas, relaciones, detalle,
preview de búsqueda y pestañas. Elegir color conserva una imagen existente.
No se usan URLs remotas ni reglas basadas en nombres de proyectos en runtime.

## Recuperación desde la fuente autorizada

La entrega del23/10 recuperó cuatro imágenes para cinco proyectos. Antes se
conservó un backup canónico. El importador previo no había creado relaciones:
se añadieron14 vínculos tarea/proyecto explícitos observados en la referencia,
con coincidencia única de título/fecha y revisión esperada. Un título duplicado
se omitió inicialmente y luego se resolvió por ID original, propiedades y
correspondencia completa de las578 filas del manifiesto de importación. No se modificaron los documentos, estados ni fechas civiles.
Los1802 IDs se conservaron; el digest de documentos antes/después es idéntico.

Los manifiestos, correspondencias, backups y recursos personales viven fuera
de Git. Los importadores auxiliares son dry-run por defecto:

```sh
python3 scripts/import-notion-icons.py --workspace RUTA --manifest MANIFIESTO --report INFORME
python3 scripts/import-notion-task-links.py --workspace RUTA --manifest MANIFIESTO --report INFORME
```

Aplicar exige `--apply --backup DESTINO_NUEVO`. No se eliminan adjuntos ni se
adivinan duplicados. Para revertir se usan los informes y operaciones con
revisión vigente; restaurar un backup completo reemplazaría cambios posteriores
y requiere elegir explícitamente ese alcance.

Ver [ADR0036](decisions/0036-local-image-icons-and-task-projections.md) y
[revisión de esta entrega](verification-task-reference.md).


El ajuste posterior del 24/10 incorpora la página Tasks del menú y retira los
contadores de pie: la cobertura se consulta en Opciones de vista. Load more
acumula filas y las acciones textuales ocupan el ancho completo. Ver
[página Tasks](tasks-page.md) y [revisión actual](verification-tasks-page.md).


## Detalle ampliado al 24/10/2026

El ajuste posterior de [contenido Task](task-page-content.md) añade documento
opcional en la misma identidad y entrada visible al editor, con propiedades
principales próximas a la referencia de Notion. No implica subtareas, comentarios,
recurrencia ni seguimiento de tiempo. [Revisión](verification-task-page-content.md).
