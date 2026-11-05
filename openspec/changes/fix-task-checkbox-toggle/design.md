# Design

## Context

`HomeItem` mostraba la marca de estado como un `span aria-hidden` dentro del
botón `.home-open`. El clic abría la tarea y el estado solo se podía cambiar
desde el detalle. La proyección All reutiliza la misma fila para tareas
pendientes y completadas.

## Decisions

Separar el checkbox del botón de apertura: un botón `role="checkbox"` con
`aria-checked` y etiqueta, dentro de `.home-task-main`, hermano del botón
`.home-open`. El checkbox mide 24 px de área clickeable y conserva la marca
visual de 14 px. `HomeTaskSection` envía el parche `{state}` a `entity.update`;
Rust fusiona el payload, valida y conserva el resto de los campos. Después,
`onchanged` refresca Tasks y Projects; el control se deshabilita durante la
operación y los errores se anuncian. El pendiente es un cuadrado vacío con borde
neutro y el completado un check verde, de modo que la diferencia no dependa solo
del color. El detalle de proyecto conserva su marca de lectura anterior. No se
anida un control interactivo dentro de otro.

## Risks / Trade-offs

El checkbox agrega un control por fila y desplaza levemente el título; el área
clickeable es mayor que la marca. El detalle de proyecto usa otra fila y no se
incluye en este recorte.
