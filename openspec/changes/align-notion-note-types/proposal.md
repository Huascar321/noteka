# Proposal

## Why

La referencia distingue Daily y Journal y permite otros tipos explícitos que Noteka no puede guardar. Daily debe aparecer en Journal y quedar fuera de Notes generales.

## What Changes

- Admitir Daily, Book, Recipe, Outlier Content, Creator Tips y Tools como tipos separados y editables.
- Presentar Daily junto a Journal en una proyección nueva; mantener la consulta anterior `notes_journal`.
- Recuperar los tipos explícitos de filas inequívocas sin reemplazar una clasificación local.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `knowledge-entities`: tipos adicionales y recuperación de origen.
- `productivity-workflows`: proyección conjunta Daily/Journal.
- `desktop-shell`: selección y vistas de los tipos.

## Impact

Contratos, Rust, UI y recuperador; sin migración ni cambios automáticos de datos. La regla de URL sigue pendiente.
