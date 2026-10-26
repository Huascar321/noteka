# Lectura de tablas PDF OCR

Integración verificada `add-local-pdf-table-reader`,8/8: API57 compartida,
sin migración. [Aceptación y límites](verification-pdf-table-reader.md).

Usá una generación `pdf_ocr` ya confirmada. Elegí página física1–20,
bordes de columnas como cadenas decimales canónicas `['0','0.5','1']`,
rectángulo `y_min/y_max` y opcionalmente una fila de encabezado1-based.
Coordenadas normalizadas respecto del raster de página con origen inferior
izquierdo; no son coordenadas del PDF. No se infieren columnas/encabezados,
no se corrige texto OCR ni se dividen regiones que cruzan bordes.

```sh
noteka --json --workspace "$WORKSPACE" extraction pdf-ocr-table <<'JSON'
{"id":"ID_DE_TRABAJO","page_number":2,"x_edges":["0","0.5","1"],"y_min":"0.4","y_max":"1","header_row":1,"limit":20}
JSON
```

CLI, core, MCP y lector existente comparten Rust. La interfaz ofrece
`pdf_ocr_table` dentro de «Leer generación local»; no prepara ni selecciona
para búsqueda al consultar. Siguiente reemplaza veinte celdas, con los mismos
criterios/cursor. Cambiar criterios retira la página y descarta respuestas tardías,
pero no libera el cupo físico hasta terminar IPC. Buffer/guardias permanecen.

Cada celda conserva ordinal row-major0-based, fila/columna1-based, regiones
completas y ordinales globales/locales. Celdas sin regiones no desaparecen.
`header_row` sólo identifica una fila; sus celdas siguen en la secuencia.
El manifiesto incluye todas las páginas, motor, raster y hashes de procedencia.
`outside` conserva referencias exactas de regiones excluidas del rectángulo;
el lector `pdf_ocr` permite consultarlas. `covers_table` sólo puede ser verdadero
en una única respuesta que incluye todas las celdas. `covers_generation` siempre
es falso: una tabla no devuelve todo el derivado original.

Se validan original y canónico completos, incluso fuera de la página solicitada,
antes de proyectar. Cursor<=2KiB ligado a workspace/ruta/operación/job/generación,
criterios y límite; restore en otra ruta requiere iniciar de nuevo. Límites
1–64/default32 celdas por respuesta,512 regiones/celdas,32 columnas,64 filas,
respuesta<=2MiB. Exceso, cruces o filas ambiguas fallan sin truncar ni reparar.

No persiste layouts, duplica FTS ni cambia SQLite16/snapshots/retención. Search
sigue recuperando las regiones OCR seleccionadas explícitamente, con la misma
procedencia de las celdas. Originales y generaciones previas se conservan.
