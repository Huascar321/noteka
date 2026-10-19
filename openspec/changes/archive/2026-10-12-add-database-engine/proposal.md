# Proposal

## Why

La productividad usa tipos fijos y no permite construir pequeños sistemas nuevos. La fase 3a añade colecciones con propiedades y vistas sobre las mismas entidades, antes de fórmulas o agentes.

## What Changes

- Colecciones y propiedades tipadas: texto, número, decimal/moneda exacta, checkbox, fecha/rango, select/multiselect, URL/email, archivos, relación y timestamps; título compartido con entidad.
- Registros son membresías de entidades existentes, no una segunda identidad; se crean páginas con bloques o se agregan entidades built-in sin copiarlas.
- Relaciones de propiedad con destino validado e inversa consultable, una sola representación lógica; conservar referencias archivadas.
- Vistas tabla/lista persistentes con columnas, filtros básicos y orden/paginación; escritorio permite operar el motor compartido.
- SQLite v4 con backup y migración no destructiva. Tipos de propiedad inmutables en esta entrega: cambiar tipo/eliminar datos exige futuro plan de migración explícito.
- Sin fórmulas/rollups, tablero/calendario/galería, bases incrustadas, templates, lotes de agente ni IA: pertenecen a 3b/3c/4.

## Capabilities

### New Capabilities
- `database-engine`: colecciones, propiedades, valores, registros compartidos y relaciones genéricas.
- `collection-views`: consultas/vistas tabla y lista persistentes con apertura del original.

### Modified Capabilities
Ninguna: built-ins y operaciones previas conservan IDs y reglas; pertenecer a una colección no reemplaza su payload.

## Impact

Nuevas operaciones Rust/CLI/Tauri, migración 004 y controles de UI. Las elecciones de representación, límites y filtros se documentan en diseño/ADR. No requiere cuentas, red, credenciales ni modelos. No se promete paridad total con bases de Notion.
