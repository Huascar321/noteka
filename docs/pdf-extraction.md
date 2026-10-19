# Extracción PDF textual acotada

## Estado de implementación

Cambio [add-local-pdf-text-extraction](../openspec/changes/archive/2026-10-14-add-local-pdf-text-extraction/tasks.md)
cerrado **21/21**, sincronizado y archivado el 14/10/2026.
El código integra **SQLite v9/API v1/49 operaciones**, `pdf_text` explícito,
helper descartable, publicación fenced y lector `extraction.pdf_fragments`.
La implementación está verificada; [evidencia y trazabilidad](verification-pdf-extraction.md).
La cola sigue pausada por defecto: importar, encolar, abrir o leer no extrae PDF.
La [verificación integrada](verification-pdf-extraction.md) separa checks realizados,
mediciones y cualquier gate pendiente; las secciones históricas de abajo no describen
el runtime actual.
Home sigue 9/17, sin cambios ni aceptación nativa. No se instalaron modelos.

Corpus actual: **235 casos** propios MIT, comparados en procesos y por CLI real.
Se conservan los 207 originales byte a byte; tres motivos pasan de soporte no
resuelto a `invalid_pdf` después de resolver su precedencia: un free de un objeto
referenciado invalida el documento, y `/Prev 0` no apunta a un índice válido.

## Contratos definidos

Entrada del lector: `{id, limit?, cursor?}` (`ReadPdfFragments`), estricta:
ID UUID canónico, límite 1–64/default 32, cursor opaco <=2 KiB. No admite contraseña,
comando, ruta de helper, SQL ni tuning del parser. La validación de scope se realiza en
el lector; el DTO verifica formato/cap del cursor.

`PdfFragmentPage` conserva `job_id`, `generation` (el **mismo shape** de
`ExtractionGeneration`), `result_from_previous_attempt`, `items`, `covers_generation`
y `next_cursor`. Añade un `manifest` completo, no otro cuerpo de texto.

| DTO / campo | Semántica |
| --- | --- |
| Manifest `version`, `parser_name`, `parser_version` | 1, `lopdf`, `0.45.0`; versiones desconocidas no se reinterpretan. |
| `scope` / `limitations` | `page_text_objects`; lista ordenada `no_ocr`, `no_visual_order`, `no_geometry`, `no_structured_tables`. |
| `page_count` / `pages` | 1–100 páginas físicas consecutivas, siempre todos los descriptores. |
| `extracted_bytes` | Suma de bytes UTF-8 del texto canónico de las páginas, <=1 MiB. |
| Descriptor | `page_number`, `state:text/no_text`, `text_bytes`, `text_sha256`, `fragment_count`. SHA-256 hexadecimal minúsculo del texto derivado; `no_text` exige cero bytes/fragments y hash del texto vacío. |
| Fragment | `ordinal` global 0–511, `text` no vacío <=4.096 bytes y `locator`. |
| Locator | `{kind:"pdf_page_text",page_number,text_byte_start,text_byte_end}`; rango 0-based half-open **en el texto de esa página**, no en el archivo PDF. |

No se normaliza Unicode, CRLF ni el texto con `trim`; el algoritmo PDF v1
fija espacios/saltos mediante golden sintético independiente. Fragmentos consecutivos
reconstruyen exactamente el texto UTF-8 de cada página y no cortan caracteres.
La validación completa verifica ordinales, contadores, rangos y hashes antes de paginar.

`source_bytes == covered_bytes` sigue midiendo el original íntegro verificado por hash,
no el texto extraído. `complete=true` certifica que el algoritmo admitido recorrió
todas las páginas; **no** una transcripción visual completa. Un PDF sólo-imagen puede
tener todas las páginas `no_text` y cero fragments sin ser visualmente vacío.
`covers_generation` sólo es true cuando la llamada devuelve todos los fragments;
el manifest completo no convierte una respuesta parcial en cobertura completa.

El lector literal `extraction.fragments` y sus offsets/líneas no cambian. Los dos
lectores rechazan el extractor ajeno con `unsupported_generation_config`.
Los roots PDF se agregaron **al final** de `Contract`; fingerprints capturados antes
del cambio comprueban las 48 entradas/salidas, errors, 140 definitions previas excepto
Operation y las referencias de roots anteriores.

## Subconjunto y límites

Original administrado `.pdf`, MIME `application/pdf` o `application/octet-stream`,
<=1 MiB, cabecera inicial PDF 1.0–1.7, no cifrado, 1–100 páginas. Texto directo con
Courier/Helvetica/Times y sus doce variantes Type1 latinas no embebidas,
StandardEncoding/WinAnsiEncoding. La [matriz del diseño](../openspec/changes/archive/2026-10-14-add-local-pdf-text-extraction/design.md)
fija rechazos: ToUnicode, Differences, glyph sin mapping, fuentes excluidas,
Form/annotations/ActualText y recuperación con pérdida; no publicar éxito parcial.
Imágenes no se transcriben. Sin PDF genérico, OCR, render, orden visual/geometría,
tablas estructuradas, Excel, FTS de adjuntos, embeddings, modelos ni red.

Presupuestos integrados: 512 fragments, 1 MiB de texto por resultado,
32 MiB de texto UTF-8/PDF retenido real y snapshots SQLite completos de 64 MiB.
Fuente, texto lógico, heap Rust, RSS, JSON y tamaño SQLite **no son la misma medida**.

## IPC privado v1

`noteka-pdf-protocol` no depende de parser/SQLite. Header de 24 bytes:
magic ASCII `NKPDF001` (8), versión u16 LE (1), reservados (2 bytes cero),
longitud fuente u32 LE (1–1.048.576), presupuesto restante u32 LE (1–5.000 ms),
reservados (4 bytes cero); luego bytes crudos, sin base64/temporales/rutas.
El presupuesto es el restante desde claim, no otros cinco segundos.

Reply JSON estricto: `protocol_version`, `extractor_version`, `parser_name`,
`parser_version`, `outcome:{status:"success",source_sha256,pages:[{page_number,text}]}`
o `outcome:{status:"error",reason}` de enum cerrado. Identidad/versiones, hash,
páginas consecutivas y texto total son validados; el padre además compara el hash
con los bytes enviados. Sin tokens, workspace, SQL ni mensajes arbitrarios.
Caps privados: stdout 8 MiB, stderr 8 KiB; no aumentan el transporte público 2 MiB.

## Uso local y recuperación

```sh
npm run pdf:prepare
cargo build --locked --offline -p noteka-cli
# Recorrido reproducible con originales propios sintéticos, sin ventanas:
python3 -B scripts/test-pdf-cli.py --profile debug
# Release, con el mismo corpus y el mismo golden:
npm run pdf:prepare -- --release
cargo build --locked --offline --release -p noteka-cli
python3 -B scripts/test-pdf-cli.py --profile release
```

Los scripts requieren `NOTEKA_TEST_TMP` fuera del checkout. Para un original
seleccionado por vos, importalo con `attachment import {source_path}`; encolá
con `extraction enqueue {attachment_id,extractor:"pdf_text"}`, habilitá mediante
`extraction control {action:"set",paused:false,expected_revision}` y pedí
`extraction run {id}`. Son payloads JSON estrictos por stdin/`--input`, no argumentos
de shell generados desde el documento. Leé con `extraction pdf-fragments {id,limit}`;
para continuar repetí exactamente job/límite y el `next_cursor` devuelto.

Cada lector valida fuente y la generación completa en un snapshot, luego pagina.
Concatená por página/ordinal, no juntes offsets con el binario PDF. Leer no requiere
helper, no recupera leases y no repara resultados. `retry` conserva la generación
previa hasta una publicación válida; pausa/cancelación/control ABA/fuente alterada
impiden al intento viejo reemplazarla. Los errores incluyen job/revisión/intento
confirmados, nunca raw stderr ni objetos PDF.

`009.sql` sólo agrega manifest, páginas y fragmentos PDF. La migración hace backup
previo y una transacción; no reescribe 001–008, configura ni recupera jobs. Cliente
v8 rechaza v9: rollback de software requiere restaurar su backup en **otra ruta**,
conservando escrituras posteriores. Backup/restore validan hashes/rangos/filas y
originales sin parser, incluso con el helper ausente. No copies SQLite vivo a mano.

El presupuesto retenido suma `length(CAST(text AS BLOB))` de ambos extractores bajo
el lock de publicación y descuenta una sola vez el job reemplazado. Los snapshots
de lotes incluyen las tres tablas completas; sólo la huella base excluye derivados.
Se conservan `noteka.batch.base.v2`, receipt/plan v1 y la whitelist anterior.

Preparación/resolución: helper sibling de `current_exe`, padre de `deps/examples`
para Cargo, o `Contents/MacOS` en el layout de prueba. Nunca PATH/CWD/workspace ni
una ruta en JSON. El script prepara además el nombre target-triple ignorado para
`externalBin`; hooks Tauri eligen debug/release sin instalar globalmente. Bundle
sigue inactivo: `tauri build --no-bundle` sólo compila, no ejecuta ni distribuye.

## Evidencia de contratos

Pruebas de DTO/protocolo: `cargo test --offline -p noteka-pdf-protocol` (3) y
`cargo test --offline -p noteka-core pdf_contracts` (6). Verifican campos desconocidos,
versiones, límites, hashes, rangos, reconstrucción multibyte y fingerprints v8.
Estos tests específicos no prueban por sí solos parsing/persistencia; las pruebas
integradas están separadas en la verificación enlazada arriba.

## Dependencia revisada y compilación local

`noteka-pdf-worker` fija `lopdf =0.45.0`, `default-features=false`; el tree de Cargo
no activa default/async/rayon/imagen/font_embedding del parser. El core sólo comparte
`noteka-pdf-protocol`: ni core ni host enlazan lopdf o dependen del worker.
Compilación debug/release Apple Silicon: Rust 1.99.0, `aarch64-apple-darwin`.
MSRV declarado del parser: 1.88; la compilación real pasó, no se deduce sólo del MSRV.

Resolución para este target: 72 paquetes contando las dos crates locales, 70 externos;
33 versiones externas agregadas al lockfile, sin cambiar versiones previas. El árbol
resuelto y sus expresiones declaradas son permisivos: MIT/Apache-2.0 y alternativas
0BSD/Unlicense/Zlib/BSD-3-Clause, además de Unicode-3.0. Excepciones a revisar al
redistribuir: `encoding_rs` requiere también BSD-3-Clause, `unicode-ident` Unicode-3.0;
`brotli-decompressor` declara `BSD-3-Clause/MIT`, su LICENSE publicado es BSD-3-Clause
y se considera esa licencia (no se interpreta el slash como SPDX válido).
`alloc-stdlib 0.2.4` no incluye LICENSE en el tarball: se comprobó el
[LICENSE BSD del repo en el commit del paquete](https://raw.githubusercontent.com/dropbox/rust-alloc-no-stdlib/ae42d22078b98549e987d2f03d12df7b984fde47/LICENSE).
El inventario local registra nombres/versiones/licencias y hashes de notices del
árbol completo; no supone que la licencia MIT del parser cubra sus dependencias.
Redistribución/firma/notarización siguen fuera del cambio; conservar todos los avisos
correspondientes antes de distribuir, no sólo este resumen.

Auditado el 13/10/2026 con cargo-audit 0.22.2, DB RustSec
`6aed84668c23d7f535077a9fe918bf1fa0f6a5bc` (1.310 advisories): cero vulnerabilidades
conocidas en 476 dependencias del lockfile, cero warnings del árbol del helper.
Warnings heredados fuera del helper: proc-macro-error 1.0.4 (unmaintained,
RUSTSEC-2024-0370), glib 0.18.5 (unsound, RUSTSEC-2024-0429, target Linux).
No se actualizaron dependencias ajenas para ocultarlos. RUSTSEC-2026-0187 de lopdf
está parcheado desde 0.42.0; no constituye garantía de ausencia de otros fallos.

Reproducción (no abre ventanas):

```sh
cargo build --locked --offline -p noteka-pdf-worker --bins --examples
cargo build --locked --offline --release -p noteka-pdf-worker --bins --examples
cargo tree --locked --offline -p noteka-pdf-worker -e features
cargo tree --locked --offline -p noteka-core
cargo tree --locked --offline -p noteka-desktop
```

## Contención implementada

Antes de leer el header: CPU blanda/dura 4 s, core dump 0 y panic hook que termina
sin imprimir el payload. Si setrlimit o watchdog fallan, no hay procesamiento.
Watchdog monotónico propio de 5 s desde la primera instrucción de main; el header
sólo puede acortarlo. Cubre lectura bloqueada, CPU/parsing/salida aunque muera el padre.
No depende de checks cooperativos del parser. El supervisor del core usa pipes no
bloqueantes y checks de fence/deadline cada tick de 5 ms; mata y recoge al hijo ante
fallo, sin mantener un lock SQLite durante lectura/parsing/salida.

Allocator global del helper: cap de **128 MiB de tamaños Layout solicitados vivos**,
reservas CAS/sumas comprobadas, alloc/alloc_zeroed/realloc/dealloc y pico atómico.
Realloc creciente reserva el delta antes de System; decreciente libera después del
éxito. Ante fallo de System revierte reserva y termina; el cap no mide padding,
fragmentación interna de malloc, stacks ni memoria nativa/mapped/RSS. Realloc puede
tener buffers temporales del allocator del sistema: tampoco son un techo de RSS.
El código unsafe se limita a esta delegación, POSIX y probes sólo de desarrollo,
con obligaciones de Layout/puntero explícitas; el core de producción conserva System.
El binario de unit tests tiene contador de Layouts sólo para mediciones, sin cap.

Salidas sin asignación: 70 setup/IO, 73 heap/OOM, 74 panic y 75 deadline.
Abort es SIGABRT; macOS entrega SIGXCPU al alcanzar CPU 4 s. Los errores IPC contienen
sólo el enum cerrado. El padre mapea heap a storage_error/pdf_memory_limit,
deadline/SIGXCPU a workspace_busy/execution_deadline y crash a error sanitizado.
El helper operativo recibe header más bytes y puede retornar páginas; no es una
CLI pública ni una sandbox de privilegios.

Pruebas realizadas: dos unit tests de overflow/frontera/rollback de reservas y
80.000 reservas concurrentes; harness debug/release con doce casos de procesos reales:
limits, accounting, alloc/realloc concurrentes, OOM/realloc-OOM, panic, abort, CPU,
presupuesto acortado, input bloqueado, reply cerrado y SIGKILL del padre.
El harness exige stderr vacío, terminación del hijo y ausencia de archivos/core dumps.
No provoca agotamiento de la RAM del equipo: OOM se induce al exceder el cap lógico.

```sh
# NOTEKA_TEST_TMP debe ser un directorio temporal local, fuera de Git.
cargo test --locked --offline -p noteka-pdf-worker
python3 scripts/test-pdf-containment.py --profile debug
python3 scripts/test-pdf-containment.py --profile release
```

Probe accounting release: heap solicitado observado 801 bytes vivos / 842 pico;
`/usr/bin/time -l` reportó RSS máximo 2.031.616 bytes, **otra medida**. Es sólo un probe,
no memoria del parser ni benchmark PDF. No valida todavía el corpus ni el presupuesto
end-to-end desde claim. Proceso separado no es sandbox de privilegios.

## Regresión inicial de infraestructura

Recorrido de la etapa inicial 1.1–1.3, sin ventanas (no confundir con el avance
experimental posterior):

| Check | Resultado |
| --- | --- |
| `sandbox-exec -p '(version 1)(allow default)(deny network*)' cargo test --offline --workspace` | 153 aprobados, 3 ignorados intencionales previos; incluye los 11 nuevos tests (6 DTO, 3 protocolo, 2 contención). |
| Harness de contención con la misma red denegada | 12 casos debug + 12 release aprobados, incluido OOM del allocator global real. |
| `cargo fmt --all --check` | Aprobado. |
| `cargo check/clippy --offline --workspace --all-targets --features native-smoke,custom-protocol` | Aprobados; Clippy `-D warnings`. Compilación del host, no ejecución nativa. |
| `npm run contracts:check`, `npm run check` | Aprobados; cero diagnósticos Svelte/TypeScript. |
| `npm run test:editor`, `npm run test:headless-bridge` | 13 + 3 aprobados; puente no es IPC nativo. |
| `npm run build` | Aprobado; bundle 507,67 kB conserva warning >500 kB. |
| OpenSpec `validate --all --strict --json` | 14/14; valida documentos, no extracción PDF. |

Baseline independiente de 249 archivos, capturado fuera de Git antes de editar.
Revisión comparativa: ningún archivo eliminado; migraciones 001–008, main specs,
archivos de Home/UI (salvo DTOs generados aditivos), host y tests previos permanecen
iguales. Las 438 versiones/checksums de dependencias de registro previas se conservan;
hay referencias de versión desambiguadas en el lockfile y la feature transitiva nueva
`getrandom/sys_rng` añade su edge `rand_core`, no un upgrade encubierto.

Los 606 checks UI previos **no se reejecutaron** en esa etapa; tampoco aceptación
nativa, `tauri build --no-bundle`, corpus PDF, benchmarks end-to-end o CLI PDF.
No se completó la tarea integrada 5.2 ni la revisión de todos los escenarios 5.4.
Sin sync/archive de PDF o Home. Próximas tareas: parser/preflight (1.4) y corpus/golden
(1.5), luego persistencia/integración. No se necesitan modelos ni proveedores remotos.

## Historia: adapter y corpus experimentales, antes de integrar PDF

**Registro histórico, no estado vigente.** Las referencias de este bloque a
«pendiente», 207 casos, v8/48, runtime deshabilitado y tareas 1.4/1.5 abiertas
describen el baseline anterior. Framing histórico/linearización, free/hybrid,
streams de carga y publicación/lector se resolvieron en la integración actual;
la evidencia vigente está en [verificación PDF](verification-pdf-extraction.md).
El xref actual audita el original y añade una vista de carga efectiva sin Prev,
con offsets originales (no reconstrucción/repair), evitando la precedencia
incorrecta de tombstones/suplementos en lopdf. [ADR 0012](decisions/0012-bounded-pdf-worker.md).

Código en `noteka-pdf-worker::{parser,text,streams,framing,xref,codecs,predictors,diagnostics}`. El
example `parser_probe` recibe bytes por stdin bajo el allocator/límites/watchdog
existentes; sólo devuelve Reply privado y no tiene DB/rutas/acciones. **El binario
runtime continúa validando el header y devolviendo un error, nunca éxito PDF.**
El probe recibe PDF crudo, **no** el header IPC `NKPDF001`; no valida todavía
el flujo runtime de claim/handshake/supervisión/publicación.

| Superficie del adapter actual | Comprobación parcial implementada |
| --- | --- |
| Fuente/carga | Firma inicial 1.0–1.7, EOF, <=1 MiB; LoadOptions strict/password None/stream de carga 1 MiB; rechaza cifrado original incluso vacío. |
| Diagnósticos | Logger Warn/Error marca degradación por hilo, sin formatear, almacenar ni imprimir argumentos. Fallo al instalar impide parsing. |
| Framing | Scanner léxico acotado de objetos originales, miembros ObjStm y operandos de contenido; exige endobj real, rechaza claves duplicadas incluso escapadas y objetos activos solapados. Trailers se releen sobre bytes originales, no sólo sobre el AST. |
| Xref incremental/hybrid | Valida sufijo startxref/EOF, filas de 20 bytes, subsecciones/duplicados/Size/W/Index, tipos y ciclos Prev/XRefStm; calcula precedencia y exige igualdad con el índice cargado. Revisiones normales y suplemento sin discrepancia tienen positivos; discrepancias de free/hybrid fallan cerradas, todavía no son soporte final completo. |
| Streams de carga | Revisa miembros/índices/offsets/conteos ObjStm y XRef para evitar defaults/filter_map silenciosos. Reparsea ObjStm desde bytes decodificados estrictamente y compara el AST activo; referencias exigen entrada xref viva y generación correcta, no mera presencia en doc.objects. |
| Árbol | Parent físico, Kids/Count, ciclos/duplicados, 1–100 hojas, recursos heredados y override de hoja. Profundidad acotada a 64. |
| Texto | Doce Type1 estándar no embebidas; Standard/WinAnsi/ausente. Preflight rechaza ToUnicode, Differences, subtipos/nombres excluidos y FontFile; cada byte usado requiere mapping Unicode válido no NUL/replacement. |
| Operaciones | Tj/TJ/'/", estados BT/ET/Tf, q/Q y marked content; operandos tipados. Orden de operaciones, sin ordenar por geometría. Corpus fija espacios/saltos, no extract_text de lopdf. |
| No-texto | XObject Image se ignora sin decode/OCR; enlace sin texto/appearance y acciones se conservan como datos pasivos, sin evaluación ni IO. Form/inline image/AcroForm/annotation textual/ActualText/Alt y operadores desconocidos fallan completos. |
| Decode del adapter | Raw, Flate, ASCIIHex, ASCII85, RunLength y LZW MSB (EarlyChange 0/1), sin reparación de EOD/checksum/truncado y con cap por etapa. TIFF 2, bits 1/2/4/8/16; PNG 10–15 con tags por fila autoritativos, filas completas y aritmética comprobada. No decodifica codecs de imagen/Crypt como texto. |
| Caps | Texto UTF-8 total 1 MiB; contenido de página concatenado 4 MiB; descompresión de carga 1 MiB. Fronteras/exceso tienen fixtures. Estos buffers no equivalen al heap/RSS ni al presupuesto retenido del core. |

El mapping usa la tabla tipada de lopdf **después** del preflight, no su
`bytes_to_string` (omite glyphs) ni `extract_text` (fallbacks). Flate2 ya formaba
parte del árbol auditado: se añadió solamente un edge directo del helper con
`rust_backend`; no se actualizaron versiones/checksums ni se agregaron paquetes.

Corpus propio MIT: [generador/procedencia/reglas](../tests/fixtures/pdf/README.md).
207 entradas reproducibles, golden explícito de texto/error y manifiesto calculado
desde esos strings, no desde la salida del parser. Huellas en `golden.sha256`
fijan generador y manifiesto (zlib de referencia 1.2.12); el harness las exige antes
de ejecutar. PDFs/manifest detallado de ejecución quedan fuera del checkout.

Se verificaron y corrigieron dos recuperaciones **silenciosas** adicionales de
lopdf 0.45.0: `parser/mod.rs::_indirect_object` permite endobj opcional y el AST
de Dictionary pierde claves duplicadas. Se reprodujeron ambos éxitos incorrectos
antes del scanner y se incorporaron cuatro fixtures negativos (incluidos nombres
escapados y diccionario comprimido), sin rebajar los esperados.

**Gates pendientes, no excepciones aceptadas:** completar auditoría de framing de
revisiones históricas/linearización y precedencia free/hybrid sin depender de
recuperaciones de lopdf; ampliar corpus de recursos/operadores y correlación
xref/objetos comprimidos, incluidos codecs de carga y combinaciones de predictors;
validar manifest/fragmentación reales del core. Lo ya implementado no certifica
todas esas combinaciones. Por eso no se marcan 1.4/1.5 ni se habilita runtime/PDF.
No hay medición claim/publicación, integración CLI, migración v9 ni benchmark end-to-end.

Reproducción sin ventanas:

```sh
cargo test --locked --offline -p noteka-pdf-worker
cargo build --locked --offline -p noteka-pdf-worker --examples
python3 -B scripts/test-pdf-parser.py --profile debug
cargo build --locked --offline --release -p noteka-pdf-worker --examples
python3 -B scripts/test-pdf-parser.py --profile release
```

Se requiere `NOTEKA_TEST_TMP` externo al checkout, igual que en contención.

### Verificación anterior del adapter experimental

| Check | Resultado |
| --- | --- |
| Workspace Rust con red denegada | **157 aprobados**, 3 ignores heredados; cuatro tests nuevos de logger/Flate/framing. |
| Corpus en procesos contenidos con red denegada | **110 debug + 110 release**, sin stderr ni archivos/core dumps; texto/hash/orden y motivos exactos. No es la integración pública. |
| Contención real, repetida sobre esta base | **12 debug + 12 release**; runtime main sigue devolviendo error cerrado. |
| fmt, check/Clippy workspace/all-targets/native-smoke/custom-protocol | Aprobados; Clippy `-D warnings`, sin ejecutar el host. |
| Contratos/editor/check/build | Aprobados, 13 tests adaptador/cola/tema; cero diagnósticos Svelte/TypeScript. Bundle sin cambios: 507,67 kB, warning >500 kB pendiente. |
| Puente/consentimiento | 3 aprobados permitiendo HTTP loopback del harness; sus procesos CLI conservan red denegada. No es IPC nativo. |
| OpenSpec estricto | 14/14; sólo valida documentos. |

Un primer intento de correr Node/puente bajo `deny network*` falló 2/3 por
`listen EPERM 127.0.0.1`: ese perfil impide también el HTTP local requerido
por la prueba. Se conservó ese resultado y se repitió el runner normal de
loopback sin cambiar assertions ni la sandbox de los subprocesses CLI. Los
tests Rust, corpus/contención y build frontend sí pasaron con red denegada.

Revisión contra baseline externo de **259 archivos**: 12 existentes cambiados,
10 agregados, ninguno eliminado y 247 existentes idénticos. Todos los paquetes
de registro (**471 versiones/checksums**) permanecen iguales; Cargo.lock sólo
añade el edge directo flate2 del helper. Main specs, Home/UI/host, núcleo/CLI,
migraciones 001–008, contratos/tests previos y `main` del helper son idénticos.
No se agregaron binarios, PDFs, datos privados ni archivos de ejecución a Git.

Evidencia temporal:
`/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-pdf-parser-d_45dei9/`:
baseline, manifiesto/golden detallado, logs JSON de ambos perfiles, regresión,
árbol/features, diff e inventario comparado. No es contenido portable del producto.

En esa etapa no se reejecutaron los 606 checks UI, pruebas nativas, tauri --no-bundle,
CLI PDF ni métricas end-to-end. Las tareas 1.4/1.5 y 5.2/5.4 siguen abiertas;
sin sync/archive ni activación de modelos/remotos. La evidencia del probe no
se atribuye al host nativo ni a la operación pública PDF aún inexistente.

### Segunda revisión experimental: xref, codecs y referencias vivas

Se conservan íntegros los **110 casos previos** (bytes/hash/expected); se agregan
97. Los nuevos positivos cubren revisiones con reemplazo de contenido, suplemento
hybrid sin conflicto, codecs/cadenas, transiciones LZW 9→12 bits/EarlyChange 0/1,
TIFF/PNG y fuente comprimida con TIFF. Los negativos incluyen ciclos Prev,
trailers duplicados/escapados, subsecciones/filas mal formadas, precedencia free,
objetos dentro de streams, referencias ausentes y recursos/herencia adversariales.

La revisión reprodujo otro éxito incorrecto: lopdf inserta un miembro ObjStm
ausente del xref y permite usarlo como fuente. Ahora toda referencia consultada
exige una entrada viva; la fixture `compressed-member-absent-xref` falla completa.
Un test reproduce también el ciclo Prev que el loader acepta y borra del trailer;
la segunda lectura lo rechaza sin modificar el original.

Las reglas de PNG y precedencia xref se contrastaron con [ISO 32000-1, §7.4.4.4 y §7.5](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf)
y [la documentación de xref de qpdf](https://qpdf.readthedocs.io/en/stable/object-streams.html).
En particular PNG 10–15 **no exige** que el tag de la fila coincida con Predictor;
se corrigió esa suposición durante desarrollo y hay cinco positivos específicos.
El golden sigue escrito desde reglas/strings, no capturado del parser. Un test LZW
contrasta códigos explícitos y texto literal con el decoder weezl ya transitivo;
no se agrega una dependencia ni se delega a él el decoder estricto del adapter.

Verificación fresca con red denegada: **164 tests Rust aprobados**, tres ignores
heredados; **207 casos de parser y 12 de contención en cada perfil debug/release**;
fmt/check/Clippy workspace/all-targets/native-smoke/custom-protocol aprobados,
Clippy `-D warnings`. No se ejecutó el host al compilar sus features.
OpenSpec estricto **14/14**: comprobación documental, no del parser.

Regresión fresca del producto vigente, sin ocupar pantalla:

| Check | Resultado |
| --- | --- |
| Contratos/adaptador/cola/tema | `contracts:check`, 13 tests y check/build aprobados. |
| Puente/consentimiento | 3 aprobados con loopback local; CLI bajo red denegada. |
| UI headless | **592 + 14 = 606 checks**, seis sesiones dark/light/main/reopen/restored; cero orígenes externos, service workers bloqueados y sin perfil personal. |
| Host `tauri build --no-bundle` | Aprobado con red denegada, Mach-O arm64; sólo compilación, sin abrir Tauri ni instalar/empaquetar el helper PDF futuro. |
| Bundle | 507,67 kB; se conserva warning >500 kB, sin optimización ajena. |

El primer intento de UI falló **antes de abrir el navegador**: el TMPDIR aislado
produjo un Unix socket demasiado largo. Se conservó el log y se repitió íntegro
con `PWTEST_SOCKETS_DIR` corto en el temporal aprobado, sin cambiar código ni
assertions. npm/npx se ejecutaron con `npm_config_offline=true`; el runner UI
necesita HTTP loopback, por eso no se atribuye una sandbox `deny network*` a Node.
Los subprocesses CLI sí la mantienen. Headless/no-bundle no cierran la aceptación
nativa de Home ni prueban todavía la integración del parser con la cola.

Revisión final contra baseline: **269 → 272 archivos**, 18 existentes cambiados,
3 agregados, ninguno eliminado y 251 idénticos. Los 123 archivos protegidos
(núcleo/CLI/protocolo, Home/UI/host, main specs, ADRs, manifiestos/lockfiles y main
runtime del helper) son byte-idénticos. Sin nuevas dependencias, upgrades,
migración, binarios/PDFs de ejecución en Git ni activación de modelos/proveedores.
Enlaces relativos modificados y sintaxis Python comprobados.

Evidencia temporal de esta continuación:
`/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-pdf-xref-akxjqcje/`.
Contiene baseline de 269 archivos, manifiesto independiente, reproducciones,
logs de procesos y diff. El runtime sigue sin extraer PDF; las tareas 1.4/1.5
y las pruebas públicas/integradas de PDF siguen abiertas. No se ejecutaron
pruebas nativas, CLI PDF ni métricas end-to-end. Sin sync/archive de PDF o Home.

## Escritorio al 17 de octubre de 2026

La [preparación explícita](attachment-preparation-ui.md) permite seleccionar un
adjunto administrado, consultar y ejecutar extracción y seleccionar generación
para Search, con las revisiones del núcleo. El [lector de generaciones](attachment-reader-ui.md)
recorre todo el derivado por páginas sin exigir indexación. La
[consulta agrupada](grouped-search.md) usa ambos motores sin fusionar cursores o
scores. [Aceptación técnica conjunta](verification-local-preparation-closure.md).
Los límites de formato/capacidad del núcleo no se amplían con estas UIs; abrir o
buscar no prepara ni repara. Los hitos históricos anteriores conservan su alcance.
