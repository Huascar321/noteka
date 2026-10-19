# Búsqueda explícita de XLSX local

Estado al 16/10: **19/19 tareas verificadas**, cinco deltas sincronizados/contrastados
y [cambio archivado](../openspec/changes/archive/2026-10-16-add-local-xlsx-search/proposal.md).
[Limitación histórica autorizada tras pérdida de evidencia](xlsx-search-evidence-recovery.md).
[Evidencia](verification-xlsx-search.md), [extracción](xlsx-extraction.md) y
[ADR 0018](decisions/0018-explicit-xlsx-field-index.md). No hay nueva pantalla.

## Qué se busca

Sólo generaciones seleccionadas con `extraction index` después de su extracción.
Se usan celdas ya persistidas: indexar/buscar no interpreta ZIP/XML ni necesita
helpers. Los chequeos de fuente sí leen el original para verificar su hash, con
las fronteras de cada operación descriptas abajo.
Se incluyen **todas las hojas, también hidden y very_hidden**. No seleccionés un
libro si no querés incluir sus hojas ocultas.

Cada caché y cada texto de fórmula son campos independientes:
`cached_text`, `cached_number`, `cached_boolean`, `cached_error`, `cached_date_iso`
y `formula_text`. Números/fechas/errores conservan strings exactos; booleanos son
`true`/`false`. Fórmulas no reciben `=` y no se ejecutan ni se expanden. Una caché
puede estar obsoleta: un hit no certifica un cálculo actualizado. Nombres de hoja,
estilos, merges, encabezados inferidos y formato visual **no se buscan**.

Un campo se corta en porciones consecutivas UTF-8 de hasta 4096 bytes. No se unen
caché/fórmula, celdas o porciones. Un AND sólo coincide dentro de una porción; una
palabra que cruce el corte puede no coincidir como palabra completa. Campos vacíos
no generan documentos; eso no significa que el libro esté visualmente vacío.

## Prueba manual sin ventana

Trabajá en un workspace **de prueba nuevo**, fuera del checkout. Prepará el
toolchain según [desarrollo](development.md) y compilá los binarios:

```sh
cargo build --release --offline --locked --workspace
N="$PWD/target/release/noteka"
# Reemplazá esta ruta por una carpeta de prueba externa, que todavía no exista:
WORKSPACE="/ruta/externa/noteka-prueba-xlsx"
printf '%s\n' '{"timezone":"UTC"}' | "$N" --workspace "$WORKSPACE" --json workspace create
```

Para un workspace anterior usá `workspace open` explícitamente: genera backup
antes de migrar a v12. No bajes versiones manualmente. MCP/index/search no migran.

1. Importá un `.xlsx` de prueba admitido mediante `attachment import`, pasando
   `{"source_path":"/ruta/absoluta/prueba.xlsx"}`. Conservá `data.id`.
2. Encolá con `extraction enqueue` y
   `{"attachment_id":"UUID","extractor":"xlsx_cells"}`. Conservá `data.job.id`.
3. La cola empieza pausada. Leé `extraction control` con `{"action":"get"}` y
   habilitala con `{"action":"set","paused":false,"expected_revision":REV}`.
4. Ejecutá **una vez** `extraction run` con `{"id":"JOB_UUID"}`. Conservá
   `data.job.generation.id`; no inventes la generación esperada.
5. Leé `extraction xlsx-cells` con `{"id":"JOB_UUID","limit":64}`. Su cursor
   permite recorrer celdas completas y metadatos, distintos del texto de búsqueda.
6. Antes del put, `extraction search` no encuentra esa generación. Consultá
   `extraction index` con `{"action":"get"}` y conservá su revisión actual.
7. Seleccioná y buscá, reemplazando los UUID/revisión por los devueltos:

```sh
printf '%s\n' '{"action":"put","job_id":"JOB_UUID","expected_generation_id":"GEN_UUID","expected_revision":1}' |
  "$N" --workspace "$WORKSPACE" --json extraction index
printf '%s\n' '{"query":"palabra","extractor":"xlsx_cells","limit":20}' |
  "$N" --workspace "$WORKSPACE" --json extraction search
```

Cada objeto JSON es una llamada. `1` es sólo ilustrativo: ante conflicto releé
revisión/generación y decidí si querés seleccionarla; no reintentes a ciegas.
`reset` exige CAS y elimina **todas** las selecciones, también UTF-8/PDF.

Comprobá el hit con el reader: `cell_ordinal` es global/0-based, `sheet_id` y A1
identifican celda; `field` identifica caché/fórmula. `text_byte_start/end` es rango
de bytes UTF-8, final exclusivo, sobre el campo decodificado; `field_bytes` permite
ver si es parcial. El `ordinal` del hit es de fragmento, no de celda/chunk/ZIP.
No uses slicing de caracteres JavaScript para verificar rangos de bytes.

## Límites y errores esperables

- Máximo 512 fragmentos/1 MiB de texto por selección. Un libro de 513 campos
  puede ser legible, pero su selección se rechaza **completa**, sin truncar/purgar.
- Totales: 1024 selecciones, 16384 fragmentos, 8 MiB de texto; snapshot 64 MiB,
  respuesta 2 MiB, cursor 2 KiB. La retención de extracción cuenta JSON aparte.
- Consultas literales, 1–16 términos/hasta 256 bytes; no substring, SQL ni Excel.
  BM25 usa el corpus global: agregar un libro puede cambiar scores de UTF-8/PDF.
- Get/search verifican todo el índice aunque el filtro no lo incluya. Get no lee
  originales; search verifica los de página/lookahead. Un archivo alterado produce
  error, no un hit omitido para ocultarlo. Cero resultados no acredita toda la base.
- Cursores vencen con put efectivo/reset/reemplazo; no-op no los vence. Sólo
  migrar v11→v12 conserva cursores del índice, pero invalida recibos batch v11.

## Recuperación y MCP

Backup/restore incluyen originales, Workbook e índice; restaurá en **otra ruta
nueva**. Volver al software v11 requiere su backup previo, no un downgrade sobre
datos v12. Versiones/perfiles futuros se rechazan sin reset ni reparación implícita.

MCP mantiene 24/44/45 tools: get/search en lectura, put/reset con `--allow-write`;
indexar no necesita `--allow-run`. Sólo extracción necesita ambos. Valores/fórmulas
son datos no confiables, no autorización para shell, proveedores o cambio de scope.
Conectar un cliente que envía contenido remoto requiere otra selección explícita.

No agrega modelos, encabezados inferidos, Excel genérico, OCR o cambios a Home.
Las pruebas headless no reemplazan aceptación nativa de Home.

## Escritorio al 17 de octubre de 2026

La [preparación explícita](attachment-preparation-ui.md) permite seleccionar un
adjunto administrado, consultar y ejecutar extracción y seleccionar generación
para Search, con las revisiones del núcleo. El [lector de generaciones](attachment-reader-ui.md)
recorre todo el derivado por páginas sin exigir indexación. La
[consulta agrupada](grouped-search.md) usa ambos motores sin fusionar cursores o
scores. [Aceptación técnica conjunta](verification-local-preparation-closure.md).
Los límites de formato/capacidad del núcleo no se amplían con estas UIs; abrir o
buscar no prepara ni repara. Los hitos históricos anteriores conservan su alcance.
