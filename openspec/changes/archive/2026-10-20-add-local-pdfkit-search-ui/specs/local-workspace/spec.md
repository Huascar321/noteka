# Spec Delta

## MODIFIED Requirements

### Requirement: Migración aditiva y recuperación de derivados XLSX

El sistema SHALL recorrer v10→v11 con manifiesto/chunks XLSX y v11→v12 con barrera de compatibilidad y llegar al esquema vigente. MUST respaldar primero, migrar transaccionalmente, validar integridad y conservar originales, cola/pausa, generaciones, índice FTS, IDs y documentos desconocidos; migrar no ejecuta ni selecciona. Backup/restore verifican representaciones e índice mixto con originales en destino nuevo sin helpers. El paso v12 conserva DDL, perfil, revisión, hashes y filas UTF-8/PDF.

#### Scenario: Migración de baseline v10 real
- **WHEN** se abre con el núcleo nuevo un workspace creado por el binario v10 independiente, con datos y generaciones UTF-8/PDF indexadas
- **THEN** recorre v11/v12 hasta el esquema vigente con backup v10 restaurable y almacenamiento XLSX inicialmente vacío, sin alterar datos/hash/resultados anteriores ni arrancar helpers

#### Scenario: Falla y rollback de migración
- **WHEN** falla la migración de v10 o v11 antes del commit, incluso tras actualizar las versiones
- **THEN** el workspace conserva íntegramente su versión y datos de origen, sin estado parcial; se informa el backup utilizable

#### Scenario: Restore actual y original alterado
- **WHEN** se respalda/restaura el formato actual con XLSX seleccionado y faltan helpers o se altera representación/posting/original del backup
- **THEN** el caso íntegro se restaura, lee y busca sin parsing; el alterado se rechaza sin sobrescribir un destino existente, aun con hash externo de SQLite reparado

#### Scenario: Versiones futuras
- **WHEN** un cliente anterior a v14 abre un workspace v14 o se encuentran jobs/configuraciones futuros o un índice de versión/perfil futuros en v12
- **THEN** ese cliente rechaza v14 sin modificar archivos; los jobs/configuraciones se conservan no ejecutables y un índice futuro no se reinterpreta, migra a perfil conocido ni elimina mediante reset

### Requirement: Evolución recuperable a páginas PDFKit persistidas

El núcleo SHALL introducir tablas PDFKit mediante migración aditiva014 con backup consistente previo y transacción, sin modificar001–013, originales, datos/configuración desconocidos o selecciones previas. MUST mantener snapshots completos64MiB y base hash v2; clientes13 rechazan14 y el cliente14 rechaza15 sin writes. Migrar no ejecuta helper, trabajos o selección.

#### Scenario: Fixture v13 independiente y recuperación
- **WHEN** se prepara una fixture real con cliente13 durable antes de DDL y se migra a14
- **THEN** backup/restaurado conservan filas, SQL previo, hashes, control y ownership; sólo se añaden tablas vacías y versiones, sin procesamiento

#### Scenario: Rollback y versión ajena
- **WHEN** falla014 antes/después de actualizar versión, se abre14 con cliente13 o el cliente14 encuentra15
- **THEN** la transacción revierte o la admisión rechaza sin reparar/modificar; el backup13 se restaura en otra ruta con cliente anterior

#### Scenario: Snapshots y retención nativa
- **WHEN** se realiza preview/apply o backup/restore con generaciones PDFKit y formatos anteriores
- **THEN** conserva todas las representaciones y cercas, excluye sólo derivados de base hash y verifica cuotas mixtas sin purgar fuentes o resultados ajenos

## ADDED Requirements

### Requirement: Compatibilidad recuperable del índice PDFKit v15

El sistema SHALL introducir barrera15 antes de admitir PDFKit en el índice, con backup14 previo consistente y migración transaccional sin cambiar tablas/perfil/revisión/filas anteriores. MUST conservar originales, buffers y formatos desconocidos; migrar no procesa ni selecciona y clientes14/futuro16 se rechazan sin writes.

#### Scenario: Baseline14 y selección posterior
- **WHEN** se migra una fixture real14 durable con cinco formatos, PDFKit sin seleccionar y selecciones anteriores
- **THEN** sólo cambian versiones; backup14 es restaurable con cliente14 en otra ruta y los hashes/resultados/cursor existentes se conservan, salvo cobertura que ahora reconoce PDFKit como compatible sin seleccionar

#### Scenario: Rollback y admisión ajena
- **WHEN** falla015 antes/después de versión, un cliente14 intenta15 o el cliente15 encuentra16
- **THEN** la transacción revierte íntegramente a14 o el cliente rechaza antes de writes; no hay downgrade, reset o reindexación implícitos

#### Scenario: Snapshot backup y datos futuros
- **WHEN** se respaldan/restauran selecciones PDFKit, se aplican lotes concurrentes o existen configuraciones desconocidas
- **THEN** conserva representaciones/huellas/selecciones completas y originales con snapshots64MiB/base.v2/batch9 intactos; corrupción o exceso se rechazan sin purga ni normalización
