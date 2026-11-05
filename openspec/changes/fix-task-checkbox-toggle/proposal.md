# Proposal

## Why

Las marcas de estado de Tasks parecen checkboxes pero son `span` decorativos
dentro del botón que abre la tarea: no se pueden clickear ni cambiar el estado.
En la vista All conviven marcas verdes de completadas y azules de pendientes, lo
que aumenta la confusión sobre si son controles.

## What Changes

- Convertir la marca de estado de las filas de Home y Tasks en un checkbox real,
  accesible y separado del botón de apertura.
- Actualizar `state` con `entity.update` y refrescar Tasks/Projects después del
  cambio; deshabilitar mientras hay una operación en curso y mostrar errores.
- Distinguir pendiente (cuadrado vacío con borde neutro) de completada (check
  verde) sin depender solo del color.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: interacción y accesibilidad del checkbox de tareas.

## Impact

UI de Home/Tasks y una llamada existente a `entity.update`; sin migración, IPC
nuevo ni cambios de formato persistente. El detalle de proyecto queda fuera de
este recorte.
