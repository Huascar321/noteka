# Proposal

## Why

El usuario pide vidrio perceptible en Settings y señala que Search sigue pareciendo opaco y su campo no coincide con Search lateral. Settings conserva fondos sólidos; el buscador acumula tinte y gradientes blancos, y su campo tiene tipografía y padding mayores.

## What Changes

- Settings adopta un marco translúcido con borde/reflejos compartidos, navegación de vidrio y contenido con velo más denso para lectura.
- Search reduce capas opacas para mostrar el contenido real de la aplicación detrás, con un solo blur local.
- Los campos Search de ambos popups usan cápsulas compactas de 32 px, tipografía del acceso lateral y placeholder Search, manteniendo nombres accesibles específicos.
- Conservar búsqueda, filtros, foco, navegación, tema, color, scroll y cambios de workspace; fallbacks sólidos accesibles y verificación sobre fondo compuesto.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: material de Settings y búsqueda, coherencia de sus campos y transparencia perceptible sin pérdida de legibilidad.

## Impact

CSS, SettingsDialog/SearchDialog y documentación. Sin operaciones, datos, dependencias ni cambios de ventana nativa. El material permite ver contenido de la app, sin refracción nativa ni transparencia al escritorio. El pedido posterior del usuario autoriza el commit local conjunto, sin push, sincronización o archivo automático.
