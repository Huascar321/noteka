## ADDED Requirements

### Requirement: Checkbox de finalización de tareas

El shell SHALL mostrar un checkbox accesible por cada tarea de las listas de Home y Tasks. MUST permitir alternar `state` sin abrir la tarea, conservar la apertura en el resto de la fila y refrescar la vista. MUST deshabilitar el control mientras hay una operación en curso y anunciar los errores. MUST distinguir pendiente (cuadrado vacío con borde neutro) de completada (check verde) sin depender solo del color.

#### Scenario: Completar una tarea pendiente

- **WHEN** la persona activa el checkbox de una tarea pendiente
- **THEN** el estado pasa a `done`, el checkbox queda marcado y la vista se actualiza

#### Scenario: Reabrir una tarea completada

- **WHEN** la persona activa el checkbox de una tarea completada
- **THEN** el estado pasa a `todo` y la vista se actualiza

#### Scenario: Abrir sin togglear

- **WHEN** la persona activa el título de la tarea
- **THEN** se abre el detalle y el checkbox no cambia de estado

#### Scenario: Teclado y lector

- **WHEN** la persona llega al checkbox con teclado
- **THEN** el control expone `role="checkbox"`, `aria-checked` y una etiqueta que identifica la tarea y la acción
