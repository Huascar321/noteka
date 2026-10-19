# Verificación de fase 0 — bootstrap-local-core

Fecha: 11 de octubre de 2026, `America/La_Paz` (12 de octubre UTC). Equipo: macOS Apple Silicon. Alcance: sólo las cuatro capabilities del cambio activo; no implica fases posteriores ni métricas de rendimiento.

## Evidencia ejecutada

- **15 tests Rust aprobados**: 12 de integración del núcleo, 1 de migración fallida y 2 de procesos CLI. SQLite real, conexiones concurrentes y fallos por triggers; no mocks de persistencia. Ejecución final bajo sandbox con red denegada y Cargo offline.
- **25 comprobaciones de UI nativa aprobadas en 3 sesiones Tauri/WKWebView**, con red denegada por sandbox macOS. IPC real y CLI como proceso independiente. Últimos resultados sintéticos fuera de Git: `noteka-native-rzQeGC/results.json` bajo el temporal aprobado de la sesión.
- `npm run contracts:check`: esquemas/types consistentes con Rust. `npm run check`: sin errores ni warnings. `npm run build`: frontend de producción construido.
- `scripts/cli-walkthrough.mjs`: ejemplo documental ejecutado; fixtures `noteka-demo.7gywPR` fuera del checkout.
- Playwright CLI en navegador: inicio y mensaje explícito de falta de backend de escritorio; no crea almacenamiento alternativo.
- `cargo fmt --all --check` y `cargo clippy --workspace --all-targets -- -D warnings`: aprobados, también incluyendo el feature de pruebas nativas.
- `npm ci` y `npm run tauri build -- --no-bundle`: aprobados. Binario release optimizado `target/release/noteka-desktop`, sin firma/distribución.
- `openspec validate bootstrap-local-core --strict`: aprobado; valida documentos, no sustituye los tests.
- `npm run tauri dev -- --no-watch`: inició Vite y el host nativo sin errores; Rust no se instaló globalmente.
- Revisión final de archivos nuevos con `git diff --no-index`: sin errores de whitespace ni runtime/credenciales elegibles para Git. No existe commit inicial, por eso `git diff` normal no incluye estos archivos untracked.

Reproducción: [development.md](development.md). La instrumentación nativa existe solamente en el host con feature `native-smoke`; no está en el binario normal.

## Trazabilidad por escenario

Núcleo: `crates/noteka-core/tests/local_core.rs`; CLI: `crates/noteka-cli/tests/cli.rs`; migración fallida: tests de `workspace.rs`; UI: `tests/native/smoke.js`. Los nombres abreviados remiten a esos tests.

### local-workspace

| Escenario | Evidencia |
| --- | --- |
| Creación fuera del checkout | `workspace_selection_default_and_restart_are_local`: resolver default fuera del cwd; UI crea en temporal y muestra ruta. |
| Selección de workspace | Mismo test: dos workspaces aislados e ID ajeno not_found; UI rechaza ruta inválida sin mostrar otra selección. |
| Operaciones sin red | Tres sesiones nativas y subprocesses CLI bajo `(deny network*)`, sin cuentas/modelos. |
| Reapertura | Test de selección conserva IDs/revisión/documentos/relación; tres procesos nativos verifican cierre/reapertura. |
| Migración compatible | `older_workspace_migrates_future_main_and_wal_remain_untouched`: v0 → backup → v1, nota y relación preservadas. |
| Migración fallida | `migration_failure_is_atomic_and_has_backup`: falla SQL, rollback a v0 y ruta de backup disponible. |
| Apertura con aplicación antigua | Test de versiones compara nombres/bytes antes/después, con WAL abierto y luego cerrado. |
| Backup mientras el workspace está abierto | `live_backup_restore_integrity_and_no_overwrite`: escritor con título no confirmado, backup sólo contiene estado confirmado; también UI/CLI con ventana abierta. |
| Restauración válida | Mismo test y walkthrough comparan ID de workspace/entidad, revisión y documento; UI abre el backup en otra sesión. |
| Backup corrupto o destino ocupado | Mismo test: corrupción rechazada y staging eliminado; sentinel de destino ocupado y workspace fuente intactos. `invalid_identity_and_missing_backup_schema_are_rejected` también rechaza un hash correcto con esquema incompleto y una identidad inválida. |

### knowledge-entities

| Escenario | Evidencia |
| --- | --- |
| Creación y modificación | `all_types_defaults_and_invalid_payloads`: cinco tipos, identidad, título, timestamps y revisión. |
| Tipo o título inválido | Mismo test: tipo desconocido, whitespace y campos extra rechazados. |
| Área y recurso | `tags_filters_and_stable_pagination`: tres clasificaciones/filtros conservan ID; reclasificación sin duplicado. |
| Clasificación inválida | Test de tipos/payloads rechaza clasificación ajena y tag sin clasificación. |
| Completar tarea | Test de tipos/payloads: todo → done y listado filtrado. |
| Estado desconocido | Mismo test: actualización inválida preserva done. |
| Documento con varios párrafos | `semantic_roundtrip_limits_and_atomic_validation`: versión/orden/IDs/texto exactos; UI → CLI verifica dos párrafos. |
| Contenido inválido | Mismo test: duplicados, exceso de bloques/bytes/profundidad y rechazo sin modificación; documento cercano al máximo aceptado. |
| Bloque desconocido ya almacenado | `unknown_documents_preserve_every_field_and_archive`: versiones/bloques/campos futuros legibles y reemplazo rechazado; UI sólo lectura/JSON. |
| Relación de nota a proyecto | `relations_integrity_duplicates_remove_archive_and_restore`: ambos extremos, edge único. |
| Extremo inexistente o incompatible | Mismo test: ID ajeno, self-link, tipo incompatible y FK inválida rechazados. |
| Relación repetida | Mismo test: changed:false, mismo edge, sin nueva revisión. |
| Archivar y restaurar | Mismo test: filtros de archivo, IDs/documentos y relaciones preservadas; también UI. |
| Proyecto archivado con notas activas | Mismo test; UI refresca y muestra «Proyecto relacionado · Archivado». |
| Páginas sucesivas | Test de tags/paginación fuerza timestamps iguales: 12 notas en páginas de 3, sin duplicados; límites inválidos rechazados. |

### core-operations

| Escenario | Evidencia |
| --- | --- |
| Clientes equivalentes | `real_processes_all_groups_and_parity`: actualización con origen desktop y proceso CLI sobre fixtures equivalentes; mismo título/documento. Handler Tauri delega al mismo execute_json, ejercitado por UI real. |
| Descubrimiento sin interfaz | `contract_catalog_is_complete_and_strict` y CLI: 15 operaciones con esquemas de entrada/salida/errores. |
| Documento inválido durante actualización | Test semántico: título válido y documento inválido conservan contenido/revisión/historial. |
| Falla de persistencia | `competing_writers_history_missing_revision_and_storage_rollback`: trigger falla tras editar título/documento y revierte toda la operación. |
| Dos clientes editan una nota | Mismo test: dos conexiones, segundo revision_conflict; UI también compite con CLI real. |
| Cliente omite revisión | Mismo test: update/archive/restore sin expected_revision → validation_error. |
| Escritor mantiene bloqueo | `busy_writer_times_out_without_mutation`: BEGIN IMMEDIATE, ~2 s acotados, workspace_busy, sin modificación; CLI exit 4. |
| Entidad no encontrada | Tests de selección/CLI: request ID y not_found sin traza interna. |
| Nota creada por CLI | Procesos CLI/walkthrough; host abre fixtures CLI del mismo workspace. |
| Operación estructurada exitosa | Tests CLI parsean stdout como único JSON, stderr vacío y exit 0. |
| Entrada inválida | `json_channels_exit_codes_and_invalid_arguments`: JSON malformado, argumento/payload inválido → JSON/error/exit 2, sin escritura. |
| Mutación rechazada y aceptada | Test de dos escritores registra origen/request ID/revisiones confirmadas, no conflicto ni falla como aplicados. |

### desktop-shell

| Escenario | Evidencia |
| --- | --- |
| Primer workspace | Host sin selección: creación por ruta, seis secciones y ruta visible. |
| Workspace no disponible | Ruta inexistente: error, sin sidebar/datos ajenos, elección posterior de otra ruta. |
| Captura y reapertura | Cmd+N, dos párrafos, Cmd+S, lectura CLI y posteriores sesiones. |
| Fallo al guardar | Trigger SQLite → storage_error, buffer intacto, Sin guardar y Reintentar. |
| Cierre con cambios pendientes | Cierre nativo cancelado conserva buffer; otro guarda antes de salir; otro descarta y sesión siguiente comprueba ausencia de escritura. Navegación también prueba tres opciones. |
| CLI modifica nota abierta | Save UI conflictivo conserva buffer/referencias; copiar y recargar con descarte explícito. |
| Archivo reversible en UI | Cmd+Shift+A → Archive → Restaurar → Notes; identidad/contenido conservados. |
| Documento de una extensión futura | Fixture v7 con widget/keep.value: sin editor, limitación y JSON intacto. |
| Recorrido offline por teclado | Cmd+N/S/Shift+A y foco con outline en host con red denegada. |

## Límites explícitos

- Sin benchmark de arranque/guardado ni FTS5; no son resultados medidos.
- Sin firma/notarización/instalador, sincronización ni recuperación de borradores tras crash.
- Toolchain de sesión temporal, sin cambios de PATH global; ver exports o instalar Rust permanente.
- Fixture v0 de pruebas, no versión histórica distribuida.
- DOM/atajos sintéticos y cierre nativo real: no certifica lectores de pantalla, teclado físico, todas las resoluciones ni otra Mac.
- No se leyó ni copió contenido de tu sesión de Notion.
- Cambio activo: no se archivó ni se publicaron specs principales automáticamente.
