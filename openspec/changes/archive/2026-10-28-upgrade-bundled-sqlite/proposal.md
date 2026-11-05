# Proposal

## Why

La versión incluida de SQLite 3.51.1 tiene un error infrecuente de WAL corregido por SQLite. Antes de optimizar los refrescos se actualiza el motor y se mide una línea de base reproducible, sin asumir que haya corrupción o saturación.

## What Changes

- Actualizar rusqlite a 0.41.0 y fijar libsqlite3-sys 0.39.1, con SQLite 3.54.1 incluido y las funciones de backup y hooks existentes.
- Medir consultas sintéticas de Home antes y después, y verificar integridad, formatos, recuperación y clientes desktop/CLI/MCP.
- Conservar el esquema 20, las operaciones públicas, la comprobación de integridad y la operación offline.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

Ninguna: no cambia el contrato observable de los workspaces; el cambio sustituye el motor incluido y preserva los requisitos existentes de `local-workspace` y `core-operations`.

## Impact

Dependencias Rust, lockfile, benchmark y pruebas con bases sintéticas. No modifica workspaces personales ni implementa el coordinador de refrescos, que es un cambio posterior.
