# Spec Delta

## MODIFIED Requirements

### Requirement: Listados en el área principal

Las secciones de entidades SHALL mostrar su listado en el área principal. Abrir una entidad o crear un borrador MUST usar el modo de apertura elegido, inicialmente Side peek derecho, conservando la vista de origen durante el peek. El detalle MUST omitir un listado propio; Full page y New tab ocupan el ancho disponible.

#### Scenario: Sección y apertura de un concepto
- **WHEN** se abre Tasks, Notes, Projects, Tags o People y se elige un registro
- **THEN** el listado inicial ocupa el área principal y el original se abre según el modo predeterminado; Side peek conserva el contexto a la izquierda

#### Scenario: Apertura lenta de un detalle
- **WHEN** se abre un registro y la lectura del original sigue en curso
- **THEN** se muestra la carga del detalle sin montar transitoriamente un listado propio ni sustituir el contexto de origen

## ADDED Requirements

### Requirement: Presentación configurable de detalles

Cada detalle SHALL ofrecer Switch peek mode con Side peek derecho, Center peek, Full page y New tab. Cambiar presentación MUST conservar buffer, revisión, editor e historial sin guardar ni descartar. Cerrar o reemplazar un peek SHALL resolver la guardia existente; los cierres agregados MUST incluir peeks de pestañas inactivas.

#### Scenario: Cambiar de presentación con edición pendiente
- **WHEN** se edita Note o Task y se alterna entre Side peek, Center peek, Full page y New tab
- **THEN** el mismo contenido pendiente e historial permanecen, la revisión persistida no cambia y New tab crea una pestaña de sesión con ese mismo editor

#### Scenario: Entidades y borradores
- **WHEN** se abre o crea Note, Task, Project, Tag, Person, Goal o Milestone
- **THEN** se presenta su editor real en el modo elegido y se conservan contexto y semillas de captura sin writes implícitos

#### Scenario: Cancelación o guardado fallido
- **WHEN** se cierra o reemplaza un peek pendiente, o se cierra su pestaña/workspace/host, y se cancela o falla guardar
- **THEN** el detalle conserva su buffer, el cierre no se confirma y los demás paneles siguen montados

#### Scenario: Contexto y foco
- **WHEN** se usa Side peek, Center peek o un viewport estrecho en cualquier tema
- **THEN** el detalle tiene scroll propio y controles accesibles, Center peek contiene el foco y atenua el contexto, Escape permite cierre protegido y cerrar devuelve el foco al origen; no hay overflow horizontal global

### Requirement: Modo de apertura predeterminado local

El selector SHALL permitir elegir un modo predeterminado global para nuevas aperturas. La preferencia MUST persistir localmente con validación, inicialmente Side peek. Cambiarla no altera la presentación actual ni documentos; un valor inválido o almacenamiento inaccesible MUST permitir continuar, con fallback seguro y fallo recuperable de escritura.

#### Scenario: Preferencia persistida
- **WHEN** se elige Center peek como predeterminado y se reabre la aplicación
- **THEN** las nuevas aperturas usan Center peek y los documentos conservan revisiones y contenido

#### Scenario: Preferencia no disponible
- **WHEN** el valor guardado es desconocido o falla el acceso al almacenamiento
- **THEN** se usa Side peek como fallback, se informa un fallo de escritura si corresponde y se puede seguir editando
