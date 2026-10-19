# Verificación de PDF textual acotado

Fecha: 14 de octubre de 2026. Cambio `add-local-pdf-text-extraction`.
**Implementación verificada:** esta página registra evidencia, no certifica PDF genérico,
distribución firmada ni aceptación nativa de Home.

**Cierre:** 21/21 tareas, tres deltas sincronizados y [cambio archivado](../openspec/changes/archive/2026-10-14-add-local-pdf-text-extraction/tasks.md)
como `2026-10-14-add-local-pdf-text-extraction`. El CLI archivó con `--skip-specs`
porque el merge agent-driven ya estaba realizado y comparado; ese flag no omitió
el sync. OpenSpec estricto antes de archivar: 14/14 documentos válidos.
Después del archivo: **13/13** (doce specs y Home activo), cero errores.

## Alcance y entorno

SQLite v9/API v1/49 operaciones; originales y documentos v1/v2/v3 conservados.
`pdf_text` es explícito/offline; Type1 latino estándar, Standard/WinAnsi,
1 MiB de fuente, 1–100 páginas, 1 MiB de texto y 512 fragmentos por resultado.
No hay OCR/render, ToUnicode/fuentes embebidas, tablas estructuradas, indexación,
modelos ni proveedor remoto. Home sigue 9/17, sin sincronizar ni archivar.

Entorno de medición: Apple M1 Max/64 GiB, aarch64-apple-darwin, Rust 1.99.0,
lopdf =0.45.0 sin features default, Node 22.22.3; locks sin actualizaciones de
versiones registradas por la integración (sólo arista libc del core).

Evidencia externa al checkout:
`/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-pdf-finish-jyh3l3sz/`.
Contiene baseline de 272 archivos, manifest, logs, corpus, pruebas de migración
desde CLI v8 compilado por separado y revisiones de conservación. No se copian
originales personales ni PDFs de ejecución a Git.

## Corpus y golden

[Generador/procedencia/reglas](../tests/fixtures/pdf/README.md): 235 casos propios
MIT y golden independiente de lopdf; zlib runtime 1.2.12. Se compararon los 207
anteriores: todos conservan bytes/hash/texto; sólo tres motivos negativos se
precisaron a invalid_pdf al resolver xref/free/Prev. Los demás cambios son aditivos.

- SHA-256 generate.py: `aa0f7b27dbe46dc7b407bccd03e9bae67b64426355fcdae5c05af6c4ce06eb4c`.
- SHA-256 corpus-manifest.json: `82efbdd679db47bbc36b43c77a9856a0a3cae728a532f892851ca0d22fc2a5a1`.
- Debug/release: 235 casos por parser_probe y 235 por CLI real en cada perfil;
  12 casos de contención por perfil. Las comprobaciones rechazan stderr/contenido
  ajeno, confrontan texto, manifest, hashes, fragmentos/rangos, códigos/envelopes.
- CLI real simula `Contents/MacOS` sin lanzar el host, usa el helper sibling,
  deduplica y restaura/lee sin helper; su ausencia falla sólo el nuevo run.
- Migración real desde CLI v8: dos estados por perfil (pausado/habilitado con
  running), configura JSON futuro raw, compara todas las filas previas, exige
  receipt anterior rechazado y cliente v8 sin writes ante v9.

Comandos (con `NOTEKA_TEST_TMP` externo y toolchain disponible):

```sh
npm run pdf:prepare
cargo build --locked --offline -p noteka-cli -p noteka-pdf-worker --bins --examples
python3 -B scripts/test-pdf-parser.py
python3 -B scripts/test-pdf-cli.py
python3 -B scripts/test-pdf-containment.py
npm run pdf:prepare -- --release
cargo build --locked --offline --release -p noteka-cli -p noteka-pdf-worker --bins --examples
python3 -B scripts/test-pdf-parser.py --profile release
python3 -B scripts/test-pdf-cli.py --profile release
python3 -B scripts/test-pdf-containment.py --profile release
python3 -B scripts/test-pdf-migration.py --baseline-cli "$V8_CLI"
python3 -B scripts/benchmark-pdf.py
```

Rust/CLI/parser/contención/benchmark y host build se ejecutan con
`sandbox-exec -p '(version 1)(allow default)(deny network*)'` y modo Cargo offline.
Los runners UI necesitan HTTP loopback; sus subprocess CLI mantienen red denegada,
y los checks verifican ausencia de requests externos. No se ejecuta `test:native`.

## Concurrencia, almacenamiento y conservación

Los tests PDF ejercitan procesos reales con barrera tras enviar input privado:
edición concurrente, segundo cliente sin ownership, pausa, cancelación, control
ABA y parent SIGKILL, ocho rondas de cada variante. Helper sólo lleva bytes/tiempo,
no SQLite/token/ruta de workspace; vence con su propio timer aunque muera el padre.
Restore de un intento activo conserva datos pero no habilita ownership/cursor de
otra ruta; recuperación/retry son explícitos. No hay autorización basada en PID.

Publicación y rollback mantienen generación previa y no modifican entidades,
FTS/historial, adjuntos ni huella original. Se prueban páginas vacías/Unicode,
65+ fragmentos/cursor regenerado, corrupción de hash/rango/manifest/cross-type
sin reparación ni incrementos de attempts por fallar antes de claim. La fixture
de corrupción se confirma en SQLite: no se usa un SAVEPOINT que oculte el error
de integridad detrás de un BEGIN anidado.

El presupuesto mixto se fija exactamente en 32 MiB con texto UTF-8 y 31 generaciones
PDF estructuralmente válidas; son semillas SQLite de presupuesto, no 31 ensayos
del parser. Un PDF sin texto cabe, un texto adicional falla sin purga y reemplazar
descuenta una sola vez la generación anterior. Backup durante helper/precommit/
commit conserva una generación; hash recalculado no habilita manifest corrupto
ni tabla ausente. Original/hash stale no se certifica como resultado válido.

## Mediciones release

Cinco warmups/veinte repeticiones por cada caso admitido del corpus y uno con
31 MiB de derivados previos. Se separan run, claim, publicación y lock efectivo,
texto/respuesta/SQLite, Layouts Rust vivos y high-water RSS de padre/hijos.
El contador del padre sólo existe en unit tests; el core productivo no tiene cap
de heap. `runtime_metrics` usa el mismo entrypoint/IPC/IO/allocator del helper,
muestrea después del flush y emite métricas separadas, fuera del runtime público.
RSS no es heap ni un límite de 128 MiB. CHILDREN es high-water de hijos recogidos,
no un muestreo RSS por PID durante cada run.

Primer ensayo: máximo lock p95 **180,54 ms**, gate fallido; no se descartó.
La reconstrucción/hash de todos los derivados PDF estaba dentro del escritor.
Se movió a snapshot read-only previo al claim; estado/ownership/configuración/
pertenencia/conteos/bytes se revalidan bajo lock, y fence/fuente/budget al publicar.
No se eliminaron validaciones ni aumentaron caps. Ensayo definitivo:
`tmp/pdf-bench-e7tl800j/results.json`, **98 casos admitidos + 1 saturado**,
2.475 ejecuciones (495 warmups/1.980 medidas). Máximo lock p95 **20,19 ms**,
objetivo <=150 ms cumplido. La pasada intermedia fallida se conserva en
`tmp/pdf-bench-66eoma_g/run.log`; no es evidencia sustituida silenciosamente.

| Caso | Run p50/p95 ms | Claim lock p50/p95 ms | Publicación lock p50/p95 ms |
| --- | --- | --- | --- |
| WinAnsi | 22,62 / 27,78 | 0,51 / 4,30 | 1,15 / 5,46 |
| 100 páginas | 23,34 / 24,27 | 0,66 / 0,80 | 1,99 / 2,16 |
| 1 MiB texto | 149,03 / 153,72 | 0,85 / 1,21 | 10,71 / 14,69 |
| 1 MiB fuente | 141,23 / 147,99 | 0,70 / 0,89 | 8,74 / 10,35 |
| 31 MiB previos + 1 MiB texto | 344,65 / 347,91 | 13,19 / 13,74 | 17,72 / 20,19 |

Máximos observados: padre Layouts **6.302.468 B**, RSS **30.441.472 B**;
helper Layouts **16.859.638 B**, RSS **13.500.416 B**. CHILDREN high-water:
13.533.184 B. No sumar máximos de distintos casos como si fueran memoria simultánea.
SQLite máximo 38.563.840 B; respuesta privada 1.048.821 B; página pública PDF
271.396 B (DTO, no envelope). El log conserva cifras por caso/percentiles completos.
El corpus y contención prueban también exceso/bombas/abort/deadline, no sólo éxitos.

## Regresión y revisión final

Corrida definitiva `*-closure*`: **177 tests Rust en debug y 177 en release**,
cero fallos y cinco ignores intencionales (benchmarks batch/UTF-8/PDF y dos helpers
invocados por sus tests de procesos). Corpus 235 parser/235 CLI/12 contención y dos
estados de migración en cada perfil, sin fallos; fmt/check/Clippy -D warnings,
contracts, 13 tests adaptador/cola/tema, tres puente/consentimiento y cuatro tests
de suscripción por perfil aprobados. Build frontend y `tauri build --no-bundle`
aprobados sin abrir host; persiste el warning heredado de chunk JS >500 kB.

Headless: hubo una corrida integrada previa completa de 592+14 checks, y dos
intentos de cierre terminaron en waitForFunction/480 s sin completar las seis
sesiones. El segundo conserva tres sesiones dark aprobadas; no se cuentan como
606 checks finales. No se cambió ninguna assertion ni UI para aprobarlos. La
repetición definitiva con `caffeinate -i` aprobó **592 + 14 = 606 checks**,
seis sesiones dark/light × main/reopen/restored, cero requests externos y sin
perfil personal. Logs `headless-closure-awake.log` y resultados en
`tmp/noteka-headless-JaEXr0/`. No cambió preferencias del sistema ni abrió ventanas;
no se atribuye causa de los timeouts anteriores sin evidencia.

Se preservaron byte a byte 001–008, host Rust,
Home/código UI (salvo contratos generados) y main specs antes de sincronizar PDF.
Los fingerprints independientes comprueban los 48 inputs/outputs/error previos,
140 definiciones previas y orden/referencias de roots; sólo Operation/metadata y
los roots PDF son aditivos.

La comparación de sync ignora sólo líneas vacías: los **13 bloques/56 escenarios**
de delta coinciden íntegramente con main; los otros **27 requisitos** de esas tres
capabilities y las otras nueve specs permanecen sin cambios. Home no se sincronizó.
Revisión baseline final: **272 → 284 archivos**, 47 existentes modificados,
225 idénticos y 12 agregados; ocho artefactos PDF movidos al archivo, ningún otro
archivo eliminado. Los 128 archivos de código/scripts/tests comparados contra el
manifest de la corrida fresca conservan sus hashes. Sin procesos de pruebas,
helpers ni sesiones headless pendientes; sin commit, push o publicación.

| Dimensión | Resultado de revisión |
| --- | --- |
| Completitud | 21/21 tareas contrastadas con código, logs y documentación; sin tareas PDF diferidas. |
| Corrección | 13 requisitos/56 escenarios con cobertura de dominio, negativas, procesos y CLI; sin discrepancias pendientes detectadas. |
| Coherencia | Diseño/ADR 0012 y límites operativos coinciden; UTF-8/contratos previos y migraciones protegidas conservados. |

No hay bloqueos críticos ni advertencias de alcance pendientes para este recorte.
Persisten las limitaciones explícitas del subset/contención, warning de bundle,
avisos heredados de dependencias y aceptación nativa de Home. La revisión no es
auditoría exhaustiva de seguridad ni garantía para todos los PDFs.

## Trazabilidad de los deltas

Se revisaron **13 requisitos/56 escenarios**; los escenarios previos en bloques
MODIFIED también se conservan. Esta matriz indica cobertura, no sustituye los logs.

| Requisito (escenarios) | Escenarios contrastados / evidencia |
| --- | --- |
| Selección explícita PDF (5) | Seleccionado offline, lecturas/pausa, nombre/firma/tamaño, cifrado/conversión y contenido activo: corpus/CLI completo, `pdf_admission_safe_source_dedup_future_and_paused_never_launch_helper`; contador de spawn cero antes de run compatible. |
| Cobertura/localizadores PDF (4) | Unicode/fragmentación, vacío/imagen/mixta, estructura textual excluida, fuente vs texto: corpus/CLI y `real_pdf_pagination_reconstruction_and_literal_contract_remain_separate`; reconstrucción/hash por página. |
| Contención/interrupción (5) | Crash/recursos, reply inválido, pausa/cancel/fence, muerte del padre/recovery y publicación atómica: containment, supervisor adversarial y `two_real_pdf_processes_*` (ocho rondas por variante), `pdf_source_change_and_each_table_failure_*`, `pdf_publication_fences_*`. |
| Lectura PDF (3) | Paginación/regeneración, original/derivado alterado, cursor ajeno/pureza: `real_pdf_pagination_*`, `pdf_backup_integrity_*`, CLI sin helper. |
| UTF-8 literal (4) | Unicode/CRLF, instrucciones pasivas, formato/codificación y vacío: suite heredada extraction_tests y fronteras de lectores separados; no se reinterpretan sus offsets. |
| Presupuestos/errores (3) | Fronteras/backpressure, respuesta limitada, límites PDF/mixto: corpus, pdf_contracts, `mixed_text_budget_counts_actual_utf8_bytes_*` y tests heredados de respuesta/caps. |
| Contratos PDF aditivos (3) | Lectores/DTO previos, input estricto, helper ausente: fingerprints independientes, pdf_contracts, CLI y supervisor. |
| Contrato compartido/versionado (5) | Clientes equivalentes, descubrimiento sin UI, catálogo completo, paridad lote y catálogo extracción: contratos/catálogo/CLI 49, CLI del corpus y suscripción debug/release. |
| CLI con ventana cerrada (4) | Nota, lote, extracción y PDF: suite CLI/core, suscripción y corpus operativo con red denegada, sin host. |
| Operaciones compartidas de extracción (3) | Estado/error, entrada estricta/control lectura y exclusión batch: CLI, DTOs y `extraction_operations_are_rejected_as_batch_steps_and_checks_before_any_effect`, incluidas las nueve extraction.*. |
| Recibos/derivados (6) | Procesamiento entre preview/apply, control original, schema/tamaño, preview sin reclamar, PDF post-preview y receipt v8: `mixed_pdf_batch_snapshot_*`, `pdf_batches_snapshot_all_tables_*`, migración CLI real y tests heredados de ownership/ABA. Las tres tablas PDF se comparan completas; 64 MiB exactos pasan y una página adicional falla. |
| Migración recuperable (5) | v7, fallo/restore, versiones anteriores/futuras, v8 con procesamiento y rollback software: suites workspace/anteriores, `v8_migration_*` y CLI v8 real; 001–008 intactas, cliente viejo sin writes. |
| Backup/restauración (6) | Durante extracción, ownership ajeno, recovery explícita, futuro/backup inválido, PDF durante parsing/commit e integridad PDF inválida: suites heredadas, `pdf_backups_during_prepare_publish_commit_restore_one_fenced_generation`, fixture de helper activo y CLI sin helper; manifest hash recalculado no oculta daño. |

La revisión de huecos agregó negativos PDF específicos de metadatos/IO seguro,
dedup concurrente/config futura, fuente alterada tras lectura/prepublicación,
fallo de cada tabla PDF/cleanup y schema/snapshot mixto, además de pruebas genéricas
heredadas. Las expectativas de config explícita y colisión de IDs del plan se
corrigieron desde sus errores reales, no para convertir fallos de producto en éxito.
