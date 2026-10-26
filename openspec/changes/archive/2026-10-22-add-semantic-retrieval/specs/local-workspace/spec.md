# Spec Delta

## ADDED Requirements

### Requirement: Evolución recuperable del índice semántico

El sistema MUST versionar el esquema semántico como migración aditiva 017, con backup previo y transacción atómica. SHALL conservar originales, documentos, entidades, revisiones, jobs, FTS y formatos desconocidos e inicializar el índice vacío. Una falla o versión futura MUST conservar el workspace anterior sin reparación ni descarte.

#### Scenario: Migración desde 16 con datos existentes
- **WHEN** se abre un workspace 16 con entidades, documentos, extracción, FTS y adjuntos
- **THEN** la migración 017 crea el esquema semántico vacío, conserva todos los datos previos y deja el workspace en 17 con integridad válida

#### Scenario: Fallo y versión futura
- **WHEN** la migración 017 falla o el archivo declara una versión posterior a 17
- **THEN** el workspace anterior queda intacto, se informa el backup o la versión y no se borran tablas, originales ni índices

#### Scenario: Backup y restauración semánticos
- **WHEN** se respalda y restaura un workspace con una generación semántica activa y otra retirada
- **THEN** la restauración valida manifiesto, integridad y originales, conserva la generación activa y sus fuentes, y no activa generaciones en construcción
