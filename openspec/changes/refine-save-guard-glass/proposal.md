# Proposal

## Why

El usuario pide aplicar Liquid Glass al diálogo de cambios pendientes y al Save de cabecera. El diálogo tiene una base translúcida previa, pero sus botones y las acciones de guardado conservan superficies planas distintas al material aprobado.

## What Changes

- Diálogo de cambios pendientes con marco y material compartidos con los popups aceptados.
- Cancel/Discard con cápsulas neutras; Save and continue y Save/Retry de cabecera con el vidrio teñido de New.
- Mantener etiquetas, orden, atajo, estado de guardado, deshabilitados, foco y operaciones existentes.
- Verificar los tres caminos de la guardia, guardado fallido/reintento, ambos temas, acentos, ancho estrecho y alternativas accesibles.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: coherencia visual accesible del diálogo y acciones de guardado.

## Impact

CSS, clases de WorkspacePane y documentación. Sin nuevas operaciones, reglas de guardado, formatos, dependencias o acceso remoto. El pedido posterior del usuario autoriza el commit local conjunto, sin push, sincronización o archivo automático.
