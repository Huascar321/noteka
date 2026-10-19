# Verificación del lector OCR independiente

`add-bounded-macos-ocr-reader`, aceptación técnica aprobada; cerrado 6/6, sincronizado/contrastado y archivado el 17/10. La planificación no acredita soporte
OCR integrado. Ruta durable en `~/.local/share/noteka-verification/active-ocr-reader-path.txt`.
Baseline previo: 469 archivos tar/SHA-256 y ocho binarios CLI/MCP/helpers copiados;
catálogos completos debug/release capturados. No se recupera el v11 histórico.

## Hitos y fallos conservados

- Protocolo inicial: once tests Rust aprobados. Clippy detectó una variante enum
grande; se cambió a Box sin alterar JSON, sin exceptuar el warning. Clippy posterior
aprobó. La aceptación final volverá a probar las fuentes actuales por perfil.
- El preparador confundió symlinks del SDK con ausencia y luego invocó el destino
swift-frontend sin modo driver. Se valida el destino pero se invoca swiftc. El
plugin SwiftifyImport intentó crear un sandbox anidado y no pudo bajo deny-network;
el flag del compilador evita sólo ese sandbox interior, conservando la política
externa. Compilación posterior sin advertencias; no se instaló toolchain/modelo.
- El generador de fixture necesitó anotar su acumulador como MainActor bajo
Swift 6; corregido sin bajar la versión o desactivar checks de concurrencia.
- Primer corpus: 22 fixtures, con dieciséis de calidad limpia y 706 caracteres.
Se recuperó de las respuestas una CER de 5/706 (0,7082%); no es la aceptación
completa, que se detuvo antes de procesos. Se distinguió imagen irreconocible
de tipo reconocido no admitido sin reflejar contenido en errores.
- La fixture TIFF inicial excedía 1 MiB sin compresión y fallaba en framing antes
del rechazo de formato buscado. Se redujo sólo esa fixture a 100×100 y se agregó
precondición de tamaño; los textos esperados de calidad no cambian. El corpus
inicial y sus hashes se conservan, no se sobreescriben.
- La prueba CPU esperaba SIGKILL/watchdog, pero Darwin terminó correctamente por
SIGXCPU. La prueba reconoce esa señal y comprueba límite configurado 4 s y CPU
medida <=4,5 s de tolerancia. No se cambió el límite del helper.
- Antes de la regresión del workspace se detectó que el nuevo ejemplo había usado
el nombre validate_reply ya existente en XLSX. Se renombró validate_ocr_reply y se
reconstruyen ambos ejemplos originales/perfiles; no se alteran fuentes XLSX ni
se utiliza la sonda OCR para acreditar sus contratos.
- La búsqueda de un literal UTF-8 dentro del ejecutable falló en release: Swift
optimizado puede representar strings cortos sin un bloque contiguo. Se sustituyó
esa suposición de layout por comprobación ejecutable de los ocho flags: el normal
debe rechazarlos y el binario de probes debe ejecutar sus casos. Se agregó además
un header real de plazo menor con cuerpo bloqueado, sin modificar producción.
- La revisión del protocolo detectó que se dividía os_version antes de comprobar
su tamaño. Se movió el límite antes de split/collect y se agregó una respuesta
hostil con un millón de separadores; evita expansión de memoria previa al rechazo.
Se repiten las pruebas de protocolo y aceptación con los validadores actuales.

`logs/`, `runtime/corpus-first/`, `runtime/accept-debug-first/`,
`runtime/accept-debug-second/` y el corpus/corridas siguientes preservan entradas,
respuestas, timings, reports parciales y fallos. Ningún esperado se ajusta al OCR.
El runner guarda checkpoints, limita pipes y mata/recolecta sólo su grupo hijo.

## Alcance de evidencia

Las respuestas se validan con Rust real, hash de cada archivo y metadatos
orientados. CER se calcula sobre Unicode exacto y LF sin normalización, WER sobre
palabras separadas por espacio. Se mantienen errores de cada fixture visibles;
una media buena no garantiza que un número o acento concreto sea correcto.

Los probes del perfil prueban su predicado con revisión/idiomas ausentes; no se
presentan como pruebas reales de otro OS o una instalación sin recursos. RSS
muestreado y ru_maxrss observado no son heap Rust, límite duro o memoria GPU.
No hay workspace, persistencia, llamadas a operaciones públicas, UI nativa o
inferencia remota. Aceptación final por perfil aprobada; revisión y sync/archivo se registran abajo.

## Aceptación técnica final

- Workspace debug/release: **298 tests Rust por perfil**, diez ignores
intencionales; 287 previos y once nuevos del protocolo. Después del guard de
metadatos se repitieron los once del protocolo y se reconstruyeron ambos
validadores OCR, sin cambiar código de los 287 previos.
- **53 casos por perfil** sobre las fuentes finales: 22 fixtures reales, diez
entradas malformadas y 21 de procesos/controles. Los ocho flags de probes se
ejecutan en su binario y se rechazan en el normal. Header real de 100 ms con
cuerpo bloqueado comprueba acortamiento del watchdog; stdin/stdout bloqueados,
CPU, RSS observado, core dumps, errores, exceso de pipes y reap aprobados.
- En ambos perfiles: **CER 5/706 = 0,7082%** (objetivo <=5% fijado antes),
**WER 5/106 = 4,7170%**, dieciséis fixtures limpias de impresión en una columna.
No se eliminan sus cinco errores ni se corrigen los textos esperados: acentos,
serif y Courier siguen mostrando errores concretos. Las ocho orientaciones EXIF
pasaron, con dimensiones crudas/orientadas y hashes exactos.
- Máximo wall observado por fixture: debug **102,91 ms**, release **121,85 ms**;
máximo ru_maxrss **37.142.528 / 37.240.832 bytes** y peak footprint
**17.187.656 / 17.335.088 bytes**, respectivamente. Son máximos observados del
corpus, no p95/SLA ni memoria GPU. Los probes de exceso se reportan separados.
- Fmt/check/Clippy workspace y Clippy final del protocolo, contratos completos,
frontend, **75 tests de cliente/editor** y **11 de puente/lotes**, aprobados.
Swift debug/release/probes compiló offline con red denegada, sin warnings finales.
El warning Vite previo por chunk >500 kB permanece sin excepción/umbral nuevo.

Resultados actuales: `runtime/accept-debug-guard/results.json` y release;
`acceptance-summary.json`, `regression-stages.log`, `resumed-stages.log` y
`logs/metadata-guard-*`. La primera secuencia se detuvo ante el assert de layout
del binario; las etapas siguientes están identificadas como reanudadas, no
reescritas como si hubieran pasado de entrada. Los primeros 45 casos de debug
son un hito anterior y no se suman a los 53 finales.

## Conservación y reutilización delimitada

Los 469 archivos originales siguen presentes. Antes de actualizar documentación,
467 eran idénticos: sólo Cargo.toml añade un miembro y Cargo.lock un paquete
noteka-ocr-protocol. Quitando esa única entrada, el lock anterior es exactamente
igual; no cambiaron dependencias previas. Rust/core, migraciones, contratos,
generadores/goldens y Home permanecen intactos.

La comparación inicial de debug después de build workspace mostró CLI/helpers
distintos del baseline. Se investigó el recorte de compilación: compilar CLI sola
y luego PDF/XLSX juntos reprodujo **exactamente** los cuatro hashes debug; no se
copiaron binarios del baseline ni se normalizaron sus bytes. MCP debug y los cuatro
release ya eran idénticos. La CLI debug también coincide con la copia durable
utilizada por la revalidación SQL previa. `debug-*-build*.log`, los dos reports de
variantes y `preservation-review.json` conservan comandos y resultados.

Los catálogos completos debug/release son idénticos quitando sólo request_id:
SQLite v12/API v1/52, MCP 24/44/45, nueve mutaciones batchables. La sonda XLSX
original se recompiló y comprobó con respuesta XLSX válida y rechazo de OCR en
ambos perfiles. El nuevo protocolo no se enlaza al core/host.

Por esa igualdad se reutilizan, **sin declararlas nuevas ejecuciones**, las 36
etapas recientes de corpus/procesos/CLI/MCP, los 3.116 checks UI y los cuatro gates
release de [aceptación UI](verification-local-preparation-closure.md): query p95
113,11–210,00 ms y lock máximo p95 19,03 ms con corpus/cuotas/validación/targets
intactos. Este cambio aislado no añade procesamiento al montar o buscar. El fallo
anterior de 355,94 ms y su causa no demostrada permanecen en esa evidencia.

Matriz de seis requisitos/catorce escenarios en
[trazabilidad](verification-ocr-reader-scenarios.md). Home sigue activo 9/17; no se
usa su pantalla ni se acredita IPC/cierre/foco nativo. La integración OCR con
fuentes/cola/persistencia/FTS/UI continúa pendiente y no se declara 5a completa.


## Archivo comprobado

[Seis tareas completas](../openspec/changes/archive/2026-10-17-add-bounded-macos-ocr-reader/tasks.md).
El único delta local-extraction añade seis requisitos/catorce escenarios;
main pasa de 32 a 38 conservando los 32 bloques, título/Purpose previos. Comparación
posterior y validación estricta aprobadas; artefactos y .openspec.yaml conservan
hashes al archivar. Sólo Home permanece activo 9/17 antes del siguiente recorte.
El [review final](review-ocr-reader.md) y `archive-proof.json` registran el alcance;
no se declara OCR persistido, 5a o fases posteriores completadas.
