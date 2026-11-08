# Proposal

## Why

El usuario acepta la transición Due acelerada y pide el mismo comportamiento para los demás menús y popups de Noteka. Las superficies actuales usan montajes, cierres y animaciones independientes que no expresan esa continuidad.

## What Changes

- Compartir expansión elástica de 400 ms y repliegue de 200 ms entre menús, pickers, herramientas, Search, Settings, confirmaciones y Center peek.
- Conectar al control o selección que origina cada panel; usar expansión central contenida cuando no hay ancla visible.
- Aplicar el movimiento a Nav y opciones de vista, conservando su condición de contenido en el flujo cuando corresponda.
- Mantener foco, teclado, posición, borradores, selección y operaciones. Reducir movimiento elimina transiciones; contraste/transparencia mantienen sus alternativas.
- Revelar contenido real desde el inicio en diálogos y Center peek, con vidrio detrás del texto y recorte progresivo, para evitar una fase vacía seguida de aparición repentina.
- Excluir controles nativos del sistema, tooltips de ayuda no interactivos, disclosures documentales y navegación Side/Full peek, que conservan sus patrones propios.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: movimiento coherente y accesible de superficies interactivas transitorias.

## Impact

Interfaz Svelte, utilidades de movimiento, CSS y verificación. Sin dependencias de producto, Rust, contratos ni migración. Due conserva la receta aceptada mediante funciones compartidas. No incluye commit, publicación, sync o archivo.
