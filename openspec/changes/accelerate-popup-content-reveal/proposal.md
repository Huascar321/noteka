# Proposal

## Why

El usuario observa demasiado tiempo contenido borroso antes de poder leer un popup, con el selector de proyectos como referencia. Los menús mantienen una capa de vidrio en top layer sobre su texto durante 400 ms; adelantar sólo la opacidad no elimina ese blur.

## What Changes

- Acortar a 160 ms la entrada de menús/pickers compartidos, retirando antes la capa temporal que cubre el contenido.
- Revelar opacidad/contenido en 100 ms sin espera inicial, incluyendo diálogos y Due; conservar material de diálogos/Due a 400 ms y cierres a 200 ms.
- Mantener vidrio en reposo, curvas, geometría, foco, interrupciones, buffers y preferencias; actualizar el sistema de diseño y conservar capturas reales.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: claridad temprana de popups compartidos.

## Impact

Presentación y pruebas focales. Sin Rust, datos, dependencias, cambios de red, commit o publicación. Conserva el trabajo local previo de rendimiento.
