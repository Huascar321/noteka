# Spec Delta

## ADDED Requirements

### Requirement: Evolución recuperable de trazas del harness

El sistema MUST versionar el esquema de trazas como migración aditiva 018 con backup consistente previo y transacción atómica. La migración SHALL conservar todo el contenido, índices y datos previos e inicializar las trazas vacías sin ejecuciones activas.

Una migración fallida MUST conservar el esquema anterior. Una versión futura MUST rechazarse sin reparación. El backup/restauración SHALL conservar ejecuciones y pasos confirmados y no reactivar reservas de otra ruta.

#### Scenario: Migración desde 17
- **WHEN** se abre un workspace 17 con entidades, documentos, extracción e índice semántico
- **THEN** la migración 018 crea las tablas de trazas, conserva los datos previos y deja integridad válida

#### Scenario: Fallo o versión futura
- **WHEN** la migración 018 falla o el archivo declara una versión posterior
- **THEN** el workspace anterior queda intacto y no se borran trazas, originales ni índices
