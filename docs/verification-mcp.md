# Verificación de MCP local

## Estado

[`add-local-mcp`](../openspec/changes/archive/2026-10-15-add-local-mcp/proposal.md), implementado y archivado el **15/10/2026**: **22/22 tareas verificadas**, dos deltas sincronizados y contrastados con main. Scope, servidor, cliente y regresión integrada aprobados: nuevo recorrido exacto FTS5, query p95 **134,96–204,81 ms** en dos rondas consecutivas (<=300 ms). Cuatro de seis mediciones previas fallaron y se conservan; no se aceptó una excepción. Home conserva 9/17 sin cambios ni aceptación nativa nueva.

## Baseline independiente — tarea 1.1

Raíz comprobada con OpenSpec 1.14.1: `/Users/huascar321/Projects/noteka`; schema spec-driven, instrucciones apply ready y 22 tareas inicialmente sin marcar. Los cinco archivos contextFiles de apply fueron leídos junto con la propuesta/diseño y reglas del proyecto.

Auditoría externa aprobada: `/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-mcp-apply-evTrgS/`. Incluye `env.sh`, copia `baseline/`, SHA-256 de los **309 archivos** en `baseline.json`, binarios CLI/helper **debug y release** independientes bajo `bin/`, `toolchain.txt` y `catalog-baseline.json` completo.

- Los 309 hashes coinciden con el manifiesto final de la planificación: no había modificaciones intermedias del usuario.
- CLI release independiente: `sandbox-exec -p '(version 1)(allow default)(deny network*)' "$MCP_AUDIT/bin/release/noteka" --json operations describe < /dev/null`; no usa workspace ni abre ventanas.
- Datos completos del catálogo iguales a la captura de planificación, no sólo cantidad de operaciones: **51 inputs y 51 outputs**, API v1 y baseline SQLite v10.
- No hay commit inicial; el manifiesto/copia independiente es la referencia de revisión. No se hizo commit, push, publicación ni acceso a datos personales.

| Entorno observado | Valor |
| --- | --- |
| Equipo | Apple M1 Max, 68.719.476.736 bytes / 64 GiB |
| Rust / Cargo | 1.99.0; aarch64-apple-darwin |
| Node / npm | 22.22.3 / 10.9.8 |
| Toolchain/variables reproducibles | `source "$MCP_AUDIT/env.sh"`; hereda entorno offline/zlib del cierre anterior |

## Evidencia fresca de ejecución

Todos los logs siguientes están bajo `$MCP_AUDIT`, la raíz externa anterior. Tooling se descargó aparte; pruebas Rust/SDK/raw ejecutadas con `sandbox-exec -p '(version 1)(allow default)(deny network*)'`, `--offline --locked` para Cargo. Sin pantalla, LLM ni workspace personal.

| Área | Prueba/log y resultado |
| --- | --- |
| SDK/dependencias (1.2) | `tree-features-final.txt`, `metadata-final.json`, `mcp-normal-build-licenses.json`; rmcp 3.5.1 Apache-2.0/MSRV declarado 1.88, Tokio 1.53.2 MIT/1.71, tokio-util 0.7.20 MIT/1.85. TS client/core 2.3.1 Apache-2.0, Node >=20; Ajv 8.20.0 MIT, dev-only. Rust real 1.99.0, Node 22.22.3. Cierre normal/build de 94 paquetes: **91 externos con licencia declarada**, ninguno GPL/AGPL; tres locales (pdf-protocol MIT, core/MCP sin licencia de distribución declarada). No se publica nada. No features servidor HTTP/auth; default-members intactos. |
| Preservación | `catalog-final7.json` completo idéntico a `catalog-baseline.json`: inputs/outputs/defs/errores/límites y root properties, 51 operaciones. Ningún paquete previo desaparece/cambia de versión en Cargo; seis entradas nuevas (incluido crate local). Ningún paquete npm previo cambia; 18 añadidos entre cliente/validador. Frontend `frontend-final7.log` aprobado, sin cliente en bundle. |
| Scope (2.1–2.4) | `workspace-{debug,release}-final7.log`: **9/9** tests scoped en cada perfil. 51 enums, acciones mixtas, counts 23/43/44, run solo inválido, permiso antes de abrir, versiones/ruta no UTF-8, overrides/bytes/profundidad, main READ_ONLY y WAL confirmado. Identidad cambiada incluso después de preflight privado y sobre conexión/snapshot real; origen/CAS, future jobs/claim vencido/índice dañado sin recuperación ni reparación. Test adicional ejecuta **23/23 lecturas exitosas**, incluidas preview y fragmentos UTF-8/PDF indexados: tablas/revisiones/originales iguales antes/después. |
| Discovery/schemas (3.1–3.3) | Unitarios en workspace final7 y SDK final7: refs locales/root object/nombres únicos/51 mapeos, todos los schemas compilados con Ajv 2020-12, páginas wire <=8, 23/43/44 herramientas completas, cursores de otra política/inválidos/excesivos, recurso estático sin ruta ni URIs genéricas. Casos de éxito/error reales validados sin coerción/defaults. |
| SDK real (5.1–5.3) | `sdk-{debug,release}-final7.log`: **17/17 por perfil**, modern pin, legacy explícito 2025-11-25 y auto; nueve combinaciones era/política. Discovery/resource/DTO/errores/denegación incluso después de leer instrucciones hostiles, preview/apply BOB **145,69**, IDs exactos, lote inválido/recibo obsoleto/CAS, documentos/relaciones/colecciones/records/fórmula/vista/template/Home/workflows. UTF-8 y PDF real con procedencia/index put/reset/search; helper ausente en ambas eras. Mensajes en frontera stdio y archivos `wire-*.json` externos; no atribuye acceso de una cuenta ChatGPT. |
| Transporte raw/concurrencia (4.x/5.4) | `wire-{debug,release}-final7-round{1,2,3}.log`: **13 grupos por ronda/perfil**, seis rondas aprobadas. Flags/versiones/pureza física, metadata/estado inválidos, modern y handshake legacy en bytes, arguments no objeto (incluido null), línea exacta 4 MiB/+1/EOF sin newline, stdout sin lector, error de entrega oversized **después de commit real**, writer ocupado/cancelación sin liberar cupo ni retry, reemplazo de identidad vivo, CLI cancel operativo, EOF/broken output/helper retenido reaped y helper ausente. |
| Cancelación/budgets (4.4–4.6) | Unitarios en workspace final7: barreras antes de despacho/después de commit, abort de waiter sin liberar permiso síncrono, supresión de salida pendiente, cap exacto/+1 y framing escapado >16 MiB, salida acotada/timeout virtual de 5 s. Cuatro requests admitidos antes de tareas SDK, cancel con canal lleno, initialized legacy ocupando un slot sin cerrar por saturación, EOF durante read síncrono retiene permiso y descarta entrega. |
| Rust integrado (6.1) | `workspace-{debug,release}-final7.log`: **235 tests aprobados por perfil**, siete ignorados intencionales preexistentes, cero fallos. `fmt-final7.log`, `check-final7.log` workspace/all-targets y `clippy-final7.log` con **-D warnings**, aprobados; `build-{debug,release}-final7.log` y `build-benchmark-final7.log`, offline/red denegada. Cinco tests nuevos de API/oráculo/fallback/corrupción/deadline; todos los anteriores conservan sus asserts. Examples se compilan en all-targets; schema/parser/containment/benchmark se ejecutan por sus harnesses. |
| Regresiones independientes | `pdf-{parser,cli,containment}-{debug,release}-final7.log`: **235 parser + 235 CLI + 12 contención por perfil**; originales, manifest, layout y lectura/restauración sin helper. `attachment-concurrency-{debug,release}-final7.log`: **20 rondas CLI por perfil** más tres checks de migración/compatibilidad, backup/put/reset/editor concurrente; migración v9/rollback/admisión en tests Rust. `migration-v8-{debug,release}.log`: **dos estados reales por perfil** desde CLI v8 independiente, futuros/filas conservados y cliente viejo sin escrituras (corrida anterior conservada; el recorrido FTS5 no cambia migraciones). |
| Frontend/host/headless | `editor-final7.log` **13/13**, `batches-final7.log` **4/4**, `bridge-final7.log` **3/3**; `frontend-final7.log` contratos generados/check/build aprobados. `host-final7.log`: Tauri no-bundle **compilado, nunca lanzado**. `headless-final7-short-sockets.log`: **606/606**, seis sesiones, keyboard/interleaving, cero requests externos (`network-results.json`) y sin perfil personal. El puente sólo necesita loopback; cada CLI corre con deny network*. No es un navegador bajo denegación total de red ni prueba de IPC nativo. |

Los siete ignores Rust son cuatro mediciones explícitas (`release_attachment_index_resources`, `release_batch_resources`, `release_extraction_resources`, `release_pdf_resources`), un diagnóstico comparativo (`attachment_posting_diagnostics`) y dos fixtures de hijos ejecutadas por sus padres. No se añadieron ignores para pasar este cambio. El benchmark de adjuntos se invoca explícitamente; no se atribuyen nuevas mediciones de extracción/lotes/PDF a los tests ordinarios. Persiste el warning Vite de chunk >500 kB y el aviso experimental Node `node:sqlite` del harness; no se subieron umbrales ni silenciaron.

### Reproducción de checks finales

Con las dependencias ya preparadas y el entorno de [desarrollo](development.md), sin abrir ventanas:

```sh
source "$MCP_AUDIT/env.sh"
cargo fmt --all --check
sandbox-exec -p '(version 1)(allow default)(deny network*)' cargo check --offline --locked --workspace --all-targets
sandbox-exec -p '(version 1)(allow default)(deny network*)' cargo clippy --offline --locked --workspace --all-targets -- -D warnings
for profile in debug release; do
  extra=; [ "$profile" = release ] && extra=--release
  sandbox-exec -p '(version 1)(allow default)(deny network*)' cargo test $extra --offline --locked --workspace
  sandbox-exec -p '(version 1)(allow default)(deny network*)' cargo build $extra --offline --locked -p noteka-cli -p noteka-mcp -p noteka-pdf-worker
  NOTEKA_CLI="$PWD/target/$profile/noteka" NOTEKA_MCP="$PWD/target/$profile/noteka-mcp" \
    sandbox-exec -p '(version 1)(allow default)(deny network*)' npm run test:mcp
  for round in 1 2 3; do
    sandbox-exec -p '(version 1)(allow default)(deny network*)' python3 -B scripts/test-mcp-transport.py --profile "$profile"
  done
done
```

Los comandos PDF/concurrencia, generación/frontend/host y headless son los harnesses anteriores **sin modificar**, descritos en [desarrollo](development.md), [PDF](verification-pdf-extraction.md) e [índice](verification-attachment-search.md). Los logs **final7** sustituyen los totales intermedios 226/227/230 de corridas previas, no se suman a ellos.

Después de retirar la proyección experimental se repitieron fmt/check/Clippy (`*-resume20.log`) y workspace debug/release: **230 por perfil**, siete ignores previos, cero fallos (`workspace-debug-resume20.log`, `workspace-release-resume20-retry.log`). La primera cadena de comandos agotó el timeout externo de 120 s durante compilación release; su log incompleto se conserva como `workspace-release-resume20.log`, **no** se cuenta como aprobado. La repetición con plazo suficiente completó la suite. OpenSpec estricto de esa etapa: **14/14 items**, doce specs/dos cambios; apply informaba **20 completas / 2 pendientes**. Esa validación documental no aprobaba el target; no es el estado final7 ni el cierre posterior.

## Fallos encontrados y conservados

- `cargo-fetch.log`/`npm-install.log`: dependencias nuevas no cacheadas en modo offline; preparación online posterior sin activar proveedores/runtime remoto.
- `scoped-tests-first.log`: patrón Get requería `{}`; corregido, sin retirar asserts.
- `sdk-first/second` y `sdk-modern-probe`: el SDK agrega páginas y reasigna onmessage durante discovery. Corregido el harness para medir páginas/mensajes wire, no reinterpretar el aggregate como una página ni perder respuestas modernas. Cierre de cliente registrado incluso si falla setup.
- `wire-first/second`: sondeo/cierre normal no debe loguearse como fallo; además un cliente puede reutilizar su ID justo después del newline. Se retira el slot antes de publicar el delimitador, con comparación por identidad del estado para no retirar un request nuevo. Se agregó repetición de ID con request_ids core únicos.
- `sdk-fourth/sixth`: fixtures usaban alias CLI incorrecto `pdf_fragments` y omitían clasificación obligatoria del tag; se corrigieron las fixtures, no el contrato.
- `sdk-seventh`: **bug real de entrega**, no tolerancia numérica: reparsear text con serde_json podía mover un score BM25 un ULP y romper equivalencia structuredContent/text. Se construye structuredContent directamente desde Response, sin reparseo; `sdk-eighth` pasa exacto sin normalizar scores.
- `clippy.log`: usar is_multiple_of en cursor; corregido sin suprimir lint.
- `bridge-final.log`: sandbox externo negó loopback. `bridge-loopback-final.log`: sandbox externo seguía impidiendo el sandbox anidado que el bridge ya aplica a cada CLI. Se repite con el harness previo intacto: loopback para UI, CLI deny network*, rutas/orígenes de navegador controlados y reporte de cero requests externos. No se retiran asserts ni se habilita pantalla.
- `scoped-success-all-reads.log`: fixture tomó property.define como wrapper inexistente; usar su dato Property directo corrige el input record.inverse, sin cambiar DTOs ni validadores. `workspace-debug-final3.log`: fixture renombró la carpeta mientras SQLite seguía abierto; se corrigió la prueba de ruta no UTF-8 sin alterar la base activa. La entrada scoped rechaza ruta de entrada y ruta canónica no UTF-8 antes de serializar un Request.
- `wire-null-regression-before-fix.log`: **bug real de admisión**, rmcp deserializa `arguments:null` como Option ausente. Se rechazan explícitamente null/scalars/arrays antes de esa normalización; ambas eras pasan el probe final5. Sólo omitir arguments produce el objeto vacío permitido.
- `mcp-benchmark-round1.json`: el sandbox negó ejecutar `ps`; no es una medición válida. El sampler queda fuera, pero **cada proceso producto medido** (CLI/MCP/core) corre con deny network*. Round2/round3 y final5 contienen mediciones válidas, no se elimina el intento fallido.
- `workspace-debug-final6.log`: el nuevo recorrido auxiliar asumía que el vocabulario siempre provenía del FTS real; el test heredado también admite un stream materializado independiente. Se conserva el test sin cambios y se comprueba la procedencia exacta antes de elegir la API; el stream independiente sigue por el comparador íntegro anterior. `postings-full-tests-debug.log` aprueba **134 unitarios core**, siete ignores, tras la corrección; cinco tests nuevos contrastan API/oráculo, corrupción oculta, fallback, repetición y deadline. La cadena final6 se interrumpió en ese fallo y no certifica release/SDK/benchmarks.
- `headless-final7.log`: Playwright rechazó su directorio de sockets por superar 127 bytes; el browser no llegó a abrir la sesión. Se repite el harness intacto con PWTEST_SOCKETS_DIR corto bajo `/private/tmp`, **sólo para sockets**; fixtures/resultados siguen en el temporal externo aprobado, sin perfil personal ni ventanas nativas. No se cuenta el intento fallido como ejecución de los 606 checks.

## Mediciones release — tarea 6.2

Equipo/toolchain de la tabla inicial; una nota sintética con documento de **10.240 bytes ASCII**, workspace v10, sin otros registros ni índice. **5 warmups + 20 muestras por caso**; p95 por nearest-rank (muestra 19 ordenada). Sin descargas, UI, LLM ni procesos de compilación simultáneos. Comando: `cargo build --release --offline --locked -p noteka-core --example mcp_benchmark`, luego `python3 -B scripts/benchmark-mcp.py` (deniega red individualmente a los binarios; sampler RSS externo). Captura final del binario actual: `mcp-benchmark-final7.json`, más `results.json` y fixture externos identificados allí; repeticiones previas conservadas: `mcp-benchmark-round{2,3}.json` y `mcp-benchmark-final5.json`.

| Caso | MCP mediana / p95 ms | CLI proceso mediana / p95 ms | Core histórico mediana / p95 ms | Core scoped mediana / p95 ms |
| --- | --- | --- | --- | --- |
| entity.get | 2,82 / 3,59 | 20,56 / 23,79 | 2,64 / 3,32 | 2,33 / 3,14 |
| entity.list | 3,11 / 4,41 | 22,75 / 25,65 | 2,60 / 2,83 | 2,52 / 4,38 |
| home.query notes_all | 3,69 / 5,24 | 22,02 / 27,33 | 2,98 / 3,48 | 2,99 / 3,79 |

Arranque + server/discover: mediana **29,76 ms**, p95 **32,36 ms**, máximo **40,83 ms**. RSS residente muestreado tras discovery/casos **23.969.792–26.001.408 bytes**; no high-water, memoria exclusiva ni pico de una extracción. CLI incluye spawn/preflight/JSON; core in-process no incluye wire y su apertura histórica conserva sus pragmas. Diferencias entre medianas MCP y scoped core: **+0,49/+0,59/+0,70 ms** respectivamente; son comparaciones secuenciales con ruido/caché, **no aislamiento causal del coste de transporte** ni garantía universal de rendimiento. Las cuatro corridas válidas y sus outliers se conservan.

Tres rondas raw **final7** por perfil dan estos rangos observados; **no son p95 de 20 muestras**:

| Probe | Debug ms | Release ms |
| --- | --- | --- |
| Frame 4 MiB fragmentado → respuesta | 116,69–132,38 | 33,91–41,99 |
| Exceso +1 sin newline → salida del proceso | 84,78–94,32 | 31,99–38,72 |
| Cancelación recibida + segunda llamada busy con core retenido | 0,62–0,89 | 0,27–0,44 |
| Sin lector de stdout → cierre | 5.087,41–5.124,58 | 5.030,35–5.075,31 |
| EOF durante espera de escritor → cierre | 1.947,71–1.979,85 | 1.974,69–1.990,10 |
| EOF durante helper retenido → cierre/helper reaped | 3,86–8,98 | 8,77–19,22 |
| Broken pipe durante helper retenido → cierre/helper reaped | 8,66–9,09 | 4,00–9,02 |

La espera de escritor sigue configurada en **2.000 ms**; la medida de cierre incluye scheduling y IO, no aumenta ese timeout. El cierre ante cliente sin lectura refleja el timeout stdout de 5 s; el límite global sigue siendo 30 s desde la primera escritura bloqueada, no 5+30. Se verifican por separado barreras deterministas y deadlines virtuales.

### Target heredado de búsqueda de adjuntos

Repetido `attachment_search_tests::release_attachment_index_resources` con red denegada, offline/locked, release, un thread; logs `attachment-benchmark-round{1,2}.log` y `attachment-benchmark-summaries.json`. **Mismo corpus de 8 MiB / 8 entradas / 2.048 fragmentos**, 5 warmups + 20 muestras/caso, comprobación completa de integridad/postings/source, sin cambiar caps/algoritmo/TEMP ni retirar asserts. No hay LLM, HTTP ni UI en la medición.

| Consulta | P95 ronda 1 ms | P95 ronda 2 ms |
| --- | --- | --- |
| query-hit | 241,19 | 236,84 |
| query-zero | 232,24 | 235,27 |
| query-filter | 234,12 | 283,98 |
| query-page | 278,78 | 240,01 |
| query-page-next | 298,54 | 238,27 |

Estas dos corridas iniciales aprobaron **<=300 ms**; p95 máximo de lock **18,30/18,79 ms** frente a <=150 ms; espera máxima p95 del editor independiente **22,70/22,69 ms**. El caso de 298,54 ms dejaba poco margen. **Una tercera corrida con el script completo falló el target** (`attachment-full-benchmark-final.log`): hit 305,47 ms, filter 359,84 ms, page 352,74 ms, page-next 371,65 ms; zero 279,76 ms. Todos los resultados funcionales y el lock (máximo p95 19,74 ms) pasaron, pero el exit 0 del script no certifica latencia. Se conservó el reporte y entonces se mantuvo 6.2 abierta. Se observó load average 19,13 en 10 CPUs lógicas; eso no demuestra por sí solo la causa. El diagnóstico pareado y la aceptación final posterior se detallan abajo; no se detuvieron procesos ajenos ni se rebajaron objetivos.

La tercera corrida registra RSS high-water del proceso de tests **44.335.104 bytes**, TEMP **lógico** 11.489.280 bytes, SQLite 21.524.480 bytes y WAL 5.697.992 bytes. El scratch físico **21.254.144 bytes** es evidencia **histórica**, no nueva medición MCP; este cambio no modifica sus caps/formato. No confundir estos resultados con los 214,20–231,07 ms del cierre anterior ni con latencia end-to-end MCP.

Diagnóstico externo conservado en `paired-attachment-diagnostic.py/.log` y su `results.json`: mismo workspace sintético de 8×1 MiB/2.048 fragmentos, 5 warmups/20 muestras, CLI baseline v10 independiente frente al actual, orden alternado AB/BA y resultados de dominio **exactamente iguales**. Medianas CPU por caso entre ambos difieren hasta **3,30 ms**; el baseline también excede 300 ms en hit (p95 **317,80 ms** frente a **287,87 ms** actual). Esto no muestra una regresión causal de MCP ni prueba que toda la variación sea carga del host. **No sustituye el gate core.** Dos repeticiones planificadas del gate (`attachment-postdiagnosis-round{1,2}.log`, resumen con load/CPU/RSS) volvieron a fallar: hit **336,25 ms** y filter **316,10 ms** respectivamente; el resto de queries y ambos locks pasaron. No se repite hasta obtener un resultado conveniente ni se omiten esas corridas. El diagnóstico de posting marca **130,40 ms** en `same_repeated_postings` de **208,96 ms** de verificación en conexión real; esa lectura recorre **1.199.240 postings**, no una muestra.

Se evaluó una proyección SQL equivalente que trasladaba el check exacto `col='text'` a CASE para reducir una celda por fila en FFI, manteniendo términos/doc/offset, bitset y fallback completos. **No mejoró**: `attachment-diagnostic-projection.log` midió repeated **136,44 ms**, verificación **217,57 ms**; `attachment-projection-benchmark.log` falló hit p95 **307,33 ms** (el resto de queries y locks pasó). **Se retiró íntegramente ese experimento** y el verificador volvió entonces al baseline. Se conservaron las seis mediciones válidas hasta ese punto: dos aprobadas/cuatro fuera de objetivo; no se desactivaron checks ni se aceptó un target distinto.

### Recorrido exacto FTS5 — gates finales aprobados

[ADR 0015](decisions/0015-exact-fts5-instance-traversal.md) documenta el recorrido por API auxiliar pública FTS5, con fts5vocab(row), xInstCount/xInst/xRowid y bitset independiente por documento. Verifica todos los términos/instancias, columna/posición/cardinalidad y duplicados, sin modificar los checks de fuente/copias/estadísticas. Más de 256 términos o vocabulario no vinculado al FTS real usa el stream íntegro anterior; no hay caché entre snapshots. FFI aislado, statements/punteros/TEMP con ciclo de vida acotado y deadline cada 1.024 instancias.

Diagnóstico inicial `postings-api-diagnostic.log`: recorrido repeated **20,62 ms**, verificación en conexión real **102,66 ms**; no es una distribución p95. Benchmark exploratorio `postings-api-benchmark1.log` aprueba queries p95 **131,90–185,66 ms**, lock máximo p95 **19,93 ms**. Son resultados previos al check final de procedencia, no se usan como aceptación definitiva.

**Aceptación 6.2:** `postings-api-final7-round{1,2}.log` / `attachment-final7-summary.json`, dos invocaciones consecutivas del wrapper heredado **sin modificar**, con source final7 (`source-final7.json`), offline/locked/red denegada. Corpus y SHA-256 de origen iguales: `fd7657afedbf7018407b0dcc402f3ab2eb1e929737efd307d727e84987a21f4c`, 8×1 MiB/2.048 fragmentos, cinco warmups/veinte muestras. Ambas informan query_target_met/lock_gate_met=true y todos los resultados funcionales correctos.

| Consulta | P95 final7 ronda 1 ms | P95 final7 ronda 2 ms |
| --- | --- | --- |
| query-hit | 135,52 | 161,14 |
| query-zero | 134,96 | 174,50 |
| query-filter | 139,16 | 204,81 |
| query-page | 136,09 | 150,10 |
| query-page-next | 138,82 | 146,73 |

Lock máximo p95 **19,59/19,80 ms** frente a <=150 ms; espera máxima p95 de editor independiente **22,73/22,74 ms**. RSS high-water SELF **42.926.080/43.139.072 bytes**, TEMP lógico **11.489.280/11.493.376 bytes**, no scratch físico ni memoria exclusiva MCP. Se conserva también el outlier hit de la primera ronda (**459,55 ms máximo**, p95 135,52 ms): cumplir p95 no significa que toda llamada cumpla 300 ms. No se elimina esa muestra, ni se cambia el corpus, ni se aumentan caps, ni se acepta una excepción. El resultado corresponde a este corpus/equipo; el fallback íntegro para vocabularios/textos diferentes mantiene integridad, no una promesa universal de latencia.

## Revisión de preservación — tarea 6.3

`preclose-preservation-review.json`: catálogo completo CLI idéntico al baseline independiente, no sólo nombres/counts. Ninguna versión Cargo previa desaparece; las seis entradas añadidas son rmcp, futures, pastey, tokio-macros, tracing-attributes y el crate local. Las 18 entradas npm añadidas son dev-only cliente/validador y transitivas: cada entrada previa sigue idéntica. Frontend/host, Home, migraciones 001–010, fixtures, generación de contratos y asserts/scripts de regresión anteriores conservan sus bytes del baseline. La proyección experimental fue retirada; el recorrido auxiliar posterior sí añade `attachment_postings.rs` y una selección de algoritmo en storage, documentados en ADR 0015.

Revisión del diseño contra código: permisos centrales antes del opener; dispatcher genérico único, scope no serializable y origen host; conexión/snapshot/lock especializados comprobados antes de efectos; sin CREATE/migrate/recovery en lectura. Transporte sin SQL de dominio, sin apertura por rutas aportadas, IDs generados/retirados sin interferir reutilización, JSON exacto en ambos outputs; cupo síncrono retenido y salidas canceladas no enviadas. stdout sólo protocolo; errores/diagnósticos mínimos, sin argumentos personales ni stacktraces. Caps/eras/hints documentados coinciden con catálogo y pruebas. El cierre PDF observa la señal del host, no confunde cancelación MCP con extraction.cancel y se prueba con proceso retenido real.

Matriz siguiente: **15 requisitos / 45 escenarios**, de los cuales cinco son heredados de C4 y sus cuerpos permanecen idénticos (`inherited-scenarios-review.json`). No se rebajó ningún WHEN/THEN. Revisión antes de archivo (`review-prearchive.json`): **309→325 archivos**, 19 modificados/16 nuevos/cero borrados, **71 archivos protegidos** idénticos, hashes source-final7 intactos y **311** referencias relativas válidas. Ese informe precede la actualización de la guía de adjuntos; la revisión final completa se conserva en `review-final.json`/`diff-final.patch`/`manifest-final.json`. Se contrastan nuevamente las dos deltas al sincronizar y se preservan todas las otras capabilities/Home.

**Revisión final tras archivo:** **309→326 archivos**, 21 modificados/17 nuevos y siete traslados de artefactos sin pérdida; cero eliminaciones ajenas. **83 archivos protegidos** idénticos (incluidas las otras specs), **316 links relativos** válidos, source final7 intacto. Los cuatro bloques delta core y once MCP coinciden exactamente con main; core queda con 25 requisitos, MCP con once, y sus cinco escenarios heredados conservan los mismos cuerpos. `spec-sync-comparison.json` y `review-final.json` contienen las comprobaciones. `openspec-final-archived.json`: **14/14 items** estrictos, trece specs/un cambio activo (sólo Home **9/17**). Se conservan `.openspec.yaml` y los siete archivos del cambio; los dos deltas archivados permanecen intactos. El diff usa el manifiesto independiente, no un Git diff vacío.

## Matriz individual de requisitos y escenarios

Identificadores L1–L11/C1–C4: [checklist](../openspec/changes/archive/2026-10-15-add-local-mcp/tasks.md#traceability). Esta matriz no se basa en que un test tenga un nombre parecido: contrasta sus asserts con WHEN/THEN y los puntos de ejecución. Los logs finales indicados en el resumen de evidencia son la aprobación; los logs parciales previos se conservan como historia, no se suman como tests nuevos.

**Código:** `scoped` = `crates/noteka-core/src/scoped.rs`; `dispatch` = `crates/noteka-core/src/lib.rs:177–333`; `specialized` = `batches.rs:424–565` + `attachment_search.rs:35–111,238–271,351–365,438–490` dentro de core/src; `catalog`, `server`, `transport`, `main` = módulos de `crates/noteka-mcp/src/`.

**Pruebas:** `scope` = nueve tests `scoped_tests::*`; `admission/cancel/output` = tests unitarios de `transport`; `SDK` = `tests/local-mcp.mjs` (17 recorridos por perfil, ambos protocolos); `raw` = `scripts/test-mcp-transport.py` (proceso/SQLite/stdout bytes/helper retenido, tres rondas por perfil); `legacy core/CLI` = suites históricas íntegras en workspace debug/release; `UI` = 606 checks del harness headless previo, no aceptación nativa.

### local-mcp — 11 requisitos / 29 escenarios

| ID | Escenario completo | Implementación | Prueba / comprobación |
| --- | --- | --- | --- |
| L1 | Lectura offline con ventana cerrada | main/server/dispatch, sin servidor de red | SDK nueve combinaciones con deny network*, lectura real sin Tauri/LLM |
| L1 | Arranque y diagnóstico separados del protocolo | main argumentos/preflight; transport stderr fijo | raw startup puro/no cero/stdout vacío; SDK stderr vacío y stdout JSON válido |
| L2 | Formato anterior o futuro | scoped new/current + preflight privado | scope creación; raw v9/futuro/inconsistente/ausente con manifiestos iguales |
| L2 | Cambio de destino durante la sesión | scoped check/open + specialized conexión/transacción/snapshot | scope reemplazo antes/después de preflight, raw sesión viva contra reemplazo; ninguno de los dos dominios mutado |
| L2 | Sondeo sin efectos | constructor privado + catálogo en memoria | scope archivos y SDK/raw arranque/discovery/close con hashes físicos iguales |
| L3 | Lectura predeterminada y llamada directa | policy/operation_access antes de abrir | scope permiso con ruta movida y llamadas directas SDK/raw sin depender de tools/list; get mixto separado de set/put/reset |
| L3 | Escritura sin ejecución | allow_write != allow_run; sin scheduler | SDK 43 tools, enqueue+resume deja queued/intentos 0/generación null; run denegado |
| L3 | Ejecución explícita y flags incompatibles | Policy::new + run compartido | unit flags/scope; raw run-solo inválido; SDK UTF8/PDF con ambos y helper local |
| L3 | Lote no elude permisos | scope + batch_plan whitelist intacta | SDK preview read-only sin persistir, apply denegado, workspace.create anidado inválido; nota/metadata nunca modifican Policy |
| L4 | Catálogo completo y subset efectivo | catalog names/filtrado/resource | unit catálogo, SDK 23/43/44 y equality del catálogo CLI completo v1/51 |
| L4 | Filesystem y recursos excluidos | siete Access::Excluded, resource URI exacta | scope siete exclusiones todos los modos; SDK import/read/workspace sin permiso; raw file/URI/subscribe rechazados |
| L5 | Esquemas completos y acciones mixtas | catalog input/output_schema/rebase | refs unit y compilación Ajv 2020-12 de todos, validación get/set y root object; SDK éxitos/errores sin data |
| L5 | Entrada inválida sin efectos | DTO/límites core, checks antes de commit | scope campos/bytes/profundidad; raw no objeto/JSON; SDK revisión omitida/CAS/cursor/formula/documento/lote inválido y revisión intacta |
| L6 | Éxito y fallo de dominio | server envelope/text/isError | SDK compara exactamente ambos formatos, not_found/revision_conflict/success y datos CLI; sin tolerancia BM25 |
| L6 | Error de protocolo y permiso | SDK modelos/error + scoped denied | raw método/nombre/params/framing inválidos; SDK permission_denied en envelope validation_error, no fallo MCP |
| L6 | Origen e identificación confiables | main fija mcp, core genera UUID | raw ID JSON-RPC repetido 50 veces con UUID distintos; SDK metadata origin/request_id falsos no afectan historial mcp; overrides DTO rechazados |
| L7 | Cliente moderno sin handshake legacy | supported_protocol_versions + SDK lifecycle | SDK pin/auto y raw discovery sin initialize, metadata requerida y resultType completo wire |
| L7 | Cliente legacy y compatibilidad limitada | SDK initialize/initialized, dos versiones | SDK legacy y raw handshake/23 herramientas/llamada; 2025-11-25 efectivo, sin resultType moderno |
| L7 | Versión o estado no admitidos | SDK validación + rechazo inline-legacy adaptador | raw petición antes de handshake, metadata ausente/versión 2099 o inline 2025; dominio sin efectos |
| L8 | Recorrido de páginas | Catalog::page, PAGE=8 | unit concatena exactamente tools; SDK compara páginas wire con aggregate/repeticiones y única entrada resources/list |
| L8 | Cursor manipulado o ajeno | huella de catálogo/política, parse estricto | unit cursor de otra política/formato/posición 0; SDK >2048 bytes; inválidos no son páginas parciales |
| L9 | Línea incompleta o excesiva | codec SDK limitado + lector incremental | raw fragmentada exacta 4 MiB, +1 sin newline cierra, EOF no despacha, JSON inválido da parse error |
| L9 | Saturación mientras ejecuta | admisión antes del SDK, try-acquire core sin cola | unit cuatro requests/core0pending y cancel por canal lleno, legacy initialized + input lleno retorna busy; raw writer retenido y segunda llamada busy |
| L9 | Respuesta que excede el límite | LimitedWriter + delivery, completado registrado | raw read/update sobre doc futuro oversized confirma UPDATE antes de -32603; unit exacto/+1 y frame escapado >16 MiB, request_id/efectos conservados |
| L10 | Cancelación antes o después del despacho | run_core permiso dentro de spawn_blocking; writer suprime pendiente | unit barreras antes/después de commit, waiter abortado no libera; raw lock real+cancel no confirma/reintenta ni responde tarde |
| L10 | EOF o salida perdida | signal/stop, plazo total, PDF Process Drop | unit EOF durante read y timeout virtual; raw EOF durante mutación/run, stdout cerrado/no lector y helper retenido reaped; estados reabiertos sin generación falsa |
| L10 | Cancelación operativa separada | extracción core/fences/ownership intactos | raw CLI extraction.cancel vigente durante run MCP, resultado interrupted/cancelled, helper terminado y sin generación obsoleta |
| L11 | Recorridos con cliente real sin LLM | rmcp 3.5.1 vs cliente TS 2.3.1 | SDK 17/perfil, raw bytes aparte, fixtures sintéticas y todas las operaciones de esos recorridos bajo deny network* |
| L11 | Configuración y riesgo de retransmisión | docs/mcp.md y ADR 0014 | revisión: rutas ficticias/read-only, advertencia de retransmisión y consentimiento; ningún perfil/modelo/OAuth/HTTP/túnel instalado |

### core-operations — 4 requisitos / 16 escenarios

| ID | Escenario completo | Implementación | Prueba / comprobación |
| --- | --- | --- | --- |
| C1 | Scope actual y versión incompatible | scoped new/current, workspace preflight | scope/raw formatos + ausencia/pureza física; no migración/backup/recovery |
| C1 | Identidad comprobada en la operación | scoped check/open + specialized snapshot/lock | scope reemplazos genéricos/especializados, hook después de preflight y conexión/snapshot distintos; raw sesión viva |
| C1 | Clientes anteriores conservados | dispatcher compartido y open histórico intacto | catálogo completo baseline=current, legacy core/CLI, host sin lanzar y UI; migraciones 001–010 byte-identical |
| C2 | Lecturas generales y acciones get | genéricas main READ_ONLY; specialized equivalente, TEMP privado | scope 23/23 lecturas **exitosas**, consulta main sin DELETE permitido; SDK derivadas/fórmulas/vistas/fragmentos reales y paridad CLI |
| C2 | WAL y estado operativo pendientes | sin immutable/recovery y misma conexión visible | scope WAL >32 bytes, doc/future jobs intactos y running vencido sigue recoverable tras consultas/preview; originales/historial/tablas iguales |
| C2 | Daño o lectura incompatible | validadores core sin fallback/reparación | scope índice future_profile devuelve error y conserva tablas; legacy index tests corrupción/falta de fuente/TEMP y límites preservados |
| C3 | Acción mixta y operación excluida | match enum exhaustivo + permiso antes del opener | scope 51 enums/tres modos/siete exclusiones y acciones mixtas con ruta ausente para demostrar no apertura; SDK run denegado |
| C3 | DTO inválido y lote permitido | DTOs/validadores/whitelist/CAS existentes | SDK read-only preview/apply, paso excluido, plan inválido, recibos obsoletos y CAS CLI; legacy lotes rollback/binding/deadline |
| C3 | Origen compartido sin falsificación | scope recibe Operation no Request; host fija origin | scope mutación/revisiones/UUID, SDK historial mcp incluso metadata forjada; origin/request_id DTO inválidos |
| C4 heredado | Clientes equivalentes | execute_json/dispatch_on_workspace | suites CLI real_processes/paridad y UI 606 intactas; no certifica IPC nativo de Home |
| C4 heredado | Descubrimiento sin interfaz | Describe especial antes de abrir | CLI independiente/current equality del catálogo sin workspace ni GUI |
| C4 heredado | Catálogo y transportes completos | describe/CLI catalog sin cambios | unit CLI catalog_and_public_operation_sets y hash/equality 51 inputs/outputs/defs/errores/capacidades |
| C4 heredado | Paridad del lote | batches dispatcher/receipts compartidos | CLI batch test previo + SDK preview/apply IDs idénticos a receipt; BOB 145,69 exactos, sin whitelist nueva |
| C4 heredado | Catálogo de extracción acotado | contratos anteriores íntegros | catálogo equality, SDK UTF8/PDF/procedencia, parser/CLI/contención y concurrencia PDF; sin OCR/Excel/models |
| C4 | Paridad de herramientas MCP habilitadas | entrada scoped sin SQL de negocio en adapter | SDK entidad/doc/relación/colección/record/formula/view/template/Home/workflow/extracción/batch y CLI exactos; normalización sólo de metadata indicada |
| C4 | Catálogo completo y disponibilidad por transporte | metadata scoped separada de describe | equality catálogo v1/51; unit+SDK tools/resources por modo; las siete permanecen en CLI y se deniegan por MCP |

## Cierre de verificación

| Dimensión | Resultado |
| --- | --- |
| Completitud | **22/22 tareas**, 15 requisitos/45 escenarios con evidencia individual; ningún criterio aplicable omitido |
| Corrección | Source final7, **235 Rust + 17 SDK por perfil**, 13 grupos raw ×3 por perfil; corpus PDF/contención, migraciones/backup/concurrencia y **606 headless** aprobados; dos gates de consulta/lock aprobados |
| Coherencia | Diseño y ADRs 0014/0015 coinciden con scope, permisos, ambas eras, entrega/cancelación/cierre y verificación exacta FTS5; contratos/migraciones/originales/Home preservados |

**CRITICAL/WARNING del cambio: ninguna abierta.** Warning de chunk Vite y aviso experimental Node preexistentes conservados, no ocultados ni tratados como fallos del protocolo. No se sustituye aceptación nativa de Home por headless: sus ocho tareas permanecen pendientes y fuera de este cambio. No se probaron cuentas/modelos/proveedores/clientes gráficos ni plataformas distintas del M1 Max/macOS; no se anuncian como verificadas. Las dos deltas implementadas se sincronizaron, se contrastaron requisito a requisito y el cambio independiente se archivó como `2026-10-15-add-local-mcp`, conservando `.openspec.yaml` y todos sus artefactos.
