# Proposal

## Why

El texto negro calculado por la paleta de acento no coincide con la presentación
que pide el usuario para los botones azules New y Save. Solicita texto blanco.

## What Changes

- Texto blanco compartido por los botones primarios, incluidos sus atajos.
- Mantener fondos, hover, estados deshabilitados y eventos existentes.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: texto blanco de botones primarios.

## Impact

CSS de botones compartidos y documentación de presentación. Sin cambios de
dominio, datos, dependencias o algoritmo de color de iconos/enlaces. Verificación
por build y revisión headless del CSS de producción en ambos temas.
