# Spec Delta

## ADDED Requirements

### Requirement: Extracción PDF textual universal con PDFium

El sistema SHALL ofrecer `pdfium_text` para extraer texto por página con PDFium y persistirlo en el formato PDF canónico con parser, versión y localizadores. MUST cargar PDFium desde una ruta explícita, validar el PDF, respetar límites y fallar de forma recuperable si falta la librería. MUST NOT descargar binarios, pesos ni contenido remoto. `pdfium_text` SHALL ser la opción prioritaria para PDF nuevo cuando esté disponible; `pdf_text`/`pdfkit_text` MUST seguir seleccionables.

#### Scenario: PDF con texto nativo
- **WHEN** se ejecuta `pdfium_text` sobre un PDF válido con texto
- **THEN** se publican páginas/fragmentos con parser PDFium, la misma procedencia de página y el lector `extraction.pdf_fragments` los devuelve sin cambios

#### Scenario: PDFium ausente o PDF inválido
- **WHEN** la librería no está disponible, el PDF está cifrado/corrupto o supera los límites
- **THEN** el trabajo falla con un error recuperable, no publica una generación parcial y las generaciones/otros extractores permanecen intactos

#### Scenario: Prioridad sin ruptura
- **WHEN** se prepara un PDF nuevo con la opción universal
- **THEN** la UI/CLI usan `pdfium_text`; las generaciones `pdf_text`/`pdfkit_text` existentes siguen legibles y `pdf_text`/`pdfkit_text` pueden elegirse manualmente
