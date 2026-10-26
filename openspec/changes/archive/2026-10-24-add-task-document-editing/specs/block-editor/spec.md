# Spec Delta

## ADDED Requirements

### Requirement: Inicio visible del contenido de una Task

El editor SHALL ofrecer un cuerpo de Task con nombre accesible propio, indicación visible de dónde escribir, bloques por `/`, undo/redo y foco desde Enter en el título. Para un cuerpo vacío SHALL ofrecer Página vacía, Lista de pasos y Notas de tarea como estructuras locales. MUST aplicar una estructura sólo por selección explícita y mientras el cuerpo siga vacío, asignar identidades válidas a sus bloques y conservar el guardado explícito y las guardias del buffer existente.

#### Scenario: Empezar una Task vacía
- **WHEN** se abre una Task sin cuerpo y se escribe en su área de contenido o se usa Enter desde el título
- **THEN** el mismo editor recibe foco y permite escribir bloques; sólo una edición real o elección de estructura marca el cuerpo pendiente, sin guardar al abrir o enfocar

#### Scenario: Estructura y contenido previo
- **WHEN** se elige Lista de pasos o Notas de tarea con cuerpo vacío
- **THEN** se prepara contenido semántico local en el borrador actual, con IDs propios y undo/redo; con contenido existente esas opciones no reemplazan bloques

#### Scenario: Salida o fallo de guardado
- **WHEN** el cuerpo contiene cambios y se cierra, navega, cancela o falla guardar
- **THEN** se aplican Guardar/Descartar/Cancelar y los errores existentes sin borrar el buffer ni afirmar persistencia, manteniendo Notes y sus documentos importados intactos
