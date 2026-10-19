# Matriz de escenarios: búsqueda XLSX

Los cinco deltas contienen **14 requisitos/49 escenarios**: 32 heredados y 17
nuevos. Los nombres heredados se conservan. Esta matriz identifica evidencia
ejecutable, no reemplaza los logs y el estado de [aceptación](verification-xlsx-search.md).
Cinco deltas sincronizados/contrastados y cambio archivado el 16/10, 19/19 tareas;
las limitaciones H indicadas abajo forman parte explícita de ese cierre autorizado.
**Adenda 16/10:** los harnesses con baseline independiente/catálogo/binario viejo
identifican comprobaciones históricas cuyos artefactos se perdieron. La
[revalidación autorizada](xlsx-search-evidence-recovery.md) repite el resto y
añade fixtures SQL/contratos actuales, pero no convierte esas fixtures o schemas
en evidencia independiente v11. Esas filas conservan una limitación explícita.
**H** = comprobado antes del incidente, logs/baseline perdidos; no repetido.
**R** = revalidación fresca durable del 16/10. Las filas que no necesitan el
baseline se repitieron por Rust/CLI/SDK/raw/gates/headless. Las filas H se aceptan
con la limitación autorizada, no se presentan como un PASS fresco independiente.
`xs` = `crates/noteka-core/src/xlsx_search_tests.rs`; `as` = attachment_search_tests;
`xt` = xlsx_tests; `sc` = scoped_tests. Las funciones se nombran sin abreviarlas.

## core-operations

| Requisito / escenario | Test o harness |
| --- | --- |
| Contrato compartido y versionado / Clientes equivalentes | `tests/block_editor.rs`, `shared_reads` y contratos frontend/CLI compartidos |
| Descubrimiento sin interfaz | R `check-xlsx-search-current-contracts.py`, enumeración SDK/raw MCP; comparación v11 H |
| Catálogo y transportes completos | Catálogo 52/allowlist exacta; tests de paridad CLI y schemas |
| Paridad del lote | `batch_tests`, `tests/subscription-batch.mjs` |
| Catálogo de extracción acotado | R checker actual, lector CLI/golden y rechazo steps/checks; checker v11 H |
| Paridad de herramientas MCP habilitadas | `tests/local-mcp.mjs`, `test-xlsx-processes.py` y raw ambos protocolos |
| Catálogo completo y disponibilidad por transporte | R SDK/raw 24/44/45 tools/siete exclusiones y contratos actuales; igualdad exacta de los 50 frente a v11 H |
| Ejecución scoped sobre workspace actual / Scope actual y versión incompatible | R `sc::scope_creation_is_physically_pure_and_versions_never_migrate`, fixture SQL v11 y raw v13; harness independiente v11 H |
| Identidad comprobada en la operación | `sc::replacement_after_private_preflight_is_rejected_on_actual_connection_and_snapshot`, raw sustitución de identidad |
| Clientes anteriores conservados | H checker v11/CLI/MCP viejos; R contratos actuales y open/admisión sobre fixtures SQL, no cliente viejo |
| Contrato compartido de lectura XLSX / CLI y core equivalentes | `xt::explicit_worker_pagination_lexemes_and_cursors`, CLI 216 casos y oracle 83 casos |
| Conservación y ámbito limitado | H comparación completa v11/52; R oracle reader/contratos actuales y `xs::derived_xlsx_selection_between_preview_apply_preserves_original_hash_and_index` |

## local-extraction

| Requisito / escenario | Test o harness |
| --- | --- |
| Lector XLSX independiente y sin activación implícita / Lectura explícita offline | `test-xlsx-reader.py`: parser/helper, 216 casos y contención con red denegada |
| Producto existente conservado | R catálogo actual/415 refs, `test-xlsx-search.py` sin helper y pins actuales de Home/parsers/lectores; comparación exacta v11 H |
| XLSX excluido de indexación textual implícita / Intento de indexar XLSX | `xs::explicit_selection_noop_provenance_and_no_cross_field_matching`, corrupción/cuotas/CAS y preservación |
| Regresión de búsqueda y contratos | Gates UTF-8/PDF intactos y mixtos; Rust/CLI/parser/SDK/contratos/frontend/headless |

## local-mcp

| Requisito / escenario | Test o harness |
| --- | --- |
| Workspace fijo y admisión sin migración / Formato anterior o futuro | R fixture SQL v11, `test-mcp-transport.py`, sc anterior/futuro/inconsistente/ausente; harness v11 independiente H |
| Cambio de destino durante la sesión | Raw sustitución de identidad, sc conexión/snapshot efectivo; SDK campos extra |
| Sondeo sin efectos | Raw huellas antes/después startup/discovery y datos/jobs running sin recovery |
| Índice y búsqueda XLSX con política compartida / Paridad XLSX modern y legacy | SDK golden sin helper/allow-run, raw seis combinaciones de política, paridad CLI |
| Denegación y contenido hostil | SDK `XLSX untrusted fields are evidence, never authority`: valores/fórmulas inertes, permisos/scope rechazados |
| Pureza y conflicto de revisión | SDK paginación/reset, procesos publicación/retry/cancel, cursores ligados tras restore |

## local-text-search

| Requisito / escenario | Test o harness |
| --- | --- |
| Selección explícita de generaciones extraídas / Extracción no implica indexación | `xs::explicit_selection_noop_provenance_and_no_cross_field_matching`, v11→v12 con XLSX sin seleccionar |
| Indexación local de ambos extractores | `as::explicit_literal_index_noop_and_exact_multibyte_provenance`, `pdf_page_text_and_no_text_have_distinct_coverage_and_typed_locators` y gates UTF-8/PDF |
| Fuente o generación inadmisible | `as` matriz de fuentes/config/CAS, `xs::source_change_after_preparation_cannot_publish`, sc |
| XLSX persistido sin helper | Oracle 83 casos, SDK/raw; comparación de filas antes/después salvo índice derivado |
| Hits con fragmento canónico y localizador tipado / Evidencia literal | R tests UTF-8 y oracle/hash independiente; cursor v11 auténtico H, cursor sintético actual conservado R |
| Evidencia PDF sin renderizador | Corpus PDF 235 casos y tests PDF/search sin helper |
| Original cambiado después de indexar | `xs::xlsx_lookahead_checks_a_different_original`, `as` fuentes inseguras/ausentes/alteradas |
| Evidencia XLSX verificable | Oracle 83 casos: reader completo/fields/UTF-8/hashes/locators; `xs::projection_unicode_ranges_reconstruct_without_overlap_or_trim` |
| Cobertura explícita del conjunto seleccionado / Cobertura parcial y texto ausente | `as::get_coverage_future_raw_configs_and_expired_running_are_pure_and_filtered`, vacío/no_text y jobs sin generación |
| Configuración futura conservada | R fixture SQL v11 running/futuro y `xt::xlsx_admission_future_configs_and_mime_do_not_spawn`; harness v11 independiente H |
| Adjunto compartido y entidad archivada | Tests heredados de adjuntos compartidos y rechazo de filtros no soportados; sin dueño inventado |
| XLSX vacío y rechazo por cuota | `xs::empty_projection_and_excess_selection_preserve_full_reader`, `exact_xlsx_global_quotas_reject_one_more_without_eviction` |
| Proyección textual XLSX determinista por campos / Tipos léxicos y fórmulas separadas | Oracle de corpus lector + `xs::projection_types_order_hidden_sheets_and_separate_formulas` |
| Campos largos y Unicode | Oracle/cortes >4 KiB, reader completo y rangos consecutivos reconstruidos byte a byte |
| Frontera de selección completa | 512/513 real por CLI y unit; saturación 1024/16384/8 MiB, sin purga |
| Coincidencia no cruza campos ni fragmentos | `xs::literal_matching_does_not_join_slices_or_repair_cut_tokens`, fórmula/caché/celdas separadas |
| Integridad completa del índice mixto XLSX / Corrupción ajena a filtro o página | `xs::corruption_outside_filter_or_page_never_returns_empty_success`, `same_text_but_changed_valid_metadata_invalidates_selection_hash`, dos recorridos exactos |
| No-op y reemplazo concurrentes | `xs::prepared_xlsx_put_loses_to_other_put_or_published_generation`, reset/fuente/rollback, `test-xlsx-processes.py` |
| Presupuestos y rendimiento sin excepciones | Fronteras/snapshot/deadlines/contención/cursor/response heredados y XLSX; dos gates intactos y dos mixtos, sin subir caps/target |

## local-workspace

| Requisito / escenario | Test o harness |
| --- | --- |
| Migración aditiva y recuperación de derivados XLSX / Migración de baseline v10 real | H `test-xlsx-migration.py` con binarios independientes; R `test-xlsx-search-revalidation.py` usa SQL versionado/filas v12 sintéticas, NO v10 independiente |
| Falla y rollback de migración | R `xs::migration_rollback_before_and_after_version_update_keeps_v11`; harness v10 independiente H |
| Restore actual y original alterado | `xs::mixed_backup_restore_and_repaired_outer_hash_are_fully_validated`, xt representación alterada con hash reparado |
| Versiones futuras | sc/raw v13; `xs::xlsx_snapshot_cap_and_future_index_are_closed_without_cleanup`, jobs futuros preservados |
| Compatibilidad recuperable del índice XLSX v12 / Baseline v11 poblado | H harness independiente v11; R fixture SQL v11: 36 tablas/DDL/shadows/originales/running/unknown, no binario viejo |
| Cliente anterior y backup previo | H CLI/MCP v11 y restore con cliente anterior, no repetidos; R backup exacto/restore con cliente actual sólo, no sustituto del cliente viejo |
| Cursores y recibos al migrar | H cursor/recibo v11 reales; R cursor sintético generado por v12 sobre misma ruta/identidad sobrevive migración y vence por revisión; no se fabrica recibo v11 |
| Snapshot con índice mixto | `xs::derived_xlsx_selection_between_preview_apply_preserves_original_hash_and_index`, snapshot 64 MiB rechazado sin purga |
