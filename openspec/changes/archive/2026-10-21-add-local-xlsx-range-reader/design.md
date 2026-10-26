# Design

## Context

Ver proposal. Loader XLSX ya reconstruye/valida canónico<=1MiB, todas las hojas/celdas y fuente antes de paginar. Baseline625 fuentes/clientes16 preservados y coincidentes; no formatos nuevos. El lector UI comparte ReadLifetime/ReadQueue(1).

## Goals / Non-Goals

**Goals:** consultar regiones dispersas verificables, no convertir una decisión de presentación en metadatos fuente.
**Non-Goals:** guardar configuración del rango, filtrar selección FTS por rango, cambiar parsers, originales, Home o protocolos nativos.

## Decisions

1. Operación aditiva `extraction.xlsx_range` con `{id,sheet_id,range,header_row?,limit?,cursor?}`. Rango A1 o A1:B2 uppercase canónico; coordenadas1-based, fila<=1048576/columna<=16384, extremos ordenados y ancho<=64. Encabezado opcional dentro de filas del rango. Alternativa: ampliar inputs/outputs de xlsx_cells rompe igualdad de55 contratos previos; endpoint separado conserva lectores existentes.
2. Compartir loader y validación fuente, con transacción Deferred. Filtrar sólo después de validar toda la representación. Headers por cada columna del rango con reference y `item` nullable (celda completa+ordinal), sin asignar nombres, rellenar valores ni expandir merges. Items incluyen también fila encabezado. Columnas ausentes tienen null; cached null dentro de celda existe y es distinto. Orden y ordinal global del libro se conservan. Se escanea máximo10000 celdas, sin grilla densa.
3. Cursor privado JSON estricto <=2KiB contiene Cursor compartido y sheet/range/header; ordinal es offset de celdas coincidentes. Comparar scope completo y no admitir cursor con ordinal0/fuera. Output incluye manifest completo, sheet descriptor, range, header_row, headers, matching_cell_count, items, covers_range y covers_generation (verdadero sólo si todos los items del libro están en esa única página). Payload<=2MiB conserva rechazo explícito para celdas grandes repetidas como encabezado, nunca truncamiento.
4. UI añade opción xlsx_range (extractor real xlsx_cells), inputs sin validación de dominio duplicada. Number pasa valores al Rust; strings A1 exactos, sin canonicalizar silenciosamente. Controlador retiene criterios completos para siguiente, invalida cambios y comprueba generación/extractor. Presentación sparse reutiliza celdas tipadas y expone headers completos, sin tabla visual engañosa.

## Risks / Trade-offs

- Repetir header largo consume respuesta → cap existente2MiB y error, sin truncar.
- Rango no detecta tabla semántica → elección explícita y límites visibles; PDF tables siguen pendientes.
- Cursor no firmado → validación completa scope/posición, no permiso.
- Lectura sigue validando todo libro → no eliminar verificaciones para acelerar; gates originales300/150ms intactos.

## Migration Plan

Sin DDL ni persistencia nueva: SQLite16, canónico XLSXv1, índices, snapshots y backup idénticos. Volver al cliente16 anterior conserva lectura de los mismos datos; mantener binarios previos independientes y comprobar antes/después con backup/restore. No downgrade ni replay necesarios. Nueva operación read-only no batchable; MCP28/48/49.
