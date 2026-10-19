# Verificación del lector PDFKit independiente

Aceptación técnica aprobada. Sincronizado/contrastado y archivado como2026-10-17-add-bounded-pdfkit-reader; no declara integración con el núcleo ni cierre de 5a.
Evidencia durable: ~/.local/share/noteka-verification/2026-10-17-pdfkit-reader-205729,
referenciada por active-pdfkit-reader-path.txt. Baseline tar SHA256
c7a0c0bd26105e46713a01f7ba41eee1e4ba1c50a5bd9684c78b3e365061891e:
522 fuentes y16 binarios, copiados/verificados antes de modificar el repositorio.

## Resultados comprobados

- Siete tests de protocolo por perfil; regresión Rust del workspace sin el paquete
  desktop:326 por perfil, diez ignores intencionales. No se reconstruyó el host
  ni se abrió ventana; el core/UI/Home conservan todas sus fuentes. Clippy con
  -D warnings y fmt aprobados.
- Corpus propio MIT21 PDF,13 entradas malformadas y24 casos de proceso por perfil:
 58/58 en debug y58/58 en release, aceptación3. Todas las respuestas de archivos
  pasan por el validador Rust real. Manifest fijo SHA256
 3ef4cf66511c3d1b1b02c855d7d71b120b71e0b61547bd48aec1da30d7be3d7c.
  Se amplió el generador con una página de texto rasterizado y expectativa vacía
  anterior a ejecutarla: corpus v2 de22 archivos pasó completo en ambos perfiles,
  con los mismos hashes de helpers/validador y expectativas anteriores intactas.
  Su manifest SHA256 es aef8f130aa3a23580c8bf70a74857495c8b64e90e921ec9ed7d10caf2111b3be.
  Son59 casos únicos por perfil al unir22 archivos,13 wires y24 procesos; las
  repeticiones no se suman. Tres guardias de preparación rechazaron opción ajena,
  target no admitido y destino symlink, conservando el marcador original.
- macOS27.0.1/build26A434, aarch64, Swift/SDK instalado. Sólo este entorno/corpus
  está probado. Máximo RSS observado en casos de archivos25.608.192 B debug y
 25.526.272 B release; máximos wall de una muestra463,732/256,589 ms. Son muestras,
  no p95 de extracción ni garantía universal. El target300 ms se aplica a queries.
- CPU real terminó por SIGXCPU:4,011279 s debug y4,004454 s release. Watchdog,
  presupuesto menor, RSS observado, stdin/stdout bloqueados, pérdida de consumidor,
  salida del padre/recolección del OS, límites de pipes, fallo interno y señal
  pasaron. El probe de texto verifica UTF16 inválido, NUL,1MiB/frontera, exceso
  multibyte y combinación Unicode sin normalizar. No se invocó SIGABRT ni UI.
- Los53 contratos completos de entrada/salida y todo data de operations.describe
  son exactamente iguales al baseline en ambos perfiles. Se ignora únicamente
  request_id aleatorio del envelope. MCP25/45/46 y batch9 se conservan.
- Cargo.lock añade sólo el paquete local nuevo; todas las entradas anteriores son
  idénticas. Las522 rutas previas permanecen; antes de actualizar documentación o
  sincronizar, sólo Cargo.toml/Cargo.lock diferían. El núcleo no depende del paquete
  nuevo; no hay DDL, perfil público, job o staging nuevo.

## Fallos previos preservados

La primera preparación usó un define mal capitalizado: produjo el flujo normal
en el archivo de probes. Se corrigió y se verificaron las acciones reales en otro
build. native-debug-1 esperaba SIGKILL; Darwin entregó SIGXCPU al alcanzar4s CPU.
Se corrigió el harness sin modificar el límite. native-debug-2, concurrente con
Rust, alcanzó watchdog5s con3,905133s CPU: no demostraba el límite CPU. Ambos perfiles
de aceptación3 verificaron CPU sin esa carga, sin ampliar presupuestos. Todos los
logs fallidos permanecen. La primera comprobación Python usó tomllib no disponible
en el intérprete del sistema; la comparación exacta de bloques de Cargo.lock usa
Counter de texto, sin instalar dependencias.

## Procedencia y gates

Cargo test --workspace --exclude noteka-desktop --all-targets recompiló únicamente
los dos CLI normales entre los16 artefactos previos. Sus hashes difieren del
baseline; no se restauraron ni se afirma identidad de rebuild. Se guardaron las
  variantes y los53 contratos coinciden exactamente. Los14 otros binarios permanecen
byte a byte; las16 copias anteriores siguen durables. Gates frescos aprobados sobre
  los artefactos actuales fijados en product-before-gates-pins.json. Dos cadenas
  mixtas iniciales fallaron: query-filter-xlsx315,243/page-next547,528 ms y luego
  distinct312,764/filter-xlsx494,709 ms. Locks aprobó ambas. Corpus/objetivos y
  validación permanecen intactos, sin optimización ni excepción. Se registró carga
  concurrente de otro laboratorio; no demuestra una causa única. El binario del
  test usa el mismo path y mtime anterior al cierre previo y no se recompiló en
  estos gates; no hay pin criptográfico histórico independiente de ese ejecutable.
  Un diagnóstico intercalado100 llamadas CLI completas compara baseline/current:
  CPU p95~211–226 ms, wall p95~249–315 ms y espera no-CPU p95~30–102 ms. No sustituye
  el gate oficial. La tercera cadena completa conserva logs propios. Los cuatro gates
del cierre OCR previo se conservan como evidencia previa, sin contarlos de nuevo.

Los3442 checks UI previos mantienen su atribución al cierre OCR; no se reejecutaron
ni se cuentan como nuevos en este lector aislado. No se atribuye aceptación nativa
de Home al headless. La comparación histórica exacta con v11 sigue no repetible
según autorización previa; no se fabricó nueva evidencia histórica.

## Matriz de escenarios

| Requisito | Evidencia |
| --- | --- |
| Lector PDF nativo independiente y explícito |22 fixtures por perfil; Unicode/multipágina/imagen/raster con texto/no_text/instrucciones como texto; fuente sólo por stdin; sin workspace |
| Admisión nativa y límites de fidelidad declarados | Cabecera/EOF, falso PDF, cifrado vacío/bloqueado; strict-utf16-and-text-limits; cinco limitaciones verificadas por Rust |
| Protocolo PDF nativo validado y acotado | Siete tests Rust,13 wires malformados, hash/metadatos/campos desconocidos/bytes/páginas y fronteras; pipes acotados |
| Contención nativa de PDF descartable |24 procesos por perfil, incluidos flags normales rechazados, RLIMIT0/4, CPU/RSS/plazos, bloqueo y recolección |
| Entrega nativa aislada y compatibilidad preservada | Preparación real debug/release normal/probes, fuente/pins/catálogos,326 Rust; no host/staging/DDL |
| Tres requisitos modificados heredados | Catálogo53 exacto, núcleo/lectores/MCP/cuotas sin cambios; regresión319 previa repetida dentro de326; reconciliación de literales con integración OCR ya verificada |

El parser del sistema puede recuperar anomalías internas; no se afirma validador
de integridad completo. El texto seleccionable no equivale a orden visual o tablas.
no_text no descarta texto rasterizado. Resultado dependiente deOS. Integración,
OCR dePDF, ampliaciones Excel y fase5a completa permanecen pendientes.


## Gates finales aprobados

La tercera cadena completa pasó consecutivamente, sin cambios de implementación,
corpus, cuotas, validación u objetivos entre los fallos y esta aceptación:

| Gate | Query p95 ms (mínimo–máximo) | Lock p95 máximo ms |
| --- | --- | --- |
| Mixto1 |195,959500–253,781083 |17,357125 |
| Mixto2 |192,479875–267,457625 |16,930250 |
| UTF-8/PDF1 |143,648208–226,828875 |22,331125 |
| UTF-8/PDF2 |138,067917–155,041625 |20,727666 |

Targets conservados: query <=300 ms y lock <=150 ms. Resultado acotado al corpus y
entorno medidos; no garantía bajo cualquier carga. Logs retry3-gates-stages.log y
logs/retry3-gate-*; los dos intentos fallidos y el diagnóstico permanecen separados.
No se demostró una causa única y no se atribuye la aprobación a una optimización.


## Sincronización y archivo

Único delta sincronizado/contrastado: cinco requisitos añadidos y tres modificados,
20 escenarios, siete nombres heredados y45 bloques ajenos de extracción intactos.
Las otras12 capacidades, título/Purpose y los seis archivos de Home son idénticos.
Main contiene176 requisitos en13 capacidades,53 de extracción. Validación estricta
15/15 antes/después de sync; archivo2026-10-17-add-bounded-pdfkit-reader, cinco
artefactos exactamente iguales tras moverlos. No se recreó ni resincronizó un
cambio anterior. Validación final/listado se conserva en evidencia externa.
