# Spec Delta

## ADDED Requirements

### Requirement: Operación de estructura PDF universal

El núcleo SHALL exponer `extraction.pdf_structure` en API v1 y CLI/MCP con esquema estricto, lectura por job y salida acotada. MUST conservar las 62 operaciones previas y no otorgar escritura ni acceso genérico.

#### Scenario: Catálogo y lectura
- **WHEN** se consulta el catálogo o se lee la estructura de un job Paddle
- **THEN** la operación aparece con su esquema y devuelve datos sin modificar el workspace

#### Scenario: Entrada inválida
- **WHEN** el job/generación no existe o el input tiene campos desconocidos
- **THEN** el núcleo rechaza con validación/no encontrado
