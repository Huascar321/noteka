# Proposal

## Why

El usuario pide que Search deje ver el contenido detrás con un vidrio gris transparente como la cápsula de referencia. El blur de 24 px y los velos blancos actuales borran ese contenido y dan aspecto de panel blanco.

## What Changes

- Dar a Search una variante de material neutro con transmisión más clara, desenfoque corto, reflejos y borde luminoso.
- Evitar velos blancos acumulados en resultados, preview, filtros y pie; conservar tinta legible sobre fondos variables.
- Conservar geometría, búsqueda local, navegación, foco, transición compartida y alternativas sólidas accesibles.
- Actualizar el design system y registrar capturas reales antes/después en ambos temas.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: variante de vidrio transparente de Search con material propio, legibilidad y fallbacks.

## Impact

CSS y documentación. Sin cambios Rust, SQLite, operaciones, dependencias, Settings ni transparencia de ventana al escritorio. El CSS aproxima el material de referencia; no implementa refracción nativa.
