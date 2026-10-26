# Proyector independiente de tablas PDF OCR

`noteka-pdf-table` es una biblioteca Rust local, aún sin integración en el producto.
No añade operación API57, cola, persistencia, lector UI o índice. SQLite16/API56,
MCP28/48/49 y batch9 permanecen iguales. No recrea ni modifica PDFKit/Vision ni sus
protocolos: recibe evidencia OCR ya obtenida por otro componente.

## Contrato y autoridad

Input estricto: hash declarado lowercase64, página física1–20, recognition OCRv2
completa, `x_edges` (2–33 escalares canónicos crecientes entre0 y1), `y_min/y_max`
y fila de encabezado opcional1-based. PNG orientación1 del raster, coordenadas
normalizadas con origen inferior izquierdo. No son coordenadas del PDF original.
Cada escala fuente y texto permanecen como strings exactos; f64 se usa sólo para
comparar geometría. Se rechazan geometría degenerada, evidencia no canónica,
campos desconocidos o perfiles no admitidos, sin normalizarlos silenciosamente.

El hash/página proceden del llamador. La biblioteca valida evidencia y proyección,
**no verifica el archivo ni el workspace**. Integrarla requerirá cargar/revalidar
el canónico y original completos, vigencia de generación/configuración y límites
del núcleo antes de usar su resultado. Ningún texto concede permiso para escritura,
red o ejecución. La biblioteca no tiene I/O ni dependencias del núcleo.

## Columnas, filas y conservación

El llamador elige rectángulo y bordes de columnas. Una región totalmente exterior
queda en `outside`; una que cruza un borde o columna produce `crossing_region`.
No se usa el centro para asignar una región que cruza, ni se divide el texto
reconocido para inventar celdas. Vision puede reconocer una fila como un solo
texto: en ese caso el proyector puede rechazar el grid elegido.

Regiones interiores se ordenan por centro vertical descendente y se agrupan sólo
cuando comparten un intervalo vertical común. Una cadena de solapamientos sin
intersección común, o una región que toca varias filas, produce `ambiguous_rows`.
Filas se presentan de arriba abajo; columnas siguen `x_edges`. Cada celda contiene
ordinales originales en orden de fuente, incluso si hay varias regiones por celda.
No es una afirmación de orden de lectura o de cálculo de valores.

El layout conserva el input/evidencia completo una sola vez y sólo añade
referencias por ordinal: **cada región aparece exactamente una vez en una celda
o en outside**. Incluye celdas vacías, no filtra confianza baja/cero y no transforma
textos numéricos. El encabezado elegido es referencia a fila, no títulos únicos,
y sigue formando parte de las celdas. No se infieren encabezados, columnas, spans,
merges o transcripción de texto no reconocido.

## Capacidad y ejemplo

Máximos512 regiones,32 columnas,64 filas y512 celdas, recognition canónica<=1MiB,
JSON de desarrollo/salida<=2MiB. Exceso rechaza completo sin truncar. Algoritmo
O(n²) acotado a512 regiones; sin modelo, red, archivos o procesos dentro de la
biblioteca. El futuro host debe imponer deadline/cancelación de su operación.

Ejemplo de desarrollo, JSON por stdin:

```sh
cargo run --offline -p noteka-pdf-table --example project < <evidencia.json>
```

No es CLI de Noteka ni helper de producción. Su lectura stdin acota bytes y
rechaza JSON extra/desconocido, pero no promete watchdog de pipes/proceso.
Errores devuelven sólo `ok:false` y razón; no incluyen texto fuente.

## Verificación

Seis tests por perfil debug/release: golden disperso independiente, encabezados
hostiles/repetidos, confianza0, celdas vacías/exteriores, roundtrip completo,
cruces/ambigüedad, límites32/64/512, ordinales, header fuera y geometría efectiva.
`cargo clippy -p noteka-pdf-table --all-targets -- -D warnings` y fmt aprobados.

`scripts/test-pdf-table.py --profile debug|release --output <ruta-nueva>` genera un
PDF propio MIT y fija el golden **antes** del OCR. Usa CLI vigente con red denegada
para extraer, lector público para obtener evidencia y el ejemplo para proyectarla.
En ambos perfiles:7 regiones, seis celdas exactas Item/Count, Alpha/12.50, Beta/003,
pie Footer fuera; ningún texto corregido, normalizado o descartado. Cinco casos de
framing y cruce real rechazados sin contenido en errores. Original/hash y evidencia
se conservan íntegros. No es garantía de calidad OCR en otros PDF.

Extra release sintético:512 regiones/512 celdas,938150 bytes de entrada JSON,
5 warmups/20 muestras, p95 **27,275459ms** de CLI completo frente a300ms. No es un
corpus nativo de512 regiones ni garantía universal. Se mantienen los cuatro gates
del producto, query máximo195,630542ms/lock17,499958ms, y fuente/binarios protegidos.
Evidencia en runtime/table-native-debug y table-native-release bajo el directorio
externo indicado por active-phase5-path.txt. La aceptación final y estado OpenSpec
se registran en [verificación](verification-pdf-table-projector.md).

La integración de tablas PDF sigue pendiente de5a. No habilita5b/5c ni fase6.
