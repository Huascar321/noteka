# Proposal

## Why

Al animar el lateral, ResizeObserver borra repetidamente el bitmap del océano ASCII y programa su repintado para el siguiente RAF. El usuario observa desaparición/parpadeo hasta que termina de cambiar el ancho.

## What Changes

- Repintar el canvas sin presentar un bitmap vacío al redimensionarlo.
- Conservar reloj/RAF activos durante el cambio de ancho y reutilizar el atlas cuando su tema/DPR/color no cambian.
- Comprobar píxeles antes/después en el flujo real de Home y mantener pausas/movimiento reducido.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

Ninguna. Corrección de la estabilidad temporal y redimensionado exigidos por desktop-shell; `skip_specs: true`.

## Impact

AsciiShark y su renderer local. Sin cambiar la transición del lateral, las trayectorias, datos, contratos, dependencias o ajustes persistentes. Meeting Notes permanece fuera de alcance.
