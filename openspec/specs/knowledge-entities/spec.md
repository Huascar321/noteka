# knowledge-entities Specification

## Purpose
Representar el conocimiento y los elementos básicos de productividad con identidades estables, contenido estructurado y relaciones que puedan consultar todos los clientes de Noteka.

## Requirements

### Requirement: Entidades mínimas con identidad estable

El sistema SHALL permitir crear, leer, listar y modificar entidades `note`, `task`, `project`, `tag` y `person` con ID estable, título no vacío, timestamps, revisión y estado de archivo.

#### Scenario: Creación y modificación

- **WHEN** se crea una entidad de cada tipo y se modifica su título con la revisión vigente
- **THEN** cada entidad conserva su ID, aumenta su revisión y expone su nuevo título

#### Scenario: Tipo o título inválido

- **WHEN** se solicita una entidad de tipo desconocido o con título vacío después de normalizar espacios
- **THEN** se devuelve `validation_error` sin crear ni modificar entidades

### Requirement: Clasificación de tags PARA

El sistema SHALL clasificar cada tag como `area`, `resource` o `topic`, sin duplicar su identidad al mostrarlo como área o recurso.

#### Scenario: Área y recurso

- **WHEN** se crean tags con clasificación `area` y `resource`
- **THEN** ambos se consultan como tags y por su clasificación conservando sus IDs

#### Scenario: Clasificación inválida

- **WHEN** se envía una clasificación no soportada
- **THEN** se devuelve `validation_error` sin persistir el valor

### Requirement: Estados básicos de productividad

El sistema SHALL admitir `todo` y `done` en tareas, y `planned`, `doing`, `done`, `ongoing`, `on_hold` en proyectos. La creación sin estado explícito SHALL usar `todo` para tareas y `planned` para proyectos.

#### Scenario: Completar tarea

- **WHEN** una tarea `todo` se actualiza a `done` con revisión vigente
- **THEN** se confirma el nuevo estado y se puede filtrar por él

#### Scenario: Estado desconocido

- **WHEN** se envía un estado ajeno al conjunto admitido para ese tipo de entidad
- **THEN** se devuelve `validation_error` y permanece el estado previo

### Requirement: Documento semántico mínimo de nota

El sistema SHALL persistir el documento de una nota como JSON versionado con párrafos identificables y texto, conservando orden, IDs y contenido en una lectura posterior.

#### Scenario: Documento con varios párrafos

- **WHEN** se guarda una nota con dos párrafos de IDs diferentes
- **THEN** al leerla se obtienen la misma versión, orden, IDs y texto

#### Scenario: Contenido inválido

- **WHEN** el documento contiene IDs de bloque duplicados o supera límites documentados de tamaño o profundidad
- **THEN** se devuelve `validation_error` y no se modifica la nota

### Requirement: Conservación de formatos no soportados

El sistema MUST rechazar modificaciones de documentos con versión o tipos de bloque no soportados sin convertirlos a texto ni descartar campos, y permitir recuperar el contenido almacenado mediante lectura estructurada.

#### Scenario: Bloque desconocido ya almacenado

- **WHEN** una fixture contiene un bloque no soportado y un cliente intenta editar su documento
- **THEN** se devuelve `unsupported_document_format` y la lectura estructurada devuelve el original intacto

### Requirement: Relaciones explícitas e íntegras

El sistema SHALL permitir vincular notas y tareas a proyectos, y entidades a tags y personas, validando extremos existentes del mismo workspace y los tipos compatibles de la relación.

#### Scenario: Relación de nota a proyecto

- **WHEN** se relaciona una nota con un proyecto existente
- **THEN** ambos se pueden consultar por la relación sin duplicar entidades

#### Scenario: Extremo inexistente o incompatible

- **WHEN** se solicita una relación a un ID inexistente, de otro workspace o de un tipo incompatible
- **THEN** se devuelve un error estructurado y no se almacena la relación

#### Scenario: Relación repetida

- **WHEN** se solicita una relación lógica que ya existe
- **THEN** se informa su existencia sin crear un segundo vínculo

### Requirement: Archivo reversible

El sistema SHALL archivar y restaurar entidades conservando IDs, documentos y relaciones; los listados activos SHALL excluir archivadas y una consulta explícita SHALL poder recuperarlas.

#### Scenario: Archivar y restaurar

- **WHEN** se archiva una nota relacionada a un proyecto y después se restaura
- **THEN** desaparece y reaparece en listados activos sin perder contenido ni relaciones

#### Scenario: Proyecto archivado con notas activas

- **WHEN** se archiva un proyecto que contiene notas activas
- **THEN** las notas conservan su estado y la relación sigue identificando al proyecto archivado

### Requirement: Listados estables y acotados

El sistema SHALL permitir listar entidades por tipo, estado, clasificación o archivo con paginación, orden documentado y desempate estable, sin devolver ilimitadamente todo el workspace.

#### Scenario: Páginas sucesivas

- **WHEN** se recorren las páginas de un listado sin escrituras concurrentes
- **THEN** cada entidad que cumple el filtro aparece una vez y el cliente sabe si hay otra página

### Requirement: Clasificación y procedencia opcionales de notas

El núcleo SHALL validar tipo y favorito opcionales de notas y procedencia explícita con proveedor, colección, ID y timestamps originales opcionales. MUST conservar documentos e IDs sin falsificar timestamps locales, admitir notas anteriores sin esos campos y rechazar valores inválidos sin escrituras parciales.

#### Scenario: Enriquecer con revisión vigente
- **WHEN** se añade metadata de fuente a una nota con revisión vigente
- **THEN** conserva su documento e identidad, registra la revisión normal y devuelve los campos con creación local intacta y actualización local normal

#### Scenario: Valor inválido o revisión vencida
- **WHEN** se envía tipo, procedencia o fecha inválida, o una revisión anterior
- **THEN** rechaza la operación y conserva la nota previa sin convertir su documento

### Requirement: Procedencia y orden opcionales de proyectos

ProjectFields SHALL aceptar procedencia explícita opcional y orden de tablero opcional entre 0 y 10000. MUST validar límites y timestamps en Rust, aceptar payloads anteriores y conservar campos no representados al editar. La procedencia archivada no equivale a archivar la entidad local.

#### Scenario: Compatibilidad y campos inválidos
- **WHEN** se recibe un payload antiguo sin procedencia/orden o uno con metadata inválida
- **THEN** el primero conserva su comportamiento y el segundo se rechaza sin modificar la entidad; no se cambia el esquema persistente

### Requirement: Entity y metadatos opcionales de tags

TagFields SHALL admitir clasificación Entity además de Area/Resource/Topic, favorito y procedencia opcionales y actividad de origen opcional RFC3339 acotada. MUST validar en Rust, aceptar payloads previos, conservar campos no representados al editar y mantener Topic independiente de Entity, sin migración SQL.

#### Scenario: Compatibilidad y validación
- **WHEN** se recibe un payload previo o uno con actividad/procedencia inválidas
- **THEN** el primero conserva su significado y el segundo se rechaza sin modificar datos; el filtro general de entidades admite Entity y el detalle preserva los metadatos enriquecidos

### Requirement: Metadatos opcionales de personas

PersonFields SHALL admitir relaciones textuales acotadas, empresa, foto local UUID con posición vertical entre 0 y 100, cumpleaños/check-in/último check-in civiles, pipeline Prospect/Contacted/Negotiating/Rejected/Closed, procedencia y orden de galería opcionales. MUST validar en Rust, aceptar payloads anteriores, rechazar valores inválidos sin escrituras y conservar propiedades no representadas, documentos e IDs sin migración SQL.

#### Scenario: Compatibilidad y validación
- **WHEN** se recibe un payload previo o metadata de persona inválida
- **THEN** el primero sigue siendo válido y la segunda se rechaza antes de persistir; el detalle permite editar los campos representados y conserva la procedencia

### Requirement: Goals y Milestones sobre páginas de identidad estable

El núcleo SHALL admitir extensiones opcionales y mutuamente excluyentes Goal y
Milestone en páginas note, conservando ID, documento, archivo y revisión. Goal
SHALL validar Active/Dream/Achieved, Dream por defecto, fechas civiles opcionales
de inicio, plazo y logro y snapshot de actividad acotado. Milestone SHALL validar
plazo, fecha completada y un Goal opcional del mismo workspace. MUST compartir
CRUD/CAS con CLI, MCP y escritorio sin migración SQL ni conversión de originales.

#### Scenario: Página anterior y captura nueva
- **WHEN** se enriquece una página importada o se crea un Goal/Milestone
- **THEN** se conserva el documento e identidad anteriores o se crea una página
  nueva normal, con validaciones Rust y campos representables sin guardar porcentajes

#### Scenario: Campos o vínculo inválidos
- **WHEN** se envía estado/fecha/snapshot inválido, ambas extensiones, un Goal
  inexistente/incompatible o se elimina un subtipo Goal aún referenciado por hitos
- **THEN** la transacción devuelve un error estructurado sin escritura parcial

### Requirement: Relaciones y recuperación de objetivos acreditadas

Goals SHALL reutilizar sus relaciones Note→Project y Note→Tag, limitando su tag
directo a uno y permitiendo un Goal por Milestone. La recuperación MUST usar IDs
originales inequívocos y metadata observada, backup, CAS e informe incremental
privado; conservar IDs, documentos, estados elegidos y datos desconocidos, y no
convertir la plantilla Goal en un objetivo real.

#### Scenario: Recuperación e identidad
- **WHEN** se recuperan Goals, hitos y relaciones desde metadata de origen
- **THEN** se enriquece sólo cada identidad acreditada y el progreso se deriva de
  hitos únicos; notas no vinculadas conservan su identidad y el template queda fuera

#### Scenario: Conflicto y archivo
- **WHEN** cambia una revisión durante la recuperación o se archiva un objetivo
- **THEN** el conflicto rechaza esa escritura sin forzarla y el archivo conserva
  los vínculos/documentos para lectura o restauración posterior


### Requirement: Documento opcional de tarea con identidad estable

El núcleo SHALL admitir un documento enriquecido opcional en Task mediante create/update compartidos, con el mismo ID, revisión y relaciones. Una Task anterior sin documento SHALL seguir sin contenido persistido al abrirla o modificar sólo propiedades; document_editable SHALL ser true con cuerpo ausente y false ante un formato almacenado no editable. MUST reutilizar validación de versiones/bloques/referencias/límites, CAS, auditoría, índice textual, exportación y backup existentes. Las otras entidades conservan su contrato y la ausencia de documento en una actualización no elimina contenido.

#### Scenario: Primer contenido y reapertura
- **WHEN** se guarda el primer documento de una Task con revisión vigente
- **THEN** se confirma un único cambio atómico en esa Task con sus bloques/IDs originales, y las lecturas compartidas, búsqueda textual, exportación y backup acceden a ese cuerpo sin crear una Note asociada

#### Scenario: Task anterior y propiedades
- **WHEN** se abre una Task sin cuerpo o se actualiza su título/estado omitiendo documento
- **THEN** puede comenzar contenido pero sigue sin fila documental nueva hasta que el usuario guarda su cuerpo; no se cambian relaciones ni fechas omitidas

#### Scenario: Conflicto o formato desconocido
- **WHEN** se envía una revisión vencida, documento inválido o se intenta reemplazar un cuerpo almacenado con formato desconocido
- **THEN** se rechaza sin escritura parcial, el original permanece recuperable y no se normalizan ni descartan propiedades ajenas
