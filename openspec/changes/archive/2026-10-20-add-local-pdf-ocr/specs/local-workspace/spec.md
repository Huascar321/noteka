# Spec Delta

## MODIFIED Requirements

### Requirement: Compatibilidad recuperable del índice PDFKit v15

El sistema SHALL introducir barrera15 antes de admitir PDFKit en el índice, con backup14 previo consistente y migración transaccional sin cambiar tablas/perfil/revisión/filas anteriores. MUST conservar originales, buffers y formatos desconocidos; migrar no procesa ni selecciona y cliente14 ante15 y cliente15 ante16 se rechazan sin writes.

#### Scenario: Baseline14 y selección posterior
- **WHEN** se migra una fixture real14 durable con cinco formatos, PDFKit sin seleccionar y selecciones anteriores
- **THEN** sólo cambian versiones; backup14 es restaurable con cliente14 en otra ruta y los hashes/resultados/cursor existentes se conservan, salvo cobertura que ahora reconoce PDFKit como compatible sin seleccionar

#### Scenario: Rollback y admisión ajena
- **WHEN** falla015 antes/después de versión, un cliente14 intenta15 o el cliente15 encuentra16
- **THEN** la transacción revierte íntegramente a14 o el cliente rechaza antes de writes; no hay downgrade, reset o reindexación implícitos

#### Scenario: Snapshot backup y datos futuros
- **WHEN** se respaldan/restauran selecciones PDFKit, se aplican lotes concurrentes o existen configuraciones desconocidas
- **THEN** conserva representaciones/huellas/selecciones completas y originales con snapshots64MiB/base.v2/batch9 intactos; corrupción o exceso se rechazan sin purga ni normalización

## ADDED Requirements

### Requirement: Migración recuperable PDF OCR v16

El sistema SHALL añadir dos tablas derivadas mediante016 transaccional con backup15 previo consistente y conservar001–015, filas/SQL/selecciones/hashes/originales existentes. MUST mantener snapshots64MiB/base.v2/retención32MiB, preservar documentos/configuraciones futuros y rechazar clientes15 ante16 o cliente16 ante17 antes de writes; migrar no procesa ni selecciona.

#### Scenario: Baseline15 independiente y restore
- **WHEN** se migra una fixture15 creada antes de016 con cinco formatos seleccionados y se restaura el backup con cliente15 en destino nuevo
- **THEN** conserva exactamente filas/SQL/hashes/cursores/originales previos; sólo añade tablas vacías/versiones, sin activar helpers o cambiar documentos desconocidos

#### Scenario: Fallo antes después y admisión incompatible
- **WHEN** falla016 antes/después de versión o un cliente encuentra una versión ajena
- **THEN** revierte el origen15 íntegro o rechaza sin reparar/modificar; el backup independiente sigue utilizable y no se hace downgrade in-place

#### Scenario: Snapshot completo y corrupción
- **WHEN** se respalda/restaura o hace preview/apply con seis formatos y se altera representación/posting/original
- **THEN** conserva todos los derivados/selecciones en el caso íntegro y rechaza corrupción/exceso sin purgar o sustituir contenido del usuario
