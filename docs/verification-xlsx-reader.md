# Verificación del lector XLSX independiente

Fecha: **15/10/2026**. Cambio `add-bounded-xlsx-reader`, seis requisitos y
dieciséis escenarios añadidos a `local-extraction`. No habilita XLSX en la cola,
MCP, SQLite, UI o FTS. Ver [alcance/límites](xlsx-reader.md),
[ADR 0016](decisions/0016-bounded-xlsx-reader.md) e investigación del cambio.

## Estado de evidencia

**8/8 tareas verificadas**, seis requisitos/dieciséis escenarios sincronizados y
contrastados íntegramente con main; [cambio archivado](../openspec/changes/archive/2026-10-15-add-bounded-xlsx-reader/proposal.md).
Implementación, corpus, regresión final3 debug/release y 606 checks headless
aprobados; [revisión final](../openspec/changes/archive/2026-10-15-add-bounded-xlsx-reader/verification.md).
No atribuye al producto extracción XLSX integrada ni aceptación nativa de Home.

## Equipo y preservación

- Apple M1 Max / 68.719.476.736 bytes (64 GiB), macOS, Rust 1.99.0
  (`b940084d7`, 28/09/2026), Cargo 1.99.0, Node 22.22.3.
- Auditoría externa:
  `/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-xlsx-apply.NjmvQr`.
  En adelante `$XLSX_AUDIT`; `source "$XLSX_AUDIT/env.sh"` restaura tooling y
  `NOTEKA_TEST_TMP` externo.
- No hay commit inicial. `baseline.json` y `baseline/` conservan **326 archivos**
  con SHA-256 y seis binarios CLI/MCP/PDF debug/release independientes; el catálogo
  baseline fue contrastado con el cierre MCP final7. Git vacío no se usa como
  referencia de conservación.
- Primer audit de implementación: sólo Cargo.toml/lock y Cargo.toml/containment
  de PDF modificados entre los archivos previos. La implementación nueva está en
  crates privados; core, frontend, migraciones y Home no se tocaron.
- La contención movida es idéntica al baseline salvo el nombre del thread
  `pdf-budget` → `parser-budget`; los límites, algoritmos y tests se conservan.

## Pruebas iniciales y aceptación

`corpus-debug-initial.json`: **161 casos × parser/helper**, protocolo,
concurrencia y pipes aprobados. `corpus-debug-review2.json`: **197 casos × dos
entradas**, incluidos descriptors Stored/Deflate firmados/sin firma, CRC real,
Unicode/UTF-16, números léxicos, fórmulas y todas las fronteras. `unit-review2.log`:
dos tests del allocator, seis de protocolo y tres del parser. `clippy-initial.log`
aprobado con `-D warnings` antes de completar la revisión.

El corpus final fijado tiene **216 casos: 77 positivos/139 negativos**. El harness
añade 512 mutaciones deterministas (seed 20261015), doce ejecuciones concurrentes,
validación independiente de salidas adulteradas, framing crudo y contención de
pipes/padre. El golden completo se calcula en Python, no de la salida Rust.
`golden.sha256` fija generador y manifiesto; no se omiten los fallos con un skip.

| Comprobación final3 | Evidencia externa y resultado |
| --- | --- |
| Rust | `fmt-final3.log`, `check-final3.log`, `clippy-final3.log`: workspace/all-targets, offline/locked y Clippy `-D warnings` aprobados. `rust-{debug,release}-final3.log`: **245 tests por perfil**, siete ignores preexistentes de probes/benchmarks explícitos, ninguno nuevo. |
| XLSX | `xlsx-{debug,release}-final3.json`: **216 casos × parser/helper por perfil**, golden completo; 512 mutaciones deterministas sin crashes, doce lectores concurrentes, diecisiete respuestas adulteradas + dos tamaños/truncamiento rechazados y dieciséis errores crudos de framing. Pipes/padre comprobados; cwd sin archivos, stderr vacío. |
| No activación | `nonactivation-{debug,release}-final3.json`: v10/API v1/51 y catálogo completo idéntico al baseline; XLSX rechazado por extractores actuales, cola pausada/sin jobs y lecturas sin cambios de SQLite. |
| Regresión PDF | `pdf-{parser,cli,containment}-{debug,release}-final3.json`: 235 casos por parser y CLI real en cada perfil; doce probes de contención, restauración/lectura sin helper. No se modificó corpus ni expectativas PDF. |
| MCP | `sdk-{debug,release}-final3.log`: **17 tests por perfil**. `wire-{debug,release}-final3.log`: trece grupos de protocolo crudo/concurrencia por perfil, incluidos cierre/cancel/EOF/broken pipe. |
| Frontend | `frontend-final3.log`: contratos, Svelte/TypeScript y build aprobados. `editor-final3.log`: trece tests; `bridge-final3-loopback.log`: tres tests del puente/consentimiento. Warning Vite de chunk >500 KB y Node SQLite experimental conservados, no silenciados. |
| Headless | `headless-final3-retry.log`: **592 checks + catorce de teclado/intercalados**, seis sesiones/fixtures dark/light y reporte `external_origins: []`; service workers bloqueados, red CLI denegada. Perfil efímero propio, sin pantalla ni perfil personal. |
| Huella de fuente | `source-final3.json`: 107 archivos de código/contratos/corpus/harness fijados y contrastados después de pruebas. Baseline: sin cambios en core/CLI/MCP/migraciones/frontend/host/Home; sólo cuatro archivos previos de implementación cambiaron, más documentación de estado. |

Los siete ignores heredados son cuatro benchmarks/diagnóstico explícitos y dos
fixtures de procesos hijos, más el benchmark PDF. El gate de adjuntos se ejecutó
aparte dos veces, sin cambiar fuente ni expectativas. No se presentan ignores
como pruebas pasadas ni se usa `validate` como validación de aplicación.

## Medición reproducible release

`benchmark-final3.json`: **81 casos**, todos los 77 admitidos y cuatro rechazos
por saturación (celdas, resultado, entrada y expansión total), tres warmups y
veinte muestras por caso. Comando real: `sandbox-exec -p '(version 1)(allow default)(deny network*)' python3 -B scripts/benchmark-xlsx-reader.py`.
Cada muestra verifica la salida completa y usa `/usr/bin/time -lp` del hijo; wall
incluye arranque del proceso/launcher/pipes, no sólo el parser. Sin compilaciones
paralelas. No se extrapola a latencia integrada de cola ni a libros fuera del corpus.

- Máximo p95 por caso: **25,849 ms**, `logical-escaped-inline-131072`;
  máximo de ese caso **26,269 ms**. RSS máximo de todo el corpus:
  **36.323.328 bytes**, `total-expansion-16777216` (p95 **19,654 ms**).
  Máxima muestra individual global: **27,405 ms**, `logical-escaped-str-131072`;
  RSS observado global **2.195.456–36.323.328 bytes**.
- Saturación significa aquí límites de contenido, no una afirmación de throughput
  a carga sostenida. El harness comprueba aparte doce lectores concurrentes y
  exceso de heap/CPU/deadline mediante el runtime compartido.
- `fts-gate-final3-{1,2}.log` y `fts-summary-final3.json`: dos gates consecutivos
  sin cambiar el corpus de adjuntos, caps, validación completa ni target **300 ms**.
  Query p95 **121,38–146,34 ms**, máximo lock p95 **20,86 ms** (<150 ms).
  Es evidencia sobre el corpus fijado, no una garantía universal de rendimiento.

Comandos principales (con `env.sh` y `NOTEKA_TEST_TMP` externo):

```sh
cargo fmt --all --check
cargo check --offline --locked --workspace --all-targets
cargo clippy --offline --locked --workspace --all-targets -- -D warnings
# Repetir con --release para el otro perfil; Rust/CLI/helpers con red denegada.
cargo test --offline --locked --workspace
cargo build --offline --locked -p noteka-cli -p noteka-mcp -p noteka-pdf-worker \
  -p noteka-xlsx-worker -p noteka-xlsx-protocol --bins --examples
python3 -B scripts/test-xlsx-reader.py --profile debug
python3 -B scripts/test-xlsx-nonactivation.py --profile debug \
  --baseline-catalog "$XLSX_AUDIT/catalog-baseline.json"
python3 -B scripts/test-pdf-parser.py --profile debug
python3 -B scripts/test-pdf-cli.py --profile debug
python3 -B scripts/test-pdf-containment.py --profile debug
NOTEKA_CLI="$PWD/target/debug/noteka" NOTEKA_MCP="$PWD/target/debug/noteka-mcp" \
  node --test tests/local-mcp.mjs
python3 -B scripts/test-mcp-transport.py --profile debug
cargo test --release --offline --locked -p noteka-core \
  attachment_search_tests::release_attachment_index_resources -- \
  --ignored --exact --nocapture --test-threads=1
```

Cada comando Rust/CLI anterior fue envuelto en
`sandbox-exec -p '(version 1)(allow default)(deny network*)'`. El puente headless
necesita loopback; mantiene denegada la red de los hijos CLI.

## Cierre documental y conservación

`closure-audit-archived-final3.json`: siete artefactos archivados (seis idénticos
byte a byte, el reporte sólo ajusta su enlace relativo tras moverlo), seis
requisitos/dieciséis escenarios idénticos a main y los doce requisitos anteriores
de `local-extraction` íntegros. **107 pins final3 intactos**, 110 archivos protegidos
de core/CLI/MCP/frontend/host/Home sin cambios; 322 enlaces relativos válidos en
el barrido ampliado. Dos enlaces rotos de archivos históricos anteriores ya
estaban en el baseline y se conservaron sin alterarlos; ningún enlace nuevo roto.
OpenSpec estricto después de archivo: **14/14**. Único cambio activo restante:
Home **9/17**, sin sync/archivo. El siguiente cambio de integración partirá de
un baseline independiente nuevo; esta entrega no migra ni habilita XLSX en la cola.

## Matriz de trazabilidad

| Requisito | Escenario | Evidencia ejecutable |
| --- | --- | --- |
| Lector independiente | Lectura explícita offline | Corpus parser/helper, comparación completa; sandbox deny network |
| Lector independiente | Producto existente conservado | test-xlsx-nonactivation.py, catálogo externo baseline, v10/cola pausada/sin jobs |
| Contenedor cerrado | Contenedor malformado o ambiguo | CRC/descriptors/duplicados/offsets/truncado/headers/padding Deflate |
| Contenedor cerrado | Contenido activo o externo | macros/content types/connections/ActiveX/enlaces/TargetMode externos rechazados |
| Contenedor cerrado | XML hostil o variante no admitida | DTD externo/interno, entidades, encoding, namespace Strict/ajeno y alternate content |
| Celdas tipadas | Números fechas y valores vacíos | ocho lexemas numéricos, booleanos/errores/fecha declarada, strings vacíos/cached null, estilos/1904 |
| Celdas tipadas | Cadenas y hojas dispersas | shared/inline/rich/CDATA/escapes, visibilidad/nombres Unicode, XFD1048576 y merge gigante sin expansión |
| Celdas tipadas | Celdas o referencias incoherentes | coordenadas/filas/orden duplicados, índices/metadata/extensiones/tipos inválidos |
| Fórmulas/caché | Fórmula con caché y sin caché | normal con/sin v, array, caché ausente distinta de v vacío inválido |
| Fórmulas/caché | Fórmula compartida | ancla/si/ref válidos; sin ancla, si inválido, seguidores fuera de rango/anclas duplicadas rechazados |
| Fórmulas/caché | Fórmula no admitida | data-table/flags rechazados; WEBSERVICE literal conservado sin red |
| Presupuestos | Fronteras y expansión | source/entries/entry/total/XMLdepth/nodes/sheets/cells/styles/strings/merges/text/result exacto y +1 |
| Presupuestos | Exceso de respuesta | result 1 MiB/+1, json_size incluye escaping, Reply::decode/version/hash/tipos/rangos/8 MiB+1 |
| Vida acotada | Pipes y caída del consumidor | stdin header/source/EOF bloqueados; stdout sin lector; consumidor cerrado; padre SIGKILL con stdin aún abierto |
| Vida acotada | Fallos de memoria panic y presupuesto | probes de allocator/realloc/concurrente, panic/abort/CPU/deadline y límites POSIX heredados íntegros de PDF |
| Vida acotada | Lecturas concurrentes y regresión PDF | doce lectores paralelos; corpus parser/CLI PDF y doce casos de contención por perfil |

## Dependencias y límites de la revisión

`dependency-closure-final1.json` registra la clausura **standalone** del helper para
`aarch64-apple-darwin`, edges normal/build: **38 paquetes, 35 externos y tres
locales**, todos con licencia declarada. No es el lock completo multiplataforma
ni la clausura de toda la app; compilar junto con otros miembros puede unificar
features. Las dependencias nuevas de registro son zip 9.0.3, roxmltree 0.21.1,
typed-path 0.12.3, nix 0.31.3 y cfg_aliases 0.2.2; ninguna versión previa se cambia.

Investigación oficial de API/versiones/licencias y RustSec documentada en
[research.md](../openspec/changes/archive/2026-10-15-add-bounded-xlsx-reader/research.md).
No se ejecutó cargo-audit (no está instalado) ni se
declara seguridad universal, conformidad OOXML completa o licencia de distribución
de la app. Nuevos crates MIT/publish=false, corpus propio, sin publicación.

## Historial y brechas que no se ocultan

- Primer comando de Cargo falló por PATH ausente; el env reproducible lo resolvió.
  El fetch siguiente falló por offline heredado: sólo la descarga de dependencias
  se ejecutó con CARGO_NET_OFFLINE=false. Pruebas/builds van offline y red denegada.
- El wrapper zsh del primer corpus intentó asignar la variable readonly `status`:
  falló después de que el harness terminara aprobado. JSON y stderr vacío se
  inspeccionaron; no se cuenta ese wrapper como aceptación final.
- Dos patches no aplicados por contexto/transporte no modificaron archivos; se
  rehicieron explícitamente. No se retiró ninguna aserción heredada.
- `xml-review-probes-final1.json` reprodujo dos defectos adicionales: declaración
  XML con tabulador/ISO-8859-1 aceptada indebidamente y texto a profundidad 64
  rechazado como 65. La tarea de contenedor se reabrió; los recorridos final1 no
  bastaron para cierre aunque pasaran el corpus anterior. Se corrigió contando
  elementos y reconociendo todo whitespace XML de la declaración; doce casos
  añadidos y aceptación final3 completa aprobada, sin cambiar los 198 anteriores.
- `bridge-final1.log` falló dos tests por EPERM al imponer deny network sobre el
  servidor loopback del puente. `bridge-final1-loopback.log` repitió los tres sin
  cambiar tests y pasó; CLI hijos siguen con sandbox de red del harness existente.
  No se presenta ese puente como ejecución totalmente sin red ni aceptación nativa.
- `escaped-text-review-final2.json` reprodujo otra frontera: 131.072 bytes lógicos
  codificados con `_x0041_` fallaban en inline/shared/str por contar el texto XML
  antes de decodificar. Fuente de ~920 KB y salida lógica dentro del cap. Se reabrió
  2.2: se corrigió el conteo después de unescaping, manteniendo fuente/entrada/
  expansión/representación/heap; seis casos frontera/+1 añadidos, los 210 anteriores
  idénticos y aceptación final3 completa aprobada. No se subieron caps.
- Dos scripts ad hoc de resumen final fallaron por `tomllib` ausente en Python del
  sistema y por asumir `lock_ms` en filas de consultas read-only. No fueron fallas
  de aplicación ni alteraron logs/tests: se corrigió el resumen y verificaron datos
  crudos, pins y diff. La medición original y gates se conservaron íntegros.
- `headless-final3.log` falló antes de abrir el navegador: ruta de socket de
  Playwright de 127 bytes. Se repitió sin tocar tests con `PWTEST_SOCKETS_DIR` en
  un temporal más corto bajo la misma raíz externa autorizada: 606 checks aprobados
  en `headless-final3-retry.log`. El log fallido se conserva y no cuenta como éxito.
- No se prueba Excel real, compatibilidad visual, recálculo ni archivos del
  usuario. El corpus sintético prueba el subconjunto declarado, no todos los XLSX.
- Heap Layout vivo no equivale a RSS. Watchdog/rlimits no equivalen a sandbox de
  seguridad. Headless/compilación tampoco equivalen a aceptación nativa de Home.
- Home conserva sus ocho tareas nativas pendientes y 9/17; no se usa pantalla,
  perfiles de navegador, modelos, OAuth, proveedores, pagos o publicación.
