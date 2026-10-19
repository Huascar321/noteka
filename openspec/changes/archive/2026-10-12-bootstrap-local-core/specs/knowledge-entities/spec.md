# Spec Delta

## Purpose

Representar el conocimiento y los elementos básicos de productividad con identidades estables, contenido estructurado y relaciones que puedan consultar todos los clientes de Noteka.

## ADDED Requirements

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
