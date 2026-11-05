# Proposal

## Why

El export usa `Voice Note`, pero la recuperación sólo reconocía `voice`; una nota de voz importada podía quedar sin categoría.

## What Changes

- Reconocer explícitamente `Voice Note` como `voice` al recuperar metadata de una fila inequívoca.
- Conservar los tipos ya guardados, dry-run, backup y CAS. No modificar datos existentes automáticamente.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `knowledge-entities`: recuperación de tipo explícito del export.

## Impact

Script de recuperación y verificación sintética. Sin migración, red ni cambios automáticos al workspace.
