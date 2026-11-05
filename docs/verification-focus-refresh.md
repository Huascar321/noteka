# Verificación del refresco y SQLite

## Línea de base anterior al cambio

El ejemplo `focus_refresh_benchmark` crea workspaces temporales sintéticos,
usa seis vistas reales de Home, tres ciclos de calentamiento y treinta
mediciones con hasta dos operaciones simultáneas. Cada operación incluye
prevalidación de workspace; no mide renderizado, IPC ni una interacción nativa.
Las muestras completas de esta ejecución se conservan fuera del checkout en
`/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/noteka-refresh-bench.c1L2JURi95/`.

| Motor | Entidades / relaciones | Ciclo p50 | Ciclo p95 | RSS máximo del proceso |
| --- | ---: | ---: | ---: | ---: |
| SQLite 3.51.1 | 1.000 / 2.200 | 34,83 ms | 38,26 ms | 25.968.640 bytes |
| SQLite 3.51.1 | 8.000 / 17.600 | 69,07 ms | 80,23 ms | 270.434.304 bytes |
| SQLite 3.54.1 | 1.000 / 2.200 | 32,42 ms | 40,39 ms | 27.262.976 bytes |
| SQLite 3.54.1 | 8.000 / 17.600 | 65,20 ms | 67,87 ms | 269.238.272 bytes |

El RSS es el máximo de todo el proceso, incluida la generación de fixtures; no
representa memoria exclusiva de las consultas ni de la aplicación gráfica.
Es una observación en este equipo, sin SLA ni prueba de saturación.

La ejecución inicial de `node --experimental-strip-types --test
tests/home-reads.mjs tests/embedded-reads.mjs` pasó 6/7. El fallo espera
`/incompatible/` aunque la implementación ya devuelve «Home response does not
match the query.»; ambas versiones están en HEAD antes de este cambio.

`cargo test --locked --offline -p noteka-core --test home` pasó 5/8. Los tres
fallos en `every_view_orders_and_paginates_unique_originals_and_declared_groups`,
`timestamp_and_id_orders_civil_age_and_archived_today` y
`view_membership_orders_and_partial_metadata_are_explicit` son los mismos
documentados y reproducidos en una copia aislada del HEAD anterior en
[verification-page-lock.md](verification-page-lock.md). No se adjudican al
cambio de SQLite ni se consideran pruebas aprobadas.

La revisión posterior encontró que las expectativas eran anteriores a cambios
de Home ya archivados: Types agregó el grupo Entity y preserva Topic; People
New/Updated filtra por período y comparte el orden de galería de Contacts con
desempate por título/ID. Con las expectativas actualizadas según los diseños de
[Tags](../openspec/changes/archive/2026-10-23-match-home-tags-reference/design.md)
y [People](../openspec/changes/archive/2026-10-24-match-home-people-reference/design.md),
`cargo test --locked --offline -p noteka-core --test home` pasa **8/8**. La
prueba de metadatos inválidos ahora comprueba el `unavailable_count` y recupera
el JSON original; la de orden comprueba tanto el fallback como `gallery_order`.
La línea de base 5/8 se conserva arriba para no confundirla con esta repetición.

## Cambio de SQLite

El lock resuelve rusqlite 0.41.0 / libsqlite3-sys 0.39.1. Los tres binarios
desktop, CLI y MCP compilaron y contienen la cadena 3.54.1; una consulta SQL
ejecutada por el core confirmó esa versión y FTS5. Pasaron 19 pruebas focales
de motor, WAL con escritores/checkpoint, workspace, formatos desconocidos,
migración y backup/restauración; 14 pruebas CLI y nueve de MCP; y el recorrido
MCP de transporte real con red denegada y fixtures aisladas. El CLI también
verificó un escritor concurrente con lecturas Home. El guard de deadlines
conserva la interrupción y se probó con guards anidados, limpieza y panic.

La suite amplia del core con cuatro hilos terminó 263 aprobadas, cuatro fallidas
y 16 ignoradas: tres son las comparaciones históricas documentadas en
`verification-page-lock.md`; la cuarta fue un timeout del worker PDF OCR antes
de su barrera y pasó al repetirla aisladamente. La ejecución anterior también
tuvo un timeout PDFKit que pasó aislado y tres fallos de anidamiento, corregidos
y repetidos satisfactoriamente. No se declara la suite amplia aprobada. Los tres
fallos de Home permanecieron idénticos después del upgrade, antes de corregir las
expectativas conforme a los diseños posteriores descritos arriba.

El esquema SQLite del workspace sigue siendo 20; el cambio no agrega migración
ni altera APIs. Una app, CLI o proceso MCP anterior que continúe ejecutándose
conserva su motor viejo hasta su reemplazo y reinicio. Las mediciones no
garantizan ausencia de regresiones en equipos o corpus distintos.

## Línea de base gráfica simulada

Con SQLite 3.54.1 y antes del cambio de refresco, el benchmark adicional usó
Chromium headless, transporte de prueba HTTP y CLI real. Para cada corpus ejecutó
tres calentamientos y 30 ráfagas sintéticas de 20 eventos de foco; midió hasta
que las seis secciones quedaron listas. No simula el foco físico ni mide IPC
nativo. Las muestras completas están en `ui-before.json` en el directorio temporal
indicado arriba.

| Etapa | Entidades / relaciones | Solicitudes Home por ráfaga | Máximo simultáneo | p50 | p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Antes | 1.000 / 2.200 | 8 | 2 | 254 ms | 268 ms |
| Después | 1.000 / 2.200 | 6 | 2 | 205 ms | 225 ms |
| Antes | 8.000 / 17.600 | 8 | 2 | 427 ms | 467 ms |
| Después | 8.000 / 17.600 | 6 | 2 | 302 ms | 320 ms |

El archivo `ui-after-final.json` conserva las 30 muestras posteriores. Esta prueba
genera eventos sintéticos en una sola vista Home; la concurrencia entre paneles,
la edición pendiente y los errores se prueban por separado. El puente CLI añade
costos que no corresponden al IPC de Tauri, por lo que los tiempos no son una
promesa de rendimiento de la aplicación nativa.

Las 30 muestras también registraron el heap de JavaScript observado al finalizar
cada ráfaga. En millones de bytes, la mediana/máximo fue 39,4/60,2 antes y
51,6/75,4 después con 1.000 entidades; con 8.000 fue 47,9/59,8 antes y
54,8/95,5 después. Esta muestra aumentó, no disminuyó. La recolección de basura
no se controló y la medida excluye otros procesos, por lo que no permite concluir
que exista una fuga ni que el consumo total nativo haya mejorado; requiere una
prueba sostenida específica si se pretende evaluar eso.

La revisión headless específica pasó 19 comprobaciones sobre CLI y SQLite reales:
dos páginas visibles durante actualización, cursor deshabilitado, reinicio con
aviso, restauración del foco de teclado, error/reintento y protección del Board. Un conflicto externo conservó el
borrador y undo/redo, dos paneles visibles respetaron el límite de dos lecturas y
un foco recibido durante el guardado se procesó al finalizarlo.

La prueba headless `headless-specialized-refresh-smoke.mjs` pasó cinco
comprobaciones: las búsquedas de adjuntos y conjunta ejecutan una única pasada
posterior al foco y la descartan si se editan los criterios. Conservan sus
validaciones y errores propios.

La suite más amplia `headless-grouped-smoke.mjs` usa selectores de navegación
anteriores. En una copia temporal con sólo esos selectores actualizados llegó a
una falla de ancho en «Viewport estrecho sin overflow»; esa misma falla apareció
sin agregar los escenarios nuevos de refresco. No se declara aprobada ni se
atribuye al refresco sin un baseline de interfaz comparable. La prueba focal de
cinco comprobaciones no cubre el ancho de pantalla.

## Verificación nativa macOS

`NOTEKA_NATIVE_SCREEN=1 node scripts/native-focus-refresh-smoke.mjs` pasó con
una ventana Tauri autorizada y un workspace temporal sintético. La prueba
minimizó la ventana hasta perder el foco físico, creó una persona por CLI y
comprobó que apareciera en Home tras restaurar y enfocar la ventana. También
retuvo un bloqueo SQLite y ejecutó escrituras competidoras desde la app, CLI y
MCP: los tres cambios terminaron con revisión 2, los títulos esperados y
`integrity_check` correcto. Después del refresco se movió un proyecto sintético
de Planned a Doing mediante el diálogo del Board y se verificaron su estado y
las demás propiedades. La prueba nativa cubre el diálogo, no un arrastre físico;
el arrastre se comprobó en el navegador headless. No capturó pantalla ni usó datos personales.
El resultado y el workspace sintético están en
`/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/noteka-focus-native-QM1UEH/`.

El primer intento agotó los 60 segundos mientras macOS estaba bloqueado. Tras
desbloquearlo, otro intento verificó las escrituras de los tres clientes, pero
falló al recuperar el foco: el helper intentó enfocar antes de que macOS
terminara de restaurar la ventana minimizada. El helper de prueba se corrigió
para esperar esa restauración y tres repeticiones posteriores pasaron. El ejecutable
normal se recompiló sin la instrumentación `native-smoke`.
