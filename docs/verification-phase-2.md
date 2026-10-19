# Verificación de fase 2: productividad

Fecha: 12 de octubre de 2026. Cambio `add-productivity-workflows`.

## Evidencia

- `cargo test --offline -p noteka-core -p noteka-cli`: **27 tests** aprobados; cuatro tests de productividad SQLite y un proceso CLI nuevo, más regresiones. Incluye actualización parcial de estado sin borrar fechas, y payload futuro preservado/rechazado ante modificación incompatible.
- `npm run test:editor`: 5 tests aprobados; el editor sigue independiente de los formularios.
- `npm run check` / `npm run build`: sin diagnósticos y contratos frescos.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: aprobado.
- `npm run test:native`: **64 checks** aprobados en tres sesiones sin red; incluye fallo y reintento de tarea y cierre cancelado con buffer pendiente. Resultados: `/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-native-7Iy5aJ/results.json` (fuera de Git).
- `npm run tauri build -- --no-bundle`: release Apple Silicon aprobado, sin firma/notarización/publicación.
- `openspec validate --all --strict`: 7 items aprobados (cambio y seis specs anteriores).

## Trazabilidad

| Escenario | Evidencia |
| --- | --- |
| Planificar/completar | `task_dates_priority_completion_and_atomic_rejections`; CLI parity; UI crea/planifica/completa con mismo ID/fechas |
| Fecha/prioridad inválida | Fechas imposibles, no canónicas, instantes, año cero, enum/tipo inválido; revisión/historial intactos |
| Día civil/vencimiento | Query con as_of y Pacific/Kiritimati; runner My Day/Overdue y día/zona explícitos |
| Inbox/archivo | `workflows_inbox_day_pending_overdue_and_archive_are_explicit`; UI desmarca Inbox sin borrar nota/vínculos |
| Crear desde proyecto | Transacción contextual núcleo/proceso CLI; UI captura nota+tarea, dashboard abre IDs compartidos |
| Contexto inválido | Workspace ajeno y relación incompatible no dejan ninguna creación; paginación/deduplicación por EXISTS |
| Productividad offline | Runner Tauri/IPC/CLI: cinco tipos, área→recurso sin duplicación, dashboards proyecto/persona/tag, asignar/quitar relaciones |
| Conflicto de tarea | CLI confirma otra revisión; UI conserva prioridad pendiente, devuelve conflicto y recarga sólo tras descarte explícito |

## Coherencia y límites

Sin esquema SQLite nuevo ni coerciones destructivas; validación y filtros en Rust. No hay subtareas, recurrencias, notificaciones, colecciones genéricas, calendarios externos ni IA. Seleccionar destinos de relación usa IDs explícitos; no se promete un picker de búsqueda todavía. Ver reglas adoptadas en [ADR 0003](decisions/0003-civil-productivity.md) y [productividad](productivity.md).

Verificados 5/5 tareas, 4/4 requisitos y 8/8 escenarios. Diff/archivos nuevos revisados y checks de whitespace aplicados, sin cambios de runtime/datos personales en Git. Sin hallazgos críticos pendientes. Las revisiones de UX siguen en la cola y no son requisitos faltantes ocultos.
