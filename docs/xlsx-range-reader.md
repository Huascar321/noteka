# Lectura de Excel por hoja y rango

Cambio `add-local-xlsx-range-reader`, verificado6/6, cuatro deltas sincronizados/contrastados y archivado como2026-10-21-add-local-xlsx-range-reader. La operación nueva es aditiva:
SQLite16, representación XLSXv1 e índice FTS permanecen iguales. APIv1 incorpora
`extraction.xlsx_range` (56 operaciones; MCP28/48/49 según permisos); los55
inputs/outputs anteriores se contrastan con clientes16 independientes.

## Elegir qué leer

En «Leer generación local», elegí `xlsx_range`, ID de trabajo, ID de hoja del
manifiesto, rango como `A1:D100` y, opcionalmente, fila de encabezados. El lector
`xlsx_cells` sigue mostrando todas las hojas y sus descriptores, incluso ocultas
o vacías. No hay selección automática de hoja, tabla o encabezado. Abrir el panel
y modificar criterios no ejecutan consultas ni extracción.

CLI equivalente (JSON por stdin):

```sh
noteka --workspace <ruta> --json extraction xlsx-range
```

```json
{"id":"<id-del-trabajo>","sheet_id":1,"range":"A1:D100","header_row":1,"limit":20}
```

`header_row`, `limit` y `cursor` son opcionales; `null` en encabezado significa
que no se eligió. IDs de hoja son los declarados por el libro, no índices de la
lista. A1 es uppercase, sin `$`, nombres de hoja ni espacios; filas1–1048576,
columnas A–XFD, extremos ordenados y ancho máximo64 columnas. A1 y A1:B2 admiten
celdas o rectángulos. Encabezado debe estar dentro de las filas del rango. No se
normaliza silenciosamente el input; Rust valida rango, límites y scope.

## Celdas y encabezados

La salida conserva el manifiesto completo, descriptor de hoja, generación/fuente,
rango y fila elegidos. `items` contiene sólo las celdas guardadas dentro del rango,
en orden del libro, con ordinal global original y localizador hoja/A1. No crea
celdas por dimensiones, ni expande merges o rellena huecos. Incluye también las
celdas de la fila de encabezados para que elegirla no descarte datos.

`headers` es vacío sin elección. Con encabezado contiene una entrada por columna:
`column`, `reference` e `item` (celda completa con ordinal/localizador, o `null` si
esa celda no existe). Una celda existente con `cached:null` es distinta de una
celda ausente; texto vacío, nombres repetidos, fórmula literal y caché siguen
separados. Encabezados se muestran como datos, sin deducir nombres de propiedades.
Números largos permanecen cadenas exactas; no hay cálculo, formato visual ni
conversión de seriales a fecha. Se mantienen limitaciones del parser original.

`matching_cell_count` cuenta celdas existentes del rango, no una grilla densa.
`covers_range` sólo es verdadero si esta única página contiene todas las celdas
coincidentes. `covers_generation` además exige contener todas las celdas del libro;
una página final de un recorrido no se presenta como generación completa.
Un rango vacío válido tiene cero items y `covers_range:true`, aunque otras hojas
contengan celdas. No prueba que el archivo esté visualmente vacío.

## Páginas, integridad y capacidad

Límite1–64/default32; UI20. Siguiente reemplaza la página; no acumula el libro.
Cursor<=2KiB ligado a workspace/ruta, operación, job/generación, hoja/rango/fila y
límite. Cambiar cualquier criterio o reemplazar generación invalida ese cursor.
No se reutilizan cursores de `xlsx_cells` o Search.

Se verifica original y representación canónica completos antes de filtrar,
incluido contenido fuera del rango. No helper, parsing ZIP/XML, red, mutación de
SQLite, indexación, recuperación de jobs o reparación implícita. Un error retira
la página anterior, conserva código/razón y no reintenta automáticamente. Salir o
editar descarta respuestas tardías sin liberar anticipadamente el cupo físico IPC.

Respuesta total<=2MiB, sin truncar. Encabezados completos pueden repetir celdas
largas: el corpus de representación exactamente1MiB provoca `response_size_limit`
con ocho encabezados y límite64; consultar límite1, un rango menor o sin encabezado
permite leer sin ampliar caps. La UI conserva20 y explica reducir rango/omitir
encabezado, sin cambiar automáticamente la decisión. Retención32MiB y snapshots64MiB
siguen iguales porque esta consulta no persiste una segunda representación.

## Compatibilidad y verificación

No hay DDL/migración ni cambios de workers/protocolos nativos. Cliente16 previo
puede abrir, leer y restaurar el mismo workspace/backup. Batch mantiene sus nueve
mutaciones y rechaza esta lectura; MCP permite consultar en modo lectura, con
workspace fijo y siete exclusiones, sin conceder extracción/escritura por contenido.
Home9/17 se conserva sin sincronizar ni archivar.

`scripts/test-xlsx-range.py --profile debug|release --old-bin <binarios16> --output
<ruta-nueva>` usa originales y golden XLSX propios, creados/extraídos con el cliente16
conservado. El cliente nuevo sin helper contrasta hojas/celdas/headers/ordinals,
cursor, pureza, clientes anteriores, desconocidos y backup/restauración. Sólo datos
sintéticos en `NOTEKA_TEST_TMP` fuera de Git y CLI con red denegada.

Tests Rust `xlsx_range_tests` cubren rangos, integridad exterior, scope y reemplazo,
lectura scoped/restore y exclusión batch. Controlador: `npm run test:generation-reader`.
`npm run test:headless-reader` conserva los870 checks previos y añade rangos en
ambos temas, fórmulas/cachés/headers hostiles, errores, respuestas tardías y buffer.
La [verificación](verification-xlsx-range-reader.md) registra resultados ejecutados.
Esta ampliación de Excel no completa tablas PDF ni toda5a;5b/5c esperan su cierre.
