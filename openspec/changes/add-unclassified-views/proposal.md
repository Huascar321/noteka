# Proposal

## Why

El usuario elige conservar Inbox manual y sumar una vista automática para localizar elementos sin clasificar. Content no está modelado, por lo que no puede comprobarse su ausencia.

## What Changes

- Agregar vistas separadas de Tasks pendientes sin Project y Notes de biblioteca sin Tag, Project ni People.
- Mantener Inbox y sus marcas intactos; explicar la limitación de Content.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `productivity-workflows`: consultas aditivas de elementos sin vínculos locales.
- `desktop-shell`: acceso independiente y explicación visible en Home y páginas.

## Impact

Rust, contratos y UI. Sin migración, lectura remota o cambio automático de datos.
