# Proposal

## Why

El usuario quiere probar un acabado cercano a los controles Liquid Glass de su captura de macOS, conservando el estilo y la interacción del menú lateral. El estado previo está preservado en el commit local `affca8a`.

## What Changes

- Dar a la cápsula activa del sidebar un tinte gris translúcido, borde de luz direccional, reflejos interiores y sombra corta.
- Conservar filas inactivas, dimensiones, iconos, Jelly y desplazamiento del indicador; mantener foco y alternativas sólidas accesibles.
- Documentar el experimento y verificarlo en claro/oscuro y en la ventana nativa.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: acabado de la selección lateral y su fallback accesible.

## Impact

`src/style.css`, sistema de diseño y documentación. Sin cambios Rust, datos, contratos, dependencias ni configuración de ventana. No se transparenta la ventana ni se promete refracción nativa; la prueba queda local y reversible.
