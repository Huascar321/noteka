# Desarrollo de Noteka

## Entorno y dependencias

### Transporte MCP local

Crate independiente `noteka-mcp`, sin alterar default-members. `rmcp =3.5.1` (Apache-2.0, rust-version 1.88), default-features false, server/transport-io; `tokio 1.53.2` (MIT, 1.71), `tokio-util 0.7.20` (MIT, 1.85) codec/rt y sha2 ya compartido. No se habilitan HTTP/auth en el binario. Build/check offline verificados con Rust 1.99.0, sin prometer MSRV inferior no probado; metadata/árbol en [verificación](verification-mcp.md).

Cliente de prueba dev-only `@modelcontextprotocol/client =2.3.1` y core TS 2.3.1 (Apache-2.0, Node >=20), `ajv =8.20.0` (MIT) para validar schemas 2020-12 sin coerciones/defaults; Node de este proyecto sigue >=22.12. No se importa el cliente en src ni en el bundle. Instalación de tooling separada: `CARGO_NET_OFFLINE=false cargo fetch --locked --target aarch64-apple-darwin` y `npm_config_offline=false npm ci --ignore-scripts --no-audit --no-fund`; runtime/tests se ejecutan después con red denegada.

```sh
cargo build --offline --locked -p noteka-mcp -p noteka-cli -p noteka-pdf-worker
cargo tree --offline --locked -p noteka-mcp -e features
npm run test:mcp
python3 -B scripts/test-mcp-transport.py --profile debug
cargo build --release --offline --locked -p noteka-core --example mcp_benchmark
# El sampler necesita ps fuera del sandbox; el script deniega red a cada binario medido.
python3 -B scripts/benchmark-mcp.py
```

[Guía, flags, consentimiento y caps](mcp.md); [regresión, mediciones y matriz de aceptación](verification-mcp.md). Los builds o la validación documental no sustituyen las pruebas de ejecución.

### Lector XLSX independiente

`noteka-xlsx-worker` usa zip =9.0.3 (MIT, features default desactivadas, Deflate con backend Rust) y roxmltree =0.21.1 (MIT OR Apache-2.0). `noteka-xlsx-protocol` conserva DTOs/validación sin enlazar el parser al core. `noteka-parser-runtime` comparte la contención previa PDF sin cambiar sus límites. El lector aislado fue archivado en v10/51: [subconjunto](xlsx-reader.md), [ADR 0016](decisions/0016-bounded-xlsx-reader.md) y [evidencia](verification-xlsx-reader.md). Su [integración verificada](xlsx-extraction.md) añade cola/lectura explícitas v11/52, sin nueva UI; [aceptación](verification-xlsx-extraction.md).

```sh
# Preparar tooling sólo si falta en caché; no descarga modelos.
CARGO_NET_OFFLINE=false cargo fetch --locked --target aarch64-apple-darwin
cargo build --offline --locked -p noteka-xlsx-worker -p noteka-xlsx-protocol --bins --examples
# NOTEKA_TEST_TMP debe ser un directorio externo; los tests no usan datos personales.
sandbox-exec -p '(version 1)(allow default)(deny network*)' \
  python3 -B scripts/test-xlsx-reader.py --profile debug
cargo build --release --offline --locked -p noteka-xlsx-worker -p noteka-xlsx-protocol --bins --examples
sandbox-exec -p '(version 1)(allow default)(deny network*)' \
  python3 -B scripts/test-xlsx-reader.py --profile release
# Medir sin compilaciones simultáneas; proceso real, no estimación del allocator.
sandbox-exec -p '(version 1)(allow default)(deny network*)' \
  python3 -B scripts/benchmark-xlsx-reader.py
```

### Integración XLSX verificada

```sh
cargo build --offline --locked -p noteka-cli -p noteka-mcp -p noteka-pdf-worker -p noteka-xlsx-worker
python3 -B scripts/test-xlsx-cli.py --profile debug
python3 -B scripts/test-xlsx-migration.py --profile debug --baseline-bin "$BASELINE_BIN_V10"
python3 -B scripts/test-xlsx-processes.py --profile debug
python3 -B scripts/check-xlsx-contracts.py --baseline-catalog "$BASELINE_CATALOG_V1_51"
# Después de preparar release/helper y sin otros tests/compilaciones:
python3 -B scripts/benchmark-xlsx-extraction.py
```

Ejecutar con red denegada y `NOTEKA_TEST_TMP` externo, igual que el lector.
El baseline debe ser independiente y anterior a este cambio; no fabricarlo con
el binario nuevo. CLI `extraction xlsx-cells`/invoke `extraction.xlsx_cells` leen
resultados sin helper; run necesita el sibling XLSX del mismo perfil. El hook
Tauri existente prepara sólo PDF: para invocar extracción XLSX desde el host de
desarrollo, construir también el sibling XLSX en ese target/perfil. No se amplía
distribución/firma ni se abre la ventana para verificarlo.

`test-xlsx-nonactivation.py` conserva la aceptación histórica del lector aislado
**v10/51**: se ejecuta contra los binarios baseline mediante `CARGO_TARGET_DIR`,
no contra v11/52. La no ejecución implícita actual se prueba en el corpus CLI,
fences/supervisor y SDK; no debe confundirse con negar el extractor ya integrado.

### Búsqueda XLSX: verificación local

Revalidación durable del 16/10 tras pérdida de baselines: cargar `env.sh` de
`~/.local/share/noteka-verification/xlsx-search-20261016.fikBaL/`. Usar
`scripts/test-xlsx-search-revalidation.py --profile debug|release` para fixtures
SQL v10/v11, y `scripts/check-xlsx-search-current-contracts.py --cli <binario>
--restart-catalog <captura-v12.json>` para contratos **actuales**. No sustituyen
los clientes/catálogo viejos independientes. [Alcance y aceptación](xlsx-search-evidence-recovery.md).
La sonda reproducible está en `scripts/scratch-probe.cpp` y su autotest en
`scripts/scratch-probe-selftest.cpp`; medición separada de latencia con
`measure-xlsx-search-scratch.py`. Conservá evidencia nueva fuera de Git y fuera
de temporales descartables. Build sin ventana: `npm run tauri -- build --no-bundle
-- --offline --locked`; `--offline` es argumento de Cargo, no de Tauri.
Los comandos con baseline v11 de abajo describen la auditoría original: sus
fuentes se perdieron y no se reconstruyen como si fueran independientes.

[Guía](xlsx-search.md), [matriz](verification-xlsx-search-scenarios.md) y
[evidencia/estado de aceptación](verification-xlsx-search.md). Prepará los mismos
binarios/helpers debug/release; index/search no necesitan los helpers después de
extraer. API v1/52 y MCP 24/44/45 no cambian de conteo; actualizar bindings por los
enums cerrados ampliados de index/search.

```sh
# NOTEKA_TEST_TMP externo; baseline independiente v11, no un binario actual rebajado.
python3 -B scripts/test-xlsx-search-migration.py --profile debug \
  --baseline-bin "$BASELINE_BIN_V11" --directory "$NOTEKA_TEST_TMP/migracion-v11-nueva"
python3 -B scripts/check-xlsx-search-contracts.py --baseline "$BASELINE_CATALOG_V11_52"
python3 -B scripts/test-xlsx-search.py --profile debug
# Cada perfil usa ambos binarios del mismo build, no NOTEKA_MCP_PROFILE:
NOTEKA_CLI=target/release/noteka NOTEKA_MCP=target/release/noteka-mcp npm run test:mcp
# Medir de a una corrida, sin otros tests/builds; repetir dos veces cada gate.
python3 -B scripts/benchmark-xlsx-search.py --profile release
python3 -B scripts/benchmark-attachment-search.py
```

Denegá red al producto/harness con sandbox-exec, igual que en extracción. El gate
mixto comprueba recipe/corpus-pins SHA-256 **antes** de medir. No modifiques corpus,
consultas, caps o target tras un fallo para hacerlo pasar. El checker histórico
`check-xlsx-contracts.py` corresponde al cierre v11 frente a v10/51; el checker
`check-xlsx-search-contracts.py` contrasta v12 contra v11 y valida cada excepción
nombrada, además de los 50 pares de contratos conservados.

Para Playwright, un `TMPDIR` de auditoría largo puede exceder el límite de sockets
Unix. Elegí `PWTEST_SOCKETS_DIR` corto dentro del temporal aprobado; `/var/...`
puede usarse como alias del mismo `/private/var/...` en macOS si hace falta ahorrar
bytes. No requiere cambiar tests, abrir un browser headed o usar perfiles personales.

### Plataforma y herramientas

Primera plataforma: macOS Apple Silicon, con Xcode/Command Line Tools (`xcode-select -p`), Node.js >=22.12 y npm. Rust **1.99.0** está fijado en `rust-toolchain.toml`; Cargo usa SQLite incluido, no una instalación de SQLite del sistema.

Versiones resueltas: Node 22.22.3, npm 10.9.8, Svelte 5.57.2, Vite 8.3.4, TypeScript 6.0.3, svelte-check 4.7.6, Tauri Rust/API 2.12.3, Tauri CLI 2.12.2, Tiptap 3.31.4, rusqlite 0.38.0 y schemars 1.2.2. TypeScript 7 no es compatible con los peer dependencies del checker elegido. `package-lock.json` y `Cargo.lock` son la referencia reproducible.

En aarch64, el core habilita `sha2 0.10.9/asm` para usar el backend ARM SHA-256 con detección runtime/fallback portable. `sha2-asm 0.6.4` es la única adición al lock de este recorte de búsqueda; no cambia algoritmo ni formato de hashes persistentes. Hace falta tener ese código de build en la caché antes de usar `--offline --locked` (o ejecutar `cargo fetch --locked --target aarch64-apple-darwin` durante preparación). No descarga modelos ni habilita red en runtime. [Decisión y pruebas de compatibilidad](decisions/0013-explicit-attachment-index.md); evidencia medida en M1 Max, no aceptación de otras arquitecturas.

Necesitás `cargo` en el PATH. La toolchain temporal original se perdió; el 16/10
se restauró Rust 1.99.0 en el directorio durable de verificación, sin modificar la
configuración global. Para usar esa instalación local existente:

```sh
source "$(cat "$HOME/.local/share/noteka-verification/active-xlsx-search-path.txt")/env.sh"
```

Para un entorno permanente, instalá Rust mediante [rustup oficial](https://rustup.rs/). No hace falta conservar estos exports cuando tengas tu propia instalación.

## Instalación y ejecución

```sh
npm ci
cargo build -p noteka-cli
npm run tauri dev
```

`tauri dev` inicia Vite en `127.0.0.1:1420` y el host nativo. `npm run dev` solo inicia el frontend: abrirlo en un navegador **no** crea una segunda implementación de almacenamiento; muestra una explicación al intentar operar.

La app ofrece crear/abrir un workspace y entra en [Home](home.md). Campo vacío: ubicación por defecto. Otras rutas se escriben explícitamente; no se guardan notas dentro del checkout por defecto. La UI crea/edita los cinco tipos y muestra Inbox, My Day, pendientes/vencidas, proyectos activos, áreas/recursos y archivo. Dashboards consultan relaciones compartidas y capturan notas/tareas en contexto. Ver [reglas y contrato](productivity.md). Home está implementado en el cambio activo, con [verificación integrada y límites](verification-home.md); todavía no archivado.

## Checks y builds

### Preparación local PDF

```sh
npm run pdf:prepare
cargo build --locked --offline -p noteka-cli
# Para CLI release:
npm run pdf:prepare -- --release
cargo build --locked --offline --release -p noteka-cli
```

Los hooks Tauri preparan debug/release mediante `TAURI_ENV_DEBUG` y
`TAURI_ENV_TARGET_TRIPLE`, con sufijo target-triple en `src-tauri/binaries/`
(ignorado). El core sólo busca el sibling `noteka-pdf-worker` de su ejecutable;
Cargo `deps/examples` resuelve un nivel arriba. No usa PATH/CWD ni rutas del JSON.
`bundle.active=false` y `tauri build --no-bundle` sólo compila: no iniciar el host
para verificar PDF ni para suplir aceptación nativa pendiente de Home.
Falta/incompatibilidad del helper falla cerrado en `run`; leer/restaurar resultados
confirmados no requiere el helper. [Guía y límites PDF](pdf-extraction.md).

### Regresión

```sh
cargo check -p noteka-core
cargo test -p noteka-core -p noteka-cli
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p noteka-desktop --all-targets --features native-smoke,custom-protocol -- -D warnings
npm run contracts:check
npm run check
npm run test:editor
npm run test:batches
npm run build
npm run tauri build -- --no-bundle
npm run test:headless-bridge
npm run test:headless
npx --yes @fission-ai/openspec@1.14.1 validate --all --strict
```

`noteka-core` compila sin Tauri. El build de frontend falla si los contratos generados están desactualizados. Después de modificar DTOs Rust, ejecutá `npm run contracts:generate` y revisá tanto JSON Schema como tipos TypeScript.

### UI sin ocupar la pantalla (predeterminado)

`npm run test:headless` construye el frontend de producción y CLI Rust, inicia un puente HTTP **sólo de pruebas**, en loopback y puerto efímero, y usa Playwright CLI **0.1.22** con `headless: true`, sin perfil personal. No inicia Tauri ni toma capturas del escritorio. El navegador llama al CLI real sobre SQLite/adjuntos sintéticos; no simula entidades, reglas ni persistencia. Ejecuta las mismas aserciones DOM que el recorrido nativo en seis sesiones (main/reopen/restored por tema), más teclado real de Playwright e intercalados de apertura/captura reteniendo sólo transporte. Conserva bases, backups, resultados y capturas del browser fuera del checkout, en `NOTEKA_TEST_TMP` o el temporal del sistema, con raíz canónica para admitir TMPDIR enlazado de macOS. `test:headless-bridge` comprueba aislamiento de origen/rutas/symlinks, temporal enlazado y bloqueo del runner nativo sin consentimiento; requiere CLI construido.

Las solicitudes HTTP del contexto del browser se limitan al origen local y se bloquean service workers. En macOS, cada proceso CLI corre bajo `sandbox-exec (deny network*)`; en otras plataformas no se afirma sandbox OS. npm/npx pueden necesitar red para tooling o browser inicial, no para operar contenido. Node 22 informa que `node:sqlite` es experimental; se usa exclusivamente para inyección de fixtures/fallos de prueba.

**Límite:** el puente reemplaza IPC y eventos de cierre de Tauri. Comprueba guardias, buffers, commit y lecturas reales, pero no demuestra interceptación de cierre del NSWindow, permisos/CSP del host ni foco del WKWebView. Los nombres heredados que dicen «nativo» en el resultado no convierten esas aserciones en pruebas nativas: `transport` lo identifica explícitamente. La preferencia de tema se verifica en localStorage del browser, no como almacenamiento nativo certificado.

### Host nativo (sólo con autorización de pantalla)

`test:native` necesita ventana real macOS y **puede tomar foco**. Está bloqueado salvo `NOTEKA_NATIVE_SCREEN=1`; no ejecutar ese opt-in sin autorización explícita del usuario para usar la pantalla. No se programa automáticamente como sustituto de headless.

```sh
# Únicamente después de acordar el uso de pantalla:
NOTEKA_NATIVE_SCREEN=1 NOTEKA_CAPTURE_HELPER=/ruta/al/helper-de-capturas.py npm run test:native
```

Compila el feature **exclusivo de pruebas** `native-smoke`, usa WKWebView/IPC/CLI reales e inyecta fallos SQLite sintéticos. Son seis sesiones y red denegada por sandbox macOS. El helper externo captura sólo la ventana de prueba; requiere Screen Recording. El deadline por sesión es 480 s y waits 12 s. El foco de ventana/webview se adquiere realmente; no se falsea `document.hasFocus()` ni se retiran asserts. La instrumentación no existe en el binario normal. [Estado de la cobertura pendiente](verification-home.md).

La instrumentación de prueba no existe en el binario normal. No equivale a pruebas con todos los periféricos o lectores de pantalla. El build `--no-bundle` produce `target/release/noteka-desktop`: no firma con identidad de distribución, notariza ni publica una aplicación distribuible.

## Workspace y recuperación

Ubicación por defecto en macOS: `~/Library/Application Support/Noteka/workspace`. UI y CLI usan exactamente el mismo resolver. El workspace contiene `noteka.sqlite` y, cuando SQLite lo necesita, sus archivos `-wal` y `-shm`. Metadatos/configuración no secreta (ID, versión, zona IANA) viven en `workspace_meta`. No hay cuentas, credenciales ni modelos.

SQLite v10 agregó un [índice explícito de adjuntos](attachment-search.md) independiente de `search_fts`, con control/entries/FTS/shadows/trigger; cierre histórico **19/19**. La integración XLSX añadió sólo dos tablas en v11; [búsqueda XLSX](xlsx-search.md) usa la barrera v12 sin cambiar DDL ni las once migraciones anteriores. V8 añadió cinco tablas de extracción y v9 tres PDF. Colecciones, cálculos, vistas y templates/documentos v3 se conservan. FKs activas, WAL, `synchronous=FULL` y espera máxima de escritor **2.000 ms**. Una operación ocupada devuelve `workspace_busy`; no se repite automáticamente.

Cada apertura prevalida identidad y versión sobre una copia privada, incluyendo WAL si existe. Esa copia **no** se usa ni se anuncia como backup: evita que SQLite toque el SHM de un formato futuro. Si WAL desaparece entre detección y copia por el checkpoint de la última conexión, se descarta cualquier WAL privado parcial y se vuelve a leer la base checkpointada; otros errores de IO siguen propagándose. Si falla la prevalidación, no se escriben los archivos originales. Se vuelve a comprobar la versión antes de configurar la conexión de escritura.

Backups: directorio con `noteka.sqlite` autosuficiente, `attachments/<uuid>` y `manifest.json` (`format_version:2`, ID de workspace, versión persistente, SHA-256 SQLite y lista completa de originales/hash/tamaño). Los metadatos se enumeran desde el snapshot, no desde una lectura posterior del workspace. Se usa la API SQLite de backup, con integridad/FKs comprobadas y publicación del directorio completo. No copies un `.sqlite` abierto a mano: podés omitir escrituras que están en WAL.

Restauración: valida SQLite y cada original en staging y publica en un destino nuevo o vacío. Conserva IDs/revisiones; nunca sustituye un workspace ocupado. La selección del restaurado es una acción posterior. Manifiestos v1 sin adjuntos siguen admitidos para SQLite v0/v1.

Migraciones `001.sql`–`012.sql`, sin alterar 001–011. Fixture v0 soportada: entidades/documentos/relaciones sin historial, **no versión publicada**. Migrar v0–v11 crea backup hermano `noteka-pre-migration-<id>` antes de transacción; falla conserva versión anterior/details.backup_path. >12 se rechaza; sin downgrade ni limpieza automática. No reescribe documentos/payloads/vistas/ASTs futuros. FTS de notas sólo se reconstruye al introducir v3; v10 crea vacío el índice de adjuntos y v11 añade almacenamiento XLSX vacío sin tocar cola/generaciones, incluso running. V11→v12 sólo cambia ambas versiones, sin reset/reindexación/selección implícita. Cliente viejo requiere restaurar backup anterior en otra ruta y conservar escrituras posteriores del workspace nuevo. [Guía/rollback de búsqueda XLSX](xlsx-search.md#recuperación-y-mcp) y [límites de evidencia histórica](xlsx-search-evidence-recovery.md).

## Contrato y CLI

```sh
./target/debug/noteka --help
./target/debug/noteka operations describe --json
```

La envoltura tiene `api_version: 1`, `request_id`, `ok` y `data` o `error {code,message,details}`. El catálogo incluye los esquemas de entrada/salida/errores. Tauri acepta la misma solicitud `{workspace?, command: {operation, input}}`. Origen del historial: `desktop` o `cli`; request IDs/timestamps no deben compararse entre operaciones independientes.

Cada subcomando recibe el **input**, no la envoltura, desde `--input archivo.json`, `--input -` o stdin. Entrada máxima 2 MiB; el documento tiene un límite separado de 1 MiB. Campos desconocidos de solicitudes/DTOs se rechazan.

| Grupo | Acciones |
| --- | --- |
| `workspace` | `create`, `open`, `backup`, `restore` |
| `operations` | `describe` |
| `entity` | `create`, `get`, `list`, `update`, `archive`, `restore` |
| `relation` | `add`, `remove`, `list` |
| `changes` | `list`, `preview`, `apply` |
| `attachment` | `import`, `read` |
| `document` | `export` |
| `workflow` | `query` |
| `home` | `query` (quince vistas ligeras, páginas 1–100) |
| `search` | `query`, `rebuild` |
| `collection` | `create`, `get`, `list` |
| `property` | `define` |
| `formula` | `set` |
| `record` | `create`, `get`, `attach`, `update`, `inverse` |
| `view` | `save` |
| `database` | `query` |
| `template` | `save`, `get`, `list`, `instantiate` |
| `extraction` | `enqueue`, `get`, `list`, `run`, `cancel`, `retry`, `control`, `fragments`, `pdf-fragments`, `xlsx-cells`, `index`, `search` |

En `--json`, stdout tiene un único resultado JSON, incluso en errores de argumentos; stderr queda para diagnósticos. `--help`/`--version` son salidas humanas. Códigos: **0** éxito; **2** validación, argumentos, backup inválido o destino ocupado; **3** no encontrado; **4** conflicto/contención; **5** almacenamiento; **6** formato no soportado.

### Recorrido sintético por CLI

Elegí rutas nuevas fuera del checkout. Este ejemplo usa un directorio temporal y datos de prueba:

```sh
NOTEKA_DEMO_ROOT="$(mktemp -d "${TMPDIR%/}/noteka-demo.XXXXXX")" node scripts/cli-walkthrough.mjs
```

El script ejecuta catálogo → crear workspace → crear nota desde `tests/fixtures/note.json` → crear proyecto → relacionar → actualizar → archivar/restaurar → historial → backup/restauración. Comprueba resultados y conserva fixtures para inspección.

Ejemplo individual sobre un workspace ya creado:

```sh
./target/debug/noteka --workspace /ruta/al/workspace entity create --json --input tests/fixtures/note.json
printf '%s' '{"kind":"task","title":"Pendiente"}' | ./target/debug/noteka --workspace /ruta/al/workspace entity create --json
```

Reemplazá rutas e IDs con resultados reales. Consultas: `entity get` recibe `{id}`; `entity list` admite `kind`, `state`, `classification`, `archived`, `limit` y `cursor`. Actualizar recibe `{id,expected_revision,title?,payload?,document?}`; archivar/restaurar `{id,expected_revision}`. La omisión de revisión falla; una vieja devuelve `revision_conflict` con la revisión actual. Documento, título, revisión e historial se confirman juntos.

## Lotes locales

[Contrato completo](batches.md), [fase 4 histórica](verification-phase-4.md), [PDF](verification-pdf-extraction.md), [índice verificado](attachment-search.md) e [integración XLSX verificada](xlsx-extraction.md). API v1/52, sólo nueve mutaciones/un check agrupables; doce extraction.* excluidas. Preview usa SQLite Backup completo, apply reejecuta bajo Immediate y compara plan/scope/base/efecto. Namespace base.v2 incluye control original, no actividad operativa/derivada; índice/derivados enteros cuentan para schema/snapshot 64 MiB. No migra ni publica copia; v0–v10 requieren open explícito/preview nuevo. Sin UI/modelos/red.

```sh
./target/debug/noteka --json --workspace "$WORKSPACE" changes preview --input "$PLAN"
./target/debug/noteka --json --workspace "$WORKSPACE" changes apply --input "$APPLY"
# $APPLY contiene {plan, expected: receipt exacto}; mismo plan, sin rebase/retry automático.
```

El catálogo distingue input previo a bindings de DTO resuelto y enumera slots/exports/límites. Entrada/salida 2 MiB, snapshot 64 MiB, 100 pasos/10 checks/512 bindings, contención 2.000 ms y presupuesto cooperativo 5 s. Recibos son comparación, no permiso/firma ni idempotencia persistente. Sin migración, undo ni historial persistente de esquema. SQLite puede coordinar SHM al leer WAL.

Recorrido sintético de 16 pasos: `node scripts/subscription-fixture.mjs /ruta/nueva/al/workspace`; primero crea workspace individualmente, verifica centavos/asientos/IDs en preview y aplica el mismo plan/recibo. `npm run test:batches` comprueba CLI real, errores/concurrencia, tie-break de allocate, reapertura, recálculo, pagos y backup/restauración. En macOS se ejecutó con `(deny network*)` y sin iniciar el host.

## Extracción local explícita

[Guía](extraction.md) y [ADR 0011](decisions/0011-explicit-local-extraction.md). `.txt`/`.md` <=1 MiB, UTF-8 literal, cola pausada/enqueue separado de run, sin daemon/modelos/FTS de adjuntos. Lecturas verifican original y paginan resultados; cancel/retry/control-set requieren revisión. [Verificación](verification-extraction.md) documenta caps, recovery y tiempos release.

PDF tiene [guía/lector por página](pdf-extraction.md), [ADR 0012](decisions/0012-bounded-pdf-worker.md)
y [verificación v9](verification-pdf-extraction.md) propios; no usa offsets literales.

```sh
TEMP_ROOT="${NOTEKA_TEST_TMP:-${TMPDIR}}"
BASE=$(mktemp -d "${TEMP_ROOT%/}/noteka-extraction-XXXXXX")
sandbox-exec -p '(version 1)(allow default)(deny network*)' \
  node scripts/extraction-fixture.mjs "$BASE/workspace"
```

Medición sintética release (fuera de Git):

```sh
cargo test --offline --release -p noteka-core --lib -- \
  --ignored --exact extraction_tests::release_extraction_resources --nocapture
```

Si TMPDIR largo rompe sockets de Playwright CLI, elegí `PWTEST_SOCKETS_DIR` corto y descartable. En esta verificación se usó `/private/tmp` **sólo para sockets** por el límite de 127 bytes; workspaces/resultados siguieron en el temporal aprobado. No modificar runner/asserts ni usar perfil personal. Headless no verifica host nativo.

## Índice explícito de adjuntos

[Guía](attachment-search.md), [ADR 0013](decisions/0013-explicit-attachment-index.md) y [verificación](verification-attachment-search.md). Cambio **19/19**, sincronizado y [archivado](../openspec/changes/archive/2026-10-14-add-local-attachment-search/proposal.md): regresión/revisión aprobadas, dos benchmarks con query p95 **214,20–231,07 ms** sobre el corpus fijado de 8 MiB frente a <=300 ms; lock aprobado y scratch físico medido. El script produce mediciones válidas incluso cuando `query_target_met=false`: su exit 0 no aprueba el target. Los dos informes finales sí publican `query_target_met=true`/`lock_gate_met=true`; los fallidos se conservan. No se cambiaron objetivo/caps ni se retiró validación; no garantiza ese p95 para cualquier corpus/caché.

```sh
cargo test --offline --locked --workspace
cargo test --release --offline --locked --workspace
# Copia/CLI v9 independientes fuera del checkout, no fabricados por el binario nuevo:
python3 -B scripts/test-attachment-search.py --baseline-cli "$BASELINE_CLI_V9"
python3 -B scripts/test-attachment-search.py --profile release --baseline-cli "$BASELINE_CLI_V9"
python3 -B scripts/benchmark-attachment-search.py --baseline-root "$BASELINE_ROOT_V9" --baseline-cli "$BASELINE_CLI_V9"
```

Requieren `NOTEKA_TEST_TMP` externo y binarios/helper preparados por perfil. En macOS se ejecutan con `sandbox-exec -p '(version 1)(allow default)(deny network*)'`; sin abrir UI/host. El benchmark usa cinco warmups/veinte muestras; comparación de búsqueda de 10.000 notas y de edición CLI de 10 KiB con baseline independiente. RSS SELF y TEMP lógico no son caps ni pico físico completo de scratch; la sonda física es una medición separada. La [verificación](verification-attachment-search.md) distingue mediciones, saturación funcional por separado y límites de aceptación.

## Entidades, documentos y relaciones

- `note`: inbox booleano default true y documento v1/v2/v3. Registros creados por plantilla quedan fuera de Inbox. `person`: payload `{}`. Otras entidades todavía no tienen bloques.
- `task`: state todo/done, priority low/normal/high, planned_date/due_date civiles opcionales e inbox booleano.
- `project`: state planned/doing/done/ongoing/on_hold, purpose y start_date/end_date opcionales.
- `tag`: exige `{classification: "area" | "resource" | "topic"}`; área/recurso son los mismos IDs de tag.
- Título: trim, no vacío, máximo 4.096 bytes. Tipo/ID inmutables. Instantes UTC y zona IANA del workspace obtenida del entorno, o `UTC` si no se puede detectar.
- Archivar no borra ni hace cascada; `entity.get` puede leer archivadas. Listados excluyen archivo por defecto; `archived:true` lista solamente archivadas.
- Paginación: 50 por defecto, rango 1–200. Entidades/cambios en orden ascendente `(created_at,id)`; relaciones por ID. `next_cursor` es opaco: reenviarlo con los mismos filtros. Sin garantía de snapshot entre páginas con escrituras concurrentes.

```json
{
  "schema_version": 1,
  "blocks": [
    { "id": "id-opaco-del-parrafo", "type": "paragraph", "content": [{ "type": "text", "text": "Texto" }] }
  ]
}
```

Documento: 1 MiB UTF-8 JSON compacto, 10.000 bloques totales, profundidad JSON máxima 32. IDs globalmente únicos en el árbol. V1: párrafos de texto. V2: `paragraph`, `heading` (level 1–3), `bullet_list`, `ordered_list` (start positivo), `list_item`, `task_list`, `task_item` (checked), `blockquote`, `horizontal_rule`, `code_block` (language opcional), `image` (attachment_id/alt) y `file` (attachment_id/name). Los contenedores usan `children`; las hojas `content` inline con text/hard_break y marcas bold/italic/strike/code/link. La marca code es exclusiva; enlaces http/https/mailto. Ver fixture `tests/fixtures/rich-document.json`.

V3 conserva reglas v2 y agrega la hoja atómica `collection_view` con IDs collection_id/view_id. Editor/CLI comparten validación; ninguna fila/DOM queda en el documento. [Plantillas](templates.md) y [bases incrustadas](embedded-collections.md) documentan creación, guardias, lectura acotada y límites de export/backup.

La lectura conserva JSON crudo y `document_editable`. Versión, bloque o propiedad desconocida: no reemplazar el documento; lectura/recuperación y archivo siguen disponibles. Copiar pendiente recupera **JSON**, no una conversión de texto que pierda hijos o marcas. Al pegar en el editor se inserta texto plano; no hay descarga de URLs ni importación implícita de HTML.

### Adjuntos y exportación

`attachment import` recibe `{source_path}`: archivo regular de hasta 64 MiB, copia inmutable con UUID y SHA-256 antes del commit. `attachment read` recibe `{id}` y devuelve metadatos + `base64`, después de comprobar integridad. No se ejecuta un original. Sólo PNG/JPEG/GIF/WebP se renderizan como imágenes locales. SVG se conserva como archivo, no como imagen ejecutable. Borrar el bloque conserva el original; no hay GC automático. Un crash puede dejar una copia huérfana, nunca un commit que anteceda a la copia.

La UI permite insertar por ruta explícita y exportar a un directorio nuevo. `document export` recibe `{id,destination,format:"json"|"markdown"}`. Exportación JSON: `document.json`, `entity.json`, `relations.json` (IDs como referencias), `attachments/<uuid>` y manifiesto de originales. Markdown: `document.md` y los mismos originales, con advertencia explícita de pérdida de identidades/metadatos. No importa Markdown/Notion ni exporta automáticamente las entidades relacionadas.

Relaciones:

- `project`: note/task → project.
- `tag`: entidad no-tag → tag.
- `person`: entidad no-person → person.
- `relation add`: `{source_id,target_id,kind,expected_revision}` (revisión del origen).
- `relation remove`: `{id,expected_revision}` (también revisión del origen).
- `relation list`: `{entity_id,cursor?,limit?}`, por cualquiera de los extremos.

No hay self-links, extremos ajenos/inexistentes ni inversas duplicadas. Repetir una relación existente con revisión vigente devuelve `changed:false` sin incrementar revisión. Añadir/quitar actualiza origen e historial atómicamente. Las referencias a archivadas se mantienen y se identifican en UI.

## Uso y atajos

Guardado explícito; no hay autosave ni merge automático. Ante fallo/conflicto se conserva el buffer, se ofrece reintentar, recargar con descarte explícito o recuperar/copiar contenido. Cambiar nota/sección/workspace y cerrar con cambios pendientes exige guardar, descartar o cancelar. Cerrar con descarte pierde explícitamente lo no confirmado: no existe recuperación de borradores tras un crash en esta fase.

La misma protección aplica a formularios de tareas/proyectos/tags/personas. `entity.update.payload` actualiza sólo campos explícitos conservando omitidos (null limpia fechas); no borra propiedades desconocidas. Captura en contexto confirma entidad y relación juntas. Relacionar/quitar exige guardar el buffer primero; quitar no borra entidades. [Evidencia de fase 2](verification-phase-2.md).

- `⌘N` / `Ctrl+N`: nueva nota.
- `⌘S` / `Ctrl+S`: guardar.
- `⌘⇧A` / `Ctrl+Shift+A`: archivar/restaurar selección.
- Tab/Shift+Tab: navegación; Enter/Espacio: activar controles; Escape: cancelar diálogo.
- Botón Actualizar y activación de ventana: refrescar sin reemplazar un buffer pendiente.
- `/`: menú de bloques; Tab/Enter o click para elegir; Escape para cancelar.
- `⌘B`, `⌘I`, `⌘Z`, `⇧⌘Z`: formato y undo/redo de ProseMirror. Se aplican al contenido del editor, no al historial de dominio.

No se usan fuentes, iconos ni recursos de red. La CSP no habilita proveedores externos, shell ni acceso general al filesystem desde el frontend.

## Evidencia y límites

### Search con evidencia local

[Guía de uso](evidence-search-ui.md), [matriz](verification-evidence-search-scenarios.md)
y [verificaciones ejecutadas](verification-evidence-search-ui.md). No añade dependencias,
contratos o migraciones. Search sigue sin procesar; la preparación se solicita
explícitamente desde su panel independiente o CLI/MCP.

```sh
npm run test:attachment-search # 14 tests cliente/promesas, no IPC real
npm run test:headless-bridge   # aislamiento + fixture CLI + inmutabilidad
npm run test:headless          # seis sesiones/606 checks anteriores intactos
npm run test:headless-evidence # build + recorrido separado, CLI/SQLite real, ambos temas
```

Para conservar fixtures/logs fuera del checkout:

```sh
export NOTEKA_TEST_TMP="$HOME/.local/share/noteka-verification/evidence-ui-runtime"
mkdir -p "$NOTEKA_TEST_TMP"
python3 -B scripts/prepare-evidence-search-fixture.py --root "$NOTEKA_TEST_TMP/nuevo-fixture"
```

`--root` debe ser nuevo y confinado al directorio existente; la preparación usa CLI
debug y helpers ya compilados (opcional `--profile release`). Los generadores del
corpus no se modifican. Sólo se inyecta configuración futura incompatible en una
generación real sin seleccionar, antes del snapshot; no se inventan hits.
El recorrido headless usa perfil efímero/loopback y niega red en los subprocesos CLI
macOS. Capturas son del navegador sintético, no de la pantalla. Cada search UI
compara SQL lógico/esquema y archivos fuente/derivados antes/después, incluso en
errores. El costo de esa instrumentación no es un benchmark del producto.

Evidencia de [bootstrap](verification-phase-0.md), [editor](verification-phase-1.md), [productividad](verification-phase-2.md), [búsqueda](verification-text-search.md), [colecciones](verification-phase-3a.md), [fórmulas](verification-phase-3b.md), [extracción UTF-8](verification-extraction.md), [PDF acotado](verification-pdf-extraction.md), [índice de adjuntos](verification-attachment-search.md) y [MCP local](verification-mcp.md). Las pruebas Rust usan SQLite real; las de CLI lanzan procesos. [Contrato y medición FTS](search.md): índice reconstruible con entidad/bloque/revisión; no busca adjuntos. El recorrido nativo no sustituye firma/notarización ni pruebas de distribución. Fórmulas, extracción literal/PDF, índice y MCP funcionan sin IA; PDF amplio/Excel/OCR, harness/embeddings, MCP remoto y Drive siguen pendientes. No se activan proveedores externos ni se descargan modelos al iniciar.

### Preparar, leer y consultar en grupos

La [preparación UI](attachment-preparation-ui.md) expone operaciones explícitas
de cola/selección del índice; el [lector](attachment-reader-ui.md) pagina cualquier
generación compatible y la [consulta agrupada](grouped-search.md) conserva dos
motores/snapshots. [Aceptación conjunta](verification-local-preparation-closure.md).
El hook Tauri usa `extraction:prepare` y prepara PDF/XLSX en perfil/target solicitado.
El script `pdf:prepare` anterior sigue disponible.

```sh
npm run test:attachment-preparation
npm run test:generation-reader
node --experimental-strip-types --test tests/grouped-text-search.mjs
npm run test:headless-preparation
npm run test:headless-reader
node scripts/headless-grouped-smoke.mjs
```

Son pruebas headless/CLI con fixtures externas; no abren ventanas ni certifican
IPC/cierre/foco nativos. El lector aislado `--isolated` es evidencia distinta de
su integración App. No se instala un modelo ni se activa proveedor remoto.

### OCR y artefactos por perfil

[OCR integrado](image-ocr.md) está verificado; [resultados y límites](verification-image-ocr.md).
El preparador general agrega el helper Swift sólo para macOS; `--tauri` prepara
el staging y el overlay macOS incluye tres sidecars. Las pruebas del lector
privado permiten `--wire-version 1` o `2`; integrar OCR usa v2 y no cambia v1.

`tauri-build` copia los sidecars de `src-tauri/binaries/` al directorio del perfil
de Cargo. Un staging debug compartido puede reemplazar los helpers de release
si después se construye todo el workspace. No deducir perfil o identidad de un
helper sólo por el nombre de su directorio. Registrar hashes de los binarios que
se ejecutan y conservar copias durables antes de mezclar builds del host.

Para aceptación sin ventanas, separar build/tests del host y runtimes de CLI:
construir el host en un target externo, luego preparar los helpers del perfil
efectivo y verificar el núcleo sin volver a construir desktop entre ese paso y
los oráculos/gates. `cargo test --workspace --exclude noteka-desktop` evita esa
copia; los tests del host se ejecutan por separado. El conjunto de pruebas debe
seguir completo. Compilar paquetes concretos puede producir hashes distintos
del workspace por el conjunto de features, aunque compartan fuentes; conservar
la procedencia, sin presentar una copia restaurada como rebuild reproducible.
