# Verificación de lectura de generaciones

Cambio `add-local-attachment-reader-ui`, **integrado en App y con aceptación UI aprobada**.
Home permanece intacto; regresión/gates compartidos con preparación aprobados.

## Baseline y cliente

Baseline durable de **439 archivos** copiados en tar.gz y contrastados por SHA-256,
fuera de Git. Ruta indicada por
`~/.local/share/noteka-verification/active-attachment-reader-path.txt`.
En el hito aislado las **209 fuentes congeladas** de preparación se comprobaron
iguales y el lector todavía no estaba importado. Esto conservó el producto de esa
primera aceptación; la integración posterior se documenta abajo.

**20/20 tests** del controlador aprobados, más check Svelte sin errores/warnings.
Cubren los tres comandos exactos, páginas/identidad/generación/errores, respuesta
tardía y cupo físico/remount/cancelación en cola. No acreditan UI o IPC nativo.

## Fixture nueva y golden

`prepare-attachment-reader-fixture.py`: nueve fuentes, doce páginas y 47 llamadas
CLI reales. Raíz externa obligatoria bajo NOTEKA_TEST_TMP, sin datos personales.
UTF-8 largo con BOM/CRLF/Unicode y HTML aparente, PDF con 23 páginas (21 con texto y
dos no_text), XLSX con 25 celdas/valor largo/hoja very_hidden/merge y casos independientes
de shared/array/estilos/seriales/coordenada extrema y vacíos. Se contrasta texto y
rango original, texto por página, tipos/celdas y hash canónico del golden Python,
sin utilizar el parser Rust como oráculo. No se selecciona ninguna generación.

Primer intento de fixture: error de sintaxis Python, conservado y corregido antes
de ejecutar. La ejecución siguiente aprobó sin alterar parser/corpus ni esperados.

## Presentación aislada

**848 checks headless** (424 por tema) aprobados sobre el componente de producción
compilado aisladamente con Vite, CLI y SQLite reales: texto/localizadores/procedencia
y manifiestos exactos, todas las páginas/tipos, source/derivado alterados, error
tipado/HTML escapado, regeneración/cursor, respuestas tardías, foco y 1280/900 px,
con cero requests externos. No certifica integración App, guardias ni IPC nativo.
Reporte/capturas/operaciones: `runtime/noteka-reader-headless-ekHeS3/` y
`logs/headless-isolated-sixth.log`.

Las primeras comparaciones detectaron corrupción UTF-8 en el **nuevo puente de
prueba**: concatenaba Buffers de stdout convirtiéndolos individualmente en strings,
por lo que podía cortar un carácter entre chunks. Se corrigió con setEncoding
continuo; un test reproduce determinísticamente la falla anterior con bytes
fragmentados y comprueba igualdad posterior. No se cambió producción, el golden
ni la comparación exacta. El intento de diagnóstico con dynamic import no está
soportado por el VM de Playwright CLI y se sustituyó por Web Crypto del navegador.

La inyección inicial de corrupción violaba el CHECK de longitud de SQLite. Se
cambió por texto corrupto de la **misma cantidad de bytes**, conservando constraints
y hashes para provocar el error real de integridad; no se deshabilitó validación.
El favicon del harness aislado se hizo explícito para evitar su GET 500 artificial.
Todos los logs anteriores permanecen fuera de Git.

**Dos tests** del puente aprobaron: decoder UTF-8 exacto, origen/workspace
confinados, UI sin permiso de mutaciones y lectura real con snapshot sin cambios.
El puente sólo se usa en pruebas y no forma parte del producto.

## Aceptación integrada

**870 checks headless** aprobados en App (435 por tema), incluidos título pendiente,
apertura/salida con guardia y remount de nota. Reporte en
`runtime/noteka-reader-headless-xhhBSj/` y `logs/headless-product-first.log`.

La secuencia se ajustó para integrar después de aceptación UI aislada y repetir
ambas UIs mientras la regresión Rust/CLI de fuentes inmutables sigue en paralelo.
Revalidación conjunta: preparación **154**, Search **1.248** y anteriores **606**,
sin cambiar sus asserts; sumados al lector son **2.878 checks de producto**.
Siete tests de puente aprobados y host no-bundle offline/locked aprobado sin ventana.
La reparación UTF-8 también se aplicó al puente anterior y los nombres de operación
con underscore se traducen al subcomando CLI con guion, preservando contratos.

Se mantienen las 209 fuentes previas y sus pins; **207 permanecen idénticas**.
App y puente tienen cambios identificados y revalidados. Los pins actuales cubren
**217 fuentes**, incluidas las ocho nuevas del lector. La búsqueda conjunta aún
no importada queda explícitamente fuera de esta aceptación; no se declara probada.
En ese hito faltaba la aceptación integrada de núcleo/gates y revisión/sync/archivo;
la sección final siguiente documenta su cierre.

## Aceptación técnica final conjunta

La [aceptación conjunta](verification-local-preparation-closure.md) registra las
36 etapas frescas, cuatro gates nuevos, 222 pins y **3.116 checks headless** sobre
la integración final. Sustituye los conteos parciales de los hitos anteriores,
que se conservan como historia. En el estado final hay 206/209 fuentes originales
iguales; App, puente y ayuda Search son las tres diferencias identificadas y
revalidadas. No se cuentan la regresión/gates compartidos como ejecuciones
independientes para cada cambio. Los tres deltas se sincronizaron/contrastaron y los cambios se archivaron el
17/10; no se declara toda 5a ni aceptación nativa.
