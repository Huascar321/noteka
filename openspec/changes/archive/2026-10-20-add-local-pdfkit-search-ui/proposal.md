# Proposal

## Why

PDFKit ya publica páginas recuperables por CLI/MCP, pero la interfaz no permite
prepararlas/leerlas y FTS rechaza su selección. Completar esa integración reutiliza
la cola, los lectores y la búsqueda agrupada existentes sin rehacer el lector nativo.

## What Changes

- Proyectar sólo texto persistido PDFKit por página/rango UTF-8, con perfil,
  OS/build y limitaciones completas; selección FTS explícita/CAS e integridad total.
- Incorporar el tipo en preparación, lector paginado y Search individual/agrupada,
  conservando buffers, cupos físicos, teclado, temas y Home intacto.
- Barrera de compatibilidad SQLite15, sin nuevas tablas ni reindexación; recuperación
  real desde cliente14 durable y conservación de las selecciones anteriores.
- Mantener54 operaciones/MCP26/46/47/batch9: sólo index/search amplían sus DTOs;
  reconciliar números/versiones heredados que describen el catálogo o cliente actual.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- local-extraction: proyección explícita y preparación/lectura PDFKit en escritorio.
- local-text-search: evidencia PDFKit en los ámbitos existentes sin procesamiento.
- local-workspace: barrera15/recuperación y versiones de clientes precisadas.
- core-operations: extensión identificada de index/search y contratos conservados.

## Impact

Núcleo Rust, migración015 sólo de versión, contratos generados y paneles existentes.
Sin nuevas dependencias, operaciones, helpers o modelos. Baseline real14/54 y
cliente durable conservados fuera de Git, con checkpoint completo tras cerrar la
integración backend. Los corpus/gates300/150 y cuotas no se alteran.

Fuera de alcance: OCR dePDF, tablas, ampliaciones Excel, embeddings/harness, Home,
fase6, proveedores/OAuth/pagos/modelos/publicación o ventanas nativas. Este recorte
no cierra toda5a. La autorización del20/10 permite implementar y verificar el ciclo
completo sin una aprobación adicional entre planificación e implementación.
