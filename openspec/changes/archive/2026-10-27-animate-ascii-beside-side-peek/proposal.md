# Proposal

## Why

Home sigue visible al lado de un Side peek, pero su animación se pausa al perder el foco de edición.

## What Changes

- Mantener la portada ASCII animada mientras Home y Side peek están visibles simultáneamente.
- Conservar las pausas para pestañas ocultas, portada fuera de vista, ventana oculta y movimiento reducido, además de la separación de foco y atajos.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: animación visible junto a Side peek.

## Impact

Estado de presentación Svelte y documentación; sin modificar geometría, datos o contratos.
