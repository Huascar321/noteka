# Verificación de búsqueda XLSX

Estado al 16/10: **revalidación alternativa autorizada aprobada**, tras perder la
evidencia temporal y cancelarse `final5-*`. **19/19 tareas verificadas**;
cinco deltas sincronizados, contrastados y [cambio archivado el 16/10](../openspec/changes/archive/2026-10-16-add-local-xlsx-search/proposal.md).
[Alcance autorizado y limitaciones](xlsx-search-evidence-recovery.md).
Las secciones históricas de abajo no implican disponibilidad de las rutas perdidas.

## Revalidación durable del 16/10

Directorio nuevo: `~/.local/share/noteka-verification/xlsx-search-20261016.fikBaL/`.
`env.sh` fija Rust/Cargo 1.99.0 y temporales sintéticos bajo ese mismo directorio;
`restart-snapshot-pins.json` identifica **405 archivos actuales v12**, no v11.
La instalación está aislada: sin alterar PATH global ni actualizar lockfiles.

- `fixture-migration-fourth-debug.log` aprobó fixtures SQL v10/v11 con 34/36
  tablas anteriores preservadas, DDL/shadows/hashes/originales conservados;
  admisión CLI/MCP rechazada sin escrituras, backup exacto y restore con cliente
  **actual** que abre/migra explícitamente a v12. Helpers físicamente ausentes.
  No prueba clientes viejos ni pretende origen independiente de esas fixtures.
- `current-contracts-{debug,release}.log`: 52 operaciones/nueve mutaciones,
  415 refs resueltas por perfil, adiciones estrictas y descriptor/caps exactos;
  catálogo idéntico a la captura v12 al retomar. **No** comparación histórica v11.
- Sonda nueva `scripts/scratch-probe.cpp`, compilada con warnings como errores y
  autotest de cinco archivos, abierto sin nombre/cerrado con nombre/copia/clone,
  pico 294.912 bytes y todos liberados. Su fuente está ahora versionable; no es
  la dylib perdida ni pretende tener su hash. La medición mixta nueva pasó abajo.
- Regresión completa, cuatro gates frescos y medición física: **aprobados**,
  con resultados y logs durables detallados abajo.

### Resultado final fresco

La cadena de continuación **terminó aprobada**, con `stages.log` y
`fresh-acceptance-summary.json`; la agregación verificó las muestras crudas y los
203 hashes de fuentes versionables fijados antes de los checks. Los logs
`fresh-*` son de esta auditoría; los debug de Rust/lector/CLI/oracle/procesos se
reutilizan de la primera cadena **de esta misma auditoría**, sin repetir el oracle
de paginación de hasta 10.000 celdas ni confundirlo con el `final5` perdido.

| Check | Resultado fresco |
| --- | --- |
| fmt/check/Clippy workspace y host | aprobados, Clippy con `-D warnings` |
| Rust debug y release | **287 aprobados por perfil**, cero fallos/diez ignores intencionales |
| XLSX parser/helper y CLI | **216 casos por perfil**; 442 checks de parser/helper |
| Golden búsqueda XLSX | **83 casos por perfil**, hash/proyección/reader/hits independientes, sin helpers |
| XLSX procesos | diez checks por perfil; ambos protocolos/tres políticas |
| PDF parser/CLI/contención | **235/235/12 por perfil** |
| Migración SQL v10/v11 | dos fixtures por perfil; **34/36 tablas**, originales/DDL/shadows, backup y cursor sintético |
| Adjuntos/procesos actuales | veinte rondas concurrentes por perfil, backup/restore y CAS |
| MCP SDK y raw | **21 SDK por perfil**; tres rondas raw por perfil, trece checks por ronda |
| Contratos actuales | 52 operaciones, nueve mutaciones, **415 refs** por perfil; formas y límites exactos |
| Frontend | contratos/build y **13 adaptador/cola/tema + 3 puente/guardia + 4 lotes** |
| Host | check/Clippy y `tauri build --no-bundle -- --offline --locked` con red denegada |
| UI headless | **592 checks compartidos + 14 teclado = 606**, seis sesiones, cero requests externos |

Los diez ignores no se esconden: fixtures hijas ejecutadas por su supervisor,
gates pertinentes invocados aparte y otros benchmarks históricos que no se
declaran reejecutados. No hay prueba nativa IPC/foco/cierre ni Home completado.
Vite conserva su warning previo de chunk >500 kB; no se elevó el umbral ni se
alteró la UI para silenciarlo.

### Gates consecutivos nuevos

M1 Max/64 GiB, macOS 27.0.1 arm64; Rust/Cargo 1.99.0, Node 22.22.3/npm 10.9.8.
Suite/precompilación terminadas **antes** de medir. Cinco warmups/veinte muestras
por caso, sin compilaciones/tests/otros benchmarks competidores; red denegada.
Receta y `corpus-pins` mixtos conservan los mismos SHA-256 históricos indicados
abajo. Gate UTF-8/PDF y parsers/corpus/Home/host **sin ediciones** desde la captura.

Cada celda es **p50/p95/max ms** de las muestras frescas, sin descartar outliers:

| Caso mixto | fresh-mixed-1 | fresh-mixed-2 |
| --- | --- | --- |
| get | 158,84/161,57/171,65 | 158,78/163,93/168,85 |
| no-op | 163,97/175,97/178,31 | 171,99/184,49/199,23 |
| put | 168,45/182,77/182,81 | 176,94/277,19/362,50 |
| query-distinct | 161,28/167,75/173,74 | 169,68/194,07/196,98 |
| query-filter-pdf | 166,50/169,90/180,42 | 164,20/180,24/191,38 |
| query-filter-utf8 | 168,44/174,64/179,21 | 194,27/232,22/292,22 |
| query-filter-xlsx | 170,34/182,31/194,84 | 172,48/205,54/206,25 |
| query-hit | 172,38/176,51/177,14 | 177,81/270,74/359,66 |
| query-page | 169,26/181,74/212,66 | 170,13/176,87/177,08 |
| query-page-next | 169,90/193,03/195,95 | 171,38/181,31/193,19 |
| query-zero | 158,46/167,02/171,50 | 174,99/206,62/220,76 |

Query p95 **167,02–270,74 ms <=300**. El máximo query-hit **359,66 ms** no se
oculta: el target es p95, no máximo o garantía universal. Put no es query.
Lock p95 máximo **28,208 ms <=150**; editor wait p95 máximo **29,111 ms**
(máximo 50,448 ms). Las tres métricas completas por caso están en el JSON durable.

| Caso UTF-8/PDF intacto | fresh-unchanged-1 | fresh-unchanged-2 |
| --- | --- | --- |
| invalidation | 36,48/45,06/52,00 | 34,55/42,91/44,13 |
| no-op | 119,80/127,26/128,51 | 118,02/126,36/126,73 |
| put | 115,95/128,01/128,23 | 115,49/122,91/142,30 |
| query-filter | 123,78/140,30/141,97 | 118,37/129,06/140,11 |
| query-hit | 132,37/150,79/158,53 | 123,44/148,85/151,63 |
| query-page | 125,39/152,19/166,20 | 121,66/130,74/131,19 |
| query-page-next | 124,88/145,96/149,28 | 121,25/142,32/148,63 |
| query-zero | 113,24/118,88/128,08 | 109,35/114,31/122,86 |
| reset | 13,07/17,49/17,96 | 12,36/17,22/19,68 |

Query p95 **114,31–152,19 ms <=300**, lock p95 máximo **20,172 ms <=150**.
Las esperas de editor y locks incluyen p50/p95/max en el resumen, no sólo wall.

### Recursos nuevos

Mixto 1/2 RSS SELF **47.611.904/48.496.640 B**, CHILDREN **0/0 B**: high-water
getrusage separados en macOS, no sumables ni coste de la preparación/extracción.
Intacto SELF **42.565.632/41.500.672 B**; su harness no mide CHILDREN.
Sampler 2 ms observó bloques de archivos regulares **27.971.584/27.983.872 B**,
no un pico exhaustivo de archivos desvinculados ni uso exclusivo APFS.

`fresh-physical.log`: **diez operaciones reales**, pico observado asignado
**22.298.624 B**, con SHM igual; backup/restore mixto y consulta en destino nuevo
**sin helpers**. Sonda/autotest nuevos indicados arriba, archivos abiertos sin
nombre/cerrados con nombre retenidos hasta liberar; DB/WAL/SHM separados.
No intercepta todo syscall (por ejemplo mmap/dup/rename), no mide RAM o bloques
exclusivos APFS/dedup y no se usa para aceptar latencia. Es un diagnóstico de
flujos concretos; los 22,3 MB instrumentados y 28 MB muestreados no son un límite
universal ni magnitudes directamente comparables.

CLI release fresco SHA-256
`4413592a4c88890716d2b82b778bbcc008dbe74d675b78c30ed721c911b14bb4`;
sonda fuente SHA-256
`aa1ba91f608f4824d0aee590da7923dec1ee4af744add6a60cde57eb29d7f40f`.
La reconstrucción con toolchain/dependencias restauradas puede cambiar el binario;
no se atribuye el hash del binario anterior a esta medición.

### Incidentes nuevos de ejecución (retenidos)

- Se incluyó por error `test-xlsx-nonactivation.py` del estadio histórico v10,
  que exige que XLSX **no** esté integrado. Se detuvo por falta de su baseline
  requerido, sin ejecutar sus aserciones; no aplica al v12. Se conserva intacto,
  no se cambia para aprobar ni se presenta ese intento como regresión de producto.
- La espera por evento de proceso usó primero un context manager no admitido por
  `select.kqueue` en Python 3.10. Se corrigió a `try/finally`, sin sondeo ni cambios
  a la suite en ejecución.
- Host build recibió `--offline` directamente a Tauri y falló su parser antes del
  build. `host-build-failed-arguments.log` retiene el intento. Se pasó a Cargo con
  `-- --offline --locked`; build final sin bundle aprobado, sin tocar el host.

La igualdad exacta con los 50 contratos v11, archivos previos y clientes viejos
**no se volvió a demostrar**: es la limitación histórica autorizada, no un PASS
nuevo. Los logs históricos de fallos anteriores siguen perdidos; los nuevos sí
se conservan. La revisión de cierre contrasta el código actual con los requisitos,
esta evidencia y la captura actual, nunca con un v11 fabricado.

Revisión de cierre: `fresh-final-review-pre-sync.json` verificó las **405 copias
actuales/203 hashes congelados**, cero ediciones de producto desde reinicio y sólo
cuatro herramientas nuevas, sin runtime/modelos/credenciales/datos personales.
Home/host/parsers/corpus/gates/lockfiles y main intactos en ese momento. **259
enlaces/20 documentos** y OpenSpec estricto **15/15** aprobados. Se revisó el diff
documental y los nuevos harnesses; [revisión técnica](review-xlsx-search.md).

### Sincronización y archivo

`sync-verification.json` confirma exactamente **cinco main specs**, diez requisitos
modificados/cuatro añadidos, **14 requisitos/49 escenarios**, los **32 nombres de
escenarios heredados** y **75 requisitos no afectados** íntegros. Las cinco specs
coinciden byte a byte con el merge externo revisado y cada requisito del delta.
Las ocho capabilities restantes y los deltas de Home no se modificaron.

Tras esa comparación, `openspec archive add-local-xlsx-search --skip-specs --yes`
movió el cambio a `2026-10-16-add-local-xlsx-search`: `--skip-specs` evita aplicar
**de nuevo** los deltas ya sincronizados, no omite su integración. Al invocarlo
quedaba sólo la tarea administrativa 6.4 (el propio archivo y verificación posterior),
que se marca después de comprobar el traslado, los enlaces y OpenSpec estricto.
El CLI avisó `Consider splitting changes with more than 10 deltas`; el alcance
real comprobado sigue siendo cinco archivos delta y catorce requisitos, sin ampliar
el cambio por ese warning. `logs/archive.log` conserva ambos avisos no bloqueantes.

Los enlaces relativos de propuesta/diseño y documentos actuales se reubicaron;
el formato vigente entregado es **SQLite v12/API v1/52**, MCP 24/44/45. La
validación documental posterior no sustituye la regresión fresca indicada arriba.
Home permanece como único cambio activo **9/17**, sin sincronizar ni archivar.
`post-archive-verification.json` comprobó el traslado de diez archivos, deltas y
`.openspec.yaml` intactos, **272 enlaces/20 documentos**, más los 203 hashes de
fuentes y las 405 copias actuales. OpenSpec estricto posterior: **14/14**
(trece specs y Home; no incluye cambios archivados). Para validar también los deltas
archivados se usó una **copia documental byte a byte** en
`archive-document-validation/`, presentada al CLI bajo su nombre activo: aprobó
`validate add-local-xlsx-search --type change --strict`. Esa copia no es un
baseline de producto; sus enlaces se verifican en la ubicación real, no en el mirror.

El intento de validar directamente `2026-10-16-add-local-xlsx-search` falló porque
OpenSpec 1.14.1 buscó deltas fuera de `archive/`; su log se conserva. No se
eliminó el delta ni se usó `skip_specs` para silenciarlo. La agregación también
excluye su propio log mientras está abierto para no fijar un hash previo al flush;
los logs ya cerrados se fijan en el manifiesto final de cierre.

La revisión final `final-closure-review.json` amplía el check a **309 enlaces en
30 documentos**, sin rotos, **19/19 tareas** y todas las referencias de funciones
de la matriz resueltas. `closure-allowlist-amendments.json` explica la conciliación
documental de cinco guías relacionadas con v12 y la sincronización exacta de las
cinco specs; no habilita cambios de producto. Se conservaron los otros archivos
y cambios archivados. `final-closure-review.diff` permite revisar el diff contra
la captura actual, y `closure-evidence-pins.json` fija los archivos de evidencia
cerrados y hashes finales; no es una copia ni un baseline independiente v11.

Intentos nuevos retenidos: el harness SQL primero omitía eliminar la generación
XLSX antes del job (FK legítima), después intentaba escribir shadows protegidos
por SQLite y finalmente esperaba incorrectamente que restore del cliente actual
no migrara. Se corrigió **sólo el harness**: limpieza FK ordenada, carga de FTS por
SQL público y oracle conforme a `Workspace::restore`→`open`. No cambió producto,
caps ni aserciones de los harnesses heredados. Logs first/second/third preservados.

Comandos de recuperación reproducibles (cargar `env.sh` de esta auditoría):

```sh
python3 -B scripts/test-xlsx-search-revalidation.py --profile debug
python3 -B scripts/test-xlsx-search-revalidation.py --profile release
python3 -B scripts/check-xlsx-search-current-contracts.py \
  --cli target/release/noteka --restart-catalog "$NOTEKA_REVERIFY_AUDIT/catalog-release-before.json"
clang++ -std=c++17 -O2 -Wall -Wextra -Werror -dynamiclib -pthread \
  scripts/scratch-probe.cpp -o "$NOTEKA_REVERIFY_AUDIT/scratch-probe.dylib"
```

## Evidencia histórica anterior al incidente
Baseline de producto independiente v11/API v1/52 en
`/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-xlsx-search-plan.diMsYO/`.
No existe commit inicial: se comparan copias y SHA-256, no un diff vacío de Git.

## Baseline previo a implementación (histórico, perdido)

Se verificaron los 369 archivos originales (cuatro modificaciones documentales
de planificación), los 14 pins de planificación y ocho binarios independientes.
`apply-baseline.json`/`apply-baseline/` conservan 379 archivos anteriores al código.
Los fixtures `v11-debug` y `v11-release` se generaron con esos ejecutables **antes
de modificar código**: UTF-8/PDF indexados, XLSX extraído sin seleccionar, tablas,
originales, hashes y páginas/cursores. No se rebajó un cliente nuevo para simular v11.

Reproducción: `scripts/test-xlsx-search-migration.py --capture-only` con
`--baseline-bin`, `--profile` y `--directory` nuevo externo. No usar datos personales.

## Matriz y preservación históricas

`apply-scenario-matrix.json` registra cada uno de los 14 requisitos/49 escenarios,
los grupos de tareas y la suite prevista; la matriz nominal enlazada abajo registra
la evidencia ejecutable actual. `apply-allowlist.json` fija fronteras de edición.
Parser/protocolo/supervisores, corpus previos, gates UTF-8/PDF, Home/host/UI y
migraciones 001–011 se preservan; sólo bindings/schemas generados pueden evolucionar
en frontend. No se usan caches de confianza para omitir validación completa.

Auditoría previa a aceptación: `preservation-before-acceptance.json` confirma los
369/379 archivos de ambas copias y ocho binarios intactos; entonces 27 archivos
modificados y 17 nuevos. La repetición `preservation-final-pre-sync.json` confirma
29 modificados y 18 nuevos, sin cambios en las rutas protegidas/main/Home. Se
inspeccionaron los diffs de producto/tests, no sólo los conteos. La ampliación
explícita del allowlist incluye el fallback y cuatro expectativas de versión de
tests de integración; `apply-allowlist-amendments.json` conserva la justificación.

Siete archivos `.agents/skills/*` preexistentes (birth/mtime 11/10) no estaban en
la captura original. No se editaron ni borraron: se distinguen de los nuevos y se
fijaron hashes tardíos para la revisión restante. **No** se presentan esos hashes
como prueba pre-edit independiente ni se oculta esta limitación de la captura.

## Resultados históricos anteriores al reinicio

Progreso detallado: [tareas](../openspec/changes/archive/2026-10-16-add-local-xlsx-search/tasks.md).
Matriz nominal completa: [14 requisitos/49 escenarios](verification-xlsx-search-scenarios.md).
Los intentos siguientes quedaron documentados, pero sus logs crudos se perdieron.
`final3-*` completó ambos perfiles tras la
corrección del oracle de lectura; `final4-rust-debug.log` pasó **287 tests/diez
ignores** después del recorrido por frases. La repetición completa posterior
usó `final5-*`, que fue cancelado; no se le atribuye aprobación.

- `final-rust-debug.log`: **285 tests aprobados, diez ignores intencionales**
  (gates/diagnósticos/fixtures explícitos, no todos reejecutados en ese comando).
  Incluye backup mixto con los tres
  extractores, todos los casos de corrupción con hash exterior reparado, no
  overwrite, CAS/publicación/rollback y equivalencia invoke/DTO tipado.
- `final-check.log`/`final-clippy.log`: workspace/todos los targets, offline/locked,
  sin warnings; fmt workspace y host aprobados. `final-host-check`,
  `final-host-clippy` y `final-host-build`: host check/Clippy/no-bundle aprobados con
  red denegada, **sin ejecutar la aplicación ni aceptación nativa**.
- `final-contracts-generate`/`final-frontend-build`: bindings actuales, chequeo de
  contratos y Svelte sin errores/warnings. Vite informa el aviso previo de chunk
  JS >500 kB; no se sube el umbral ni se refactoriza UI ajena para ocultarlo.
- `final-editor.log`: **13** tests adaptador/cola/tema; `final-bridge.log`: **3**
  de aislamiento/guardia; `final-batches.log`: **4** de suscripción CLI.
- `final-headless-third.log`: **592 + 14 = 606 checks**, seis sesiones, temas
  dark/light, cero orígenes externos, service workers bloqueados y perfil efímero
  propio. No usa perfil personal, pantalla, foco ni screenshots del escritorio.
- `final-attachment-process-{debug,release}.log`: **20 rondas concurrentes por
  perfil**, v9 real→v12, conservación/restore, CAS/reset/put/backup/escritor. El
  harness del repo está intacto; `replay-attachment.patch` documenta sólo raíz cwd
  y expectativa de versión 10→12 en la copia externa. Pins originales/derivados
  quedan en `replay-attachment-pins.json`.
- `phrase-attachment-process-second-{debug,release}.log`: otras **20 rondas por
  perfil** después del recorrido final, con baseline v9 release independiente
  (también para cliente nuevo debug). El primer replay se detuvo porque la ruta
  v9 debug no existe; log conservado, sin resultado de producto ni cambio de asserts.

- `final3-v11-{debug,release}.log`: 36 tablas no-meta idénticas, DDL/shadows,
  originales, hashes/páginas/cursores, running lease y documentos/configs desconocidos
  preservados. El cursor v11 sobrevive sólo-migración y vence tras put; recibo v11
  rechazado en v12. Cliente viejo CLI/MCP rechazado sin escrituras; backup v11
  restaurado con ese cliente en otra ruta. Sin helpers.
- `final3-v10-{debug,release}.log`: estados paused y running/future por perfil,
  34 tablas históricas, originales y backup v10 conservados/restaurados; cliente
  viejo rechazado. Los tests Rust verifican rollback antes/después de versiones.
- Revisión documental previa: OpenSpec estricto 15/15. No es prueba de producto.
- `rust-xlsx-fifth.log`: 28 tests XLSX aprobados, con dos ignores intencionales
  (fixture de proceso y benchmark). Proyección de tipos/hidden/shared/rangos,
  512/513, corrupción fuera del filtro, hash con metadatos y CAS/reset/fuente.
- `oracle-debug-first.log`: **83 casos** del oracle Python aprobados, hash y
  proyección independientes, FTS5 de referencia y páginas/hits/locators públicos;
  helper ausente durante put/search. Incluye un rechazo completo a 513 campos.
- Comparación profunda `check-xlsx-search-contracts.py`: aprobada en ambos perfiles,
  50 pares input/output intactos, lector XLSX/errores/whitelist de nueve intactos;
  exactamente los enums/defs y descriptor nombrados cambian (no es igualdad falsa
  de los 52 contratos). Regeneración/bindings aprobados.
- `final3-sdk-{debug,release}`: **21 tests por perfil**; modern/legacy, tres
  políticas, esquemas estrictos, texto/structuredContent/CLI, cursor/CAS y fórmula
  hostil sólo como dato. `test-xlsx-processes` prueba seis combinaciones raw,
  24/44/45 herramientas, sin helpers para index/search. Tres rondas raw completas
  por perfil también aprobadas (discovery puro, límites/cancel/EOF/identidad).
- Cuotas/integridad/backup mixto están probados en Rust y procesos; gates y sonda
  físicos históricos aprobados abajo. La aceptación nueva está al inicio de este documento.

## Rendimiento y recursos históricos aprobados

M1 Max/64 GiB, macOS 27.0.1 arm64, release, red denegada, sin tests/builds/gates
concurrentes durante medición. Cinco warmups/veinte muestras por caso; p95 es la
muestra ordenada 19/20, no máximo ni intervalo de confianza. Cada intento tiene
fixture propia y no se descartan muestras lentas. Los logs completos y
`accepted-performance-summary.json` conservan p50/p95/max/locks/editor wait.

Corpus mixto fijado **antes** de medir: 8.388.608 bytes (2 MiB UTF-8 + 2 MiB PDF +
4 MiB XLSX), doce generaciones, fórmulas/caché, Unicode, hojas ocultas y >256
términos. Golden independiente contrasta todos los textos, hashes, hits y score
FTS5. `benchmark.json` SHA-256
`6a430b18518a88ee93a34f18f185f479bde925e7078e0632036ad6df4136af45`;
`corpus-pins.json` SHA-256
`ff758b660483713a956123baf02c93a4f10cc8ff5b07576b188d33a6b83fb9ef`.
Los pins y el corpus UTF-8/PDF previo no cambian tras medir.

### Gate mixto consecutivo 11/12

Cada celda contiene **p50/p95/max en ms**; get/put no se confunden con consultas.

| Caso | 11 | 12 |
| --- | --- | --- |
| get | 157,23/168,73/172,79 | 159,95/168,31/177,17 |
| no-op | 160,85/167,34/170,03 | 163,87/176,17/181,05 |
| put | 164,34/185,39/208,04 | 169,58/181,88/196,36 |
| query-distinct | 160,94/166,14/177,13 | 164,88/175,73/183,77 |
| query-filter-pdf | 166,92/182,67/184,26 | 166,71/178,41/187,62 |
| query-filter-utf8 | 165,58/171,93/172,22 | 170,25/182,07/186,50 |
| query-filter-xlsx | 168,65/171,72/184,71 | 168,41/172,71/173,28 |
| query-hit | 171,20/181,20/195,28 | 173,65/180,59/185,59 |
| query-page | 168,73/173,99/177,86 | 171,30/186,95/187,50 |
| query-page-next | 169,22/191,30/195,92 | 172,33/197,02/199,60 |
| query-zero | 157,38/176,96/190,32 | 158,90/164,92/175,50 |

Query p95 **164,92–197,02 ms <=300**; lock p95 máximo **14,094 ms <=150**.
No-op lock 11/12: 3,54/3,64/3,68 y 3,48/4,12/7,66; put lock:
9,77/14,09/14,14 y 9,92/14,09/16,54. Espera editor no-op:
3,85/3,90/3,92 y 3,84/8,94/12,36; put:
10,14/22,64/22,64 y 10,19/22,62/22,66. Son mediciones separadas del wall.

### Gate UTF-8/PDF intacto consecutivo 3/4

Logs `final3-unchanged-benchmark-{3,4}.log`; script/fixture/objetivos originales
sin cambios. P50 se agrega desde sus muestras crudas, sin editar el harness.

| Caso | 3 | 4 |
| --- | --- | --- |
| invalidation | 35,95/44,41/62,01 | 36,04/42,46/42,90 |
| no-op | 113,25/117,49/209,74 | 116,79/124,13/125,51 |
| put | 109,49/116,35/142,40 | 111,45/133,42/139,10 |
| query-filter | 112,77/114,75/120,25 | 113,99/118,84/130,75 |
| query-hit | 116,24/121,65/122,68 | 119,65/124,55/124,99 |
| query-page | 116,46/125,61/170,98 | 115,68/120,57/127,64 |
| query-page-next | 116,47/127,04/127,09 | 115,11/121,37/128,19 |
| query-zero | 104,67/115,47/124,92 | 107,15/112,62/120,55 |
| reset | 11,78/14,90/16,74 | 12,27/14,49/16,00 |

Query p95 **112,62–127,04 ms <=300**; lock p95 máximo **18,475 ms <=150**,
editor wait p95 máximo **22,772 ms** (máximo observado 53,088 ms). Los cuatro
casos de lock/editor incluyen p50/p95/max en el resumen externo, no sólo el gate.

### Memoria y temporales físicos

RSS nativo getrusage en macOS, bytes: mixto 11 SELF **46.383.104**, CHILDREN **0**;
mixto 12 SELF **45.645.824**, CHILDREN **0**. Son high-water separados del proceso
core medido; no crea hijos durante index/search, por eso CHILDREN es cero. No
representa el coste de extracción/preparación ni memoria simultánea sumable.
Gate intacto SELF **41.795.584/41.893.888**; su harness no captura CHILDREN y no
se inventa ese valor. Mediciones de helpers de extracción permanecen en su guía.

El muestreo de archivos regulares cada 2 ms en el gate mixto observó asignación
**27.967.488/27.971.584 bytes** (aparentes 27.963.752/27.967.848). No incluye de
forma exacta todo archivo efímero ya desvinculado; no se lo presenta como máximo
físico exhaustivo. SQLite principal de las fixtures: 22.249.472/22.253.568 bytes.

`final-physical-scratch.log`: diagnóstico separado en diez operaciones CLI reales
sobre la fixture mixta 12, **helpers ausentes**, pico observado de bloques de
scratch **22.286.336 bytes** (con SHM también 22.286.336). Interposición externa
open/openat/write/pwrite/ftruncate/close/unlink/unlinkat/copyfile/fcopyfile/
clonefile/clonefileat/fclonefileat; stat/fstat `st_blocks*512`, conserva abiertos
sin nombre y cerrados con nombre; DB/WAL/SHM se clasifican aparte. Todos los
temporales terminan sin nombre/referencias, conexiones cerradas; backup/restore
mixto conserva get y búsqueda sin helpers en destino nuevo.

Se compiló una dylib nueva desde la sonda externa fijada (`b0a7a319…a0f90fd`),
autotest de cinco archivos/copia/clone y unlink-open/named-close aprobado con
196.608 bytes. Binario CLI medido SHA-256
`1324fb037f358e4da4b7760618e3a632d22b28f3cbb59817bb42977026103f59`.
No es RAM, uso exclusivo APFS/dedup ni prueba de interceptar todo syscall. Los
22,3 MB instrumentados y 28 MB muestreados son observaciones de flujos distintos,
no límites universales comparables ni aceptación de latencia instrumentada.

Reproducción (con `env.sh` externo cargado y binarios release compilados):

```sh
python3 -B scripts/benchmark-xlsx-search.py --profile release
python3 -B scripts/benchmark-attachment-search.py
python3 -B scripts/measure-xlsx-search-scratch.py --workspace <fixture>/workspace \
  --probe-library <audit>/scratch-probe.dylib --probe-source <fuente>/scratch-probe.cpp
```

## Límites de aceptación

Se exigen los gates fijos query p95 <=300 ms/lock <=150 ms, caps intactos y nuevas
mediciones mixtas, sin reemplazar evidencia anterior ni ocultar intentos fallidos.
Home conserva ocho tareas nativas pendientes: no usar pantalla ni sustituirlas por
los checks headless. Este recorte no completa toda 5a/5b ni Excel genérico/OCR.

### Fronteras y caps dominados

`exact_xlsx_global_quotas_reject_one_more_without_eviction` puebla por SQL fixtures
explícitas para evitar miles de puts cuadráticos; el **siguiente put real** verifica
todo y rechaza exceso sobre 1024 selecciones, 16384 fragmentos y 8 MiB sin tocar
ninguna fila. 512/513 y campos >4096 bytes se contrastan además por CLI/oracle.

El texto de una proyección válida es menor que su representación JSON canónica
<=1 MiB: cada string se usa como máximo una vez; claves, metadatos y escapes añaden
bytes, y true/false mantienen longitud. `canonical_representation_cap_dominates_xlsx_projected_text_cap`
prueba una representación **exactamente 1 MiB** sin inventar un Workbook inválido
para llegar al cap de texto dominado. La cuota global sí se satura exactamente.

Para hits XLSX, 64 fragmentos ×4096 bytes =262144 bytes de texto (aun a 6× por
escapes JSON: 1572864), más localizadores y fuentes acotados: el máximo de página
domina la respuesta de 2 MiB. Se mantiene
PageLimit, el serializador y los tests heredados de respuesta; no se fabrica un hit
con campos ajenos/inválidos para alcanzar un límite que esta proyección no alcanza.
Cursores 2 KiB, deadlines cooperativos 5 s, progress handler SQLite y busy timeout
2 s se preservan y prueban con entradas hostiles, presupuesto cero y contención.

## Intentos y diagnóstico históricos (logs perdidos)

Las referencias a retención de cada intento describen lo observado **antes** del
incidente; no implican que los logs/fixtures sigan disponibles en esas rutas.

- `mixed-benchmark-first.log`: latencias query p95 263,68–280,06 ms y locks
  aprobados, pero el sampler auxiliar de temporales terminó con FileNotFoundError
  al desaparecer una carpeta efímera durante rglob. **No cuenta como aceptación**;
  se corrigió el harness con os.walk y propagación de errores. Sus observaciones
  de scratch son parciales, no se presentan como pico físico completo.
- `mixed-benchmark-2.log`: corrida válida con p95 de primera/siguiente página
  **366,10/315,88 ms**, target 300 ms **fallido**, lock máximo p95 12,74 ms.
  Se conservan corpus/consultas/pins/caps/validación y la evidencia fallida; no se
  acepta excepción ni se busca una corrida afortunada como sustituto de optimizar.
- `mixed-cost-before.log`: diagnóstico sobre esa misma fixture, no gate. Tras la
  primera muestra fría, canonical tomó 29,07–31,61 ms y verify 248,03–257,96 ms;
  el recorrido de postings completo por sí solo tomó 160,95–167,42 ms. Esto ubica
  un costo verificable en el fallback de vocabularios grandes, no en el ZIP/XML.
- Se midió una implementación del **mismo** stream completo mediante C API
  tipada/fija y statement RAII, conservando tipos/columna/término/offset/rowid,
  bitsets por documento, duplicados, cardinalidad y deadlines. El umbral de 256
  no cambia y los streams independientes siguen comprobándose completos. No es
  una caché de confianza ni un permiso para ignorar términos. La aceptación seguía pendiente en ese momento.
- `mixed-cost-after.log`: mismo workspace y diagnóstico pareado, fallback completo
  142,33–145,19 ms después de la primera muestra, frente a 160,95–167,42 ms antes;
  verify 232,79–244,76 ms. Pasaron tests attachment y XLSX, incluidos stream/set,
  materializados, corrupción y >256 términos. Esto prueba una mejora local del
  recorrido, **no** el cumplimiento del gate de consulta; las nuevas corridas
  deben cumplir independientemente.
- `mixed-benchmark-7.log`: la primera optimización no alcanzó; página query p95
  **336,382 ms**. No hubo gates 5/6 completos; la cadena previa se interrumpió en
  PDF. El intento 8 no se inició al fallar 7. Logs parciales se conservan.
- `mixed-cost-batched.log`: OR de hasta 64 términos con `xInst` redujo postings
  a 91,09–93,79 ms warm. Gate 9 aprobó (216,58–273,04 ms) pero gate 10 falló
  query-hit p95 **343,139 ms**: no son dos corridas aceptadas.
- `mixed-cost-phrase.log`: misma fixture, `xPhraseFirst/Next` público evita ordenar
  el array combinado y conserva cada término/instancia/columna/posición/bitset/
  cardinalidad/deadline; warm postings **54,13–54,89 ms**, verify 148,34–153,65 ms.
  Umbral 256 queda como frontera de algoritmo y el stream íntegro como fallback,
  no se cambian caps, fixtures ni persistencia. Gates finales 11/12 sí pasan.
  Tests diferenciales cubren 255/256/257, lotes incompletos, >700 términos,
  Unicode/repeticiones/filas dispersas, corrupciones y streams hostiles sin tipos.
- El primer intento del nuevo conjunto (shell `sh_140c42f77001JRXtTCFM5Jl2eD`)
  se interrumpió por el timeout externo de **120 s** de la orquestación; no produjo
  un informe completo ni cuenta como corrida aprobada/fallida del target. No se
  encontraron hijos de benchmark restantes. Se retiene su log vacío y la fixture;
  la repetición usa timeout externo ilimitado sin tocar los 5 s del producto.
- `final-headless.log` y `final-headless-second.log` no alcanzaron los checks de
  UI: el daemon CLI rechazó la longitud del socket Unix (127 y 85 bytes de
  directorio, respectivamente). Se conserva el fallo. Se usa una carpeta corta
  propia en el mismo temporal aprobado mediante su alias macOS `/var/...`, con
  `PWTEST_SOCKETS_DIR`; no se cambian assertions ni se abre pantalla/perfil personal.
- `final-test-xlsx-search-debug.log`: la ampliación del harness de paginado usaba
  sheet_id en el nivel equivocado del item. El DTO existente lo conserva en
  `item.locator.sheet_id`; se corrigió sólo el oracle de lectura. No hubo cambio
  de producto, esquema ni expected. `final2-*` se interrumpió luego en PDF sin
  resultado; `final3-*` terminó ambos perfiles. `final4-*` se interrumpió durante
  oracle debug después de 287 tests Rust y corpus lector/CLI; no acredita release.
  `final5-*` intentó repetir lo pendiente y todo release, pero quedó cancelado sin resultado.
- La validación documental estricta detectó dos textos ADDED >500 caracteres:
  detalles de los mismos requisitos se trasladaron a sus escenarios existentes,
  sin quitar obligaciones, nombres ni escenarios (siguen 14/49 y 32 heredados).
  `apply-all-validate-second.log`: **15/15**, sin warnings. Esto no acredita el gate.
