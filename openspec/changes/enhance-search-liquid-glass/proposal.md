# Proposal

## Why

La búsqueda ya usa un vidrio neutro, pero el panel se lee como una superficie plana: el tinte es casi opaco y no tiene el brillo, el borde luminoso ni la profundidad que el usuario pidió para el menú contextual.

## What Changes

- Dar al diálogo de búsqueda un material de vidrio líquido: tinte más translúcido, blur local más amplio con saturación, velo radial y sheen compartido.
- Reforzar el borde, el reflejo superior y la sombra interior; aplicar el mismo material al campo de búsqueda, la fila activa y el pie.
- Mantener el fallback sólido con transparencia reducida o contraste aumentado y conservar la legibilidad.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: material del diálogo de búsqueda.

## Impact

CSS, documentación y verificación visual; sin cambios de datos, contratos, dependencias ni ventana nativa.
