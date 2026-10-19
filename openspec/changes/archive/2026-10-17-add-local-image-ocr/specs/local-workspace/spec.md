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
- **WHEN** se abre un workspace v14 o se encuentran jobs/configuraciones futuros o un índice de versión/perfil futuros en v12
- **THEN** v14 se rechaza sin modificar archivos; los jobs/configuraciones se conservan no ejecutables y un índice futuro no se reinterpreta, migra a perfil conocido ni elimina mediante reset

### Requirement: Compatibilidad recuperable del índice XLSX v12

El sistema MUST incrementar la versión de workspace a v12 antes de admitir selecciones XLSX, aunque las tablas FTS sean genéricas. SHALL rechazar acceso a v12 desde un cliente v11 sin escrituras, y acceso index/search/scoped a v11 desde el cliente nuevo sin migración implícita. La apertura explícita conserva el índice literal_unicode61_v1 y permite restaurar el backup v11 con el cliente anterior en otra ruta; no hay downgrade in-place, reset ni reindexación como sustituto de migración.

#### Scenario: Baseline v11 poblado
- **WHEN** se migra un workspace creado por un binario v11 independiente con UTF-8/PDF seleccionados, XLSX extraído sin seleccionar, jobs running y configuraciones futuras
- **THEN** el paso v11→v12 sólo cambia versiones y la continuación aditiva a la versión vigente conserva contenido/filas/hashes/revisiones/ownership y XLSX sigue sin seleccionar, compatible para un put posterior dentro de cuotas

#### Scenario: Cliente anterior y backup previo
- **WHEN** un CLI/MCP v11 intenta usar el workspace v12 y luego su CLI restaura el backup previo en otra ruta
- **THEN** el acceso al nuevo formato falla antes de escrituras y el backup recupera el formato/resultados v11 completos sin borrar escrituras posteriores del workspace v12

#### Scenario: Cursores y recibos al migrar
- **WHEN** se conserva ruta/identidad/perfil/revisión del índice durante v11→v12 y luego se agrega una selección XLSX
- **THEN** el cursor previo del índice conserva validez antes de ese put y vence por revisión después; los recibos de lote v11 se rechazan en v12 y requieren preview nuevo

#### Scenario: Snapshot con índice mixto
- **WHEN** se indexa XLSX entre preview y apply de un lote válido en v12 o un put excede el snapshot de 64 MiB
- **THEN** apply conserva el índice actual sin ampliación de whitelist ni invalidación por cambio puramente derivado; el exceso revierte completo y no elimina selecciones para caber

## ADDED Requirements

### Requirement: Evolución recuperable a OCR persistido v13

El sistema MUST añadir el almacenamiento OCR mediante migración 013 con backup previo consistente y transacción, sin modificar migraciones 001–012, originales, documentos, cola/ownership o selecciones previas. SHALL rechazar clientes antiguos/futuros incompatibles y conservar configuraciones/datos desconocidos sin OCR, selección, reparación o purga implícitos al migrar.

#### Scenario: Baseline v12 real preservado
- **WHEN** se abre una copia poblada creada por el binario v12 durable, con originales y generaciones/selecciones UTF-8/PDF/XLSX, jobs y datos futuros
- **THEN** el backup previo es v12 restaurable y v13 conserva exactamente filas/hashes/revisiones/control/ownership previos; sólo se añaden tablas OCR vacías y cambian versiones

#### Scenario: Migración fallida y cliente anterior
- **WHEN** falla 013 antes del commit o el cliente v12 intenta usar v13
- **THEN** el fallo conserva versión/datos anteriores y backup indicado, o el cliente antiguo rechaza antes de escrituras; el backup previo se restaura en otra ruta sin mezclar escrituras posteriores

#### Scenario: Versión futura sin reparación
- **WHEN** un cliente v13 encuentra un workspace posterior o configuración/representación no soportada
- **THEN** conserva archivos/datos y devuelve incompatibilidad sin convertir, ejecutar, borrar o reparar automáticamente

### Requirement: Recuperación y snapshots de OCR íntegro

Backup/restore y snapshots SHALL conservar representación, manifiesto, índice/huellas y originales OCR como parte del estado completo, validándolos en destino nuevo sin helper. MUST sumar OCR al cap retenido y mantener snapshot <=64 MiB, excluir derivados de base.v2 y rechazar corrupción/exceso sin purga o sobreescritura; una migración invalida recibos previos conforme al contrato existente.

#### Scenario: Restore OCR sin motor
- **WHEN** se restaura un backup con OCR seleccionado, vacío, intento anterior y metadata futura, sin helper disponible
- **THEN** el estado íntegro se conserva y sus lectores/búsqueda funcionan sin inferencia; la representación/posting/original alterados se rechazan sin destino parcial o modificación del origen

#### Scenario: Snapshot lote y cuota
- **WHEN** se crean previews/applies con OCR y cambios derivados concurrentes o una operación excede snapshot/retención
- **THEN** el lote conserva todo el esquema/estado OCR y las nueve mutaciones, base.v2 no cambia sólo por derivados y el exceso revierte completo sin aumentar caps ni eliminar datos
