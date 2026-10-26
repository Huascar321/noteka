# Spec Delta

## ADDED Requirements

### Requirement: Detalle Goal con cabecera y hitos canónicos

Goal SHALL presentar Status/Tag/Progress compactos, fechas editables en View details, navegación protegida y sus hitos con Target Deadline y Date Completed. El progreso MUST proceder del resumen Rust existente con carga/error/reintento/ausencia explícitos, sin inventar cero cuando falta o no hay hitos. MUST conservar el mismo editor de documento, todos sus bloques/prefijo, CRUD/CAS, acciones de hitos, proyectos originales y condiciones de PageTools; no ampliar el comportamiento del cuerpo ni escribir al abrir.

#### Scenario: Edición y navegación preservadas
- **WHEN** se abre un Goal y se edita estado, fechas o contenido, se abre un hito/proyecto o se prepara un hito vinculado
- **THEN** se usan los mismos borradores, identidades y Guardar/Descartar/Cancelar; el contenido original no se reemplaza por una plantilla visual y los hitos conservan apertura, actualización y continuación

#### Scenario: Resumen ausente o respuesta tardía
- **WHEN** falla el resumen, no incluye el Goal en su página acotada o responde tras cambiar de selección
- **THEN** se ofrece reintento/No disponible o se descarta esa respuesta, sin calcular otro progreso en el cliente ni convertir estados desconocidos

### Requirement: Detalle Person con portada y contexto operativo

Person SHALL usar su foto local acotada como portada cuando está disponible y conservar icono/título, campos editables en View details, relaciones y herramientas existentes. MUST presentar notas, proyectos y tareas del contexto local con identidades, cursor común, captura Note/Task y guardias existentes. Las ausencias, errores y cobertura parcial SHALL ser explícitos, sin copiar contadores, cuerpo libre, campos ni relaciones remotos no soportados. Temas, 420 px y teclado MUST conservar controles accesibles sin overflow global.

#### Scenario: Propiedades y acciones propias
- **WHEN** se editan campos de Person, se abre un relacionado o se prepara Note/Task vinculada
- **THEN** se conservan sus operaciones y guardias, la captura mantiene el ContextLink y no se agrega un documento/editor a Person ni se inician operaciones remotas al abrir paneles

#### Scenario: Foto o contexto no disponibles
- **WHEN** falta/falla la foto, falla el contexto o se continúa una página con entidades repetidas
- **THEN** la portada usa un espacio neutro, el contexto permite reintentar o acumula IDs únicos y expresa su alcance cargado, sin ocultar acciones ni afirmar ausencia total durante una lectura parcial
