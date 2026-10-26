# Proposal

## Why

La captura del usuario muestra controles superiores con centros verticales distintos. El lateral desaparece al contraerlo sin transición; el usuario pidió alineación y movimiento suave en ambos sentidos.

## What Changes

- Centrar lateral, pestaña, cierre y nueva pestaña en la misma fila y ajustar la posición configurada de los controles Mac.
- Animar expansión/contracción del lateral y del contenido disponible, también al cerrar el panel estrecho.
- Conservar accesibilidad, movimiento reducido e instancias/buffers de pestañas.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: alineación de la barra integrada y transición bidireccional del lateral plegable.

## Impact

CSS de presentación, fondo del lateral en App y configuración macOS. Sin dependencias, operaciones Rust, persistencia, migraciones o datos personales. La revisión se realiza en navegador headless; no acredita controles ni arrastre nativos. Meeting Notes sigue fuera de alcance.
