# Verificación de extracción XLSX local

Fecha: 15/10/2026. **Aceptación final3 aprobada, 11/11 tareas; sincronizado y archivado.**
[Cambio](../openspec/changes/archive/2026-10-15-add-local-xlsx-extraction/proposal.md),
[guía](xlsx-extraction.md) y [ADR 0017](decisions/0017-persisted-xlsx-cells.md).
SQLite v11/API v1/52, `xlsx_cells` v1 y MCP 24/44/45, sin nueva UI ni FTS XLSX.

## Alcance y baseline

- Lector Transitional previamente archivado, sin cambios a ZIP/XML/runtime PDF;
  core enlaza sólo sus DTOs/protocolo con JsonSchema. Persistencia nueva 011,
  cola explícita y lectura tipada CLI/MCP; sin UI ni FTS XLSX.
- Baseline independiente externo: **355 archivos**, ocho ejecutables debug/release
  y catálogo v1/51. No existe commit inicial: una comparación con Git vacío no
  demuestra preservación. Los hashes/copia previos son la referencia.
- Evidencia externa en
  `/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-xlsx-integration.E3aeuW/`.
  `env.sh` recupera tooling y `NOTEKA_TEST_TMP` externo. No datos personales,
  modelos, credenciales, backups ni transcripts en Git.

## Evidencia ya observada

| Check | Resultado observado | Evidencia externa |
| --- | --- | --- |
| Rust workspace debug/release | 261 tests aprobados por perfil, ocho ignores intencionales de medición | `rust-{debug,release}-final2.log` |
| CLI/golden XLSX debug/release | 216 casos por perfil; páginas, manifiestos, JSON/chunks/hash, restore/read sin helper | `cli-{debug,release}-final3.log` |
| Migración desde binario v10 debug/release | Dos estados por perfil, 34 tablas anteriores conservadas por estado, backup v10 recuperado con cliente anterior y originales idénticos | `migration-{debug,release}-final3.log` |
| MCP SDK debug/release | 19 tests por perfil; modern/legacy, golden XLSX, permisos, lectura sin helper y generación previa tras retry fallido | `sdk-{debug,release}-final3.log` |
| Procesos CLI/MCP debug/release | Diez checks por perfil: dos eras/tres permisos, cancelación/publicación con barrera, lectura durante run, escritor disponible, singleton y apagado del helper | `processes-{debug,release}-final3.log` |
| MCP raw debug/release | Startup puro, protocolos, caps, contención, EOF/broken pipe, respuesta excedida y helper PDF | `raw-{debug,release}-final3.log` |
| Lector XLSX parser/helper | 216 casos por vía y perfil; mutaciones deterministas, protocolo y contención | `xlsx-reader-{debug,release}-final3.log` |
| PDF parser/CLI/contención | 235 casos por parser/CLI y perfil; doce casos de contención por perfil | `pdf-{parser,cli}-{debug,release}-final3.log`, `containment-{debug,release}-final3.log` |
| Frontend/adaptadores/lotes | 13 tests de adaptadores/cola/tema, cuatro de lotes y tres del puente/consentimiento | `{editor,batches,bridge}-final3.log` |
| UI headless | 592 checks compartidos + 14 de teclado, seis sesiones, ningún origen externo | `headless-final3.log`, `runtime/noteka-headless-oD0jye/` |
| Contratos completos | 51 inputs/outputs/policies y 164 definiciones previas; sólo adiciones XLSX nombradas y nueve mutaciones idénticas | `compat-final3.log`, `contracts-final3.log` |
| Calidad/build | fmt/check/Clippy workspace y host con features; frontend y host no-bundle sin ejecutar ventana | `{fmt,check,clippy,clippy-host}-final2.log`, `host-final3.log` |

Todos estos checks pasaron. Un build o `openspec validate` no sustituye las pruebas
de ejecución; no se ejecutó ni se anuncia aceptación nativa de Home.

El código Rust/SQL definitivo pasó fmt/check/Clippy (incluido host con features
de pruebas, sin abrirlo). `source-pins-final3.json` fija **153 archivos de código,
scripts, contratos y fixtures**; entre final2 y final3 cambió solamente la
expectativa del diagnóstico terminal en el harness de procesos. El apagado real
XLSX observado fue **4,18–10,64 ms** entre EOF/broken pipe y salida, por debajo del
límite de prueba de 4 s y sin esperar el watchdog de 5 s.

## Mediciones release

M1 Max, **68.719.476.736 bytes/64 GiB**, aarch64, tooling registrado en
`environment.txt`. Ejecución secuencial, sin compilaciones/pruebas concurrentes.
Original <=1 MiB, JSON canónico <=1 MiB, chunks <=4096 B/512, retención mixta
<=32 MiB y response <=2 MiB; helper conserva heap solicitado 128 MiB/CPU 4 s/
watchdog 5 s. No se elevó ningún cap ni se retiró validación.

`benchmark-xlsx-final3.log` apunta a
`runtime/xlsx-integration-bench-yhmmhj4h/results.json`: **77 casos admitidos**,
cinco warmups/veinte muestras por caso, core real y helper descartable por run.
Máximo p95 de ejecución **85,82 ms**, máximo individual **98,93 ms**, lectura de
hasta 64 celdas p95 máximo **5,65 ms** y locks claim/publicación p95 máximo
**6,90 ms** frente a 150 ms. Mide operaciones core, no render/UI ni todas las
combinaciones de workspace/CLI. La saturación exacta/+1 se verifica aparte en Rust.

RSS high-water del proceso de tests: **34.455.552 B**; RUSAGE_CHILDREN para hijos
recolectados: **40.189.952 B**. Son las magnitudes informadas por getrusage en
macOS, no heap solicitado, un pico simultáneo sumable ni una garantía RSS futura.

Dos gates FTS consecutivos, script/corpus/asserts intactos, cinco warmups/veinte
muestras por caso y corpus fijado de 8 MiB:

| Gate | Query p95 (todos los casos) | Máximo lock p95 | Resultado |
| --- | --- | --- | --- |
| `runtime/attachment-bench-kjiybirq/` | 111,15–125,81 ms | 19,86 ms | query <=300 ms y lock <=150 ms |
| `runtime/attachment-bench-7i0ss86s/` | 133,19–146,42 ms | 18,05 ms | query <=300 ms y lock <=150 ms |

En el segundo gate, no-op tuvo p95 237,14 ms y máximo 466,21 ms: no es un caso
query ni se oculta su variabilidad. Los objetivos se aplican a los casos declarados;
no se afirma que cualquier operación/corpus/caché dure menos de 300 ms.

## Matriz de requisitos y escenarios

Los cuatro deltas contienen **14 requisitos/44 escenarios** (incluidos los
heredados modificados). Esta matriz identifica los tests concretos; su aprobación
final depende de los logs del código definitivo, no de la existencia del test.

| Requisito | Evidencia ejecutable |
| --- | --- |
| Contrato compartido y versionado | `check-xlsx-contracts.py`, enumeración CLI, `complete_schemas_pages_and_scoped_cursors`, SDK catálogo |
| Operaciones aditivas de indexación y búsqueda de adjuntos | Huellas de 51 contratos, regresión attachment_search/scoped, SDK UTF-8/PDF y gates intactos |
| Ejecución scoped sobre workspace actual | `scoped_tests`, startup raw anterior/futuro/inconsistente, checks de identidad efectiva y lectura READ_ONLY |
| Contrato compartido de lectura XLSX | CLI 216 casos subcomando/golden, request execute_json usado por invoke comparado con reader Rust en `explicit_worker_pagination_lexemes_and_cursors`, catálogo completo y rechazo batch |
| Workspace fijo y admisión sin migración | Raw startup/discovery puro e identidad sustituida; harness real v10; scoped tests |
| Permisos locales efectivos por operación y acción | SDK modern/legacy/read-write-run y raw seis combinaciones XLSX, permisos antes de efectos, batch/cancel anteriores |
| Superficie acotada y catálogo descubrible | SDK y raw tools/list, siete exclusiones, resource/schema y nombres derivados del core |
| Lectura XLSX bajo permisos MCP existentes | SDK golden tipado/sin helper, raw `test-xlsx-processes.py` modern/legacy y comparación CLI |
| Lector XLSX independiente y sin activación implícita | Corpus `test-xlsx-reader.py` parser/helper, `explicit_worker_pagination_lexemes_and_cursors`, CLI admisión sin intentos |
| Selección explícita de XLSX administrado | CLI 216 casos, dedup/pausa/retry sin helper, `xlsx_admission_future_configs_and_mime_do_not_spawn` |
| Publicación XLSX cercada y presupuesto compartido | Fences 3 etapas × 6 causas, deadline antes de commit, 32 MiB exacto/+1 mixto, supervisor adversarial, barreras CLI/MCP y EOF/broken pipe |
| Lectura paginada de celdas XLSX con procedencia | Golden completo/empty/sparse/hojas/caches, Unicode largo/chunks, cursores ligados y reemplazados, matriz de corrupción, open y restore alterados |
| XLSX excluido de indexación textual implícita | Unit preservación de índice UTF-8, SDK rechazo XLSX, corpus PDF y dos gates FTS sin caps/asserts modificados |
| Migración aditiva y recuperación de derivados XLSX | Real v10 dos estados por perfil; rollback Rust; restore válido sin helper; backup alterado con hash externo reparado; futuros sin descarte |

## Fallos conservados

- Primeras regresiones: registry cerrado sin 011 y fixtures históricas que aún
  asumían v10/futuros y filtrado de contratos. Se conserva la huella histórica;
  sólo se excluyen adiciones al contrastar contratos anteriores.
- El primer harness CLI esperaba un error del parser para `not-zip`, pero la firma
  PDF se identifica al importar. Ahora prueba rechazo por MIME en admisión y
  mantiene los **216 casos**, sin cambiar el golden.
- El harness de migración inicialmente usó una lista de objetos PDF como bytes y
  asumió `index_revision` en vez de `revision`. Se corrigió el harness, no el DTO.
- La tercera regresión falló en la prueba de kill/reap: el PID escrito por el hijo
  no estaba garantizado antes del deadline. El test observa ahora el PID registrado
  por spawn y comprueba `ESRCH` después del retorno; no se relajó el kill/reap.
- La primera ejecución del test de puente denegó toda la red al proceso Node y
  recibió `listen EPERM` en loopback. El runner necesita HTTP local; cada CLI
  continúa bajo red denegada. Se preserva el fallo y se repite sin cambiar asserts.
- La revisión del código encontró que open v11 sólo comprobaba estructura y que
  faltaba probar restore con un backup XLSX alterado. Se añadió validación XLSX
  integral en el snapshot efectivo de open y tests de corrupción fuera de página,
  originales alterados, hash externo reparado y destino ocupado intacto.
- La revisión también detectó que el supervisor XLSX no consultaba la señal scoped
  de apagado que ya consulta PDF. Ahora se comprueba antes de spawn y durante el
  loop, con pruebas reales EOF/broken pipe y preservación de la generación previa.
- `final1` pasó Rust, corpora y SDK de ambos perfiles, pero falló la nueva prueba
  de procesos por exigir stderr vacío en broken pipe. `final2` identificó el
  diagnóstico real `MCP: output_closed.` en transport.rs, además del diagnóstico
  de lifecycle de main.rs. El harness acepta hasta dos líneas, sólo esas constantes,
  conserva el límite de 4 s y exige helper recolectado, job failed
  sin ownership y generación previa íntegra. No se cambiaron asserts anteriores.
- Clippy inicial señaló un `format!` literal y un assert constante del benchmark;
  se corrigieron sin desactivar lints. El comparador inicial omitía las dos nuevas
  propiedades del root del catálogo; ahora elimina sólo las adiciones nombradas
  y exige igualdad profunda del resto, incluidos 164 DTOs anteriores y nueve
  mutaciones. `compat-final1.log` ya pasó esa comparación.

## Revisión y preservación

Revisión final de código/diseño y matriz completa de **14 requisitos/44 escenarios**
concluida; cuatro deltas contrastados por bloques exactos con main. Se conservaron
**59 requisitos no afectados** de esas cuatro capabilities (22/8/17/12), incluidos
los escenarios heredados de los requisitos modificados.

Baseline: **314/355 archivos idénticos**, 41 cambios intencionales revisados y
ningún archivo anterior eliminado. **169 archivos protegidos** idénticos:
Home/UI/host, migrations 001–010, parsers/runtime PDF/XLSX, corpus/golden,
gates/headless y archivos históricos incluidos en el baseline. Los ocho binarios
baseline y nueve fingerprints históricos también permanecen idénticos. Los únicos
cambios en el protocolo XLSX son derives JsonSchema. Lock: sólo enlaces de
dependencias locales/core/protocolo a Schemars ya fijado; sin nuevos paquetes.
Evidencia: `reviewed-baseline-final3.diff`, `closure-audit-final3.json`,
`protected-audit-in-progress.json`, `sync-comparison-final3.json` y 153 source pins.

Auditoría de links locales: ningún enlace roto nuevo. Dos links relativos del
bootstrap archivado ya estaban rotos antes y se conservaron sin reescribir historia.
No se usa el diff de un Git sin commit como prueba de preservación.

Archivo: `2026-10-15-add-local-xlsx-extraction`, ocho artefactos idénticos al
snapshot listo para archivar salvo el link relativo de evidencia en tasks.
`openspec validate --all --strict`: **15/15 antes, 14/14 después**; main conserva
cuatro deltas exactos. Sólo Home sigue activo 9/17. El CLI emitió la advertencia
genérica de tamaño de propuesta, no un error de validación. No commits, pushes,
publicación ni archivos de ejecución/modelos/credenciales añadidos a Git.

## Límites de la evidencia

No Excel genérico, cálculo, OCR, inferencia de encabezados, indexación XLSX,
embeddings, modelos, remotos o consentimiento de envío. Originales/SQLite siguen
siendo autoridad. Los helpers tienen heap solicitado/CPU/watchdog acotados, no
sandbox OS de filesystem/red ni garantía RSS universal.

Home continúa **9/17**, independiente y sin sincronizar/archivar. Headless no
acredita IPC/foco/cierre nativos; no se abrió ninguna ventana de escritorio.
Este recorte no completa toda 5a ni habilita por sí solo todas las dependencias 5b.
