# Spec Delta

## MODIFIED Requirements

### Requirement: Migración aditiva y recuperación de derivados XLSX

El sistema SHALL recorrer v10→v11 con manifiesto/chunks XLSX y v11→v12 con barrera de compatibilidad. MUST respaldar primero, migrar transaccionalmente, validar integridad y conservar originales, cola/pausa, generaciones, índice FTS, IDs y documentos desconocidos; migrar no ejecuta ni selecciona. Backup/restore verifican representaciones e índice mixto con originales en destino nuevo sin helpers. V12 conserva DDL, perfil, revisión, hashes y filas UTF-8/PDF.

#### Scenario: Migración de baseline v10 real
- **WHEN** se abre con el núcleo nuevo un workspace creado por el binario v10 independiente, con datos y generaciones UTF-8/PDF indexadas
- **THEN** recorre v11 hasta v12 con backup v10 restaurable y almacenamiento XLSX inicialmente vacío, sin alterar datos/hash/resultados anteriores ni arrancar helpers

#### Scenario: Falla y rollback de migración
- **WHEN** falla la migración de v10 o v11 antes del commit, incluso tras actualizar las versiones
- **THEN** el workspace conserva íntegramente su versión y datos de origen, sin estado parcial; se informa el backup utilizable

#### Scenario: Restore actual y original alterado
- **WHEN** se respalda/restaura el formato actual con XLSX seleccionado y faltan helpers o se altera representación/posting/original del backup
- **THEN** el caso íntegro se restaura, lee y busca sin parsing; el alterado se rechaza sin sobrescribir un destino existente, aun con hash externo de SQLite reparado

#### Scenario: Versiones futuras
- **WHEN** se abre un workspace v13 o se encuentran jobs/configuraciones futuros o un índice de versión/perfil futuros en v12
- **THEN** v13 se rechaza sin modificar archivos; los jobs/configuraciones se conservan no ejecutables y un índice futuro no se reinterpreta, migra a perfil conocido ni elimina mediante reset

## ADDED Requirements

### Requirement: Compatibilidad recuperable del índice XLSX v12

El sistema MUST incrementar la versión de workspace a v12 antes de admitir selecciones XLSX, aunque las tablas FTS sean genéricas. SHALL rechazar acceso a v12 desde un cliente v11 sin escrituras, y acceso index/search/scoped a v11 desde el cliente nuevo sin migración implícita. La apertura explícita conserva el índice literal_unicode61_v1 y permite restaurar el backup v11 con el cliente anterior en otra ruta; no hay downgrade in-place, reset ni reindexación como sustituto de migración.

#### Scenario: Baseline v11 poblado
- **WHEN** se migra un workspace creado por un binario v11 independiente con UTF-8/PDF seleccionados, XLSX extraído sin seleccionar, jobs running y configuraciones futuras
- **THEN** sólo cambian versiones; contenido/filas/hashes/revisiones/ownership se conservan y XLSX sigue sin seleccionar, compatible para un put posterior dentro de cuotas

#### Scenario: Cliente anterior y backup previo
- **WHEN** un CLI/MCP v11 intenta usar el workspace v12 y luego su CLI restaura el backup previo en otra ruta
- **THEN** el acceso al nuevo formato falla antes de escrituras y el backup recupera el formato/resultados v11 completos sin borrar escrituras posteriores del workspace v12

#### Scenario: Cursores y recibos al migrar
- **WHEN** se conserva ruta/identidad/perfil/revisión del índice durante v11→v12 y luego se agrega una selección XLSX
- **THEN** el cursor previo del índice conserva validez antes de ese put y vence por revisión después; los recibos de lote v11 se rechazan en v12 y requieren preview nuevo

#### Scenario: Snapshot con índice mixto
- **WHEN** se indexa XLSX entre preview y apply de un lote válido en v12 o un put excede el snapshot de 64 MiB
- **THEN** apply conserva el índice actual sin ampliación de whitelist ni invalidación por cambio puramente derivado; el exceso revierte completo y no elimina selecciones para caber
