# Proposal

## Why

El texto PDF seleccionable depende hoy de PDFKit en macOS o de un parser acotado; el usuario pidió priorizar componentes universales. El benchmark local eligió PDFium como base portable de PDF. Agregar un extractor PDFium permite obtener texto en macOS y preparar el mismo camino para Windows/Linux sin reemplazar generaciones existentes.

## What Changes

- Nuevo extractor `pdfium_text` (versión 1) con worker Rust independiente que carga PDFium dinámicamente.
- Reutiliza las tablas `extraction_pdf_*` y el lector `extraction.pdf_fragments`, registrando `parser_name=pdfium` y la versión del binding/PDFium en el manifiesto.
- `pdfium_text` queda como opción prioritaria para PDF textual nuevo en UI/CLI cuando la librería está disponible; `pdf_text` (lopdf) y `pdfkit_text` siguen disponibles como alternativas y las generaciones previas no se reinterpretan.
- Catálogo/UI/búsqueda reconocen el nuevo extractor; el filtro FTS y la proyección de páginas son los mismos.
- Sin migración: no cambia el esquema. La librería PDFium se configura por ruta (`NOTEKA_PDFIUM_LIB` o sibling) y no se descarga al abrir/buscar.

## Capabilities

### New Capabilities
- Ninguna.

### Modified Capabilities

- `local-extraction`: nuevo extractor universal `pdfium_text`, perfil/procedencia y prioridad.
- `core-operations`: catálogo/CLI/UI y filtros reconocen el extractor.
- `local-text-search`: el índice de adjuntos acepta `pdfium_text` con los mismos localizadores de página.

## Impact

- Rust: nuevo protocolo/worker `noteka-pdfium-*`, supervisor en `noteka-core`, ramas de extracción/lectura/búsqueda.
- Dependencia opcional: `pdfium-render 0.9.4` (MIT OR Apache-2.0) y una librería PDFium del usuario; no se versionan binarios ni pesos.
- Frontend: opción de preparación/lectura y filtro; sin pantalla nativa.
- SQLite sigue en 18; API/MCP cambian sólo por el enum de extractores, no por nuevas operaciones.
