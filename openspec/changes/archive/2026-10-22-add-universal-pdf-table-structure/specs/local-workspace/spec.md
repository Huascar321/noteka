# Spec Delta

## ADDED Requirements

### Requirement: Migración recuperable de estructura PDF

El sistema MUST versionar la estructura como migración aditiva 019 con backup previo y transacción atómica. SHALL conservar todas las generaciones previas y dejar la estructura nullable; una falla o versión futura MUST conservar el workspace anterior.

#### Scenario: Migración desde 18
- **WHEN** se abre un workspace 18 con extracciones existentes
- **THEN** la migración agrega las columnas, conserva filas/índices y deja integridad válida

#### Scenario: Fallo o versión futura
- **WHEN** falla la migración o el archivo declara una versión posterior
- **THEN** no se publica un estado parcial ni se descartan datos
