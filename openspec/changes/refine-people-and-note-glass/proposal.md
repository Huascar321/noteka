# Proposal

## Why

El usuario aprobó continuar el acabado Liquid Glass en tarjetas de People y controles del detalle de nota. Los bordes rectos de relaciones/propiedades y las acciones superiores separadas todavía presentan otra jerarquía visual.

## What Changes

- Tarjetas de People con radio de 16 px, borde iluminado y sombra corta, base sólida y reflejo discreto en hover/foco; fotos y colores de relaciones conservados.
- Cápsulas compactas para relaciones vinculadas, con apertura y eliminación separadas; Add project/person como acción secundaria con vidrio en hover/foco.
- Tag y Type de notas con material de campo tenue; URL conserva presentación de lectura y Favorite el checkbox real.
- Agrupar Locked, Refresh, estado de guardado y menú en una cápsula superior para notas, conservando operaciones y guardias.
- Alternativas sólidas, foco y contraste en ambos temas/acento, ancho estrecho y movimiento reducido.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: coherencia visual y agrupación accesible de tarjetas People, relaciones y controles de notas.

## Impact

Svelte/CSS y sistema de diseño. Sin nuevas operaciones, datos, dependencias, migraciones o sincronización. Títulos, contenido y bloques de código mantienen su nivel sólido. Se comienza por tarjetas/relaciones y se continúa con propiedades/barra. El pedido posterior del usuario autoriza el commit local conjunto, sin push, sync ni archivo automático; no hay dependencias externas pendientes.
