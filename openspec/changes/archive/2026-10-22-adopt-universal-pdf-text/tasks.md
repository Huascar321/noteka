# Tasks

## 1. Worker y protocolo

- [x] 1.1 Crear `noteka-pdfium-protocol` con header/reply/páginas y validación estricta.
- [x] 1.2 Crear `noteka-pdfium-worker` con `pdfium-render 0.9.4`, carga dinámica, límites y errores acotados.
- [x] 1.3 Añadir supervisor Rust con deadline, pipes no bloqueantes, hash y mapeo de errores.

## 2. Persistencia y extractor

- [x] 2.1 Registrar `pdfium_text` en `supported`, `eligible_for`, ramas de extracción y `pdf_storage::prepare_with_parser`.
- [x] 2.2 Ampliar lectura/integridad/retención de páginas PDF y el lector público sin migración.
- [x] 2.3 Probar extracción real, PDF vacío/texto, librería ausente, PDF inválido y no publicación parcial.

## 3. Búsqueda y superficies

- [x] 3.1 Ampliar `IndexedExtractor`, canonical/proyección y filtros de búsqueda.
- [x] 3.2 Actualizar catálogo, UI de preparación/lector, filtro y contratos generados.
- [x] 3.3 Probar índice, filtro y procedencia `pdfium_text`.

## 4. Verificación y cierre

- [x] 4.1 Ejecutar Rust/CLI/MCP/UI/headless, fmt, Clippy, contratos y OpenSpec estricto.
- [x] 4.2 Documentar guía/ADR, actualizar plan y conservar límites/fallos.
- [x] 4.3 Sincronizar deltas terminados y archivar; no iniciar fase6 en este cambio.
