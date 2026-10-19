# ADR 0011: extracción explícita y derivados aislados

Fecha: 13/10/2026. Estado: adoptado, implementado y [verificado](../verification-extraction.md). No cierra 5a ni Home nativo.

## Decisión

- SQLite v8 mediante 008 aditiva y backup previo. No reescribir 001–007/IDs/documentos/payloads/ASTs futuros. Rollback de software restaura backup anterior en otra ruta; no descarta automáticamente escrituras posteriores.
- Control es configuración original; runtime/jobs operativos; generaciones/fragmentos derivados. Config futura conservada cruda, soporte explícito, sin conversión/ejecución.
- Cola pausada por defecto. Ocho operaciones Rust/CLI/Tauri genérico no agrupables, enqueue separado de run de un job. Sin daemon/modelo/UI/red.
- Claim/publicación Immediate cortas, trabajo fuera del lock. Token privado + revisión de job/control + scope ID/ruta y lease 15 s cercan ABA/publicación tardía. Presupuesto cooperativo 5 s, contención 2 s, guardia SQL y cleanup separado acotado.
- Pausa del ownership válido devuelve queued; vencido/ajeno pasa a failed sin retry automático. Revisión antes de no-ops; open/restore/lecturas no recuperan.
- UTF-8 literal `.txt`/`.md` <=1 MiB sin NUL/conversión con pérdida; chunks <=4096 en frontera UTF-8, offsets y líneas por LF. Generación completa única reemplazada atómicamente; fallos conservan anterior.
- Caps 1000 pending/10000 jobs/32 MiB texto; sin purga/VACUUM ocultos. Cursores/páginas/envelope acotados. Estado persistido distinto de disponibilidad calculada.
- Registry batch 001–008 completo. Huella `noteka.batch.base.v2`: trece originales previos + control; excluye contenido operativo/derivado, no schema. Snapshot completo <=64 MiB sin worker; apply reejecuta en original, no promueve copia ni borra actividad posterior.

## Alternativas y límites

No extraer al importar, incrustar derivados en documentos, mantener lock durante procesamiento ni identificar sólo por PID/state. Token no autentica/autoriza. No prometer transacción filesystem/SQLite frente a procesos con acceso arbitrario: hash antes de publicar/leer y originales inmutables por contrato. Reloj hacia adelante falla conservadoramente; hacia atrás puede demorar recuperación, con cancel explícito.

32 MiB es texto lógico, no tamaño SQLite: páginas libres/índices consumen snapshot. Restore conserva IDs/revisiones/resultados/token, pero scope de otra ruta informa ownership inactivo; sólo acción explícita recupera.

## Verificación y pendientes

SQLite real, barreras, dos procesos vivos y crash tras claim, fallos precommit/deadline/IO/lock, backups y lotes. [Medición release/regresión completadas](../verification-extraction.md), 20/20 tareas y cambio sincronizado/archivado; `openspec validate` no sustituye aplicación. PDF/Excel/OCR/indexación, limpieza/scheduler, embeddings/harness/MCP/remotos son cambios separados. Home sin sincronizar/archivar.
