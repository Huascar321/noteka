## Why

El usuario pidió arrastrar y reordenar tarjetas en Projects Board, dentro de columnas y entre estados. El tablero ya tiene estado y board_order persistibles, pero carece de interacción de movimiento. La operación debe preservar los documentos y rechazar conflictos sin escrituras parciales.

## What Changes

- Arrastre nativo de tarjetas en Board, indicadores de destino y movimiento entre estados o dentro de una columna.
- Alternativa accesible mediante botón Mover y selección de estado/posición, con confirmación propia de esa interacción.
- Persistencia mediante lotes existentes de actualización de entidades, preview/apply atómico, revisiones y comprobación del snapshot del tablero. No se añade una API ni migración.
- Board carga hasta cien proyectos por página, dentro del límite actual; mover requiere una página completa y metadata interpretable. No se reordenan registros ocultos.

## Capabilities

### New Capabilities

### Modified Capabilities

- `desktop-shell`: movimiento de tarjetas con ratón/teclado, persistencia, feedback y conflictos.

## Impact

Componentes de Projects y un adaptador de orden/persistencia usando changes.preview/changes.apply existentes. Rust conserva validación, CAS y transacción. Se conserva metadata, iconos y documentos; un cambio de columna modifica estado. No es un canvas libre ni afecta relaciones de tareas.
