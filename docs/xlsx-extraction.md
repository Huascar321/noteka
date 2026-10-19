# Extracción local de celdas XLSX

**Integración verificada y archivada, 11/11 tareas.** SQLite v11/API v1/52,
conservando los 51 contratos previos. El [lector independiente](xlsx-reader.md)
se integró por un cambio separado, con [aceptación final3](verification-xlsx-extraction.md):
[`add-local-xlsx-extraction`](../openspec/changes/archive/2026-10-15-add-local-xlsx-extraction/proposal.md).
Ese cierre describe el baseline histórico v11. El producto vigente es v12 por
[búsqueda XLSX](xlsx-search.md), verificada en otro cambio sin alterar este reader.

## Selección y ejecución

Original administrado `.xlsx` (case-insensitive), MIME workbook OOXML u
`application/octet-stream`, <=1 MiB. `xlsx_cells`/config v1 usa la cola compartida;
importar/abrir/encolar/leer/migrar no ejecuta el helper. La cola inicia pausada.
`extraction.run` procesa como máximo un job y usa un sibling descartable
`noteka-xlsx-worker`, sin parser ZIP/XML en core. No se aceptan rutas de helper,
shell, contraseñas ni proveedores desde payloads.

## Representación y localizadores

Se preserva el [subconjunto y límites del lector](xlsx-reader.md), incluidos
números como strings exactos, fórmulas y cachés separadas, metadatos shared/array,
visibilidad/rangos/merges/estilos y sistema de fechas. No cálculo, conversiones a
fechas, formato visual, inferencia de encabezados o expansión de grillas/merges.

`extraction.xlsx_cells` recibe `{id,limit?,cursor?}`, con ID del job, límite 1–64
(32 por defecto). Devuelve manifiesto completo y celdas completas con ordinal
global y localizador `{kind:"xlsx_cell",sheet_id,reference}`. La celda tiene
coordenada/fila/columna, índice de estilo, valor tipado guardado opcional y fórmula
opcional. `null`/ausencia de valor no equivale a texto vacío; una caché no es un
resultado recalculado. El manifiesto incluye todas las hojas, incluso vacías.

Páginas ligadas a scope/ruta, operación, job/generación y límite; reemplazar una
generación invalida su cursor. Lectura verifica todo el resultado y el original,
no sólo items de la página, y no necesita el ejecutable del helper. La salida
completa sigue <=2 MiB; no se prometen offsets al ZIP ni citas visuales de Excel.
Open de CLI/UI verifica íntegramente los derivados XLSX en su snapshot antes de
configurar escritura, sin helper ni reparación. La admisión scoped de MCP conserva
su preflight estructural puro; cada lectura XLSX valida el resultado completo.

## Persistencia, presupuestos y recuperación

[ADR 0017](decisions/0017-persisted-xlsx-cells.md). SQLite 011 aditiva guarda una
sola serialización JSON canónica Workbook v1 <=1 MiB, dividida en chunks UTF-8
<=4096 bytes/512 con hash, conteos y FKs generation/job. `fragment_count` general
cuenta esos chunks internos, **no celdas**. Una celda de texto >4 KiB se devuelve
completa, reconstruida del JSON validado. No hay segunda copia persistida de celdas.

Los 32 MiB de retención suman UTF-8, texto PDF y JSON XLSX; sólo se descuenta la
generación del mismo job reemplazada. Error/deadline/cancelación/fence no publican
resultados parciales ni purgan el anterior. `result_from_previous_attempt` identifica
resultados retenidos si falla la regeneración. El helper mantiene heap vivo
solicitado 128 MiB/CPU 4 s/watchdog 5 s; el supervisor comparte deadline 5 s del job,
acota pipes y mata/recolecta el hijo. No es sandbox FS/red OS ni garantía de RSS.

Antes de v10→v11 hay backup consistente; migración transaccional con validación.
Una falla deja v10 y devuelve backup. Restore siempre a ruta nueva, valida JSON y
originales, sin parsing. Software v10 no abre v11: recuperar el backup previo en
otra ruta con binario compatible, sin descartar escrituras posteriores.

## Clientes y exclusiones

CLI: `noteka --workspace <ruta> --json extraction xlsx-cells` (JSON por
stdin), equivalente a invoke `extraction.xlsx_cells`. MCP proyecta esa lectura
desde core; enqueue necesita write, run write+allow-run y workspace actual.
El cambio de extracción no añadió UI, alteró Home ni habilitó índice XLSX. En su
baseline v11, index put devolvía `unsupported_index_extractor`; la [búsqueda explícita
posterior](xlsx-search.md) admite campos persistidos en v12, sin parsing ni helpers.
No modelos, remotos, Sheets, macros ni compatibilidad Excel genérica.

## Verificación ejecutable

`scripts/test-xlsx-cli.py --profile debug|release` ejecuta los 216 casos sintéticos
con golden independiente por CLI real, reconstruye páginas/metadatos/números,
contrasta JSON/chunks/hash y comprueba restore/lectura sin helper y fallo de retry.
Usar red denegada y `NOTEKA_TEST_TMP` fuera del checkout. La
[verificación final](verification-xlsx-extraction.md) registra ambos perfiles,
migración real, procesos/SDK, regresión, métricas y límites de la evidencia.

## Escritorio al 17 de octubre de 2026

La [preparación explícita](attachment-preparation-ui.md) permite seleccionar un
adjunto administrado, consultar y ejecutar extracción y seleccionar generación
para Search, con las revisiones del núcleo. El [lector de generaciones](attachment-reader-ui.md)
recorre todo el derivado por páginas sin exigir indexación. La
[consulta agrupada](grouped-search.md) usa ambos motores sin fusionar cursores o
scores. [Aceptación técnica conjunta](verification-local-preparation-closure.md).
Los límites de formato/capacidad del núcleo no se amplían con estas UIs; abrir o
buscar no prepara ni repara. Los hitos históricos anteriores conservan su alcance.
