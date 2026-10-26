# Spec Delta

## ADDED Requirements

### Requirement: Catálogo y filtros PaddleOCR

El catálogo SHALL anunciar `paddle_ocr` y CLI/UI MUST permitir encolarlo, prepararlo y filtrarlo. El filtro de adjuntos MUST distinguirlo de `pdf_ocr`/`pdfium_text` sin agregar operaciones.

#### Scenario: Catálogo y filtro
- **WHEN** se consulta el catálogo o se filtra por `paddle_ocr`
- **THEN** el extractor aparece con su perfil y los hits conservan página y procedencia

#### Scenario: Nombre no publicado
- **WHEN** se envía un extractor desconocido
- **THEN** el núcleo lo rechaza sin crear trabajos
