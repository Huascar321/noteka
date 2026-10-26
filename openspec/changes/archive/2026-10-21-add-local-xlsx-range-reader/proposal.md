# Proposal

## Why

El lector XLSX conserva todas las celdas y hojas, pero obliga a recorrer el libro completo para consultar una región. La fase 5a requiere hojas, rangos y encabezados explícitos antes de habilitar recuperación semántica.

## What Changes

- Añadir `extraction.xlsx_range`, lectura pura de una hoja y rango A1 canónico, con fila de encabezados opcional elegida por el usuario. No inferir títulos, expandir merges, recalcular fórmulas ni crear una segunda autoridad.
- Paginar celdas dispersas completas con ordinal global conservado, manifiesto íntegro, cobertura del rango y encabezados por columna que distinguen celda ausente de valor ausente.
- Compartir la operación entre núcleo, CLI, MCP y lector de escritorio, con invalidación de criterios/cursores y protección de buffers.
- Verificar integridad del libro y original completos incluso fuera del rango, compatibilidad contractual, headless y gates sin modificar sus límites.

## Non-goals

No extracción nueva, migración, configuración persistida de tablas, indexación por rangos, inferencia automática, nuevas funciones Home, PDF tables, modelos ni fases 5b/5c/6. Este recorte no completa toda 5a.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `local-extraction`: lectura XLSX acotada por hoja/rango/encabezado desde representación confirmada.
- `core-operations`: contrato aditivo API v1 y CLI, conservando las 55 operaciones previas y batch9.
- `local-mcp`: lectura equivalente con permisos actuales; catálogo 28/48/49.
- `desktop-shell`: criterios explícitos en lector existente y presentación fiel de cobertura/celdas/encabezados.

## Impact

Rust: contratos y proyección en memoria sobre loader XLSX vigente; catálogo/scoped/CLI/bindings. UI: sólo panel/controlador de lectura existente. SQLite16 y formatos canónicos/FTS no cambian; clientes16 previos siguen leyendo el mismo workspace. Baseline externo anterior de625 fuentes y clientes independientes16 disponible. Home9/17 y lectores nativos preservados. Evidencia nueva fuera de Git; límite de esta continuación20:11:26UTC del21/10/2026.
