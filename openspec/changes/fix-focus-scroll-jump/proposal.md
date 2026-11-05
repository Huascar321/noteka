# Proposal

## Why

Al volver a Noteka, los mensajes transitorios de refresco insertan altura en las secciones y desplazan el scroll.

## What Changes

- Mostrar el refresco de datos existentes sin añadir filas transitorias al contenido.
- Restaurar el foco sin desplazar la vista y conservar estados de error, datos anteriores y paginación.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: estabilidad visual durante el refresco por foco.

## Impact

Presentación y restauración de foco; sin cambios de operaciones ni datos.
