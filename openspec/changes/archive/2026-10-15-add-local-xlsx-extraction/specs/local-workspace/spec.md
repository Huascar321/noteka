# Spec Delta

## ADDED Requirements

### Requirement: Migración aditiva y recuperación de derivados XLSX

El sistema SHALL evolucionar v10 a v11 añadiendo únicamente tablas de manifiesto/chunks XLSX y los índices/constraints necesarios. MUST respaldar primero, migrar transaccionalmente, validar integridad y conservar datos originales, cola/pausa, generaciones UTF-8/PDF, índice FTS, IDs y documentos desconocidos; migrar no ejecuta extractores. Backups v11 incluyen representaciones XLSX verificadas y originales y se restauran en un destino nuevo sin helper.

#### Scenario: Migración de baseline v10 real
- **WHEN** se abre con el núcleo nuevo un workspace creado por el binario v10 independiente, con datos y generaciones UTF-8/PDF indexadas
- **THEN** queda v11 con backup v10 restaurable y tablas XLSX vacías, sin alterar datos/hash/resultados anteriores ni arrancar helpers

#### Scenario: Falla y rollback de migración
- **WHEN** falla la migración después de crear las tablas XLSX
- **THEN** el workspace conserva v10 y sus datos completos, sin tablas parciales; se informa el backup utilizable

#### Scenario: Restore actual y original alterado
- **WHEN** se respalda/restaura v11 con XLSX y el helper no está disponible o se altera una representación/original del backup
- **THEN** el caso íntegro se restaura y lee sin ejecutar parsing; el alterado se rechaza sin sobrescribir un destino existente

#### Scenario: Versiones futuras
- **WHEN** se abre un workspace v12 o se encuentran jobs/configuraciones de extractor futuros en v11
- **THEN** v12 se rechaza sin modificar archivos y los jobs futuros se conservan visibles/no ejecutables, sin reinterpretarlos como XLSX ni descartarlos
