# Verificación de Search con evidencia local

Cambio `add-local-evidence-search-ui`. Implementación y aceptación técnica aprobadas;
**17/17 tareas verificadas**, delta sincronizado y cambio archivado. Esta página registra
verificaciones ejecutadas, no aceptación anticipada.
Home permanece independiente 9/17; ninguna prueba usa la pantalla del usuario.

## Baseline pre-UI

El 16/10/2026 se verificaron los 409 archivos del baseline pre-UI, ocho copias
de binarios CLI/MCP/helpers debug/release y 283 fuentes congeladas. Catálogo v12
completo idéntico, excluyendo sólo el `request_id` aleatorio del envelope: API,
resultado y todos los contratos se compararon íntegros. El primer probe comparó
ese ID por error; se diagnosticó la diferencia y se conservó en el reporte.

Evidencia durable fuera de Git: directorio señalado por
`~/.local/share/noteka-verification/active-evidence-search-ui-path.txt`, archivos
`baseline-review.json`, `baseline-pins.json`, `baseline-all-binary-pins.json`,
`frozen-source-pins.json`, `catalog-v12-before.json` y `baseline-bin/`.
Es un baseline **v12 anterior a esta UI**, no recuperación del v11 perdido.

## Fuentes sintéticas

`scripts/prepare-evidence-search-fixture.py` prepara fuentes nuevas fuera del
checkout, dentro de `NOTEKA_TEST_TMP`, y usa CLI real con red denegada en macOS.
Reutiliza generadores PDF/XLSX y proyector golden Python sin modificar el corpus.
La única inyección SQL es config/config_hash futuros en un job/generación real
sin seleccionar. No fabrica hits ni omite validaciones.

El probe inicial detectó que el golden PDF omitía el LF derivado documentado;
se corrigió el esperado, no el extractor ni el assert. `fixture-initial.log`
conserva el fallo. El fixture verificado da dos páginas/31 hits, los seis campos
XLSX, tres localizadores y cobertura `[4,1,1,1]` (seleccionadas/sin seleccionar/
incompatibles/sin generación); compara lectores/golden y prueba intento anterior.

## Controlador

`npm run test:attachment-search`: **14/14 tests aprobados**. Cubren consulta exacta
y filtros, una página sin acumulación, reinicio explícito, fallos de núcleo y
transporte, límites defensivos de respuesta, descarte de éxito/rechazo tardío,
cancelación en cola, remount y un IPC físico compartido. `npm run check` pasó sin
errores/advertencias. Son tests de cliente, no evidencia de IPC nativo o de UI.

La ejecución definitiva del fixture (`fixture-final.log`) contrastó también los
rangos con lectores UTF-8/PDF y las 31 celdas con el golden independiente.

## UI integrada

Primer recorrido UI: **1.234 checks nuevos** (617 por tema), cero requests remotos,
con evidencia en `headless-initial-summary.json` y capturas headless sintéticas.
Cinco tests de puente aprobados (`bridge-evidence-first.log`). Comparación de
283 fuentes congeladas intacta. Se inspeccionaron capturas de ambos temas: texto
no confiable visible como datos, wrap/foco presentes y editor conservado.

La integración inicial tuvo un error de nombre local `state` que interfería con
la rune `$state`; se renombró a `view`. Build/check posterior aprobó sin errores
ni warnings Svelte. Permanece el warning conocido de Vite por bundle >500 kB;
no se aumentó ni ocultó su umbral.

La instrumentación final verifica snapshot antes/después de **cada** lectura de UI:
**1.248 checks nuevos aprobados** (624 por tema), incluidos teclado para consulta y
paginación y respuestas de notas ya obtenidas retenidas al alternar. Además, los
**606 checks anteriores** aprobaron sin alterar sus fuentes/asserts: seis sesiones,
592 compartidos + 14 de teclado/intercalados, cero requests externos. Cinco tests
del puente aprobaron, también con la preparación confinada al root efímero local.
Resultados: `headless-final-summary.json`, `logs/headless-evidence-final.log`,
`logs/headless-regression.log` y `logs/final-bridge.log`. Se cotejó el código del
recorrido completo con el log Playwright; el primer probe había retirado su LF
terminal y falló por el formato del wrapper, no por una diferencia de tests.

Build host no-bundle offline/locked con red denegada aprobado sin abrir ventana
(`logs/final-host-build.log`). Frontend check/editor/controlador/lotes aprobados;
regresión Rust/MCP y cuatro gates finales aprobados.
[Matriz de diecinueve escenarios](verification-evidence-search-scenarios.md).
La validación OpenSpec de planificación aprobada no prueba comportamiento.

## Reanudación de regresión

El coordinador se interrumpió tras aprobar `test-xlsx-search-debug`, con el log
de `test-xlsx-processes-debug` todavía vacío. No se aceptó esa etapa ni se
interpretó el proceso desaparecido como éxito. `interruption.json` conserva el
punto observado; se verificaron de nuevo los 218 pins de fuentes probadas y se
reanudó desde la primera etapa sin PASS. Los logs previos permanecen intactos y
`regression-resumed-stages.log` distingue lo ejecutado después. Los gates se
ejecutan sólo cuando termina la regresión, sin competir con ella.

## Regresión fresca completada

Las 36 etapas originales/reanudadas cerraron con PASS: **287 tests Rust por perfil**
(10 ignorados explícitos de benchmarks/recursos), fmt/check/Clippy workspace/host,
216 casos XLSX por parser/helper y CLI por perfil, 83 casos de proyección/búsqueda
XLSX por perfil, procesos/revalidación y corpus PDF/contención. **21 tests SDK MCP
por perfil**, tres rondas raw y veinte rondas CLI de adjuntos por perfil.
Frontend: 13 adaptador/colas/tema, 14 controlador, 4 lotes y 5 puente aprobados.
Catálogo completo/DTOs/referencias/límites/whitelist de nueve mutaciones idénticos
al baseline pre-UI v12 en debug/release, no sólo igual número de operaciones.

Los binarios originales siguen conservados; la regresión reconstruyó CLI debug
y helper PDF release con hashes diferentes. CLI release, MCP y los demás helpers
coinciden con sus pins originales. No se confunde identidad de catálogo/fuentes
con reproducibilidad binaria de todos los perfiles; comparaciones en
`post-regression-binary-comparison.json`. Las 218 fuentes de ejecución cotejadas
siguen intactas respecto de los recorridos headless, y Rust/corpus/gates respecto
del baseline pre-UI. No se modificó lógica, dependencia ni contrato para este cambio.

## Gate fallido conservado

Primer gate mixto: `query-page-next` **306,401375 ms p95** frente a <=300 ms;
locks aprobados y todas las operaciones válidas. Mediana de ese caso 193,194625 ms,
máximo 410,259125 ms; hubo otro pico de pared de 572,471292 ms. Evidencia íntegra en
`logs/gate-mixed-1.log` y el directorio de resultados registrado allí.

Se contrastaron fuentes/corpus/gate y CLI release, sin cambios, se revisaron muestras
y condiciones (sin swap ni avisos térmicos; carga 8,64/9,42/9,32). **No se demostró
la causa de los picos**, ni se afirma una optimización o culpa de la UI.
`gate-failure-review.json` registra análisis y límites. No se acepta el fallo como
excepción: un nuevo par completo consecutivo usa los mismos gates/caps/validación y
timing agregado externo, sin profiler dentro del benchmark. El fallo original no
forma parte del par aprobado ni se elimina del informe.

## Cuatro gates finales

Tras esa revisión se ejecutó un par nuevo completo y consecutivo de cada gate,
sin cambios de código/corpus/pins/caps/validaciones o targets:

| Gate | Ronda 1: rango query p95 | Ronda 2: rango query p95 | Lock máximo p95 |
| --- | --- | --- | --- |
| Mixto UTF-8/PDF/XLSX | 175,74–202,26 ms | 169,49–235,47 ms | 14,17 ms |
| UTF-8/PDF intacto | 121,37–182,71 ms | 128,22–149,25 ms | 18,94 ms |

Objetivos <=300 ms y <=150 ms aprobados en todas las categorías. Cinco warmups y
veinte muestras por caso, percentil por orden estadístico (19.ª de veinte); se
conservan muestras completas, errores y metodología originales en cada
`results.json`. `logs/gate-final-*.log`, `gate-final-stages.log`, reportes de
`time -l` y condiciones observadas están fuera de Git. `acceptance-summary.json`
consolida 36 etapas de regresión, cuatro gates y 74 logs cerrados/pinneados.

Son resultados locales del corpus fijado en M1 Max, no garantía universal de 300 ms.
No hubo optimización del núcleo en este cambio. El par posterior no demuestra
por sí solo la causa del pico anterior; éste queda como limitación explícita,
no como excepción aceptada. La instrumentación headless no se usa como benchmark
Rust y estos gates no acreditan IPC/foco/cierre del host nativo.

## Revisión técnica y documental

`implementation-review.json` y su diff contrastan 424 archivos actuales contra
las 409 copias pre-UI: 399 originales intactos, diez modificados y quince nuevos
(incluidos cinco artefactos del cambio). Se verificaron 282 pins congelados;
la única excepción documental en esa lista es `openspec/config.yaml`, actualizado
para comunicar el estado. No hay cambios Rust/host/parsers/contratos/locks/corpus/
gates, Home, otros cambios activos o archivos previos. Las 218 fuentes probadas
siguen byte a byte iguales a los recorridos aprobados.

Revisión de controlador, panel y diff de integración: lifetime/cupo físico correcto,
invalidación de notas al alternar, una página, texto escapado y procedencia íntegra,
sin N+1/preparación/base64. Sin divergencias del diseño ni decisión duradera nueva
que requiera ADR. Guía/matriz cubren los siete requisitos y diecinueve escenarios;
266 enlaces locales contrastados. Los comandos de pruebas/gates se ejecutaron.
No se detectaron runtime, modelos, datos personales ni patrones de credenciales
en los archivos afectados. El scan amplio marcó inicialmente el PNG del scaffold;
se verificó su hash sin cambios y se permitió únicamente ese asset conocido.

Merge preparado fuera de main y luego contrastado contra main: un requisito modificado y seis añadidos, veinte
requisitos resultantes, trece ajenos byte-idénticos y escenario heredado de notas
conservado exactamente. La revisión documental final y sync/archivo aprobaron;
esto no acepta las ocho tareas nativas pendientes de Home ni cierra 5a/5b.

## Verification report

| Dimensión | Resultado |
| --- | --- |
| Completitud | 17/17 tareas verificadas; siete requisitos/diecinueve escenarios trazados. |
| Corrección | 606 checks heredados + 1.248 nuevos, 287 Rust por perfil/MCP/oracles y cuatro gates aprobados. |
| Coherencia | Seis decisiones del diseño respetadas; fuentes y Home preservados, sin ampliación de arquitectura. |

Sin hallazgos críticos ni divergencias abiertas para este recorte. Limitaciones
conservadas: causa no probada del gate inicial, columna estrecha/IDs manuales,
puente no nativo y comparación histórica v11 no recuperada. No se transforman en
garantías de latencia universal, frescura posterior, lector visual ni cierre de Home.
Validación estricta del cambio y **15/15 items** aprobada; merge staged validado.
Estos checks documentales no sustituyen la aceptación técnica anterior.

## Cierre

Sincronizado **sólo** `local-text-search`, byte-idéntico al merge staged cuyo
SHA-256 es `b1d5a223c6e63822c03ff3758d9bb3abdba5e2339426d20f124ce24db7c47812`.
Se preservaron trece requisitos ajenos y el escenario heredado, con 281 fuentes
congeladas restantes intactas. Specs resultantes y todos los items validaron en
modo estricto. `--skip-specs` evitó aplicar el delta una segunda vez, no omitió
su sincronización.

Archivado como
[`2026-10-16-add-local-evidence-search-ui`](../openspec/changes/archive/2026-10-16-add-local-evidence-search-ui/proposal.md).
Home sigue como único cambio activo 9/17 y sin sincronizar/archivar. Snapshot de
fuentes/evidencia y manifiestos durables fuera de Git; `sync-review.json`,
`post-archive-review.json`, `final-closure.diff`, `final-versionable-pins.json`,
`closure-evidence-pins.json` y logs de validación/list posteriores permiten auditar
el resultado. No hubo commit, push, publicación, pantalla ni activación remota.

La revisión posterior confirmó **424 archivos** copiados/pinneados y **270 enlaces**
locales válidos, con los catálogos completos debug/release idénticos al pre-UI v12.
Validación estricta posterior: **14/14 items** del repositorio (trece specs y Home).
Una copia documental byte-idéntica de 26 archivos, con el cambio archivado expuesto
como activo sólo en ese mirror externo, aprobó **15/15 items** para revalidar sus
artefactos finales; no es una nueva prueba de aplicación ni evidencia histórica.
`gate-evidence/` conserva copias de muestras/reportes de los cuatro gates y del
fallido; `runtime-report-pins.json` fija 24 reportes/capturas/fixtures sintéticos
durables. Se verificó además que las 218 fuentes de ejecución seguían intactas.
