# Spec Delta

## MODIFIED Requirements

### Requirement: Apertura y navegación local

La interfaz SHALL permitir seleccionar o crear un workspace. El menú lateral SHALL mostrar únicamente Home, Tasks, Notes, Projects, Tags y People, en ese orden, con búsqueda contextual fuera de esa lista y Collections como botón de icono accesible. MUST conservar los destinos secundarios, incluido Archive, mediante accesos de búsqueda y mostrar estados explícitos para listas vacías y fallos de apertura.

#### Scenario: Primer workspace
- **WHEN** se inicia la aplicación sin un workspace seleccionado
- **THEN** ofrece crear o abrir uno y muestra su ruta después de seleccionarlo

#### Scenario: Workspace no disponible
- **WHEN** la ruta seleccionada no se puede abrir
- **THEN** muestra el error y permite elegir otra ruta sin presentar datos de otro workspace como propios

#### Scenario: Navegación primaria reducida
- **WHEN** se abre un workspace y se recorre el lateral por teclado
- **THEN** la lista contiene exactamente los seis destinos definidos, Collections es un botón de icono con nombre accesible y Archive, Inbox, My Day, Pending, Overdue, Active Projects, Areas y Resources siguen disponibles desde el buscador sin cambiar sus reglas

## ADDED Requirements

### Requirement: Pestañas de sesión independientes

El escritorio SHALL abrir secciones o entidades en pestañas de sesión con título, selección y cierre accesibles. El botón `+` MUST abrir el mismo buscador contextual de la barra y crear una pestaña sólo después de seleccionar un resultado. Alternar pestañas SHALL conservar el buffer, revisión capturada, editor, selección y scroll de cada una sin guardar ni descartar implícitamente.

#### Scenario: Nueva pestaña desde resultado
- **WHEN** se presiona `+`, se busca una entidad y se selecciona el resultado
- **THEN** se abre el original en una pestaña nueva seleccionada; la pestaña anterior conserva pantalla y edición, y cancelar el buscador no crea una pestaña vacía

#### Scenario: Alternar ediciones y respuesta tardía
- **WHEN** dos pestañas tienen entidades o buffers distintos y una consulta termina después de cambiar de pestaña
- **THEN** cada respuesta pertenece a su pestaña, los buffers/revisiones no se intercambian y volver conserva el contenido local sin autoguardado

#### Scenario: Teclado y desbordamiento
- **WHEN** se abre más de una pestaña en un viewport estrecho y se alterna con teclado
- **THEN** la pestaña seleccionada es identificable y alcanzable, los controles de cierre y `+` permanecen disponibles y el contenido inactivo no recibe foco ni atajos de edición

### Requirement: Cierre agregado de pestañas y workspace

Cerrar una pestaña con edición pendiente SHALL ofrecer guardar, descartar o cancelar. El cierre del host o cambio de workspace MUST considerar todas las pestañas, incluidas las inactivas, y no confirmar cierre mientras exista una edición no resuelta, un guardado fallido o una operación protegida en curso. Los conflictos MUST conservar el buffer y la revisión externa.

#### Scenario: Cerrar pestaña editada
- **WHEN** se cierra una pestaña editada y se cancela o falla el guardado
- **THEN** la pestaña permanece abierta con su contenido pendiente; descartar explícitamente o guardar confirmado permite cerrarla sin afectar las otras

#### Scenario: Cierre del host con pestaña inactiva pendiente
- **WHEN** se solicita cerrar el host o cambiar de workspace mientras una pestaña inactiva contiene cambios
- **THEN** se activa esa pestaña y se resuelve su guardia antes de cerrar; cancelar conserva el workspace y las demás pestañas

### Requirement: Portada local y animación ASCII accesible

Home SHALL ofrecer la portada local recuperada de la referencia autorizada y una alternativa ASCII con tiburones que nadan y giran mediante cambios continuos de forma, orientación y profundidad. MUST funcionar offline en claro/oscuro, conservar la preferencia de portada separada de los datos y detener movimiento cuando se pide movimiento reducido, la portada no es visible, el documento está oculto o la pestaña está inactiva.

#### Scenario: Foto y ASCII en ambos temas
- **WHEN** se alterna foto/ASCII y tema con edición pendiente en otra pestaña
- **THEN** la portada elegida se muestra con recursos empaquetados y contraste propio del tema, y no se escriben ni descartan entidades ni buffers

#### Scenario: Movimiento reducido y desmontaje
- **WHEN** se activa movimiento reducido, se oculta la portada o se cierra su pestaña
- **THEN** queda una composición estática o se suspende el render, sin un bucle de animación activo para contenido oculto ni listeners retenidos tras desmontar

### Requirement: Movimiento de interfaz breve y accesible

El escritorio SHALL usar transiciones suaves y breves para el buscador, pestañas, paneles y estados interactivos. MUST respetar movimiento reducido, mantener foco y controles utilizables durante la transición y no retrasar operaciones de lectura, guardado ni mensajes de error para completar una animación.

#### Scenario: Transiciones y preferencia de movimiento
- **WHEN** se abre el buscador, se alterna una pestaña o se interactúa con filas y controles en cualquiera de los temas
- **THEN** los cambios visuales usan un ritmo coherente sin destellos o saltos de layout, y al activar movimiento reducido se deshabilitan los desplazamientos/animaciones sin alterar las operaciones

### Requirement: Listados en el área principal

Las secciones de entidades SHALL mostrar su listado en el área principal. Abrir una entidad o crear un borrador MUST mostrar el contenido sin abrir automáticamente una segunda columna lateral. El detalle MUST omitir el listado; éste sólo SHALL mostrarse al navegar explícitamente a una sección.

#### Scenario: Sección y apertura de un concepto
- **WHEN** se abre Tasks, Notes, Projects, Tags o People y se elige un registro
- **THEN** el listado inicial ocupa el área principal y luego el original se abre a ancho disponible sin una columna de registros lateral

#### Scenario: Apertura lenta de un detalle
- **WHEN** se abre un registro y la lectura del original sigue en curso
- **THEN** se muestra la carga del detalle sin montar transitoriamente su listado; la lista sólo vuelve al navegar a una sección

### Requirement: Detalles con composición de documento

Las entidades SHALL presentar icono sobre el título, propiedades compactas, relaciones reales y contenido en una columna centrada. MUST mantener herramientas de formato, adjuntos, exportación y metadatos técnicos en controles desplegables accesibles, sin inventar propiedades, relaciones o permisos de Notion.

#### Scenario: Abrir notas y otras entidades
- **WHEN** se abre una nota, tarea, proyecto, tag o persona
- **THEN** se muestra una composición consistente con sus datos reales y sin listado de registros ni paneles técnicos expandidos; los controles de edición y relaciones siguen accesibles

### Requirement: Presentación conservadora de contenido importado

El escritorio SHALL reconocer el prefijo de propiedades emitido por el importador y presentar sus valores como metadatos de origen de sólo lectura. MUST conservar íntegros esos bloques al editar el cuerpo. Las tablas Markdown válidas SHALL tener una vista tabular escapada y acceso al texto original, sin cambiar su formato persistido ni eliminar contenido ambiguo.

#### Scenario: Editar cuerpo de una nota importada
- **WHEN** una nota contiene el encabezado y listado de propiedades reconocidos y se modifica su cuerpo
- **THEN** los metadatos aparecen en el encabezado y el guardado recompone todos los bloques originales de propiedades con sus IDs y valores intactos; una estructura no reconocida conserva su presentación original

#### Scenario: Tabla de texto importada
- **WHEN** un bloque de código Markdown contiene una tabla válida
- **THEN** se muestra una tabla de texto escapado y se puede editar su Markdown original; abrir o guardar otra parte del documento no convierte ni altera ese bloque
