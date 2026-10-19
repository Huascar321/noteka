# Verificación de fase 4: operaciones agrupadas locales

Cambio [`extend-agent-operations`](../openspec/changes/archive/2026-10-13-extend-agent-operations/proposal.md). **22/22 tareas implementadas y verificadas; sincronizado y archivado el 13/10/2026**. Regresión v4 completa aprobada después de la corrección acotada del probe WAL; sin relajar aserciones ni criterios nativos de Home. API v1, 40 operaciones, SQLite v7 sin migración. Sólo datos sintéticos fuera del checkout; sin modelos/proveedores/remoto ni ventana/foco/capturas del escritorio. [Guía](batches.md), [ADR 0010](decisions/0010-atomic-local-change-plans.md).

## Evidencia vigente

Baseline previo a apply: `noteka-agent-apply-4yj44tgm/baseline.tar.gz` y `manifest.json`, 209 archivos. Directorio padre aprobado: `/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/`. Repositorio sin primer commit: el diff se compara por hashes/texto contra ese baseline, no contra una historia Git inexistente.

- `helpers-full-tests.log`: 96 tests core/CLI aprobados tras extraer helpers y primeras pruebas de lote.
- `expanded-tests.log`: regresión Rust ampliada aprobada antes de las últimas pruebas de conservación/medición.
- `noteka-batch-cli-RQn2V9/subscription.json`: construcción CLI real de 16 pasos, página original y 14.569 centavos, proceso con network denegado.
- `noteka-subscription-C53P9C/`: fixture intermedia de preview/apply/reapertura/backup/restauración/recálculo/pago, tres tests Node offline aprobados; la aceptación final amplió la suite a cuatro.
- `final-rust-v4.log`: **115 tests Rust aprobados**, incluida la reproducción determinista del probe WAL ya corregida; una medición release ignorada durante la regresión normal.
- `concurrency-stress-v4.log`: ocho ejecuciones adicionales del test CLI/lecturas Home concurrentes, todas aprobadas. No se añadieron retries ni se cuentan como ocho tests distintos.
- `final-batches-release-v4.log`: cuatro tests con CLI release real y red denegada; fixture final `noteka-subscription-mDk1m1/`.
- `final-headless-v4.log` y `noteka-headless-YnfNO3/`: seis sesiones aprobadas, 592 checks compartidos y 14 de teclado/intercalados, ningún origen HTTP externo.
- `sync-assessment-prearchive.json`, `contract-compatibility.json` y `closure-prearchive.diff`: comparación del delta/main, compatibilidad de las 38 operaciones previas y revisión contra baseline.

Una ejecución intermedia de regresión falló en el test existente `concurrent_cli_writer_and_home_reads_keep_reference_metadata_coherent`, sin details en el assert. Se agregó diagnóstico; inicialmente volvió a pasar. Más tarde el headless v3 falló en activación Home, sección Tags, con `No se pudo acceder al almacenamiento local` (`noteka-headless-QI732k`, tema light). Se reabrió la tarea 6.4: no se tomó el éxito anterior como cierre de esta regresión.

Reproducción determinista `workspace::tests::preflight_survives_wal_removed_by_last_connection_checkpoint`: una conexión real v6 mantiene WAL; el probe privado copia la base, detecta WAL y la última conexión cierra/hace checkpoint justo antes de copiarlo. El código previo falla con el mismo StorageError de IO (`wal-preflight-reproduction.log`). La corrección distingue únicamente NotFound de esa copia, descarta cualquier WAL privado parcial y vuelve a leer la base ya checkpointada; la prueba exige v6, no el v7 de la copia anterior, y verifica que no hubo migración. No abre el original con SQLite, no trata el probe como snapshot definitivo y no reintenta ni rebasa escrituras. El interleaving reproducido explica una clase de fallo compatible con el síntoma UI; ese intento UI no tenía instrumentación para atribuir su IO exacto retrospectivamente. Otros errores de IO siguen propagándose.

Después de la corrección, la regresión Rust v4, las ocho repeticiones de concurrencia y el recorrido headless v4 completo pasaron. Se cerró la tarea 6.4 con esa evidencia nueva, no por ignorar el intento fallido. Esto verifica el interleaving reproducido, no garantiza ausencia de toda carrera posible ni identifica retrospectivamente el IO exacto del intento UI v3.

## Medición release en M1 Max / 64 GiB

`cargo test -p noteka-core --lib --release --offline release_batch_resources --no-run` construyó el binario final. Para RSS sin incluir rustc/cargo, se ejecutó directamente con `/usr/bin/time -l target/release/deps/noteka_core-32c28369c3b82b15 release_batch_resources --ignored --nocapture`. Evidencia vigente: **`resources-runtime-v4.log`** y `resources-summary-v4.json` en el audit de apply. Los otros logs conservan mediciones anteriores, no se mezclan las muestras.

Fixture: 10.000 entidades originales (1.000 tags y 9.000 notas con bloques de 400 caracteres), 20.000 relaciones, FTS real; snapshot final **20.353.024 bytes**. Plan medido: **un paso entity.create, cero checks**, cinco warmups y veinte repeticiones, estado creciente con 25 confirmaciones. No extrapolar a lotes de 100 pasos/consultas máximas ni a arranque/render/IA.

| Fase (ms) | p50 | p95 (nearest rank) | Máximo |
| --- | ---: | ---: | ---: |
| Preflight/apertura lectura | 3,24 | 3,75 | 4,02 |
| Snapshot Backup | 6,42 | 7,14 | 7,38 |
| Huella base | 44,89 | 47,25 | 47,67 |
| Simulación + efecto | 3,12 | 3,35 | 3,38 |
| Apertura apply | 3,72 | 4,13 | 4,41 |
| Espera Immediate, sin contención | 0,0027 | 0,0034 | 0,0060 |
| Lock mantenido, incluye commit | 59,71 | 60,93 | 61,32 |
| Commit | 0,72 | 0,80 | 0,84 |

Resultado interno de evaluación: 1.118 bytes; la respuesta pública incluye metadatos/recibo adicionales y se limita por su envoltura completa. RSS máximo runtime **44.826.624 bytes (42,75 MiB)**; peak memory footprint 37.487.072 bytes, sin swaps. El RSS del comando de compilación no se presenta como RSS de la operación. Una medición anterior tuvo máximos mayores (lock 146,05 ms y commit 79,57 ms): se conserva como variabilidad observada, sin atribuir una raíz puntual no instrumentada ni prometer que el p95 final es un máximo garantizado.

Fronteras comprobadas por tests: 64 MiB exactos aceptados y página siguiente rechazada; 2 MiB exactos de envoltura y byte siguiente; 100 pasos ejecutados/101 rechazados; deadline SQL y antes de commit; timeout independiente 2.000 ms. No se subieron límites ni se saltaron checks para obtener estos resultados.

## Matriz de requisitos y escenarios

Pruebas Rust en `crates/noteka-core/src/batch_tests.rs` salvo indicación; datos SQLite reales. Abreviaturas:

- **P** `preview_apply_all_nine_operations_preserve_ids_diff_and_history`.
- **R** `receipts_bind_plan_scope_original_state_and_effect_before_commit`.
- **A** `before_commit_storage_failure_and_reader_snapshot_are_atomic`.
- **C** `checks_default_to_complete_and_error_free_partial_is_explicit` y `checks_cover_51_and_101_rows_without_certifying_skipped_rows`.
- **F** `batch_recovery_preserves_versions_unknown_formats_fts_and_attachment_originals`.
- **CLI** `crates/noteka-cli/tests/cli.rs::batch_cli_stdin_files_receipts_transport_parity_and_structured_failures` y prueba de conjuntos en main.rs.
- **S** `tests/subscription-batch.mjs` (CLI real, sandbox sin red).

| Requisito | Escenario exacto del delta | Evidencia |
| --- | --- | --- |
| Contrato compartido | Clientes equivalentes | Regresiones CLI/core existentes, CLI; adaptador Tauri genérico compilado |
| Contrato compartido | Descubrimiento sin interfaz | CLI operations describe; igualdad de conjuntos |
| Contrato compartido | Catálogo y transportes completos | Conjuntos enum/catálogo/CLI/output = 40; DTO resuelto en nueve descriptores |
| Contrato compartido | Paridad del lote | P, CLI, S; recibos/exports iguales salvo metadatos tipados |
| CLI operativo | Nota creada por CLI | Regresión existente real_processes_all_groups_and_parity y recorrido UI compartido; no nueva certificación nativa |
| CLI operativo | Lote sin escritorio ni red | CLI, S con `(deny network*)`, sin host iniciado |
| Planes acotados | Esquema y registros encadenados | P, helpers prestados; IDs/revisiones/relación/property_key/vista/documento v3 |
| Planes acotados | Referencia inválida o clave duplicada | plan_limits_strict_shapes_and_previous_exports; bindings_reject_text_currency_unknown_attrs_wrong_type_and_collisions; binding_errors_identify_step_and_binding_and_keep_all_tables_unchanged |
| Planes acotados | Límites y operaciones excluidas | plan_limits_strict_shapes_and_previous_exports; hundred_steps_execute_as_one_batch_and_canonical_path_aliases_share_scope; snapshot_size_boundary_and_response_limit |
| Planes acotados | Texto que parece referencia | bindings_do_not_interpolate_literal_ast_text_and_reserved_upserts_fail; título y payload literales conservados |
| Preview aislado | Preview exitoso y fallido sin mutaciones | P, invalid_step_and_check_leave_all_original_tables_and_fts_unchanged, F |
| Preview aislado | Snapshot con WAL y escritor concurrente | pinned_backup_includes_one_complete_state_despite_independent_wal_writer; wal_snapshot_page_size_and_sql_extensions_fail_closed_without_migration; workspace::tests::preflight_survives_wal_removed_by_last_connection_checkpoint |
| Preview aislado | Versión o estructura no admitida | wal_snapshot_page_size_and_sql_extensions_fail_closed_without_migration; límite snapshot; migración explícita v0 comprobada |
| Diff/identidad | Crear y editar con diff verificable | P, timestamp_metadata_normalization_never_hides_unknown_payload_or_civil_dates, F |
| Diff/identidad | Reparto empatado y tamaño excesivo | S tie-break de 4 centavos; vector UUID fijo; snapshot_size_boundary_and_response_limit; envoltura 2 MiB exacta |
| Checks finales | Cálculo inválido al final | C: división por cero, flag explícito y sort inválido que sigue fallando |
| Checks finales | Página parcial declarada | C: 51/101 filas, cursor entrada, next_cursor y matching_count |
| Apply atómico | Fallo intermedio y de persistencia | A; apply_rejects_invalid_receipts_stale_revisions_and_late_step_or_check_atomically; deadline antes de commit |
| Apply atómico | Commit visible y agrupado | A, P; lector fijado ve anterior y luego completo; request/origen comunes |
| Apply atómico | Misma validación que operaciones individuales | Helpers + regresiones database/formulas/views/templates; P y errores contextualizados |
| Confirmación segura | Escritura entre preview y apply | R, S escritor ajeno; conservan esa escritura y no construyen colecciones parciales |
| Confirmación segura | Revisión vieja o escritor ocupado | independent_writer_lock_timeout_and_visibility; apply_rejects_invalid_receipts_stale_revisions_and_late_step_or_check_atomically |
| Confirmación segura | Recibo ajeno o efecto alterado | R, CLI; omisión, versión/forma, plan alterado, path restore, effect_hash adulterado |
| Confirmación segura | Respuesta perdida después del commit | P, CLI, S: repetición efectiva falla; entity.get/query por IDs confirmados |
| Conservación/recuperación | Formatos futuros y referencia existente | F; future_documents_are_not_replaced_or_hidden_in_the_state_fingerprint; omitted_payload_stays_byte_exact_and_empty_update_keeps_existing_revision_semantics; original faltante aborta apply |
| Conservación/recuperación | Reapertura y recuperación del lote | F y S: hash original, docs/valores/cálculos/vistas/historial/FTS e IDs; restore en otra ruta |
| Suscripción CLI | Preview y construcción completa | S y script: 16 pasos, 14.569 centavos, seis asientos/cinco miembros/página original |
| Suscripción CLI | Fallo sin estructura parcial | S variante inválida y escritura ajena; validación relation/formula compartida y rollback A |
| Suscripción CLI | Costo y pagos actualizados | S: 15.000 centavos exactos, cuatro pendientes, referencia original sin copias |

La matriz no sustituye tests: sus logs/funciones son la evidencia. El antiguo escenario de apertura en escritorio mantiene evidencia histórica del canal compartido y la regresión headless; no se afirma nueva aceptación nativa de Home.

## Aceptación integrada y revisión final

| Check fresco | Resultado |
| --- | --- |
| cargo test --workspace --all-targets --offline, sandbox `(deny network*)` | **115 pasados**, 0 fallos; una medición ignorada en regresión normal y ejecutada aparte en release |
| npm run test:batches, CLI real sin red | **4 pasados**, tanto debug como release; incluidos fórmula inválida posterior, escritor ajeno y desempate monetario |
| fmt --all --check | Aprobado |
| Clippy workspace/all-targets + desktop native-smoke,custom-protocol, -D warnings | Aprobados; compilación únicamente, sin host iniciado |
| contracts:check, Svelte check/build | Aprobados tras regenerar contratos; checker 0 errores/0 advertencias |
| test:editor + test:headless-bridge | **13 + 3 pasados** |
| cargo build -p noteka-cli --release --offline + tauri build --no-bundle | Aprobados; Mach-O arm64 de CLI/desktop, sin launch/firma de distribución/publicación |
| test:headless | **592 + 14 = 606 checks**, seis sesiones dark/light/main/reopen/restored, 0 orígenes externos |
| OpenSpec --all --strict previo a archivo | **13/13**; valida documentos, no sustituye las pruebas de aplicación |

Logs finales v4 en el audit indicado arriba. UI final: `/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/noteka-headless-YnfNO3/`, results/network/keyboard JSON comprobados. Los logs/fixtures v2/v3 conservan evidencia anterior, incluido el fallo investigado. Una primera ejecución headless fue cortada por el timeout de la herramienta de 120 s; se cerró su sesión propia `noteka-headless-75710` y se repitió con timeout de herramienta deshabilitado y waits internos originales. No se quitaron asserts ni se utilizó pantalla nativa.

Revisión acotada contra baseline: helpers conservan dominio/ownership; creación única también en templates; nombre inválido individual sigue validándose antes del lock; body del query conserva snapshot individual. La última revisión preservó bytes del payload omitido en edición de título/documento y completó contexto step/binding para errores estructurales; cuatro tests adicionales prueban ese caso, context/membresía, duplicados/solapados y rechazo atómico de fallos finales con recibo modificado. Una actualización vacía sigue incrementando revisión como antes, no se convirtió en no-op. Nuevos módulos mantienen whitelist, bindings tipados, IDs, Backup fijado, trece originales, diff consolidado, reloj tipado y reejecución sin copia publicada. Se comprobó presupuesto después de Backup Done y antes del commit; salidas DTO estrictas. `contract-compatibility.json` compara las 38 variantes previas y sus definiciones: ningún cambio semántico, aunque schemars renombró aliases genéricos Page/Page2/Page3 al añadir DTOs. Se añadió la corrección reproducida del probe WAL, no reintentos de confirmación ni cambios de UI. Lockfiles, migraciones, componentes/estilos y artefactos/specs de Home no cambiaron. No hubo commit, push ni datos personales en fixtures.

Scorecard: completitud **22/22 tareas, 10/10 requisitos**; corrección **29/29 escenarios trazados**; coherencia con diseño/ADR sin divergencias bloqueantes. No se omitieron dimensiones de la revisión; la aceptación nativa de Home queda explícitamente fuera de este cambio. Home sigue **9/17** con ocho brechas nativas. Seguimiento no bloqueante: instrumentar IO si reaparece un fallo de lectura Home; bundle JS **507,67 kB** mantiene warning orientativo >500 kB, sin elevar umbral ni atribuir optimización a esta entrega. Los hashes no son seguridad ni idempotencia; medición de un paso no acredita costo máximo de todos los planes permitidos.

## Sincronización y archivo

Se compararon íntegramente los dos requisitos modificados y ocho añadidos del delta con `openspec/specs/core-operations/spec.md`: **10/10 requisitos y 29/29 escenarios coincidentes**. Los otros seis requisitos previos de la capability permanecen intactos; ninguna otra spec principal cambió. `instructions apply` confirmó `all_done`, **22/22**, y la validación estricta previa aprobó **13/13 ítems**.

Se ejecutó `npx --yes @fission-ai/openspec@1.14.1 archive extend-agent-operations --skip-specs --yes`: `--skip-specs` evitó repetir la sincronización ya aplicada y comprobada, no omitió la verificación. Archivo `openspec/changes/archive/2026-10-13-extend-agent-operations/`, schema `spec-driven`. Los cinco archivos del cambio, incluida `.openspec.yaml`, conservan exactamente sus hashes anteriores al movimiento; ninguna spec cambió durante el archivo.

Después del archivo, `list --json` confirmó únicamente Home activo con **9/17**; `validate --all --strict` aprobó **12/12 ítems** (once specs y Home). Se reconciliaron README/plan/config/workflow/guías/ADR/cola y se comprobaron **187 enlaces locales sin destinos faltantes**. Evidencia: `before-archive-manifest.json`, `archive-closure-check.json`, `openspec-prearchive.log`, `openspec-postarchive.log` y revisión final `closure-final.diff`/`closure-final-audit.json` en el audit de apply. El archivo no crea commit, no publica ni sincroniza los deltas pendientes de Home.
