# google-drive Specification

## Purpose
Conectar Google Drive con consentimiento explícito, seleccionar archivos con Picker, asociarlos a entidades y previsualizar Docs/Sheets sin scopes amplios ni persistir contenido remoto.

## Requirements

### Requirement: OAuth Google de escritorio explícito

El sistema SHALL autenticar Google con Authorization Code + PKCE, callback loopback y scope mínimo `drive.file`. MUST validar state, guardar tokens en almacén seguro y MUST NOT imprimir credenciales. El login SHALL ser explícito; abrir/buscar no SHALL iniciarlo.

La sesión MUST refrescarse antes de expirar y logout SHALL borrarla. Sin cliente OAuth configurado o consentimiento, MUST devolver un error claro sin enviar contenido.

#### Scenario: Login Google
- **WHEN** el usuario configura client_id y completa el consentimiento
- **THEN** se guarda una sesión con identidad/scopes y `google files` puede listar metadatos

#### Scenario: Falta configuración o consentimiento
- **WHEN** falta client_id/secret o el usuario rechaza
- **THEN** no se guarda sesión ni se llama a Drive

### Requirement: Selección acotada con Google Picker

El sistema SHALL ofrecer una selección explícita de archivos mediante Google Picker usando sólo el scope `drive.file`, sin solicitar scopes restringidos ni enumerar todo el Drive. MUST guardar los `picked_file_ids` en una sesión separada de Keychain y MUST consultar únicamente metadatos acotados de esos IDs. La cancelación o un error MUST NOT conservar una selección parcial.

#### Scenario: Selección de archivos
- **WHEN** el usuario ejecuta `google pick` y selecciona archivos en Google Picker
- **THEN** se guardan los IDs elegidos y `google selected` devuelve sus metadatos acotados

#### Scenario: Cancelación
- **WHEN** el usuario cancela el Picker o rechaza el consentimiento
- **THEN** no se guarda una nueva selección y se informa el error sin llamar a Drive

#### Scenario: Sin Picker API
- **WHEN** Google Picker API no está habilitada en el proyecto
- **THEN** el flujo falla con un error claro y no modifica la selección anterior

### Requirement: Persistencia de asociaciones externas

El sistema SHALL persistir asociaciones many-to-many entre entidades y recursos externos con `provider`, `account_id`, `resource_type`, `resource_id` y `metadata`. MUST validar la revisión esperada de la entidad y los límites de campos/metadata. Crear, actualizar o quitar una asociación MUST incrementar la revisión y registrar el cambio; quitarla MUST NOT borrar el recurso remoto.

#### Scenario: Vincular y listar
- **WHEN** el usuario vincula un recurso externo a una entidad con la revisión vigente
- **THEN** se persiste la asociación, se actualizan metadatos y la entidad aparece con una nueva revisión

#### Scenario: Recurso compartido
- **WHEN** dos entidades vinculan el mismo recurso externo
- **THEN** ambas asociaciones comparten el recurso sin duplicarlo y quitarlas no borra el archivo remoto

#### Scenario: Entidad o asociación inválida
- **WHEN** la revisión no coincide, falta la entidad o se intenta quitar una asociación inexistente
- **THEN** no se modifica ninguna tabla y se devuelve un error explícito

### Requirement: Listado paginado de asociaciones externas

El sistema SHALL listar las asociaciones de una entidad con paginación acotada y MUST rechazar entidades inexistentes.

#### Scenario: Listado
- **WHEN** se solicita `external.list` para una entidad existente
- **THEN** devuelve sólo sus asociaciones, con cursor y límite explícitos

### Requirement: Previsualización read-only de Docs y Sheets

El sistema SHALL previsualizar Google Docs como `text/plain` y Google Sheets como `text/csv` mediante exportación de Drive API, usando la sesión Picker. MUST ser sólo lectura, MUST NOT persistir el contenido y MUST limitar la respuesta a 2 MiB. Sin red, permiso revocado, archivo inexistente o tipo no soportado MUST devolver un error claro sin mostrar contenido obsoleto.

#### Scenario: Sheet
- **WHEN** el usuario previsualiza un Google Sheet seleccionado
- **THEN** se muestra la primera hoja como CSV de sólo lectura

#### Scenario: Doc
- **WHEN** el usuario previsualiza un Google Doc seleccionado
- **THEN** se muestra texto plano de sólo lectura

#### Scenario: Permiso revocado o sin red
- **WHEN** Drive responde 403/404 o la red falla
- **THEN** la UI informa el error y no conserva una previsualización anterior

### Requirement: Lectura Drive acotada

El sistema SHALL listar hasta 20 archivos con id/nombre/mime/modificado/enlace, sin descargar contenido ni escribir en Drive. MUST NOT habilitar Google Fotos ni edición de Sheets.

#### Scenario: Listado
- **WHEN** hay sesión válida y se solicita `google files`
- **THEN** devuelve metadatos acotados y no modifica archivos

#### Scenario: Permiso revocado o sin red
- **WHEN** la API responde error de permiso/red
- **THEN** informa el error sin presentar metadatos obsoletos como vigentes
