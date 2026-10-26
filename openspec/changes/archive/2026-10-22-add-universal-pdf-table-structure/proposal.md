# Proposal

## Why

El recorte universal ya obtiene texto OCR con PaddleOCR, pero descarta la estructura de tablas que fue la principal ventaja del benchmark (86/86 celdas). Sin persistirla, Noteka no puede ofrecer tablas universales ni auditar sus celdas; la UI de tablas sigue dependiendo de CoreGraphics/Vision.

## What Changes

- `paddle_ocr` conserva por página las tablas devueltas por PP-StructureV3: filas, columnas, celdas con row/col/span y texto exacto.
- Migración aditiva 019: columnas nullable de estructura en `extraction_pdf_generations` (JSON acotado, hash y bytes). No se reescriben generaciones previas.
- Nueva operación de lectura `extraction.pdf_structure` por job, con generación/procedencia y estructura completa acotada.
- CLI/MCP y catálogo; la UI puede mostrar la estructura como JSON/lectura simple. No se inventan celdas ni se convierte Markdown en autoridad.
- Backup/restore, integrity y snapshot de lotes incluyen la estructura; no se borran originales ni otras generaciones.

## Capabilities

### New Capabilities
- Ninguna.

### Modified Capabilities

- `local-extraction`: estructura tabular Paddle persistida y lectura.
- `core-operations`: nueva operación/catálogo/CLI.
- `local-workspace`: migración 019 y recuperación.

## Impact

- Rust: `pdf_contracts`/`pdf_storage`/`pdf_structure_storage`, worker Paddle y extractor.
- SQLite 18→19; API 62→63; MCP lectura +1; batch9 intacto.
- Sin proveedores remotos: la estructura viene del runtime Paddle local ya configurado.
