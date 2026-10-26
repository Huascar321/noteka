# Proposal

## Why

La OCR de PDF depende de Vision/CoreGraphics en macOS. El benchmark eligió PaddleOCR PP-StructureV3 como el mejor resultado de tablas/texto, con licencia Apache 2.0 y ejecución local. Se necesita un extractor universal que produzca texto por página sin exigir frameworks Apple.

## What Changes

- Nuevo extractor `paddle_ocr` que ejecuta un worker Python externo con PDFium/PaddleOCR PP-StructureV3 y persiste texto por página en el formato PDF canónico existente.
- Perfil/procedencia `paddleocr_ppstructurev3` 3.7.0; rutas de Python, script y modelos configuradas por variables de entorno. No se descargan modelos al abrir, buscar o ejecutar otra operación.
- El worker no recibe workspace, SQL, shell ni red; el núcleo valida hash, páginas, texto, deadline y salida acotada.
- Reutiliza `extraction_pdf_*` y `extraction.pdf_fragments`; no hay migración.
- `paddle_ocr` queda como opción universal preferida para PDF escaneado; `pdf_ocr`/PDFKit/Vision siguen disponibles. Las tablas/estructura de Paddle no se persisten en este recorte: sólo texto por página.

## Capabilities

### New Capabilities
- Ninguna.

### Modified Capabilities

- `local-extraction`: extractor universal `paddle_ocr` y su perfil.
- `core-operations`: catálogo/CLI/UI y filtros.
- `local-text-search`: índice de páginas PaddleOCR.

## Impact

- Rust: supervisor `paddle_supervisor`, ramas de extracción/lectura/búsqueda.
- Dependencia externa opcional: Python, `paddleocr 3.7.0`, `paddlepaddle 3.3.1`, `pypdfium2 5.14.0` y modelos PP-OCRv6/estructura instalados por el usuario. No se versionan pesos.
- SQLite sigue en 18; API/MCP no agregan operaciones, sólo el enum de extractores.
