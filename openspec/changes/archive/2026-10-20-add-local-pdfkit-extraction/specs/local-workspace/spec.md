# Spec Delta

## ADDED Requirements

### Requirement: Evolución recuperable a páginas PDFKit persistidas

El núcleo SHALL introducir tablas PDFKit mediante migración aditiva014 con backup consistente previo y transacción, sin modificar001–013, originales, datos/configuración desconocidos o selecciones previas. MUST mantener snapshots completos64MiB y base hash v2; clientes13 rechazan14 y el actual rechaza futuro15 sin writes. Migrar no ejecuta helper, trabajos o selección.

#### Scenario: Fixture v13 independiente y recuperación
- **WHEN** se prepara una fixture real con cliente13 durable antes de DDL y se migra a14
- **THEN** backup/restaurado conservan filas, SQL previo, hashes, control y ownership; sólo se añaden tablas vacías y versiones, sin procesamiento

#### Scenario: Rollback y versión ajena
- **WHEN** falla014 antes/después de actualizar versión, se abre14 con cliente13 o se encuentra15
- **THEN** la transacción revierte o la admisión rechaza sin reparar/modificar; el backup13 se restaura en otra ruta con cliente anterior

#### Scenario: Snapshots y retención nativa
- **WHEN** se realiza preview/apply o backup/restore con generaciones PDFKit y formatos anteriores
- **THEN** conserva todas las representaciones y cercas, excluye sólo derivados de base hash y verifica cuotas mixtas sin purgar fuentes o resultados ajenos
