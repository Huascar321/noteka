# Proposal

## Why

Excel local sigue pendiente en 5a. Antes de evolucionar SQLite y los contratos públicos necesitamos un lector determinista comprobado que conserve números y fórmulas sin depender de Excel, modelos ni red. El flujo actual sólo soporta UTF-8/PDF; no se dará por integrado XLSX por tener un parser.

## What Changes

- Añadir un lector/helper Rust descartable para un subconjunto explícito de XLSX Transitional, con hojas/celdas dispersas, cadenas, tipos, estilos numéricos y fórmulas/resultados guardados separados.
- Preservar lexemas numéricos, coordenadas, rangos declarados/observados y sistema de fechas sin conversión a floats, inferencia de encabezados, recálculo, macros ni conexiones externas.
- Validar contenedor ZIP/XML, presupuestos, respuesta privada y corpus sintético independiente antes de una futura integración.
- Reutilizar la contención existente de PDF mediante un módulo común, sin alterar sus límites ni protocolo; verificar regresión PDF real.

## Capabilities

### New Capabilities

Ninguna: el comportamiento pertenece a extracción local.

### Modified Capabilities

- `local-extraction`: lector XLSX aislado y acotado, todavía independiente de trabajos y persistencia.

## Impact

Nuevos crates privados de protocolo/lector y runtime común, Cargo.lock, corpus propio, harness y documentación. SQLite v10, API v1, 51 operaciones, MCP y Home permanecen sin ampliación funcional. No se activa el lector al importar, abrir, encolar o consultar.

## Non-goals

No integración XLSX con cola/SQLite/CLI de dominio/MCP/FTS/UI en este cambio; será el siguiente recorte sobre evidencia de este lector. No `.xls`, `.xlsb`, `.xlsm`, ODS, OOXML Strict, libros cifrados, macros, fórmulas data-table, evaluación de fórmulas, formato visual, inferencia de tablas/encabezados, edición, Google Sheets, modelos ni proveedores. No se modifica Home ni se elude su aceptación nativa pendiente.
