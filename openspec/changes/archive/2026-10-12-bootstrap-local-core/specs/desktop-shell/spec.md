# Spec Delta

## Purpose

Dar acceso de escritorio al núcleo local de Noteka mediante una navegación mínima y captura de notas, haciendo visibles la persistencia, los conflictos y los límites del formato inicial.

## ADDED Requirements

### Requirement: Apertura y navegación local

La interfaz SHALL permitir seleccionar o crear un workspace y navegar listados de Notes, Tasks, Projects, Tags, People y Archive, con estados explícitos para listas vacías y fallos de apertura.

#### Scenario: Primer workspace

- **WHEN** se inicia la aplicación sin un workspace seleccionado
- **THEN** ofrece crear o abrir uno y muestra su ruta después de seleccionarlo

#### Scenario: Workspace no disponible

- **WHEN** la ruta seleccionada no se puede abrir
- **THEN** muestra el error y permite elegir otra ruta sin presentar datos de otro workspace como propios

### Requirement: Captura y edición mínima de notas

La interfaz SHALL permitir crear una nota con título y texto de párrafos, abrirla y editar su contenido usando las operaciones compartidas. SHALL preservar los IDs de párrafos que no se reemplazan.

#### Scenario: Captura y reapertura

- **WHEN** se crea y guarda una nota desde la interfaz y luego se vuelve a abrir
- **THEN** aparecen el título y párrafos persistidos y el CLI puede leerlos

### Requirement: Estados de guardado verificables

La interfaz SHALL distinguir cambios pendientes, guardado confirmado y fallo. MUST conservar el buffer pendiente ante un fallo y ofrecer reintento sin afirmar que ese contenido ya está persistido.

#### Scenario: Fallo al guardar

- **WHEN** una nota editada no puede guardarse por un error de almacenamiento
- **THEN** el contenido editado permanece visible, se muestra fallo y se ofrece reintentar

#### Scenario: Cierre con cambios pendientes

- **WHEN** se intenta cerrar una nota con cambios todavía sin confirmar
- **THEN** la interfaz permite guardarlos, descartar explícitamente o cancelar el cierre

### Requirement: Cambios externos sin sobrescritura

La interfaz SHALL refrescar datos al activar una página o solicitar actualización, y detectar conflictos al guardar. MUST conservar el buffer local si el CLI modificó la misma nota y permitir recargar o copiar el contenido pendiente.

#### Scenario: CLI modifica nota abierta

- **WHEN** el CLI confirma una nueva revisión mientras la interfaz contiene ediciones de la revisión anterior y luego la interfaz intenta guardar
- **THEN** muestra conflicto, conserva ambas referencias de revisión y el buffer local, y no sobrescribe la revisión del CLI

### Requirement: Archivo y restauración visibles

La interfaz SHALL permitir archivar y restaurar entidades con revisión vigente, mostrar archivadas en Archive y preservar referencias a entidades archivadas desde notas activas.

#### Scenario: Archivo reversible en UI

- **WHEN** se archiva una nota y después se restaura desde Archive
- **THEN** deja de aparecer y vuelve a aparecer en Notes conservando identidad y contenido

### Requirement: Formato no editable identificado

La interfaz MUST deshabilitar la edición de documentos con versión o bloques desconocidos, mostrar una limitación explícita y ofrecer recuperar su JSON original mediante las operaciones de lectura.

#### Scenario: Documento de una extensión futura

- **WHEN** se abre una fixture con bloques no soportados
- **THEN** la interfaz no los reemplaza por párrafos y ofrece acceso al contenido original sin modificarlo

### Requirement: Acceso por teclado y sin dependencias remotas

La interfaz SHALL ofrecer foco visible y acceso por teclado a navegación, creación, guardado y archivo, y esos recorridos SHALL funcionar sin cargar recursos remotos o modelos.

#### Scenario: Recorrido offline por teclado

- **WHEN** con la red deshabilitada se navega, crea, guarda y archiva una nota usando teclado
- **THEN** todas las acciones se completan y el foco resulta identificable
