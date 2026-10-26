# Verificación del refactor de integridad de evidencia

`optimize-local-index-verification`, verificado y archivado6/6. No está archivado ni
completa el lector de tablas6/8 o fase5a. No se inició5b/5c o fase6.

## Baseline y fallos previos

Evidencia fuera de Git en `2026-10-21-phase5-201534`, señalado por
`~/.local/share/noteka-verification/active-phase5-path.txt`.
`before-index-opt/` conserva671 fuentes y14 binarios con pins comprobados;
`opt-baseline-review.json` contrasta archivo congelado y cuerpo del oráculo.
Es baseline diagnóstico16/API57: el lector de tablas aún no tenía aceptación
de gates, no se presenta como un cambio cerrado.

Tres gates mixtos anteriores al refactor: fallo query-hit328,663ms p95;
aprobado con máximo query194,783ms; fallo query-zero320,734ms p95. Los locks
aprobaron. El primer query-hit tenía mediana182,221ms y dos picos consecutivos
440,988/328,663ms. Todos los intentos/corpus/muestras se conservan.

El A/B inicial56/57 sobre la misma fixture y64 hits fue exacto, con p95
232,612/214,756ms incluyendo CLI/sandbox. No demostró una regresión sostenida
del lector ni la causa puntual. Diagnóstico de ocho muestras: canonical30,187–
36,885ms, verify152,175–203,651ms y postings54,145–56,829ms. Se observó ausencia
de advertencia térmica y189,69MiB de swap después del fallo; no son condiciones
concurrentes de los picos ni explicación causal. No se exceptuó ningún fallo.

## Equivalencia y corrección

El cuerpo anterior de verify se conserva bajo cfg(test), con rename solamente,
contrastado byte a byte con la fuente congelada. Cinco pruebas nuevas aprobaron
en debug y release;59 pruebas anteriores de búsqueda aprobaron en debug.
Los diferenciales incluyen corpus vacío/único/repetido/mixto/sin tokens, slots
dispersos y todos los campos/tipos de copias, filas faltantes/sobrantes, metadatos
canónicos, postings ocultos y estadísticas corruptas. Comprueban pureza y budget.

Las pruebas detectaron una brecha anterior: verify podía admitir ciertos daños
en estadísticas; search los descubría sólo al contrastar candidatos/puntajes.
El nuevo control rechaza totales o docsize incompatibles con la tokenización
canónica antes de responder, incluso sin hits. No se debilitaron los tests para
aceptar esa brecha. La comparación de copias sigue siendo estricta, no coercitiva.

Search conserva su índice esperado completo para candidatos/puntajes. La ruta
de ausencia sólo se usa si tampoco coincide ninguna clase canónica de todo el
snapshot; si hay un match global, incluso fuera del filtro, usa el contraste
completo. Prueba de predicados contra FTS completo incluye Unicode, puntuación,
frases y tokens ausentes. Las tablas de clases desaparecen al salir del snapshot.

Los contadores usan varints del SQLite incorporado (libsqlite3-sys0.36.0):
`fts5StorageSaveTotals` y `fts5StorageInsertDocsize`. Se validan tipos, tamaños,
cardinalidad, ceros de columnas UNINDEXED y conteos canónicos; no se decodifica
un blob excesivo ni se acepta un campo parcial no permitido por el formato.

## Comparación medida y compatibilidad

A/B alternado anterior57/actual57, misma fixture12 generaciones/8MiB,5 warmups y
20 muestras por caso;150 ejecuciones CLI. Todos los outputs completos coinciden,
incluyendo64 hits, puntajes, cursores, metadata y cobertura. Frontera de medición:
proceso CLI+sandbox, distinta al gate interno. No sustituye la aceptación original.

| Caso | Antes p95 | Después p95 |
| --- | --- | --- |
| index get | 186,807ms | 122,885ms |
| query sin hits | 182,736ms | 121,385ms |
| query con64 hits | 198,040ms | 194,036ms |

`opt-ab-results.json` conserva todas las muestras y CPU de hijos; no atribuir
mejoras de latencia con hits que este ensayo no demuestra. El piloto mixto
original aprobó; la aceptación final siguiente se midió después de la regresión.

`opt-compatibility2.json`:57 contratos/esquemas/errores/límites/batch idénticos;
get/search equivalentes, lecturas puras y backup nuevo restaurado con cliente57
previo, con seis formatos/desconocidos y originales exactos. El primer script
falló al leer por URI ro un backup recién restaurado en WAL; la segunda versión
verifica ausencia de WAL no vacío y abre una instantánea immutable sólo después
de terminar todos los procesos. No ignora WAL pendiente ni altera la aplicación.
El fallo y su fixture se conservan; el probe RW inicial sólo se hizo en esa
fixture sintética fallida, no en fuentes originales de usuario.

## Ejecución aislada y aceptación pendiente

Una compilación Tauri preexistente compartía target con los tests y generó una
espera de build. Se canceló únicamente nuestro cargo bloqueado y se copió la
caché a `runtime/opt-target`; CLI/MCP/helpers y pruebas actuales usan ese target.
El puente headless admite NOTEKA_CLI para fijar ese binario. No se detuvo el
proceso ajeno ni se abrió ventana. Logs de cancelación y procedencia conservados.

La regresión fresca aprobó384 pruebas Rust por perfil (13 ignoradas explícitas),
28 SDK MCP por perfil,101 de frontend y5146 checks headless en ambos temas,
con cero solicitudes externas en los recorridos aprobados. Fmt, Clippy workspace
all-targets y host native-smoke/custom-protocol aprobaron sin warnings Rust.
El host se compiló no-bundle en target externo sin ejecutarlo. El aviso de Vite
por tamaño del bundle permanece visible, sin ampliar el umbral.

Después sólo cambió el helper cfg(test) de diagnóstico: descarta su scratch antes
de reconstruirlo cuando se llama con Verified vivo. Cinco diferenciales por
perfil, el diagnóstico mixto ignorado ejecutado expresamente, fmt y Clippy
verifican ese ajuste; no cambia código productivo. El CLI de tabla también
respeta CARGO_TARGET_DIR para fijar el cliente independiente de la prueba.

Treinta fuentes Home/App/helpers y ocho pins adicionales (Cargo.lock, ambos gates,
corpus, postings y pruebas compartidas nativas) conservan sus bytes. El cuerpo
del oráculo es exacto tras excluir comentario de procedencia y renombrar sólo la
función; el primer comparador incluyó ese comentario y falló, sin cambio de fuente.
Los cuatro gates finales300/150 aprobaron. SQLite16/API57,
perfiles/caps y fuentes Apple se conservan; no hay modelo/proveedor nuevo.

## Aceptación final

Dos repeticiones consecutivas de cada gate original, sin compilación o inferencia
propias concurrentes, aprobaron con corpus/caps/targets/verificadores intactos:

| Gate | Query p95 mínimo–máximo | Lock p95 máximo |
| --- | --- | --- |
| Mixto1 | 105,789–182,232ms | 14,637ms |
| Mixto2 | 100,981–179,935ms | 14,885ms |
| Original1 | 59,803–163,630ms | 18,445ms |
| Original2 | 65,551–123,340ms | 17,642ms |

`opt-final-gates-summary.json` enlaza resultados/muestras/corpus completos.
Scratch mixto observado28.049.408 bytes asignados (muestreo2ms de archivos del
TMPDIR privado, no máximo transitorio exacto ni consumo exclusivo APFS).
La lectura CLI de tabla repitió golden/fuentes/cursores/búsqueda/backup con cliente56
en ambos perfiles; release p95 primera21,365/siguiente19,241/completa22,166ms.
No hubo excepción de muestras o relajación de límites; se conservan los tres
intentos anteriores, dos fallidos. El alcance aceptado es este corpus/macOS,
no latencia universal o validación Windows.

Revisión final del diff, oráculo y pins aprobada. No hay delta de specs en este
refactor; su archivo no sincroniza el lector de tablas todavía abierto6/8.

Cierre21/10: verificado, sin deltas; archivado como `2026-10-21-optimize-local-index-verification`. Home9/17 intacto.
