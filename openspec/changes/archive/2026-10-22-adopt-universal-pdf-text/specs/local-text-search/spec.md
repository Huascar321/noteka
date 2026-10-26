# Spec Delta

## ADDED Requirements

### Requirement: Índice de adjuntos para PDFium

El índice derivado SHALL aceptar generaciones `pdfium_text` y validar su canónico con las mismas reglas de páginas/fragmentos que `pdf_text`. MUST conservar la procedencia `pdfium_text` y no mezclar sus postings con generaciones de otro extractor.

#### Scenario: Indexar y buscar PDFium
- **WHEN** se indexa una generación `pdfium_text` y se busca por un término de página
- **THEN** el hit conserva extractor `pdfium_text`, página y rangos de bytes; los filtros y la verificación de fuente siguen funcionando

#### Scenario: Generación alterada
- **WHEN** cambia el original o el canónico PDFium
- **THEN** la búsqueda falla de forma recuperable o marca la fuente obsoleta, sin devolver postings de otra generación
