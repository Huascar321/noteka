# Design

## Worker

El worker Paddle ya recibe `table_res_list[].pred_html`. Además de unir texto, parsea cada HTML con un parser de tabla estricto (filas, columnas, rowspan/colspan, celdas y texto) y devuelve `structure: {version:1,pages:[{page_number,tables:[{rows,columns,cells:...}]}]}`. No se guarda el HTML ni Markdown; sólo la estructura tipada. Si el JSON supera 1 MiB o el parser detecta HTML inválido, el job falla; no se trunca una tabla.

## Persistencia

Migración 019 agrega a `extraction_pdf_generations`:
`structure_json TEXT NULL`, `structure_sha256 TEXT NULL`, `structure_bytes INTEGER NULL`.
Sólo `paddle_ocr` la completa. `pdf_storage::publish` inserta columnas explícitas; las generaciones previas quedan NULL. `pdf_structure_storage` valida ownership, parser Paddle, hash/bytes, page numbers y que el JSON parsee el tipo público. La estructura se incluye en backup/integridad y snapshot de lotes.

## API

`extraction.pdf_structure` recibe `{id}` (job), valida job/generación, exige `paddle_ocr`, y devuelve `job_id`, `generation`, `result_from_previous_attempt`, `structure` y `manifest` (hash/bytes/format). Límite 1 MiB. No ejecuta worker. Si no hay estructura, `not_found`/`unsupported_generation_config` explícito.

## Límites

20 páginas, 1 MiB de JSON, 512 tablas, 4096 celdas por tabla, 512 caracteres por celda (los límites exactos se validan y fallan, no truncan). El texto OCR sigue en el formato PDF existente; la estructura es adicional.

## Alternativas

Guardar HTML/Markdown: descartado porque no es estable ni tipado. Convertir tablas a celdas del motor de colecciones: descartado; esta estructura es evidencia derivada, no una base de datos. Recalcular/reordenar: descartado; se conserva lo que Paddle devolvió con incertidumbre.

## Verificación

Migración 18→19 con datos, backup/restore, estructura real Paddle, lectura, ausencia, corrupción/hash, límites, CLI/MCP, UI headless y regresión.
