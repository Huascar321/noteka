# Verificación de lectura de tablas PDF OCR

Cambio `add-local-pdf-table-reader`, verificado8/8, sincronizado y archivado.
SQLite16/APIv1/57, MCP29/49/50 y nueve mutaciones batchables. Cierre y evidencia descritos abajo.

## Baseline y recuperación

Evidencia nueva fuera de Git en el directorio indicado por
`~/.local/share/noteka-verification/active-phase5-path.txt`:
`2026-10-21-phase5-201534`. Se preservaron662 archivos con pins, catálogo completo
API56 y ocho binarios por perfil. No recupera ni reemplaza evidencias históricas.

Antes de editar se creó con cliente56 un workspace16 real con seis formatos,
bloque/schema/propiedad desconocidos y backup/restauración. Las seis lecturas son
idénticas con el cliente57; el56 restaura el backup nuevo y conserva desconocidos
byte a byte (`mixed-compatibility.json`). Sin DDL, migración017 o persistencia de
layouts. El único cambio de Cargo.lock de este recorte es el vínculo del núcleo
con la biblioteca local de tablas ya existente, sin paquetes externos nuevos.

`final-catalog-comparison.json` compara ambos perfiles:56 entradas/salidas,
errores, límites, batch/capabilities y todas las raíces/defs previas idénticos.
Sólo se añaden la operación, dos raíces y cuatro defs nombradas. No se cambiaron
hashes históricos para aceptar diferencias. Los clientes56 independientes siguen
disponibles fuera del checkout.

## Núcleo, CLI y MCP

379 pruebas Rust por perfil aprobaron antes del refactor; la regresión conjunta
final aprobó384 por perfil,13 ignoradas explícitas. Seis pruebas nuevas
de tabla cubren regiones múltiples/celdas vacías,512 celdas/regiones, ambigüedad,
fuente/canónico/configuración completos fuera de página, cursores con todos los
criterios/generación/scope/restore, intento anterior, entrada estricta y exclusión
de lotes. Las guardias y permisos anteriores se conservan.

El cliente CLI real usa un PDF propio de tres páginas con golden previo al OCR:
22 celdas,11 filas, página vacía y Footer exterior. Lee sin helpers, contrasta
regiones/localizadores y Search, rechaza cursores ajenos/fuente/canónico alterados,
restaura con cliente56 y comprueba snapshots antes/después. Se ejecutó en ambos
perfiles (`runtime/table-reader-debug3` y `table-reader-release3`). Las mediciones
iniciales de lectura release dieron p95 de36,893/21,539/27,867ms para primera,
siguiente y tabla completa. La repetición final sobre binarios actuales dio
23,462/25,151/21,932ms, con5 warmups/20 muestras; la aceptación conjunta de
rendimiento se completó con los cuatro gates originales descritos abajo.

28 tests SDK MCP por perfil aprobados sobre binarios reconstruidos. Modern,
legacy y auto descubren29/49/50 tools; paginan tablas y contrastan datos/esquemas
con CLI sin helpers. Read/write/run mantienen separación y siete exclusiones.

## Interfaz y regresión

101 tests de frontend/editor/controladores aprobados; contracts/check/build,
fmt y Clippy workspace/all-targets sin warnings Rust aprobados. Se conserva el
aviso conocido de Vite por tamaño del bundle; no se ocultó ni aumentó su umbral.

La nueva UI pasó286 checks headless en dark/light, sin requests externos:
preparación real, golden22 celdas, paginación20/2, una columna con varias regiones,
columna vacía, manifiesto/exteriores, página física vacía, límite32 columnas
rechazado por Rust, cruce/corrupción de fuente y canónico, error/tardío, teclado,
buffer/guardias y concordancia de Search individual/agrupado sin duplicación FTS.

El escape de markup usa una fixture **de transporte** con texto hostil sobre
respuesta real; no se presenta como reconocimiento real de ese HTML por Vision.
El texto permanece literal, no crea nodos ni ejecuta handlers. Snapshots verifican
pureza de cada lectura. Capturas sintéticas de ambos temas se inspeccionaron:
celdas/cobertura/procedencia legibles, wrap y contraste conservados. El puente CLI
no acredita IPC/foco/cierre nativo; no se abrió ninguna ventana.

Los4860 checks anteriores aprobaron:606 base+1248 evidencia+154 preparación+
1016 lector+238 agrupada+326 OCR de imágenes+822 PDFKit+450 PDF OCR. Con los286
nuevos suman5146 checks, todos con cero solicitudes externas en sus recorridos
aprobados. Host Clippy all-targets/native-smoke/custom-protocol y build no-bundle
aprobaron en target externo sin ejecutar la aplicación. Los cuatro gates300/150
finales aprobaron tras el refactor de integridad, sin cambiar fuentes o límites.

El recorrido base aprobó606 checks y cero solicitudes externas en la repetición
`table-regression-headless-base2`. La primera ejecución aprobó los606 checks de
producto/teclado pero falló al detectar nueve intentos HTTP a otro puerto local,
todos abortados. Ese listener pertenecía a un proceso Python de otra carpeta;
no se demostró el disparador de las solicitudes. Se conserva el fallo y se añadió
registro de origen/path/tipo/frame en la rama de error, sin exceptuar destinos o
cambiar aserciones. Los dos scripts compartidos con pruebas nativas, runner original y
ambos gates permanecen byte a byte idénticos al baseline. No se detuvo el proceso
ajeno ni se inspeccionaron contenidos de esa aplicación.

El primer recorrido PDFKit agotó los30s de espera de Home después de emitir sus
cinco consultas. No llegó a la preparación PDFKit. El replay CLI sobre la misma
fixture devolvió las cinco respuestas válidas en21,8–23,4ms; no se demostró la causa
de la espera del frontend. Sin cambiar fuentes/asserts/timeouts, la repetición
`table-regression-headless-pdfkit2` pasó822 checks y cero solicitudes externas.
Se preservan log/fixture iniciales y `pdfkit-home-failure-replay.json`; no se
presenta la repetición como diagnóstico resuelto ni aceptación nativa de Home.

## Fallos conservados y límites

El primer recorrido falló al usar `process` dentro del sandbox de run-code de
Playwright; se derivó la ruta externa desde la fixture conocida. La primera
compilación Svelte detectó el tipo tuple generado de x_edges; el adaptador usa el
tipo de transporte y Rust mantiene toda validación de longitud/geometría.

El golden inicial del CLI incluyó Footer al elegir y_min0.25; se corrigió el
rectángulo explícito a0.4 y el borde de cruce se calculó desde bbox real. No se
tocaron Vision/CoreGraphics ni se corrigió texto. Una lista READ sin ordenar falló
en la prueba scoped; se corrigió su orden y la prueba pasó. Todos los logs fallidos
permanecen. No se cambiaron caps, fuentes, corpus o objetivos para aprobar gates.

Esta proyección requiere columnas/rectángulo elegidos y rechaza cruces/ambigüedad;
no ofrece tablas genéricas, spans o lectura visual. La comparación aislada de
motores Mac/Windows está fuera de Git (`research/comparacion-y-recomendacion.md`),
sin adoptar motores nuevos en el producto ni acreditar ejecución Windows. Home
sigue9/17 intacto y fuera de este cierre; no se inició fase6.

## Aceptación conjunta final

El refactor de verificación completó6/6 con [cuatro gates intactos aprobados](verification-index-verification.md):
mixto1 query105,789–182,232ms/lock14,637ms; mixto2 100,981–179,935/14,885ms;
original1 59,803–163,630/18,445ms; original2 65,551–123,340/17,642ms.
Se conservan dos fallos mixtos previos y todos los intentos, sin causa puntual
fabricada ni excepción de muestras. La regresión fresca conjunta pasó384 Rust
por perfil,28 SDK por perfil,101 frontend y5146 headless, contracts/fmt/Clippy y
host no-bundle, sin ventana. Fuentes protegidas Home/App/helpers, corpus/gates,
postings y pruebas nativas compartidas conservan sus bytes.

CLI actualizado a target aislado repitió en ambos perfiles el PDF propio real y
cliente56, snapshots/cursor/backup/restore/Search exactos. Release: p95 primera
21,365ms, siguiente19,241ms y completa22,166ms;5 warmups/20 muestras por caso,
frontera proceso CLI+sandbox. La comparación56/57 completa sigue exacta.

Revisión final de código/diseño y17 requisitos/43 escenarios aprobada. El archivo
requiere contrastar los cuatro deltas con main y conservar33 nombres heredados,
194 requisitos no afectados, títulos y Purpose; la validación OpenSpec sólo
verifica documentos. El cierre de5a se registra en su auditoría después del archivo.

Cierre21/10: verificado, cuatro deltas sincronizados y contrastados; archivado como `2026-10-21-add-local-pdf-table-reader`. Home9/17 intacto.
