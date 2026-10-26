# Spec Delta

## ADDED Requirements

### Requirement: Catálogo y filtros del extractor PDFium

El catálogo SHALL anunciar `pdfium_text` entre los extractores admitidos y el CLI/UI MUST permitir encolarlo, prepararlo y filtrarlo. El filtro de búsqueda de adjuntos y el catálogo de perfiles MUST distinguir `pdfium_text` de `pdf_text`/`pdfkit_text` sin cambiar las operaciones existentes.

#### Scenario: Catálogo y filtro
- **WHEN** se consulta el catálogo o se filtra la búsqueda por `pdfium_text`
- **THEN** el extractor aparece con su nombre exacto y los resultados usan el mismo lector/localizador de páginas

#### Scenario: Extractor desconocido
- **WHEN** se envía un nombre fuera de los extractores publicados
- **THEN** el núcleo rechaza la solicitud sin crear trabajos ni generaciones
