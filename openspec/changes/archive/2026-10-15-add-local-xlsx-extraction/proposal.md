# Proposal

## Why

El lector XLSX aislado ya está verificado, pero todavía no permite procesar y
consultar libros administrados mediante las operaciones de Noteka. Integrarlo
ahora requiere persistencia recuperable y fencing, no sólo añadir un nombre al
catálogo.

## What Changes

- Añadir `xlsx_cells` v1 a la cola explícita, reutilizando el helper descartable
  verificado sin enlazar ZIP/XML al core ni alterar su subconjunto/límites.
- Migración aditiva v10→v11 con backup previo, rollback y restauración: workbook
  canónico <=1 MiB, chunks UTF-8 <=4 KiB/512 y manifiesto con hash/conteos. No hay
  offsets del ZIP ni truncamiento de celdas largas.
- Operación de sólo lectura `extraction.xlsx_cells`, paginada y tipada, con
  procedencia de generación/fuente, hojas/estilos/fórmulas/cachés y límites claros.
  API v1 pasa de 51 a 52 operaciones; las 51 previas se conservan.
- CLI `extraction xlsx-cells` y MCP desde el contrato compartido, sin ampliar
  permisos: lectura por defecto; encolar requiere write y ejecutar write+run.
- Rechazar explícitamente indexación XLSX; eliminar supuestos binarios UTF-8/PDF
  en integridad y dispatch, conservando búsqueda/validación/caps/target <=300 ms.

## Capabilities

### New Capabilities

Ninguna; se extienden fronteras existentes.

### Modified Capabilities

- `local-extraction`: admisión XLSX explícita, publicación cercada, lectura tipada,
  integridad y presupuesto agregado sin indexación implícita.
- `local-workspace`: migración v11 aditiva, backup/restore con derivados XLSX y
  conservación de versiones/configuraciones futuras.
- `core-operations`: operación/CLI/contratos aditivos sin nueva mutación batchable.
- `local-mcp`: lector XLSX proyectado desde core bajo los permisos existentes.

## Impact

Core/storage/supervisor/migración 011, CLI, policy MCP, contratos generados,
fixtures sintéticas y harnesses. El protocolo adquiere esquemas de DTOs públicos,
no cambia sus bytes ni reglas. Sin nuevas bibliotecas de parsing ni modelos.
Baseline independiente de 355 archivos y ocho binarios fuera de Git.

## Non-goals

No Excel genérico, cálculo, macros, formato visual, inferencia de encabezados,
XLS/XLSB/ODS/Sheets, FTS XLSX, embeddings, proveedores, OAuth ni distribución.
No pantallas nuevas o cambios a Home/aceptación nativa. La autorización del
usuario cubre desarrollo local autónomo, no remotos ni archivos personales.
