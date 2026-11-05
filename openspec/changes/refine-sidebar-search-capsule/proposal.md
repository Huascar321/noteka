# Proposal

## Why

El usuario pidió que el acceso Search del sidebar adopte la cápsula clara de su referencia: borde fino, gris translúcido y luz interior, con menos relieve que la selección lateral.

## What Changes

- Convertir visualmente Search en una cápsula con material de campo neutro, borde fino y reflejos interiores discretos.
- Conservar la apertura del buscador, icono, atajo, tamaño, estados y funcionamiento en ambos temas y con preferencias accesibles.
- Actualizar el sistema de diseño y verificar el flujo focal. Las demás mejoras de vidrio se presentan sólo como recomendaciones.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: presentación del acceso lateral a búsqueda.

## Impact

CSS y documentación. Sin cambios de operaciones, datos, Rust, dependencias ni ventana nativa. El acceso sigue siendo un botón que abre el buscador; no incorpora escritura directamente en el sidebar.
