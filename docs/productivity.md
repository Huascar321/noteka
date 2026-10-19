# Productividad local: reglas y contrato

Fase 2 (`add-productivity-workflows`). El núcleo valida metadatos y consultas; no usa modelos. La verificación de escritorio se registra al completar los recorridos nativos, no por existir esta documentación.

## Metadatos y compatibilidad

- Nota: `inbox` booleano, default true.
- Tarea: `state` todo/done (default todo), `priority` low/normal/high (default normal), `due_date` y `planned_date` opcionales, `inbox` true por defecto.
- Proyecto: los estados previos, `purpose` hasta 16 KiB y `start_date/end_date` opcionales; fin >= inicio.
- Tag: clasificación area/resource/topic requerida. Persona: payload vacío.
- Fechas civiles canónicas YYYY-MM-DD, años 0001–9999, sin hora ni zona. Instantes de creación/modificación siguen UTC. No se convierten fechas inválidas.
- SQLite continúa v2. Abrir/consultar payloads previos no los reescribe; los filtros interpretan defaults ausentes. En `entity.update`, payload es un **patch**: campos omitidos se conservan, null limpia fechas y texto vacío limpia purpose. Completar con `{state:"done"}` no borra planificación/prioridad. Una escritura explícita completa defaults. Campos desconocidos no se borran por apertura ni cambio sólo de título; actualizar un payload incompatible falla conservando todo.

## Consultas

CLI `workflow query --json` recibe `{view,as_of?,related_id?,limit?,cursor?}`. `as_of` es un día civil explícito; si falta, se usa el día actual en zona IANA del workspace. Devuelve `{date,items,next_cursor}`; límite 1–200 (default 50), orden ascendente `(created_at,id)`.

| view | Filtro |
| --- | --- |
| inbox | Notas y tareas activas con inbox true/ausente; tareas done fuera |
| my_day | Tareas activas todo con planned_date igual al día |
| pending | Todas las tareas activas todo |
| overdue | Tareas activas todo con due_date menor al día; vencer hoy no equivale a vencida |
| active_projects | Proyectos activos planned/doing/ongoing; on_hold/done fuera |
| context | Entidades activas relacionadas a proyecto/tag/persona indicado por related_id, por cualquiera de los extremos y sin duplicados |

Clasificar o relacionar no saca una captura del Inbox; desmarcarlo es una decisión explícita. My Day no agrega automáticamente vencidas. Archivar excluye de vistas pero conserva relaciones. Completar una tarea no elimina fechas ni vínculos. Contexto puede seguir mostrando tareas completadas; es un dashboard de relaciones, no otra copia de pendientes. No hay snapshot entre páginas ante escrituras concurrentes.

```sh
printf '%s' '{"view":"my_day","as_of":"2026-10-12"}' | ./target/debug/noteka --workspace /ruta/al/workspace workflow query --json
```

## Captura y relaciones

`entity create` agrega `context?:{target_id,kind:"project"|"tag"|"person"}`. Entidad/documento/relación/historial se confirman juntos con revisión inicial 1. ID ausente o tipo incompatible revierte toda la creación. La captura desde dashboard usa ese input, no dos requests de crear y vincular. Las operaciones posteriores `relation.add/remove` conservan control de revisión del origen.

Los formularios mantienen el buffer ante error/conflicto. Relaciones se cambian sólo con buffer guardado. Quitar una relación entrante verifica la revisión actual de su origen; no elimina entidades. El ID de selección se muestra para compartirlo con CLI y seleccionar destinos locales explícitamente.

Pendientes de producto fuera de esta entrega: subtareas, recurrencias, recordatorios, contactos externos, colecciones/propiedades genéricas y modelos. Preferencias de Inbox/My Day quedan en review queue para revisión no bloqueante.
