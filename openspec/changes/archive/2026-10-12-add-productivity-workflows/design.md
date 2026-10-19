# Design

## Context

Fases 0/1 verificadas: operaciones compartidas, SQLite v2, documentos v1/v2, conflictos y originales. El payload actual usa estado/clasificación; la UI sólo edita notas. Se amplían metadatos y consultas, no se duplica el dominio.

## Goals / Non-Goals

**Goals:** captura/productividad utilizable offline, fechas civiles inequívocas, relaciones y dashboards por IDs.

**Non-Goals:** recurrencias, subtareas, notificaciones, calendarios externos, entidades duplicadas PARA, IA, colecciones genéricas.

## Decisions

- Payloads tipados y estrictos en Rust: nota inbox default true; tarea state todo/done, priority low/normal/high default normal, due_date/planned_date opcionales e inbox true; proyecto state existente, purpose vacío y start_date/end_date opcionales; tag classification requerida; persona vacío. Fechas canónicas YYYY-MM-DD, año 0001–9999; no instantes ni coerciones ocultas. Proyecto end_date no precede start_date.
- No migración SQLite: JSON anterior se conserva en lectura y consultas aplican defaults a campos ausentes. Payload en update es un patch: conserva campos omitidos; null limpia fechas y texto vacío limpia propósito. La validación se aplica al objeto fusionado dentro de la transacción. Una actualización explícita completa defaults. Campos desconocidos no se eliminan al abrir ni en cambios de título; una modificación incompatible se rechaza.
- Inbox: notas/tareas activas con inbox true o ausente; completadas no aparecen. Clasificar/relacionar no desmarca inbox implícitamente. My Day: tarea activa/todo con planned_date igual al día civil; no mete automáticamente vencidas. Pending: todo activo; overdue: due_date menor al día; active_projects: planned/doing/ongoing. Queries devuelven el día usado; día por defecto calculado en zona IANA del workspace. as_of permite pruebas o navegación civil explícita.
- `workflow.query` pagina con `(created_at,id)` y limit 1–200; contexto usa ambos extremos de relaciones y EXISTS para deduplicar. Sólo soporta proyecto/tag/persona como contexto. Sin snapshot entre páginas con escrituras concurrentes, igual que listados base.
- `entity.create` agrega context opcional {target_id,kind}; valida tipos dentro del lock y crea relación junto a entidad/documento/historial. Una sola revisión inicial; no transacción anidada ni crear-y-enlazar en dos requests. `relation.add/remove` conserva revisiones del origen para posteriores cambios.
- UI generaliza el buffer de entidad y reutiliza guardado/protección. Formulario no descarta propiedades desconocidas. Selección de relación explícita por tipo/ID local, lista paginada y acción quitar con revisión del origen (guardado antes de mutar relaciones). Dashboards abren IDs originales y capturan nota/tarea mediante context. No se guardan filtros de cada dashboard: vistas de colección persistentes se incorporan en 3a.

## Risks / Trade-offs

Inbox no infiere organización: requiere salida explícita. My Day no es un calendario ni hace selección por IA. Formularios antiguos muestran defaults sin confirmar hasta guardar. Campos de un formato futuro se mantienen fuera de controles y el núcleo rechaza un reemplazo incompatible. Review queue conserva decisiones de UX; no se bloquea el desarrollo por preferencias visuales.
