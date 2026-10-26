# Proposal

## Why

El usuario quiere probar tiburones, koi y medusas juntos, con tamaños, contraste y profundidad variados, y retirar el scrollbar horizontal del menú. El estado anterior de tres escenas alternadas quedó preservado en el commit local `2d43a66` antes de este experimento.

## What Changes

- Una composición ASCII continua con las tres familias presentes, ordenadas por profundidad entre especies y con escalas/opacidades distintas.
- Conservar las siluetas y nado existentes, el reloj activo, las pausas, Photo/ASCII y el repintado durante resize; retirar la alternancia de 42s de la presentación activa.
- Corregir el overflow horizontal del lateral en su origen sin perder el borde de resize, foco ni scroll vertical.
- Trabajo local en `codex/combined-ascii-ocean`; checkpoint previo recuperable. Sin publicación.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: composición mixta sustituye las variantes alternadas del cambio previo `add-random-ascii-cover-scenes`; profundidad compartida y lateral sin desborde.

## Impact

Compositor/poses Canvas, APIs de dibujo por actor, estilos del lateral, pruebas y documentación. No cambia Rust, SQLite, contratos, dependencias o preferencias. La referencia koi/Kurage y su atribución siguen siendo las ya revisadas. El checkpoint conserva todos los avances locales previos; no representa una nueva validación de la totalidad de aquel código.
