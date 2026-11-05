## ADDED Requirements

### Requirement: Overview y Journal en Goals

El detalle Goal SHALL ofrecer un Goal Overview editable con Additional Details desplegable y un Journal desplegable con acción para añadir entradas fechadas, usando documentos locales y guardado explícito. MUST conservar bloques y metadatos previos, identidades, versiones, guardias y revisión esperada; abrir o navegar no MUST escribir ni convertir contenido ambiguo o importado.

#### Scenario: Nuevo Goal y diario
- **WHEN** se crea un Goal, se escribe en Overview y Additional Details y se agregan entradas al Journal
- **THEN** las secciones se guardan en el documento del Goal, se pueden volver a editar tras reabrir y los hitos y propiedades conservan su comportamiento

#### Scenario: Contenido existente e importado
- **WHEN** se abre un Goal existente con bloques anteriores o formato no reconocido
- **THEN** sus datos no se reescriben ni descartan al abrir; el contenido interpretable permanece accesible y el formato no reconocido conserva su recuperación existente

#### Scenario: Errores y cambios pendientes
- **WHEN** se navega o se intenta guardar con cambios de Overview o Journal, o una revisión externa entra en conflicto
- **THEN** se mantienen las guardias, la recuperación del borrador y el control de concurrencia existentes sin descartar secciones

### Requirement: Proyectos relacionados en Goal

Goal SHALL presentar los proyectos relacionados en una vista legible con título, estado y plazo disponibles, y acceso para abrir y vincular originales. MUST distinguir datos ausentes o no interpretables sin inventar valores, conservar relaciones y funcionar en Side peek y ventana estrecha.

#### Scenario: Proyectos reales
- **WHEN** un Goal tiene proyectos relacionados
- **THEN** la vista muestra sus nombres, estados y plazos disponibles y permite abrir cada original

#### Scenario: Lista vacía o panel estrecho
- **WHEN** no hay proyectos o el Goal se muestra en un panel angosto
- **THEN** se ofrece acceso a vincular proyectos guardados y las columnas permanecen accesibles

## MODIFIED Requirements

### Requirement: Detalle Goal con cabecera y hitos canónicos

Goal SHALL presentar Status/Tag/Progress compactos, fechas editables en View details, navegación protegida y sus hitos con Target Deadline y Date Completed. El progreso MUST proceder del resumen Rust existente con carga/error/reintento/ausencia explícitos, sin inventar cero cuando falta o no hay hitos. MUST conservar el editor de documento, todos sus bloques/prefijo, CRUD/CAS, acciones de hitos, proyectos originales y condiciones de PageTools, sin escribir al abrir. Overview, Additional Details y Journal SHALL usar bloques locales del mismo documento y conservar contenido anterior.

#### Scenario: Edición y navegación preservadas
- **WHEN** se abre un Goal y se edita estado, fechas o contenido, se abre un hito/proyecto o se prepara un hito vinculado
- **THEN** se usan los mismos borradores, identidades y Guardar/Descartar/Cancelar; el contenido original no se reemplaza por una plantilla visual y los hitos conservan apertura, actualización y continuación

#### Scenario: Resumen ausente o respuesta tardía
- **WHEN** falla el resumen, no incluye el Goal en su página acotada o responde tras cambiar de selección
- **THEN** se ofrece reintento/No disponible o se descarta esa respuesta, sin calcular otro progreso en el cliente ni convertir estados desconocidos
