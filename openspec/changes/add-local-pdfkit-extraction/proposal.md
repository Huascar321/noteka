# Proposal

## Why

El lector PDFKit independiente está verificado y archivado, pero sus páginas no
pueden prepararse ni leerse mediante las operaciones compartidas del núcleo.
Integrarlo exige supervisión Rust, persistencia recuperable y selección explícita
de otro extractor, conservando el perfil PDF estricto existente.

## What Changes

- Incorporar consumidor Rust acotado reutilizable antes de tocar persistencia;
  validar respuesta/hash completos y conservar cancelación, deadline y recolección.
- Añadir extractor explícito pdfkit_text, representación canónica de páginas y
  metadatos, retención compartida y lectura paginada extraction.pdfkit_pages.
- Diseñar y probar recuperación real v13 antes de DDL; migración aditiva14 y
  barrera de clientes anteriores, conservando fuentes y contenido desconocido.
- Exponer operaciones compartidas CLI/MCP y empaquetado explícito del helper.
  Importar, montar, leer o buscar no lo ejecuta. No sustituir pdf_text por defecto.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- local-extraction: preparación explícita y lectura de páginas PDFKit persistidas.
- local-workspace: migración, snapshots, backup/restore y barrera v14.
- core-operations: operación de lectura y metadatos de extractor.
- local-mcp: exposición acotada de la nueva lectura compartida.

## Impact

Núcleo Rust, protocolo/runtime privado, migración, CLI/MCP, contrato generado,
preparación de helpers, tests y documentación. Baseline durable542 archivos/22
binarios, SQLite13/API53/MCP25/45/46/batch9. Nueva lectura propone API54 y
MCP26/46/47; no es estado ya implementado. Cuotas/gates y originales se conservan.

Fuera del recorte: selección FTS de este formato y su UI, OCR dePDF, tablas,
ampliaciones Excel, Home9/17, embeddings/agentes, modelos o remotos. La UI actual
continúa con sus formatos conocidos. Esos próximos recortes no se declaran
implementados al preparar este backend. No completa toda5a. El límite horario
vigente de5a sigue en2026-10-18 02:16 UTC; detener e informar si continúa abierta,
preservando estado recuperable y sin archivar trabajo incompleto.
