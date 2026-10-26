# Spec Delta

## ADDED Requirements

### Requirement: Índice de adjuntos PaddleOCR

El índice SHALL aceptar generaciones `paddle_ocr` y validar sus páginas/fragmentos con las mismas reglas PDF. MUST conservar el extractor y no mezclar postings con otras generaciones.

#### Scenario: Indexar y buscar PaddleOCR
- **WHEN** se indexa una generación `paddle_ocr` y se busca un término
- **THEN** el hit conserva extractor `paddle_ocr`, página y rangos; los filtros y la verificación de fuente funcionan

#### Scenario: Fuente alterada
- **WHEN** cambia el original o el canónico
- **THEN** la búsqueda falla de forma recuperable o marca obsoleto sin devolver otra generación
