# Verificación: cola local de extracción

Fecha: 13/10/2026. [`add-local-extraction-queue`](../openspec/changes/archive/2026-10-13-add-local-extraction-queue/proposal.md), **20/20 tareas implementadas/verificadas**, tres deltas sincronizados y archivado. SQLite v8/API v1/48 operaciones. No completa fase 5a ni certifica Home nativo.

## Entorno

Apple M1 Max/64 GiB, macOS 27.0.1, Rust 1.99.0 aislado, Node 22.22.3; rusqlite 0.38.0/SQLite incluido. Sin upgrades. Datos sintéticos/logs/backups en temporal aprobado fuera de Git. Sin pantalla/Notion/credenciales/modelos/proveedores; UI headless.

## Checks completados

`sandbox-exec -p '(version 1)(allow default)(deny network*)' cargo test --offline --workspace`: **142 tests aprobados**, tres ignorados intencionales (benchmark batch, benchmark extraction y helper de proceso hijo; los dos últimos se invocan explícitamente por sus harness). CLI: catálogo + 12 regresiones reales previas + nuevo test extraction; core: 66 unit tests y 62 integración. SQLite real, no mocks de persistencia. `cargo check` y `cargo clippy --offline --workspace --all-targets --features native-smoke,custom-protocol -- -D warnings`, fmt/check de contratos completados. Release CLI: 14 tests aprobados con red denegada.

Catálogo anterior/final: inputs y outputs de las **40 operaciones previas idénticos** (comparación JSON), ocho nuevas, nueve mutaciones batch/un check. Contrato raíz: propiedades/$defs previas idénticas excepto la ampliación de Operation; nuevas raíces al final conservan los nombres generados de Page<T>. Generated contracts/check/build válidos; Svelte 0 errores/warnings. Bundle 507,67 kB y advertencia >500 kB sin resolver, idéntico al baseline. No se modificó UI ni host Tauri.

`test:editor`: 13 aprobados; `test:headless-bridge`: tres; suscripción CLI: cuatro en debug y cuatro en release. Scripts de suscripción/extraction ejecutados bajo sandbox `(deny network*)`, ventana cerrada, sin modelo. Extracción sintética: 364.000 bytes, tres páginas, reconstrucción exacta, retry/cancel sin entidades/historial.

Headless fresco: **592 checks + 14 de teclado = 606**, seis sesiones dark/light/reopen/restore y cero orígenes externos, tráfico bloqueado fuera del puente local. Primer intento falló antes de abrir navegador por Unix socket demasiado largo con TMPDIR aislado; corrección sólo de entorno mediante `PWTEST_SOCKETS_DIR` corto bajo temporal aprobado, repetición íntegra sin relajar asserts. `npm run tauri build -- --no-bundle` produjo release sin ejecutar host. IPC/WKWebView/cierre/foco nativos no certificados. Sin ventanas, capturas de escritorio ni perfil personal.

## Fronteras y recuperación

| Frontera | Evidencia |
| --- | --- |
| 1 MiB / exceso | ASCII/multibyte/escapable exactos; import >1 MiB no enqueue; crecimiento tras claim pasa stale, conserva generación. |
| 1000 queued+running | 999 + dos enqueue concurrentes: uno admite; dedupe con cola llena no consume cupo. Dos retries compiten por un único slot y sólo uno admite. |
| 10000 jobs | Fixture SQL operacional sintética + enqueue real en frontera; siguiente rechaza sin writes, dedupe/retry no crea ID nuevo. |
| 32 MiB derivados | 32 jobs reales de 1 MiB; reemplazo descuenta anterior y acepta; siguiente byte rechaza antes de publicar, conserva originales. |
| 4096/512 | UTF-8 en fronteras y reconstrucción literal; CHECKs rechazan fragmento de 4097 bytes, ordinal 512 y count 513. El cap 512 es defensivo: chunk 4096 + fuente 1 MiB no alcanzan 512 en extractor v1. |
| 2 MiB envelope | Página de 64 fragmentos escapables ~1,58 MB; página de 100 jobs con config/error/generación futura excede y se rechaza sin writes. |
| 64 MiB SQLite | Snapshot completo preserva runtime/results; fixture de páginas libres originadas en derivados hace rechazar preview/apply, sin purga ni elevación. |
| Revisiones/config/cursores | Overflow comprobado, no-op obsoleto rechaza, get estricto, cursores de ruta/job/generación/filtro/límite/operación ajenos fallan. Config futura cruda intacta, no ejecución/retry. |
| Fencing y fallos | Pausa→resume, cancel→retry→nuevo claim, token/lease/scope ajenos; worker anterior no publica ni pisa otro estado/resultado. Deadline SQL y vencimiento justo antes de commit fallan; guardia limpia. |
| Proceso interrumpido | Hijo real termina exit 73 después de claim confirmado; open/lecturas dejan running intacto, lease vencido requiere run explícito → failed sin retry; retry posterior con revisión. |
| Dos procesos vivos | Hijo confirma claim y espera handshake por stdin; segundo proceso recibe worker_active/exit 74, otro cliente edita y al liberar handshake sólo hay un intento/generación. No sleeps como prueba de exclusión. |
| Persistencia/lectores | Trigger aborta antes de commit, original alterado, lock publicación y cleanup agotados; cleanup_pending sólo cuando no se confirmó failed. Snapshot lector conserva anterior mientras otro publica nueva completa. |

## Matriz individual de requisitos y escenarios

| Dimensión de cierre | Resultado |
| --- | --- |
| Completitud | 20/20 tareas; 16/16 requisitos de los tres deltas implementados. |
| Corrección | 51/51 escenarios contrastados con código y pruebas; matriz individual abajo. |
| Coherencia | Ocho decisiones de diseño y ADR 0011 respetados; sin divergencias pendientes del recorte. |

**CRITICAL/WARNING del cambio: ninguna abierta.** No se omitió dimensión de verificación. Fuera de alcance: aceptación nativa de Home y optimización del bundle heredado. Auditoría de enlaces locales: sin enlaces rotos nuevos; quedan dos enlaces heredados `../../../docs/plan-noteka.md` del bootstrap archivado, intactos (reparación documental separada).

Implementación por requisito (rutas bajo `crates/noteka-core/src/`, salvo donde se indica):

| Requisito | Implementación |
| --- | --- |
| Selección local explícita y ejecución separada | `extraction.rs:12–40`, `extraction_worker.rs:67–175,176–230`; apertura/importación sin llamada a worker. |
| Identidad incremental y trazabilidad | `extraction_storage.rs:190–276`, `extraction.rs:12–40,151–174`, `migrations/008.sql:9–30` (bajo noteka-core). |
| Extracción UTF-8 literal y localizada | `text_extractor.rs:4–45`, `extraction_storage.rs:292–368`. |
| Ejecución cooperativa con ownership acotado | `extraction_worker.rs:34–175,204–309`, `extraction_storage.rs:113–189,389–408`; guardia Budget compartida. |
| Pausa cancelación y reintento con revisión | `extraction.rs:97–174`, `extraction_storage.rs:42–53,370–408`. |
| Publicación atómica de una generación | `extraction_worker.rs:235–309,322–379`, `migrations/008.sql:42–69`. |
| Lectura verificable y paginada | `extraction.rs:41–96,175–246`, `extraction_storage.rs:311–368,410–465`. |
| Presupuestos explícitos y errores útiles | `extraction_contracts.rs:6–14`, `extraction_storage.rs:14–68,278–309`, `extraction_worker.rs:246–301,322–379`. |
| Evolución recuperable a cola local | `workspace.rs` SCHEMA_VERSION/schema_check/create/migrate e integrity; `migrations/008.sql` aditiva. |
| Backup y restauración de procesamiento | `workspace.rs` backup/validate_backup/restore; `extraction_storage.rs:142–183,266–274,466–534`. |
| Contrato compartido y versionado | `contracts.rs` Operation/Contract, `lib.rs:44–119,187–202`; contratos generados y CLI idénticos al catálogo. |
| Escritura atómica validada | Helpers de dominio previos sin cambios; `extraction.rs` transacciones/response precommit y `extraction_worker.rs:235–309`. |
| CLI operativo con ventana cerrada | `crates/noteka-cli/src/main.rs` ExtractionAction/Command::operation; dispatcher Rust `lib.rs:187–202`. |
| Historial local de mutaciones | Helpers previos de entities/mutations intactos; extracción sólo escribe sus cinco tablas. |
| Operaciones compartidas de extracción local | DTOs `extraction_contracts.rs:52–93`, ocho métodos Workspace y dispatcher/catálogo; sin reglas de negocio CLI. |
| Recibos independientes de derivados | `batch_storage.rs:19–36,51–98,282–322`; ORIGINAL_TABLES/registry/snapshot, namespace v2 y apply existente sobre original. |

Referencias E1–E23 pertenecen a `crates/noteka-core/src/extraction_tests.rs`; X = `text_extractor::tests::utf8_literal_ranges_reconstruct_bom_crlf_and_long_lines`; M = `workspace::tests::extraction_migration_failure_v7_and_success_preserve_all_unknown_originals`; C = `crates/noteka-cli/tests/extraction.rs::real_cli_explicit_extraction_parity_channels_and_errors`. Los escenarios heredados modificados conservan también sus tests anteriores de core/CLI.

| Ref | Test |
| --- | --- |
| E1 | explicit_queue_dedupes_and_publishes_literal_without_domain_writes |
| E2 | empty_invalid_content_and_ineligible_sources_have_explicit_errors |
| E3 | revisions_transitions_overflow_and_future_configs_are_preserved |
| E4 | pages_cursors_generations_and_reads_never_recover_or_write |
| E5 | ownership_pause_resume_cancel_retry_aba_and_other_writer_are_fenced |
| E6 | two_independent_connections_claim_once_with_barriers |
| E7 | publication_failure_deadline_stale_and_cleanup_pending_keep_previous_generation |
| E8 | backup_restore_inherits_inactive_ownership_and_original_remains_live |
| E9 | batch_receipt_preserves_current_extraction_but_control_aba_conflicts |
| E10 | pending_total_retry_and_concurrent_admission_respect_exact_caps |
| E11 | derived_cap_replacement_and_escaped_pages_use_real_generations |
| E12 | unsafe_originals_and_expired_control_recovery_are_not_hidden_noops |
| E13 | backup_rejects_missing_v8_table_matching_hash_and_retains_future_configs |
| E14 | final_lease_recheck_sql_interrupt_and_crash_do_not_confirm_results |
| E15 | concurrent_retry_uses_one_remaining_pending_slot |
| E16 | response_size_before_read_return_and_scoped_cursors_are_enforced |
| E17 | independent_process_crash_after_claim_requires_explicit_recovery_and_retry |
| E18 | preview_keeps_expired_runtime_and_future_jobs_without_recovery |
| E19 | publication_atomic_reader_and_growth_after_claim_preserve_prior_generation |
| E20 | fragment_constraints_and_physical_derivative_snapshot_cap_fail_closed |
| E21 | extraction_operations_are_rejected_as_batch_steps_and_checks_before_any_effect |
| E22 | concurrent_duplicate_enqueue_and_removed_note_reference_keep_original_provenance |
| E23 | two_real_worker_processes_share_one_ownership_and_allow_other_edits |

### local-extraction — 8 requisitos / 24 escenarios

| Requisito / escenario | Evidencia |
| --- | --- |
| Selección local explícita / Importación sin procesamiento implícito | E1, C, fixture sin red |
| Selección local explícita / Trabajo seleccionado sin red | C, extracción fixture con sandbox |
| Identidad incremental / Encolado repetido | E1, E10, E22 |
| Identidad incremental / Regeneración explícita | E1, E7, C |
| Identidad incremental / Configuración persistida futura | E3, E13, E16 |
| UTF-8 / Unicode y saltos de línea | X, E1, C |
| UTF-8 / Contenido que parece una instrucción | X, E1, C, fixture deny network |
| UTF-8 / Formato o codificación no admitidos | E2, C |
| UTF-8 / Original vacío | E2 |
| Ownership / Dos ejecutores independientes | E5, E6, E17, E23 |
| Ownership / Worker interrumpido y publicación tardía | E5, E14, E17 |
| Ownership / Deadline o contención | E7, E14 |
| Pausa cancelación reintento / Cola pausada | E1, E5, E12, C |
| Pausa cancelación reintento / Cancelación durante extracción | E5, E7 |
| Pausa cancelación reintento / Revisión antigua | E3, C |
| Pausa cancelación reintento / Reintento luego de un fallo | E3, E5, E7, E17, C |
| Publicación atómica / Generación completa observable | E19, E8 |
| Publicación atómica / Falla antes de publicar | E7, E14 |
| Publicación atómica / Regeneración fallida | E7, E19 |
| Lectura / Fuente alterada o ausente | E4, E12, E19, C |
| Lectura / Página y cambio de generación | E4 |
| Lectura / Ruta ajena y lecturas sin efectos | E4, E8, E16, E22 |
| Presupuestos / Fronteras y backpressure | E2, E10, E11, E15, E20 |
| Presupuestos / Respuesta limitada | E11, E16, C |

### local-workspace — 2 requisitos / 7 escenarios

| Requisito / escenario | Evidencia |
| --- | --- |
| Evolución / Migración desde SQLite v7 | M |
| Evolución / Falla y recuperación desde versión previa | M |
| Evolución / Versiones anteriores y futuras | M, regresiones workspace v0–v6/local_core/block_editor/database/formulas/search |
| Backup y restauración / Backup durante extracción | E8 |
| Backup y restauración / Restauración con ownership ajeno | E8, E16 |
| Backup y restauración / Recuperación explícita sin afectar origen | E8, E17 |
| Backup y restauración / Datos futuros o backup inválido | E13, local_core::live_backup_restore_integrity_and_no_overwrite, block_editor::attachments_backups_restore_exports_and_missing_files |

### core-operations — 6 requisitos / 20 escenarios

| Requisito / escenario | Evidencia |
| --- | --- |
| Contrato / Clientes equivalentes | C, cli::real_processes_all_groups_and_parity |
| Contrato / Descubrimiento sin interfaz | CLI catálogo y C |
| Contrato / Catálogo y transportes completos | tests::cli_catalog_and_public_operation_sets_are_identical; audit inputs/outputs 40→48 |
| Contrato / Paridad del lote | cli::batch_cli_stdin_files_receipts_transport_parity_and_structured_failures |
| Contrato / Catálogo de extracción acotado | catálogo 48, caps y flags false, E21 |
| Escritura atómica / Documento inválido durante actualización | local_core::semantic_roundtrip_limits_and_atomic_validation |
| Escritura atómica / Falla de persistencia | local_core::competing_writers_history_missing_revision_and_storage_rollback, E7/E14/E19 |
| Escritura atómica / Claim separado de publicación | E7, E14, E17 |
| CLI / Nota creada por CLI | cli::real_processes_all_groups_and_parity (lectura misma operación núcleo), headless existente |
| CLI / Lote sin escritorio ni red | CLI batch + scripts subscription sandbox debug/release |
| CLI / Extracción sin escritorio ni red | C, scripts extraction sandbox debug/release |
| Historial / Mutación rechazada y aceptada | local_core::competing_writers_history_missing_revision_and_storage_rollback |
| Historial / Procesamiento sin historial ficticio | E1, E22, fixture C/extraction |
| Operaciones extraction / Paridad de estado y error | C |
| Operaciones extraction / Entrada estricta y control de lectura | C, E3/E4/E16, extraction_contracts::tests::strict_extraction_shapes_and_defaults |
| Operaciones extraction / Operación excluida del lote | E21 |
| Recibos / Procesamiento entre preview y apply | E9 |
| Recibos / Configuración original entre preview y apply | E9 |
| Recibos / Esquema y tamaño siguen limitados | E20, E9, batch_tests::wal_snapshot_page_size_and_sql_extensions_fail_closed_without_migration |
| Recibos / Preview no reclama ni recupera | E18 |

## Medición release en M1 Max

Corpus en `extraction_tests::release_extraction_resources`: 1 MiB ASCII (`x`), multibyte (`é世😀\n` + padding ASCII hasta 1 MiB) y control U+0001 altamente escapable. Saturado: 32 generaciones reales de 1 MiB, 10.000 jobs; antes de cada claim hay 1.000 pending, después 999. Los demás jobs de presión usan configuración futura y no se ejecutan. Warmups 5 + 20 repeticiones por caso; p50 usa muestra ordenada índice 10 (mediana superior), p95 índice 18, max índice 19. Sin cambiar caps. La corrida final de fases fue serial, sin builds concurrentes para no mezclar su contención con la extracción.

Fases en **ms p50 / p95 / máximo**, medidas con Instant; lock hold excluye espera de adquirir lock e incluye trabajo hasta commit:

| Caso | Claim lock | Lectura/hash | Fragmentación | Publicación lock | Total run |
| --- | --- | --- | --- | --- | --- |
| ASCII | 0,69 / 0,81 / 0,81 | 4,48 / 4,57 / 4,63 | 16,07 / 16,52 / 16,87 | 7,30 / 8,69 / 8,72 | 28,86 / 30,27 / 30,50 |
| Multibyte | 0,72 / 0,78 / 0,82 | 4,45 / 4,76 / 5,94 | 16,64 / 17,56 / 17,87 | 7,43 / 8,62 / 8,72 | 29,83 / 30,67 / 32,67 |
| Escapable | 0,71 / 0,78 / 0,80 | 4,44 / 4,53 / 4,73 | 15,86 / 16,27 / 16,32 | 7,15 / 8,68 / 9,50 | 28,30 / 30,14 / 30,71 |
| Saturado | 53,83 / 55,72 / 56,94 | 4,51 / 4,73 / 4,74 | 15,90 / 16,42 / 16,66 | 9,28 / 12,59 / 12,61 | 83,65 / 87,42 / 87,61 |

Lectura de 64 fragmentos (ms p50/p95/max): ASCII 6,35/6,61/6,63; multibyte 6,72/6,90/7,68; escapable 6,74/6,96/7,07; saturado 6,50/6,66/6,68.

CLI real bajo sandbox deny network, proceso nuevo por llamada, **5 warmups + 20 repeticiones** por caso. `/usr/bin/time -l` RSS máximo por proceso; latencia de harness incluye spawn/apertura/operación/serialización/salida. No es memoria del host Tauri ni latencia de UI.

| Caso | CLI run ms p50/p95/max | RSS run MiB p50/p95/max | CLI fragments ms p50/p95/max | RSS fragments MiB p50/p95/max |
| --- | --- | --- | --- | --- |
| ASCII | 44,97 / 51,30 / 59,14 | 15,14 / 15,17 / 15,19 | 20,02 / 20,92 / 22,19 | 12,39 / 12,66 / 12,66 |
| Multibyte | 46,86 / 50,09 / 50,12 | 15,16 / 15,22 / 15,22 | 20,53 / 24,96 / 25,52 | 12,42 / 12,72 / 12,73 |
| Escapable | 46,94 / 52,16 / 56,65 | 15,14 / 15,19 / 15,20 | 23,48 / 27,36 / 37,35 | 14,08 / 15,16 / 15,17 |
| Saturado | 94,61 / 97,62 / 98,82 | 16,09 / 16,14 / 16,16 | 20,79 / 21,72 / 21,87 | 12,41 / 12,66 / 12,67 |

Envelope CLI run 1.418 bytes; fragments ASCII/saturado 268.987, multibyte 295.587 y escapable 1.579.707 bytes (incluye newline). Páginas de 64 no acreditan cobertura de la generación de 1 MiB. Memoria crece con respuesta escapable, no carga un modelo.

| Caso | page_count × 4096 | Páginas SQLite del snapshot | Texto retenido |
| --- | --- | --- | --- |
| ASCII/escapable | 349 | 1.429.504 bytes | 1.048.576 bytes |
| Multibyte | 350 | 1.433.600 bytes | 1.048.576 bytes |
| Saturado | 11.427 | 46.804.992 bytes | 33.554.432 bytes |

Estos tamaños son `page_count*page_size`, la unidad del límite de snapshot; no incluyen el pico temporal de WAL/SHM. El saturado queda bajo 64 MiB, **sin garantizar** ese resultado con cualquier configuración futura de hasta 8 KiB: el cap físico sigue independiente. P95 máximo de locks 55,72 ms < objetivo orientativo 150 ms; p95 total 87,42 ms < presupuesto 5 s, sin prometer SLA. El claim saturado domina por integridad/escaneo de trabajos; no justifica scheduler, purga ni más memoria. Tests de presión física prueban rechazo aun con huecos/páginas libres.

## Repetición y revisión final

Suite concurrente repetida ocho veces: 20 tests aprobados por corrida (160 ejecuciones adicionales), sin cambiar asserts/retries/sleeps. Sólo se excluyen de esas repeticiones el corpus 32 MiB, fixture física 64 MiB, fallo de bloqueo 2+2 s y benchmark; están en regresión completa/medición explícita. Helper de proceso es ignored en listado normal, pero los tests supervisores lo invocan de verdad.

Revisión del diff contra baseline independiente de Git sin commit inicial: comparación SHA-256 de frontend (salvo contratos generados), host Tauri, artefactos/docs Home, migraciones 001–007 y lockfiles/paquetes. Se conservan; no hay upgrades, archivos persistentes personales, secretos, pesos ni backups entre archivos versionables. Assertions históricos de versión actual pasan de 7 a 8 y downgrades sintéticos eliminan sólo tablas nuevas; no se debilitan las verificaciones originales. Revisión individual de 16 requisitos/51 escenarios más diseño/ADR, sin divergencias pendientes de este recorte. Los ajustes detectados (get Serde estricto, validación de run con ID aun pausado, integridad job/generación y lectura límite+1 aun con crecimiento) tienen pruebas y regresión posterior.

Evidencia de ejecución fuera de Git: `/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-extraction-apply-lfjkxgk0/`. Conserva baseline.tar.gz/manifest, env.sh, logs Rust/Clippy/check, catálogo anterior/final, métricas/resources y harness `cli-rss.py`, auditoría `audit-apply.py`, ejemplos/fixtures/backups sintéticos y resultados headless. Estos temporales no son garantía de retención indefinida; métricas y comandos quedan documentados aquí.

Comandos de reproducción (herramientas existentes, cargo offline y temporales fuera del checkout):

```sh
cargo fmt --all --check
cargo check --offline --workspace --all-targets --features native-smoke,custom-protocol
cargo clippy --offline --workspace --all-targets --features native-smoke,custom-protocol -- -D warnings
sandbox-exec -p '(version 1)(allow default)(deny network*)' cargo test --offline --workspace
cargo build --offline --release -p noteka-cli
NOTEKA_EXTRACTION_BENCH_KEEP=1 cargo test --offline --release -p noteka-core --lib -- \
  --ignored --exact extraction_tests::release_extraction_resources --nocapture
# workspace/job_id se imprimen por caso; conservar salida fuera de Git.
# Repetir get→retry con revisión vigente→run y fragments 25 veces por caso.
/usr/bin/time -l target/release/noteka --json --workspace "$WORKSPACE" \
  extraction fragments --input "$INPUT_JSON"
cargo test --offline --release -p noteka-cli
npm run contracts:check
npm run test:editor && npm run test:headless-bridge
npm run test:batches
NOTEKA_CLI=target/release/noteka npm run test:batches
npm run test:headless
npm run tauri build -- --no-bundle
```

## Cierre OpenSpec

20/20 tareas verificadas. Sincronización agent-driven de **16 requisitos/51 escenarios**: local-extraction nueva (8/24), core-operations (4 modificados + 2 añadidos/20 escenarios) y local-workspace (2 añadidos/7). Comparación íntegra de cada bloque delta con main, conservando requisitos/escenarios previos no mencionados y Purpose existentes. Las otras nueve specs y los artefactos/docs/host/UI de Home permanecen idénticos al baseline.

`validate --specs --strict` y `validate --all --strict` aprobados antes del archivo (14 ítems: dos cambios + doce specs). Archivo `2026-10-13-add-local-extraction-queue` mediante `archive add-local-extraction-queue --skip-specs --yes --json`: skip evita repetir la sincronización **ya realizada y comparada**, no omite specs implementadas. Metadatos `.openspec.yaml` y tres deltas conservados exactos; propuesta/diseño/tareas sólo ajustan enlaces relativos/estado de cierre después de mover. Validación posterior: trece ítems (Home + doce specs); sin enlaces rotos nuevos ni archivos de ejecución versionables. Sin commit, push o publicación.

OpenSpec estricto comprueba documentos, no aplicación. Home mantiene 9/17/ocho brechas nativas sin editar; 5a y la dependencia plena de 5b siguen pendientes. Bundle >500 kB es advertencia heredada, no una exención de un escenario de extracción.
