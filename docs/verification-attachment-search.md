# Verificación de búsqueda local de adjuntos

Fecha: 14/10/2026. **19/19 tareas verificadas, tres deltas sincronizados y cambio archivado.** Cambio [`add-local-attachment-search`](../openspec/changes/archive/2026-10-14-add-local-attachment-search/tasks.md), SQLite v10/API v1/51. Dos benchmarks finales consecutivos: query p95 **214,20–231,07 ms**, target <=300 ms y lock <=150 ms aprobados sin cambiar caps ni retirar validación. Scratch físico medido, **212 tests Rust por perfil y 606 checks headless** aprobados. Se conservan las corridas fallidas; el resultado sólo aplica a las condiciones/corpus documentados. Home permanece 9/17, sin lanzar host nativo, usar pantalla ni activar modelos/proveedores.

## Evidencia externa y conservación

Temporal aprobado: `noteka-attachment-search-apply-qLSCJI/`. Baseline de 291 archivos en `baseline/` y `baseline.json`; CLI v9 independiente en `noteka-v9`, compilado offline/release desde la copia con target separado. No hay commit inicial: un Git diff vacío no es evidencia. `migration-hashes.json` conserva hashes iguales de 001–009. `contract-comparison.json`: 49 inputs, 49 outputs y 150 definiciones anteriores, salvo la unión Operation que recibe dos variantes, iguales al CLI v9 independiente.

`attachment-tests-limits.log` y `attachment-tests-before-bench-fixed.log` conservan pruebas parciales iniciales sobre SQLite bundled real: migración/rollback, main READ_ONLY con TEMP escribible, canónicos UTF-8/PDF/no_text, cuotas independientes, CAS/no-op, cursores/fuentes inseguras, falsos ceros/corrupción/copias/postings/estadísticas, reset/rollback, backup/recibos y overflow. La regresión integrada fresca y la matriz individual se detallan más abajo; la aceptación de rendimiento procede de los dos benchmarks finales, no de esas pruebas funcionales.

La SQLite **bundled de Rust es 3.51.1** (rusqlite 0.38/libsqlite3-sys 0.36.0). `sqlite-runtime-version.txt` procede de una sonda externa enlazada al mismo rlib/native archive del build, consultando `sqlite_version()`/compile_options: FTS5 habilitado, TEMP_STORE=1. No confundir con SQLite 3.54.0 del Python de fixtures ni con el probe Node 3.51.3 de planificación. Se validan los endpoints en binarios Rust reales, no se infiere comportamiento bundled del intérprete Python.

`cli-initial-fixed2.log` preserva las tres rondas iniciales. `attachment-cli-{debug,release}-complete.json` repiten **veinte rondas por perfil**, dos migraciones v9 reales por perfil (poblado y running con resultado previo/configuración futura), restauración de backup v9/cliente anterior puro e indexación/búsqueda PDF sin helper. El harness exige un único ganador CAS, snapshots completos en carreras put/backup/reset/publicación y edición independiente. Incluye roundtrip mixto UTF-8/PDF/no_text, configuraciones futuras y comparación byte a byte de originales. Ambas repeticiones posteriores al cierre de schemas de salida terminaron correctamente, con stderr vacío.

`guide-cli-reconciled.json` verifica además la secuencia documentada `get` por stdin → `put --input` → `search` por stdin → `reset --input` sobre una fixture UTF-8/CRLF/multibyte real, con red denegada y original conservado; no son comandos inferidos sólo de schemas.

## Benchmarks de release: evolución y condiciones

Corpus fijo: ocho generaciones UTF-8 de 1.048.576 bytes, total 8 MiB/2.048 fragmentos; línea ASCII repetida `canonica evidencia cafe contrato alpha beta gamma delta\n`. Hash fuente `fd7657afedbf7018407b0dcc402f3ab2eb1e929737efd307d727e84987a21f4c`. Selecciones sembradas con fixtures SQL válidas, no ocho puts medidos; las mediciones de put sí usan el endpoint real.

Cada caso tiene **cinco warmups y veinte muestras**. Sin UI/red/modelos, cachés calientes, build release. Los timers de lock abarcan adquisición→COMMIT; la sonda de otro escritor usa conexión/barrera independiente, no UI nativa. Presupuesto y caps no se aumentaron.

| Caso | p95 wall ms | p95 lock ms | p95 espera de otro escritor ms |
| --- | ---: | ---: | ---: |
| Query con hits | 1.140,54 | — | — |
| Query sin hits | 1.048,83 | — | — |
| Query filtrada | 1.276,06 | — | — |
| Primera página | 1.161,59 | — | — |
| Siguiente página | 1.215,81 | — | — |
| Put 1 MiB sobre 7 MiB seleccionados | 1.046,70 | 25,00 | 40,36 |
| Put no-op sobre 8 MiB | 1.384,57 | 14,41 | 22,24 |
| Reset 8 MiB | 23,23 | 13,38 | 21,99 |
| Publicación/invalidation | 83,51 | 40,89 | 49,74 |

`benchmark-optimized.json`, `benchmark-optimized-summary.json` y `.log`: todas las muestras terminaron correctamente y por debajo de 5 s. **Gate lock p95 <=150 ms aprobado; target de consulta <=300 ms incumplido.** No se debe dar por cerrado ni aceptar una excepción silenciosa por haber pasado pruebas funcionales.

RSS SELF high-water del proceso de tests: 42.631.168 bytes (macOS), no el RSS del compilador. SQLite al final 21.532.672 bytes, WAL 5.697.992; TEMP lógico del verificador 11.501.568 (`page_count*page_size`). Estos valores no son un cap de memoria, pico físico de scratch ni un tamaño que prometa reset/compactación.

### Baseline fallido y diagnóstico

`benchmark-initial.json/.log` preservan el primer resultado: query p95 entre 1.839,93 y 2.028,20 ms. `/usr/bin/time -l` allí incluye Cargo/compilación y **no** se interpreta como RSS exclusivo del endpoint.

`posting-diagnostics.log` compara, sobre 1.199.240 instancias, la verificación inicial (2.423,63 ms), conteo + un EXCEPT (922,96 ms para esa comparación), GROUP/UNION (1.387,45 ms) y streams ordenados (1.327,42 ms). Es diagnóstico de una ejecución, no p95 de aceptación. El costo de comparar todos los postings, incluso para cero hits o un filtro estrecho, domina las consultas densas.

La primera optimización conserva igualdad exacta bidireccional: esperado único E, E ⊆ actual A y |E|=|A| implican A=E. Se eliminó el segundo árbol EXCEPT, **no** validación, caps, canónicos, comprobación de scores o fuente. Ese resultado todavía incumplía el target; no habilitó validación parcial ni una excepción.

### Segunda optimización exacta, todavía sin alcanzar el target

`benchmark-stream-final.json` referencia `tmp/attachment-bench-670bzqm6/results.json` y logs completos del script reproducible. Comparación inicial de secuencias exactas con memoria constante; ante distinto orden/datos se usa el fallback de conjuntos. Un test comprueba el mismo conjunto en distinto orden y duplicados con igual cardinalidad; no se asume que `fts5vocab` prometa un orden. Se conserva el diagnóstico `posting-stream-diagnostic-contended.log` (una ejecución, no aceptación).

Misma máquina **Apple M1 Max/64 GiB**, macOS 27.0.1 arm64, mismo corpus/hash, release, cinco warmups/veinte muestras por caso, sin otra suite pesada en paralelo:

| Caso | p95 wall ms | p95 lock ms | p95 espera escritor ms |
| --- | ---: | ---: | ---: |
| Query con hits | 781,37 | — | — |
| Query sin hits | 721,52 | — | — |
| Query filtrada | 733,44 | — | — |
| Primera página | 766,04 | — | — |
| Siguiente página | 774,35 | — | — |
| Put | 761,04 | 28,19 | 40,35 |
| No-op | 782,55 | 13,45 | 22,65 |
| Reset | 14,28 | 5,17 | 10,13 |
| Publicación/invalidation | 50,36 | 22,02 | 22,69 |

Todas las muestras correctas; máximo wall <0,9 s. Gate de lock pasa, **target de query sigue fallando** (2,4–2,6 veces 300 ms), sin aceptar excepción de diseño. RSS SELF high-water 45.301.760 bytes; SQLite 21.532.672, WAL 5.706.232, TEMP lógico 11.493.376 bytes. No es una medición completa de scratch físico ni un límite RSS.

Comparación fresca de **10.000 notas/80 muestras** con el mismo example y baseline independiente v9:

| Medida p95 ms | v9 | v10 |
| --- | ---: | ---: |
| Búsqueda directa | 17,25 | 17,45 |
| Operación con preflight | 23,56 | 23,97 |

Diferencias observadas de 0,20/0,41 ms en esta ejecución, no una prueba estadística de equivalencia. El corpus de notas no contiene adjuntos seleccionados. La repetición final siguiente no volvió a medir ese example: conserva esta comparación independiente en `benchmark-stream-final.json`.

### Repetición previa y edición de notas

`benchmark-complete.json` referencia el informe `results.json` de su directorio externo. Mismos corpus, hardware, build, cinco warmups/veinte muestras por caso y validación exacta; ninguna muestra falló y el máximo wall observado fue **870,31 ms**, por debajo del presupuesto cooperativo de 5 s.

| Caso | p95 wall ms | p95 lock ms | p95 espera escritor ms |
| --- | ---: | ---: | ---: |
| Query con hits | 772,12 | — | — |
| Query sin hits | 715,06 | — | — |
| Query filtrada | 717,17 | — | — |
| Primera página | 765,76 | — | — |
| Siguiente página | 825,07 | — | — |
| Put | 711,63 | 24,30 | 39,74 |
| No-op | 749,68 | 13,20 | 22,69 |
| Reset | 21,19 | 5,47 | 10,13 |
| Publicación/invalidation | 51,45 | 21,08 | 22,49 |

**Gate de lock aprobado; target query incumplido:** p95 715,06–825,07 ms frente a <=300 ms (2,38–2,75 veces el target). El script termina con éxito al producir mediciones válidas y publica `query_target_met=false`; su exit 0 no aprueba el target. No se elevó el umbral ni se aprobó una excepción.

RSS SELF high-water **42.237.952 bytes** del proceso de tests; SQLite **21.528.576**, WAL **5.710.352**, TEMP lógico **11.489.280 bytes**. Se mide `temp.page_count*page_size`, **no** el pico físico completo de scratch/temporales SQL. Esa medición todavía estaba pendiente en esta corrida; se resuelve por separado abajo, sin afirmar un límite RSS/WAL/scratch.

El mismo informe compara edición real por CLI de una nota ASCII de **10 KiB**, incluyendo startup/preflight/update, cinco warmups y veinte muestras: p95 **12,67 ms v9 / 13,37 ms v10**, diferencia observada 0,71 ms y ambas <150 ms. Son workspaces sintéticos con una nota y sin selecciones; no una prueba de equivalencia estadística ni de edición nativa bajo carga. La espera de escritor concurrente en la tabla mide otra conexión SQLite, no latencia UI.

### Diagnóstico adicional, sin nueva optimización adoptada

`profile-breakdown.log` separa una ejecución: comparación exacta de secuencias **501,45 ms**, canónicos **70,70 ms**, copias por EXCEPT **63,86 ms** y construcción FTS **38,42 ms**. No son p95 ni tiempos sumables como un perfil exhaustivo. `profile-native-{groups,encodings,json}.log` conserva experimentos de agregación SQL; aun la variante aislada más rápida rondó 299 ms sólo para postings, sin cubrir el resto de la consulta ni demostrar todos los límites/orden/corrupción. No se incorporaron esas variantes al verificador. Su fuente se conserva fuera de Git en `attachment-search-diagnostics-experiments.rs`; el test ignorado mantiene el diagnóstico de etapas/alternativas exactas. Ningún experimento justifica retirar validación, asumir orden público de vocabularios o introducir un hash aproximado.

### Apertura sin mutex redundante y copias ordenadas

Se adoptan dos cambios acotados sin modificar persistencia, hashes, contratos ni límites:

- La apertura **dedicada** de index/search añade `SQLITE_OPEN_NO_MUTEX`, como el default documentado de rusqlite. `Connection` no es Sync y pertenece a una única operación; no se comparte entre threads. Se elimina el mutex por llamada/columna, **no** el lock de escritor de SQLite, CAS, transacciones o busy timeout. No se cambia la apertura de notas, lotes, lectores o workspace.
- Las copias completas se comparan por streams con `ORDER BY rowid` explícito, incluyendo las cinco celdas y sus tipos, en lugar de construir dos árboles EXCEPT con todo el texto. El rowid de cada FTS es único; terminar ambas secuencias juntas verifica faltantes/extras sin depender de un orden implícito. `copy_stream_checks_complete_rows_types_and_rowids` cubre inserción en otro orden, tipos, identidad, texto, rowids, filas faltantes y extras. Los postings conservan la secuencia exacta de cuatro columnas y su fallback completo E ⊆ A/cardinalidad.

`posting-no-mutex-diagnostic.log` mide una ejecución: secuencia 499,63 ms con la conexión histórica frente a **231,57 ms** con la dedicada. Después de las copias ordenadas, `posting-no-mutex-copy-stream.log` mide verificación completa **358,39 ms** en la conexión dedicada. Son diagnósticos, no aceptación p95.

`benchmark-no-mutex-copy-stream.json` y su `results.json`: mismo corpus/hash de 8 MiB, M1 Max/64 GiB, release, cinco warmups/veinte muestras por caso, sin otra suite pesada en paralelo. Todas las muestras correctas:

| Caso | p95 wall ms | p95 lock ms | p95 espera escritor ms |
| --- | ---: | ---: | ---: |
| Query con hits | 383,05 | — | — |
| Query sin hits | 370,06 | — | — |
| Query filtrada | 384,11 | — | — |
| Primera página | 382,17 | — | — |
| Siguiente página | 395,37 | — | — |
| Put | 361,68 | 23,57 | 27,14 |
| No-op | 417,99 | 12,89 | 27,16 |
| Reset | 15,44 | 4,81 | 12,11 |
| Publicación/invalidation | 48,53 | 20,64 | 27,19 |

El p95 de consultas baja aproximadamente a la mitad respecto de la repetición previa, pero **sigue excediendo 300 ms** (1,23–1,32 veces el target). Gate lock aprobado, máximo p95 **23,57 ms**. No se acepta una excepción: la decisión explícita del usuario fue seguir optimizando con el objetivo y la validación completos.

RSS SELF high-water **42.844.160 bytes**; SQLite **21.536.768**, WAL **5.706.232** y TEMP lógico **11.489.280 bytes**. La comparación fresca del example de **10.000 notas/80 muestras por consulta** da p95 directo **17,40 ms v9 / 16,81 ms v10**, operación/preflight **24,03 / 23,82 ms**. Son diferencias observadas, no equivalencia estadística. El informe también conserva la comparación CLI de edición de 10 KiB (**11,38 / 12,51 ms**); ese binario de edición estaba construido antes del cambio a la apertura dedicada, que la edición no utiliza. No se presenta como una nueva prueba de la UI ni de edición con índice cargado.

Experimentos adicionales **descartados**: `posting-per-row-projections.log` (JSON/packing por fila más lentos que cuatro columnas); `raw-posting-sequence-diagnostic.log` (API C manual, mejora insuficiente para justificar unsafe/mantenimiento). Sus fuentes están en `per-row-projection-experiments.rs` y `raw-posting-sequence-experiment.rs` externos. El verificador conserva rusqlite seguro, sin ese FFI ni proyecciones empaquetadas.

La sonda externa de archivos falló inicialmente (sin informe y luego abortó antes del constructor por acceso TLV durante carga dyld); esos runs no produjeron mediciones válidas y se conservan. La corrección y la medición válida se detallan abajo. Nada de esa instrumentación se incorpora al producto. La documentación oficial de [temporales SQLite](https://www.sqlite.org/tempfiles.html) distingue TEMP, índices transitorios y journals: contar sólo páginas TEMP o nombres visibles no alcanza.

### Reutilización exacta de tokenización dentro de la misma lectura

Cuando los textos únicos ocupan <=25% del total, `same_repeated_postings` tokeniza cada texto **byte a byte distinto** una vez más con el mismo FTS5 Unicode61/columnas. Conserva el FTS esperado completo para las consultas/scores, no comparte estado entre llamadas ni omite generaciones. Un HashMap clasifica por igualdad completa de bytes (una colisión de hash no significa igualdad). El mapa completo posición→término de cada clase se aplica a cada rowid canónico. Referencias oficiales consultadas: [Unicode61](https://www.sqlite.org/fts5.html#unicode61_tokenizer) y [fts5vocab instance](https://www.sqlite.org/fts5.html#the_fts5vocab_virtual_table_module): términos por corridas de caracteres, una fila por instancia, offset ordinal de token desde cero con detail=full; no offset de bytes ni promesa de orden entre filas.

Se recorre **todo** el vocabulario real, verificando término, rowid, columna y posición/tipos; un bitset independiente por documento rechaza duplicados. Si A ⊆ E, A no tiene duplicados y |A|=|E|, entonces A=E. El conteo esperado pondera cada documento, no sólo cada texto distinto. Cero tokens/filas, faltantes, extras y reemplazar un faltante por un duplicado no eluden la prueba. La cache del último término compara sus bytes en cada fila: no presume un orden público de fts5vocab.

Sin suficiente repetición se conserva `same_posting_sequence`, y cualquier falta de igualdad en el camino nuevo conserva también `same_postings` (E ⊆ A/cardinalidad) como fallback completo. No se usa digest aproximado, muestreo ni una cache de confianza de un snapshot anterior. Memoria adicional acotada por 8 MiB de texto, 16.384 fragmentos y rowids <=524.288; no es el camino de memoria constante anterior. El bitset usa como máximo una posición por byte canónico y los textos únicos elegibles suman <=2 MiB. Se registra RSS/TEMP sin prometer un cap de RSS.

`repeated_postings_are_exact_order_independent_and_detect_duplicate_missing_pairs` compara con el verificador anterior sobre vocabularios reordenados, Unicode/diacríticos, términos repetidos, documentos idénticos, rowid máximo, faltantes/extras/duplicados, columna/término/posición/IDs inválidos y casos sin tokens. `repeated_fragment_verification_rejects_hidden_postings_and_keeps_source_checks` verifica el endpoint real: postings adulterados con copia restaurada fallan incluso con cero hits; originales modificados siguen rechazándose.

`repeated-postings-diagnostic.log` mide una ejecución, no p95: secuencia completa **239,16 ms**, camino repetido **144,80 ms**, verificación completa dedicada **269,70 ms**. Las agrupaciones SQL con NO_MUTEX siguieron más lentas (**279,59–358,61 ms** sólo postings en `grouped-no-mutex-diagnostic.log`); no se adoptaron, fuente externa `grouped-no-mutex-experiment.rs`.

Primera medición `benchmark-repeated-postings.json`: mismo corpus/hash, cinco warmups/veinte muestras, p95 query **281,57–297,94 ms**, `query_target_met=true`; lock máximo p95 **24,42 ms** aprobado. Put p95 279,22 ms, no-op 314,14, reset 16,21, invalidation 52,92; ningún caso falló. RSS **43.630.592**, SQLite **21.516.288**, WAL **5.706.232**, TEMP lógico **11.481.088 bytes**. Edición CLI de 10 KiB p95 **11,79 ms v9 / 17,64 v10**; notas/10.000 directo **17,11 / 16,82**, operación **23,61 / 21,23 ms**, sin equivalencia estadística. Esta ejecución incluye compilaciones pequeñas de la sonda fuera del benchmark, por lo que se repite al final sin otra suite/carga iniciada por la agente.

La ventaja depende de repetición: **no** se extrapola <=300 ms a cualquier texto de 8 MiB, cachés frías ni todos los caps simultáneos. El corpus fijado no se cambió para pasar. Cada posting real se sigue verificando; el perfil/esquema y el resto de invariantes no cambian.

### Réplica sin margen suficiente y diagnóstico de hashing

La suite `*-repeated-final` terminó correctamente: **211 tests Rust por perfil**, siete ignorados intencionales, PDF 235 parser/235 CLI/12 contención por perfil, 20 rondas CLI por perfil, catálogo/contratos, frontend/host sin lanzarlo y 606 checks headless sin requests externos. Aun así `benchmark-repeated-final.json` dio `query_target_met=false`:

| Caso | p95 wall ms | p95 lock ms |
| --- | ---: | ---: |
| Query con hits | 305,87 | — |
| Query sin hits | 314,20 | — |
| Query filtrada | 344,71 | — |
| Primera página | 300,17 | — |
| Siguiente página | 282,97 | — |
| Put / no-op | 282,16 / 329,74 | 23,41 / 12,95 |
| Reset / invalidation | 14,64 / 46,76 | 4,83 / 19,43 |

Todas las muestras válidas, máximo wall **460,33 ms**. RSS **44.974.080**, SQLite **21.536.768**, WAL **5.697.992**, TEMP lógico **11.517.952 bytes**. Notas/10.000: directo p95 **16,01 v9 / 16,24 v10**, operación **20,31 / 22,81 ms**. Edición CLI 10 KiB **10,80 / 11,39 ms**. No se toma la primera corrida pasada como evidencia suficiente de margen ni se oculta la fallida.

`canonical-hash-breakdown.log` separa hashing: sobre 8 MiB, SHA-256 de fuentes **23,57 ms** y hash JSON de contenido **30,30 ms**. La inspección del código instalado y la [documentación de sha2 0.10.9](https://docs.rs/sha2/0.10.9/sha2/index.html) confirmó que ARM usa el backend portable salvo feature `asm`. Se habilita **sólo en aarch64**, conservando versión, algoritmo, detección runtime de SHA2 y fallback portable. `Cargo.lock` añade únicamente **sha2-asm 0.6.4** (licencia MIT según su Cargo.toml instalado, más su referencia desde sha2 0.10.9); sus dependencias ya estaban. No se actualizan otros paquetes ni se cambia SQLite. La descarga de código de build en `sha-asm-fetch.log` es explícita; pruebas/CLI siguen con red denegada y no se envía contenido a proveedores.

El hash de contenido también serializa vistas prestadas con claves ordenadas explícitamente, sin clonar el texto en dos árboles Value. **Mismos bytes JSON, namespace/prefijos big-endian y SHA-256 v1**, sin migración. `borrowed_content_hash_preserves_v1_for_both_locators_escapes_and_nested_metadata` contrasta con el algoritmo anterior para ambos localizadores, no_text, escapes/control/multibyte y metadata anidada. `persisted-pre-asm-hashes-compatible.json` verifica además **16 generaciones preexistentes en dos workspaces** con hashes del backend/serialización anteriores por el CLI nuevo, no sólo comparaciones entre dos cálculos nuevos.

`hash-asm-diagnostic.log`: canónicos **23,36 ms**, hash de contenido **9,13**, SHA de fuente **3,90**, verificación completa dedicada **203,53 ms**. Es una ejecución diagnóstica, no p95 de aceptación. Los manifiestos/diffs declaran esta excepción acotada a la preservación anterior de dependencias: sólo Cargo.toml del core y Cargo.lock adicionales. Los hashes persistentes, contratos y perfiles no cambian; el resultado de las repeticiones finales se registra por separado.

### Aceptación final: dos corridas consecutivas

`benchmark-asm-final.json` y `benchmark-asm-repeat.json`, después de la regresión completa, sin otra suite/compilación iniciada en paralelo: mismo M1 Max/64 GiB, release, corpus/hash fijados, cinco warmups/veinte muestras por caso y verificación exacta completa. Ambas publican **query_target_met=true y lock_gate_met=true**; 225 ejecuciones correctas por corrida (warmups incluidos), máximo wall **234,66 / 228,29 ms**, dentro del presupuesto cooperativo de 5 s.

| Caso | p95 wall ms, final / réplica | p95 lock ms, final / réplica | p95 espera escritor ms, final / réplica |
| --- | ---: | ---: | ---: |
| Query con hits | 231,07 / 225,11 | — | — |
| Query sin hits | 217,51 / 214,20 | — | — |
| Query filtrada | 227,26 / 226,09 | — | — |
| Primera página | 226,31 / 223,44 | — | — |
| Siguiente página | 226,87 / 224,17 | — | — |
| Put | 215,30 / 217,69 | 15,58 / 14,68 | 26,09 / 26,32 |
| No-op | 220,02 / 221,21 | 4,32 / 4,35 | 11,15 / 4,69 |
| Reset | 15,09 / 13,95 | 4,84 / 4,40 | 11,37 / 11,28 |
| Publicación/invalidation | 40,38 / 41,30 | 16,97 / 17,66 | 12,17 / 22,47 |

El mayor p95 de consulta deja **68,93 ms** respecto de 300 ms en estas dos corridas, no una garantía universal. Lock máximo p95 **17,66 ms** (<150 ms). RSS SELF high-water **40.927.232 / 40.419.328 bytes**; SQLite **21.528.576 / 21.528.576**, WAL **5.718.592 / 5.706.232**, TEMP lógico **11.493.376 / 11.497.472 bytes**. Los tamaños son observaciones, no caps de RSS/WAL ni scratch físico.

Comparación independiente de 10.000 notas (80 muestras por consulta, sin adjuntos seleccionados): p95 directo v9/v10 **16,19 / 15,39 ms** y **15,81 / 15,73**; operación/preflight **21,40 / 20,35** y **22,89 / 21,41**. Edición real CLI de nota de 10 KiB con binarios recién construidos: p95 **10,74 / 10,97 ms** y **9,91 / 9,74**, ambas <150 ms. No es equivalencia estadística ni aceptación nativa; la comparación incluye intencionalmente el backend de compilación nuevo de v10.

El diagnóstico, las corridas fallidas y la primera favorable seguida de fallo permanecen arriba. No se cambió corpus, target, caps, controles de fuente ni validación completa para obtener estos resultados. Textos no repetidos usan el camino completo anterior; no se promete <=300 ms para cualquier corpus, caché fría o todos los caps simultáneos. Los bordes/saturación/deadline se validan funcionalmente por separado.

### Scratch físico por proceso, separado de latencia

`physical-scratch-report.json` / `physical-scratch-ten-summary.json` y `scratch-selftest-copies.json` son la evidencia final sobre el binario con backend ARM/vistas prestadas. Sonda externa macOS `scratch-probe.cpp`, `scratch-selftest.cpp`, `measure-scratch.py`: DYLD interpose de open/openat/write/pwrite/ftruncate/close/unlink/unlinkat **y copyfile/fcopyfile/clonefile/clonefileat/fclonefileat**, consulta `fstat/stat(st_size, st_blocks*512)` después de escrituras/truncates/copias/clones y antes de close. Mantiene archivos desvinculados todavía abiertos y archivos cerrados todavía con nombre; cuenta clones aunque no atraviesen open/write visibles del llamador. Sólo observa el hijo CLI y su TMPDIR/SQLITE_TMPDIR dedicado; clasifica SQLite/WAL/SHM del workspace sintético aparte, sin datos personales ni otros procesos.

El self-test exige un pico conocido **196.608 bytes**, incluyendo un archivo desvinculado abierto, otro cerrado con nombre, una copia y un clon, con extensión/truncado y cierre/borrado correctos: cinco archivos, cuatro escrituras, una truncación, cinco unlinks, cuatro closes, una copia y un clon registrados. Evitar el acceso temprano a thread-local storage durante carga dyld corrigió el aborto; el código final delega a las mismas funciones originales, sin syscalls sustitutas ni configuración SQLite distinta. Las versiones previas de la sonda y sus resúmenes se conservan, pero no se usan como instrumentación completa de cierre: no incluían hooks explícitos de copia/clon ni separación de SHM.

Fixture por **ocho puts públicos**, misma fuente/hash y 8 MiB/2.048 fragmentos. Diez casos: put, get, no-op, primera/siguiente página, sin hits, filtrada, publicación/invalidation, postings dañados/fallback y reset. Pico simultáneo de tamaños aparentes y bloques asignados de archivos scratch: **21.254.144 bytes**, dominado por la copia privada de preflight; put sobre 7 MiB **19.324.928 bytes**. El pico simultáneo scratch+SHM también fue **21.254.144 bytes**, no la suma de máximos no simultáneos. En esta sonda: bloques máximos por archivo del workspace SQLite **21.389.312**, WAL **2.039.808**, SHM **32.768 bytes**, separados de los tamaños de los benchmarks. Todos los handles registrados cerrados y nombres de scratch eliminados al salir; se conservan los originales/base persistentes. La fixture dañada se edita con SQLite Node **3.51.3** porque Python 3.54 prohíbe editar shadows; la operación medida sigue siendo Rust bundled **3.51.1**.

Son bloques de archivos regulares del proceso observado, incluidos TEMP/índices transitorios/journals/copias de preflight. No es consumo exclusivo APFS (deduplicación/metadata del filesystem), RAM scratch, una auditoría de todos los syscalls posibles, un p95 de latencia instrumentada ni una promesa universal de espacio. La instrumentación y resultados permanecen fuera de Git. Reproducción con el directorio de evidencia disponible:

```sh
c++ -std=c++17 -O2 -Wall -Wextra -Werror -dynamiclib -fPIC "$AUDIT/scratch-probe.cpp" -o "$AUDIT/scratch-probe.dylib"
c++ -std=c++17 -O2 -Wall -Wextra -Werror "$AUDIT/scratch-selftest.cpp" -o "$AUDIT/scratch-selftest"
python3 -B "$AUDIT/measure-scratch.py"
```

## Reproducción

Con toolchain/cache offline, temporales externos y red denegada para Rust/CLI:

```sh
cargo test --offline --locked --workspace
cargo test --release --offline --locked --workspace
python3 -B scripts/test-attachment-search.py --baseline-cli "$AUDIT/noteka-v9"
python3 -B scripts/test-attachment-search.py --profile release --baseline-cli "$AUDIT/noteka-v9"
python3 -B scripts/benchmark-attachment-search.py --baseline-root "$AUDIT/baseline" --baseline-cli "$AUDIT/noteka-v9"
```

En macOS se envuelven comandos con `sandbox-exec -p '(version 1)(allow default)(deny network*)'`; Cargo usa `--offline --locked`. La comparación de notas/10.000 usa la copia v9, no el p95 histórico. Los bordes independientes se verifican con SQLite real en tests de cuotas; no son un benchmark de p95 en todos los caps. El target de consulta y scratch se verifican por los métodos separados anteriores. El baseline v9 conserva su build anterior sin `asm`; v10 incorpora intencionalmente ese backend de compilación, también en sus hashes anteriores. Se compara el cambio real completo, no dos builds con flags idénticos.

## Regresión integrada final y revisión

Toolchain: Rust/Cargo **1.99.0**, Node **22.22.3**, npm **10.9.8**, Python de fixtures **3.10.19/zlib 1.2.12**, SQLite Python **3.54.0** y SQLite bundled Rust **3.51.1**. Hardware/OS arriba. Los temporales, originales sintéticos, backups y resultados están fuera de Git.

| Comprobación | Resultado y logs externos |
| --- | --- |
| `cargo test [--release] --offline --locked --workspace` | **212 tests aprobados por perfil**, siete ignorados intencionales (benchmarks/diagnóstico y fixtures de hijos invocadas por sus supervisores), cero fallas; `workspace-{debug,release}-asm-final.log` |
| Corpus parser/CLI PDF y contención | **235 parser + 235 CLI + 12 contención por perfil**, cero fallas; `pdf-{parser,cli,containment}-{debug,release}-asm-final.log` |
| Migración PDF desde CLI v8 independiente | **Dos estados por perfil**, pausa/config raw/filas/recibos/cliente viejo comprobados; `pdf-migration-{debug,release}-final-state.json` |
| Harness nuevo CLI | **20 rondas concurrentes + dos estados v9 + roundtrip mixto por perfil**; `attachment-cli-{debug,release}-asm-final.json` |
| Fmt/check/Clippy `--workspace --all-targets -- -D warnings`, builds debug/release/examples | Aprobados; `{fmt,check,clippy}-asm-final.log`, `{build,examples}-{debug,release}-asm-final.log` |
| Contratos TS/JSON generados | Aprobados, 49 inputs/49 outputs/150 definiciones anteriores conservados salvo extensión de Operation; `contracts-asm-final.log`, `catalog-v10-asm-final.json`, `contract-comparison.json` |
| Adaptador/editor/lecturas/cola/tema | **13 tests**; `npm-test-editor-asm-final.log` |
| Suscripción/lotes CLI reales | **Cuatro tests**, 16 pasos/centavos/reapertura/restauración; `npm-test-batches-asm-final.log` |
| Puente/aislamiento/consentimiento | **Tres tests**; `bridge-asm-final.log` |
| Frontend | `svelte-check`: **0 errores/0 warnings**; Vite compila con aviso previo de chunk >500 kB, no suprimido; `npm-build-asm-final.log` |
| Host | `npm run tauri build -- --debug --no-bundle` aprobado, **sin ejecutarlo**; `host-asm-final.log` |
| UI headless | **592 checks compartidos + 14 teclado/intercalados = 606**, seis sesiones dark/light/main/reopen/restored, **cero requests externos**; `headless-asm-final.log` y su `results.json` |

Las regresiones históricas `*-complete`/`*-reconciled` (208 tests) y `*-no-mutex`/`*-repeated-final` se conservan. La suite final `*-asm-final` sí repite Rust debug/release, parser/CLI/contención PDF, migraciones reales v9, contratos, frontend, host sin lanzarlo y UI headless después del último cambio de producción. La migración v8 de la fila anterior es evidencia previa identificada como tal, no una nueva corrida con el backend ARM; las migraciones 001–009, parser/helper/lectores/corpus permanecen iguales. `final-audit.py`/`final-audit.json` comprueban conteos, ambas corridas de benchmark, contratos, matriz/tests, enlaces y manifiesto externo.

La revisión externa de implementación, **previa a la sincronización**, comparó los **291 archivos** del baseline con **302**: 29 modificados, once añadidos, **262 previos idénticos y ninguno eliminado**. `implementation-review.json`, `implementation-manifest.json` e `implementation.diff` reflejan documentos/DTOs reconciliados de esa etapa. Home y sus artefactos, UI/host salvo contratos generados aditivos, código de búsqueda de notas, lectores/workers/formatos previos, migraciones **001–009**, fixtures PDF, specs principales y archivos históricos conservaban hashes; la sincronización posterior modifica sólo las tres specs descritas en el cierre siguiente. **Excepción de compilación explícita:** Cargo.toml del core activa `sha2/asm` en aarch64 y Cargo.lock añade sólo sha2-asm 0.6.4; el resto de dependencias/lockfiles se conserva. El backend también acelera hashes previos sin cambiar sus bits. Los cambios en tests históricos actualizan la versión vigente/fixtures de migración o excluyen las dos variantes aditivas de fingerprints fijados; no cambian esos fingerprints ni retiran assertions. No hay datos personales, modelos, credenciales o backups nuevos en el inventario.

Matriz individual contrastada con código y logs siguientes; sintaxis Python, enlaces locales y whitespace de líneas nuevas comprobados. Antes del archivo, `openspec validate --all --strict` validó **14/14 elementos documentales**, no comportamiento ni rendimiento. **19/19 tareas verificadas; ningún bloqueo del recorte conocido**. El cierre siguiente sincroniza y archiva sólo este cambio; Home permanece 9/17 y no se declaran satisfechas toda 5a ni las dependencias completas de 5b.

### Resultado final de revisión de implementación

| Dimensión | Resultado |
| --- | --- |
| Completitud | 19/19 tareas y 15 requisitos con implementación/evidencia; 40 escenarios trazados |
| Corrección | Matriz contrastada con los logs finales; verificación exacta/canónicos, CAS, rollback, cuotas, fuentes, contratos y ambos gates comprobados |
| Coherencia | Decisiones de diseño/ADR implementadas; excepción de dependencia y compatibilidad hash v1 documentadas; Home/lectores/migraciones previas conservados |

No se detectaron hallazgos críticos ni divergencias pendientes dentro de este recorte. `final-integration-review.diff` registra la revisión de catálogo/dispatch/CLI, v10/registry/backup y la única dependencia añadida; `final-audit.json` y `implementation-review.json` resumen controles automáticos y revisión. No se omitieron checks exigidos para este recorte. Aceptación nativa de Home, otras arquitecturas, todos los corpora/cachés, distribución y futuros componentes siguen **fuera de esta conclusión**, no se declaran probados. Persiste el aviso previo de bundle >500 kB, sin suprimirlo.

## Synchronization and Archive

Cierre del 14/10/2026, schema `spec-driven`. Raíz confirmada con `openspec context --json`: este repositorio. Los tres deltas se tomaron exclusivamente de `status --change add-local-attachment-search --json`; las reglas de specs se consultaron antes de escribir. La comparación íntegra verifica descripción y escenarios, no sólo nombres/conteos:

| Capability | Añadidos | Reconciliados | Escenarios contrastados | Requisitos previos ajenos intactos |
| --- | ---: | ---: | ---: | ---: |
| core-operations | 3 | 0 | 8 | 19 |
| local-text-search | 9 | 0 | 26 | 3 |
| local-workspace | 3 | 1 | 11 | 8 |

Son **15 requisitos añadidos/40 escenarios nuevos y un requisito reconciliado/cinco escenarios heredados** (16/45 bloques en los deltas). La spec anterior consideraba futura cualquier versión posterior a v9 y terminaba las migraciones v7/v8 en v9. Se reconcilia `Evolución recuperable a cola local` con la versión vigente, preservando los cinco escenarios y sus garantías, sin cambiar código ni migraciones históricas. La matriz adicional al final identifica la evidencia.

Verificación focalizada fresca, sin red, con el mismo toolchain/backend final:

```sh
cargo test --offline --locked -p noteka-core migration
cargo test --release --offline --locked -p noteka-core migration
python3 -B scripts/test-pdf-migration.py --profile debug --baseline-cli "$BASELINE_V8"
python3 -B scripts/test-pdf-migration.py --profile release --baseline-cli "$BASELINE_V8"
```

**15 tests de migración por perfil y dos estados CLI v8 por perfil aprobados**: preservación byte a byte, rollback/backup, cola/ownership/configuración futura, índice vacío, cliente anterior sin escrituras y rechazo de recibo viejo. `$BASELINE_V8` es el binario debug independiente conservado por el cierre PDF, no un CLI reconstruido desde v10. La evidencia v8 de la tabla de regresión anterior sigue siendo histórica; estas cuatro corridas son posteriores al backend ARM. No se repiten benchmarks/UI porque desde la aceptación final sólo cambian documentos y ubicación de artefactos.

Antes de mover el cambio: **12/12 specs y 14/14 elementos documentales estrictos aprobados**, 19/19 checks completos y ningún artefacto pendiente. Archivo ejecutado con `openspec archive add-local-attachment-search --skip-specs --yes --json`, conservando validación: `--skip-specs` evita una segunda escritura automática de specs **ya sincronizadas y comparadas**, no omite su sincronización. El resultado `specsUpdated=false` describe sólo esa invocación. Destino: [`2026-10-14-add-local-attachment-search`](../openspec/changes/archive/2026-10-14-add-local-attachment-search/proposal.md).

Después del archivo: **12/12 specs y 13/13 elementos estrictos aprobados** (doce specs y sólo Home activo 9/17; el cambio archivado deja de contar como activo). Auditoría 302→302 archivos: siete reubicados, diecisiete con contenido documental cambiado y 285 idénticos al mapear la reubicación; ningún archivo inesperado añadido o eliminado. **195 enlaces locales comprobados**, sin destinos faltantes ni whitespace nuevo incorrecto; comparación completa de los deltas repetida sobre el destino archivado.

Auditoría externa separada: `noteka-attachment-search-close-EKtnFG/`, con snapshot/manifiesto de los 302 archivos verificados, `sync-verification.json`, `sync.diff`, `migration-unit-{debug,release}.log`, `migration-v8-{debug,release}.json`, `archive-result.json`, `closure-audit.py`/`closure-audit.json`, `closure-manifest.json`, `closure-full.diff` y validaciones finales. README, guías, roadmap/contexto y enlaces del cambio archivado se reconcilian con este cierre. Las otras nueve capabilities, once cambios archivados previos, Home y todo código/tests/lockfiles conservan sus hashes respecto de la implementación verificada. No hay commit, push, publicación, modelos ni activación remota.

## Incidencias corregidas durante implementación

- Primer intento de catálogo llamó `operations.describe` en lugar de `operations describe`; no es sintaxis CLI válida. El baseline se capturó con la segunda.
- Compilación inicial intentó leer INTEGER como u64 (rusqlite no lo admite); se usan enteros comprobados y conversión explícita.
- Fixture de corrupción que eliminaba `_idx` no dañaba realmente ese segmento pequeño: se sustituyó por eliminación real de leaf blobs, y se añaden cambios de postings conservando la copia canónica.
- Un test batch asumía v9 como current; ahora deriva current/future de SCHEMA_VERSION.
- El harness inicial ordenó por rowid una shadow WITHOUT ROWID y usó `backup_path` como input de restore. Corregidos a comparación de filas y contrato `backup`.
- Primera compilación del benchmark usó un nombre de enum inexistente para schema TEMP; ahora consulta PRAGMA temp explícito.
- Dos assertions históricas multilínea de migración aún esperaban v9; se ajustaron a SCHEMA_VERSION, sin retirar validaciones de preservación.
- Clippy encontró una conversión redundante y estilo en tests; se corrigieron sin allowances. Una comprobación fmt detectó una nueva assertion agregada durante un run; se reformateó y se repite sobre estado estable.
- Denegar toda red a los tests del puente bloqueaba también loopback (`listen EPERM`); se ejecutan con HTTP local, manteniendo subprocesses CLI con red denegada. No se cambiaron tests.
- El primer headless nuevo falló por path de socket Playwright >127 bytes. Sólo se usa un `PWTEST_SOCKETS_DIR` descartable corto bajo `/private/tmp` para ese socket (el temporal aprobado es demasiado largo); workspaces y resultados siguen en el temporal aprobado. La repetición `headless-short-socket.log` pasó **606 checks**, cero requests externos, sin ventana/perfil personal ni cambios a assertions.
- Revisión del guard de páginas: se declara antes de la transacción para que en error/unwind el rollback suceda antes de restaurar el límite, no sólo en la ruta COMMIT.
- Revisión de contratos nuevos: los DTOs de salida también rechazan campos desconocidos y publican schemas cerrados, no sólo inputs/localizadores. No cambia serialización persistente ni los 49 contratos anteriores; se añaden roundtrips y pruebas negativas de schemas/salidas.
- La primera assertion nueva de schema buscó AttachmentIndexPage en `$defs`, pero el enum con tag la inlina. Se verifica su schema propio (y las definiciones realmente referenciadas), además del roundtrip get; no se modifica la lógica para satisfacer una ubicación supuesta.

No se borraron logs fallidos ni se cambiaron assertions existentes de UI para ocultar resultados.

## Matriz de 15 requisitos / 40 escenarios

IDs S/O/W siguen `tasks.md`. A = `crates/noteka-core/src/attachment_search_tests.rs`; L = `attachment_search_limits_tests.rs` en la misma carpeta; C = `scripts/test-attachment-search.py`; M = migraciones reales dentro de C. Los nombres siguientes identifican tests/ramas concretos, no sólo el módulo. Cada test funcional A/L está aprobado en `workspace-{debug,release}-asm-final.log`; C/M en `attachment-cli-{debug,release}-asm-final.json` y assertions del harness. Estos son **15 requisitos/40 escenarios funcionales contrastados**; el rendimiento de 6.1 tiene su evidencia separada arriba. Los caps/deadline se prueban funcionalmente por separado, no se extrapola su p95 a todos los corpora posibles.

| ID | Scenario | Código / evidencia funcional |
| --- | --- | --- |
| S1.1 | Extracción no implica indexación | A `explicit_literal_index_noop_and_exact_multibyte_provenance`; índice vacío tras extracción en C |
| S1.2 | Indexación local de ambos extractores | A `pdf_page_text_and_no_text_have_distinct_coverage_and_typed_locators`; C elimina helper antes de put |
| S1.3 | Fuente o generación inadmisible | A `generation_fence_rejects_replacement_even_without_index_revision_change`, `changed_or_unsafe_sources_fail_without_mutation_or_refill`; admisión/CAS en attachment_search.rs |
| S2.1 | Dos indexadores compiten | C dos procesos con misma revisión, exactamente códigos 0/4 |
| S2.2 | Repetición sin duplicados | A `explicit_literal_index_noop_and_exact_multibyte_provenance`; C changed=false |
| S2.3 | Reset deliberado | A `logical_corruption_is_not_a_false_zero_and_reset_is_explicit`, `oversized_cursor_response_and_reset_between_preview_apply_are_explicit` |
| S3.1 | Filtros y orden reproducible | A `bounded_literal_queries_filters_and_fragment_boundaries`, `cursors_paginate_ties_are_scoped_and_expire_only_on_index_changes` |
| S3.2 | Operadores como texto | A `bounded_literal_queries_filters_and_fragment_boundaries`; C literal `cafe OR NEAR` |
| S3.3 | Límites y frontera de fragmentos | A `bounded_literal_queries_filters_and_fragment_boundaries` |
| S4.1 | Evidencia literal | A `explicit_literal_index_noop_and_exact_multibyte_provenance`; golden CRLF/multibyte en C |
| S4.2 | Evidencia PDF sin renderizador | A `pdf_page_text_and_no_text_have_distinct_coverage_and_typed_locators`; golden de página en C sin helper |
| S4.3 | Original cambiado después de indexar | A `changed_or_unsafe_sources_fail_without_mutation_or_refill`, `get_and_search_cursors_reject_tampering_alias_restore_and_lookahead_corruption` |
| S5.1 | Reemplazo confirmado | A `publication_invalidates_atomically_but_failed_attempt_keeps_prior_selection`; carrera real publicación/reset en C |
| S5.2 | Falla o cancelación del reemplazo | A `failed_utf8_and_pdf_publications_roll_back_selection_trigger_and_postings`, `publication_invalidates_atomically_but_failed_attempt_keeps_prior_selection` |
| S5.3 | Publicación compite con indexación | A `generation_fence_rejects_replacement_even_without_index_revision_change`; CAS revision/generation bajo Immediate |
| S6.1 | Páginas estables | A `cursors_paginate_ties_are_scoped_and_expire_only_on_index_changes` |
| S6.2 | Cambio de revisión entre páginas | A `cursors_paginate_ties_are_scoped_and_expire_only_on_index_changes`; control/trigger + Cursor::offset |
| S6.3 | Cursor ajeno o manipulado | A `get_and_search_cursors_reject_tampering_alias_restore_and_lookahead_corruption`; C restauración/ruta distinta |
| S7.1 | Cobertura parcial y texto ausente | A `pdf_page_text_and_no_text_have_distinct_coverage_and_typed_locators`, `get_coverage_future_raw_configs_and_expired_running_are_pure_and_filtered` |
| S7.2 | Configuración futura conservada | A `get_coverage_future_raw_configs_and_expired_running_are_pure_and_filtered`; M compara todas las filas raw |
| S7.3 | Adjunto compartido y entidad archivada | A `shared_archived_and_removed_references_never_become_index_owners_or_filters` |
| S8.1 | Copia o posting inconsistente | A `logical_corruption_is_not_a_false_zero_and_reset_is_explicit`, `corruption_of_canonical_locators_postings_extras_and_bm25_stats_fails_closed`, `repeated_fragment_verification_rejects_hidden_postings_and_keeps_source_checks`; L `copy_stream_checks_complete_rows_types_and_rowids`, `repeated_postings_are_exact_order_independent_and_detect_duplicate_missing_pairs` |
| S8.2 | Reconstrucción sin parser | C elimina helper; A `logical_corruption_is_not_a_false_zero_and_reset_is_explicit`, `put_and_reset_failures_roll_back_data_revision_and_ddl` |
| S8.3 | Lectura durante trabajo interrumpido | A `get_coverage_future_raw_configs_and_expired_running_are_pure_and_filtered`; sin recovery en apertura/lecturas |
| S9.1 | Saturación del índice | A `exact_entry_fragment_and_utf8_byte_caps_are_independent_without_eviction`; bordes independientes y +1 |
| S9.2 | Deadline, contención o respuesta excesiva | A `put_deadline_and_writer_contention_do_not_confirm_or_retry`, `oversized_cursor_response_and_reset_between_preview_apply_are_explicit`; L budget/page guard |
| O1.1 | Catálogo y compatibilidad | A `all_49_v9_input_output_contracts_remain_byte_identical`; CLI tests de paridad; contract-comparison.json |
| O1.2 | Entrada estricta por acción | A `strict_dtos_and_read_only_main_with_writable_temp`; formas/exit codes en C |
| O1.3 | Estado paginado | A `get_and_search_cursors_reject_tampering_alias_restore_and_lookahead_corruption`; get ordenado en attachment_search.rs |
| O2.1 | Workspace que requiere migración | M v9 rechazado por ambas operaciones sin cambios; preflight soporta sólo current y rechaza futuro |
| O2.2 | Errores diferenciables | C códigos API/CLI; A CAS/fuente/deadline/contención/corrupción; not_found usa job_not_found/generation_not_found |
| O2.3 | Lectura sin permisos de escritura | A `strict_dtos_and_read_only_main_with_writable_temp`; main READ_ONLY rechaza DELETE en transacción y TEMP sí permite INSERT |
| O3.1 | Intento de indexar desde preview | extraction_tests `extraction_operations_are_rejected_as_batch_steps_and_checks_before_any_effect` ampliado a index/search; C prueba formas válidas no admitidas |
| O3.2 | Escritura derivada entre preview y apply | A `backup_roundtrip_and_batches_keep_current_index_without_original_hash_changes`, `oversized_cursor_response_and_reset_between_preview_apply_are_explicit` |
| W1.1 | Migración de estados reales v9 | M poblado UTF-8/PDF y running/resultado previo/config futuro; compara filas y backup restaurado por CLI v9 |
| W1.2 | Falla y cliente anterior | A `schema_empty_registry_and_migration_rollback_preserve_v9`; M cliente v9 ante v10 compara hashes de todos los archivos |
| W2.1 | Restaurar y buscar offline | C roundtrip mixto UTF-8/PDF/no_text sin helper, igualdad de estado/cobertura/hits y cursor rechazado por ruta |
| W2.2 | Backup concurrente o dañado | C carreras de procesos con backup/reset/publicación; A `backup_roundtrip_and_batches_keep_current_index_without_original_hash_changes`, backup adulterado con hash de manifiesto actualizado |
| W3.1 | Huella y preservación en lotes | A `backup_roundtrip_and_batches_keep_current_index_without_original_hash_changes`, `oversized_cursor_response_and_reset_between_preview_apply_are_explicit`; registry 001–010 |
| W3.2 | Presupuesto real y recibo viejo | A `full_snapshot_cap_denies_put_but_reset_preserves_originals_without_vacuum`; L asignación tardía FTS/COMMIT/SQLITE_FULL; M recibo v9 rechazado |

### Reconciliación de migración heredada: W4 / cinco escenarios

No añade cinco comportamientos nuevos: conserva los escenarios existentes y corrige su referencia a la versión vigente. Logs frescos `migration-unit-{debug,release}.log` y `migration-v8-{debug,release}.json` en la auditoría de cierre, separados de la matriz funcional nueva anterior.

| ID | Scenario heredado | Código / evidencia |
| --- | --- | --- |
| W4.1 | Migración desde SQLite v7 | workspace::tests `extraction_migration_failure_v7_and_success_preserve_all_unknown_originals`: documentos/AST/payloads desconocidos, originales, control pausado y cola vacía; `010.sql` crea entries/FTS vacíos |
| W4.2 | Falla y recuperación desde versión previa | Mismo test v7: rollback, backup validado/restaurado y apertura compatible hasta SCHEMA_VERSION; tests de fallas de migración v2–v9 |
| W4.3 | Versiones anteriores y futuras | Mismo test v7: v11 rechazado y archivo idéntico; tests de migración de versiones anteriores y `migration_backup_originals_and_future_schema_are_preserved` |
| W4.4 | Migración desde SQLite v8 con procesamiento | `v8_migration_is_additive_transactional_and_does_not_recover_running_jobs` y dos estados reales del CLI v8 por perfil: filas previas iguales, control/ownership/config raw conservados, estructuras PDF e índice nuevos vacíos |
| W4.5 | Rollback de software sin pérdida posterior | Evidencia histórica v8→v9 del cierre PDF preservada; nueva migración v8→v10 confirma cliente v8 sin escrituras, backup previo independiente y rechazo de recibo viejo; W1 verifica además cliente v9 ante v10 |
