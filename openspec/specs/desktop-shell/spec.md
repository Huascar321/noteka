# desktop-shell Specification

## Purpose
Dar acceso de escritorio al núcleo local de Noteka mediante una navegación mínima y captura de notas, haciendo visibles la persistencia, los conflictos y los límites del formato inicial.

## Requirements

### Requirement: Apertura y navegación local

La interfaz SHALL permitir seleccionar o crear un workspace. El menú lateral SHALL mostrar únicamente Home, Tasks, Notes, Projects, Tags, Goals y People, en ese orden, con búsqueda contextual fuera de esa lista y Collections como botón de icono accesible. MUST conservar los destinos secundarios, incluido Archive, mediante accesos de búsqueda y mostrar estados explícitos para listas vacías y fallos de apertura.

#### Scenario: Primer workspace
- **WHEN** se inicia la aplicación sin un workspace seleccionado
- **THEN** ofrece crear o abrir uno y muestra su ruta después de seleccionarlo

#### Scenario: Workspace no disponible
- **WHEN** la ruta seleccionada no se puede abrir
- **THEN** muestra el error y permite elegir otra ruta sin presentar datos de otro workspace como propios

#### Scenario: Navegación primaria reducida
- **WHEN** se abre un workspace y se recorre el lateral por teclado
- **THEN** la lista contiene exactamente los siete destinos definidos, Collections es un botón de icono con nombre accesible y Archive, Inbox, My Day, Pending, Overdue, Active Projects, Areas y Resources siguen disponibles desde el buscador sin cambiar sus reglas

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

### Requirement: Criterios explícitos de lectura de rango XLSX

El lector existente SHALL ofrecer hoja por ID, rango A1 y fila de encabezados opcional al elegir rango XLSX. MUST presentar cobertura de rango separada, encabezados/celdas/procedencia completos escapados y una página de20 items, con invalidación de criterios, errores útiles y descarte tardío sin alterar buffers/Home.

#### Scenario: Hoja rango y encabezados en ambos temas
- **WHEN** se consulta y pagina por teclado un rango XLSX con encabezados hostiles en dark/light
- **THEN** se muestran como datos las celdas tipadas, encabezados elegidos y manifiesto completo; abrir/editar criterios no consulta ni procesa y no hay requests externos

#### Scenario: Error cambio y salida
- **WHEN** se cambia criterio, falla la lectura o llega respuesta después de salir con nota sin guardar
- **THEN** se retira página/cursor sin retry automático, se conserva el buffer y se descarta respuesta tardía manteniendo el cupo físico hasta finalizar IPC

### Requirement: Lectura de tablas PDF con grid explícito

El lector existente SHALL ofrecer página/grid/encabezado elegidos sobre PDF OCR, una página de20 celdas completas y cobertura/manifiesto/exteriores visibles. MUST escapar contenido y conservar buffers/guardias, criterios descartables y cupo físico; abrir o buscar no proyecta, prepara ni selecciona automáticamente.

#### Scenario: Tabla por teclado en ambos temas
- **WHEN** se consulta y pagina tabla con huecos, texto hostil, varios textos por celda y exteriores en dark/light
- **THEN** muestra procedencia y cobertura diferenciadas, texto/regiones completos sin ejecutar HTML; buffer, originales y selección FTS permanecen iguales, sin requests externos

#### Scenario: Error edición y salida tardía
- **WHEN** se cambia grid, Rust rechaza ambigüedad/capacidad o llega respuesta después de salir
- **THEN** retira página/cursor, mantiene error útil sin retry/fallback y descarta tardíos sin liberar anticipadamente IPC ni sobrescribir edición

### Requirement: Home local con cobertura visible

El escritorio SHALL ofrecer Home de ancho amplio con portada local, icono de casa y secciones Tasks, Notes, Projects, Tags, Goals y People en ese orden. MUST presentar Goals desde sus originales y conservar la franja de clima como pendiente explícito sin cifras o acciones ficticias, recursos remotos ni bloqueo de las secciones implementadas.

#### Scenario: Workspace vacío y estructura offline
- **WHEN** se abre Home en un workspace vacío sin red ni modelos
- **THEN** aparecen composición, secciones, estados vacíos y creación local, mientras Goals muestra sus grupos vacíos y clima indica su falta de implementación sin datos simulados

#### Scenario: Navegación compacta y funciones diferidas
- **WHEN** se usan los encabezados de las secciones de Home y se consulta la cobertura de las pestañas de referencia
- **THEN** Tasks/Notes/Projects/Tags/Goals/People llevan a secciones reales y Content y auxiliares pendientes se identifican sin controles que aparenten ejecutar esas funciones

### Requirement: Presentación Home de vistas originales

Home SHALL mostrar las vistas locales habilitadas como listas compactas de tareas/notas/tags, grupos plegables, listas/tablero de proyectos y galería de personas con fotos locales opcionales. MUST presentar contexto, errores, resúmenes y cobertura del núcleo, conservar IDs al cambiar de vista y ofrecer paginación y apertura por teclado sin recalcular reglas de negocio en la interfaz.

#### Scenario: Cambio de vista y páginas
- **WHEN** se alternan vistas de las cinco secciones, se pliega un grupo y se recorren 51 elementos coincidentes
- **THEN** cada vista conserva el orden, IDs y alcance de su consulta, el grupo no borra datos y la cobertura distingue filas únicas de apariciones; abrir una fila o tarjeta conduce al original

#### Scenario: Progreso y metadatos no disponibles
- **WHEN** un proyecto carece de tareas, un resumen no es interpretable o una persona carece de foto u organización
- **THEN** la presentación distingue sin tareas y dato no disponible, usa una tarjeta con fallback textual y no sustituye desconocimiento por cero, porcentajes inventados o atributos de otra entidad

### Requirement: Navegación y captura Home protegidas

La interfaz SHALL abrir originales y comenzar creación desde Home con las guardias existentes de guardar/descartar/cancelar. MUST conservar buffers de entidades y colecciones ante cancelación, conflicto o fallo; crear tareas en un grupo de proyecto con relación atómica y proyectos en una columna con ese estado, sin writes hasta confirmar ni defaults ocultos de fechas.

#### Scenario: Llegar a Home con buffer pendiente
- **WHEN** desde una nota o colección editada se intenta navegar a Home y se cancela o falla guardar
- **THEN** permanecen buffer, IDs y revisiones capturadas, no se monta Home como sustituto del contenido pendiente y la versión del CLI no se sobrescribe

#### Scenario: Crear en proyecto o estado
- **WHEN** se crea y guarda una tarea desde un grupo de proyecto o un proyecto desde una columna del tablero
- **THEN** CLI y Home leen una única entidad con la relación o estado elegidos; descartar antes de guardar no crea entidad y capturar desde Today no asigna automáticamente due_date ni planned_date

#### Scenario: Confirmación seguida de fallo de lectura
- **WHEN** una creación se confirma pero falla el refresco posterior de Home
- **THEN** se distingue guardado confirmado de lectura fallida y se ofrece reintentar la consulta, no repetir una creación ni reponer un buffer ya confirmado

### Requirement: Lecturas Home vigentes y aisladas

Home SHALL mantener selección, página, estado de carga/error y actualización por sección. MUST descartar respuestas de vistas, días, workspaces o componentes anteriores, limpiar datos al cambiar su identidad de consulta y no etiquetar resultados viejos como actuales después de un fallo. Una lectura o refresh MUST dejar intactos originales y buffers pendientes.

#### Scenario: Respuesta tardía al cambiar vista o workspace
- **WHEN** una consulta termina después de cambiar la vista, día, workspace o desmontar Home
- **THEN** no publica sus filas, contexto, cantidades ni error en la nueva pantalla y no modifica el buffer activo

#### Scenario: Fallo y reintento independiente
- **WHEN** falla una sección y se solicita reintentar mientras otras secciones tienen resultados válidos
- **THEN** la sección afectada muestra el fallo sin filas viejas como respuesta vigente; el reintento consulta los mismos originales y no borra las otras selecciones ni escribe datos

#### Scenario: Actualizar después del CLI
- **WHEN** el CLI cambia una entidad o relación y se actualiza Home o se vuelve a activar su ventana
- **THEN** las proyecciones reflejan los originales confirmados con día/zona visibles, sin polling, rebase de revisiones capturadas ni incremento de revisiones por leer

### Requirement: Temas y foco verificables en Home

El escritorio SHALL ofrecer temas claro y oscuro con preferencia local, oscuro inicial, contraste legible y foco visible; navegación, vistas, grupos, paginación y creación Home SHALL funcionar por teclado y offline. MUST conservar datos/buffers al cambiar tema y tratar fallo de preferencia como recuperable, sin cambiar el formato del workspace.

#### Scenario: Tema persistido y buffer intacto
- **WHEN** se cambia tema con edición pendiente y se reabre la aplicación
- **THEN** la edición no se escribe ni descarta por el cambio y la preferencia reaparece; una preferencia inválida o inaccesible usa oscuro y no impide operar el workspace

#### Scenario: Foco y layouts en ambos temas
- **WHEN** se recorren Home, pestañas, grupos, tablero, tarjetas y creación por teclado en temas claro/oscuro
- **THEN** el foco es identificable, los controles tienen nombres accesibles y el contenido es legible; en viewport más estrecho no desaparecen las acciones esenciales ni se fuerza un listado lateral vacío de Home

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

La alternativa ASCII SHALL distinguir morro/aletas/cola, coordinar giros y ondulación posterior con impulso/deslizamiento y estabilizar caracteres sin ruido temporal ni estelas de celdas vacías. MUST reutilizar buffers y caracteres preparados, conservar velocidad basada en tiempo y programar dibujo con requestAnimationFrame hasta120 FPS, sujeto a la cadencia/presupuesto reales. Partículas, luz y bancos de peces ASCII SHALL ser ambientales, acotados y locales; sin consultas de datos, recursos remotos ni nuevos ajustes persistentes.

La presentación SHALL seguir la referencia de video autorizada mediante vista cenital, celdas sombreadas con letras contrastantes y acentos deterministas discretos, y grilla fina de fondo. MUST incluir tiburones y peces de cuerpo compacto/alargado con geometrías y cadencias propias, conservando render procedural, estabilidad temporal y las pausas/cadencia existentes. La interfaz del video no se incorpora a Home.

La escena SHALL seguir ambas referencias diurna y nocturna con curvatura visible
que se distribuye por el cuerpo durante los giros y llega progresivamente a la
cola; las aletas y el sombreado MUST acompañar esa postura. Los animales SHALL
conservar ritmos y trayectorias suaves propios, con tonos gris azulados y
oliva/dorados legibles según el tema, y solapamiento coherente con la profundidad.

La composición SHALL repartir los animales por distintas zonas del ancho y
alturas de la portada, conservando posiciones variables y recorridos continuos.
MUST reducir la acumulación frecuente de tres o cuatro animales en un mismo
lugar sin reemplazar el nado por saltos o posiciones estáticas; los cruces
ocasionales siguen siendo parte de la escena.

#### Scenario: Foto y ASCII en ambos temas
- **WHEN** se alterna foto/ASCII y tema con edición pendiente en otra pestaña
- **THEN** la portada elegida se muestra con recursos empaquetados y contraste propio del tema, y no se escriben ni descartan entidades ni buffers

#### Scenario: Movimiento reducido y desmontaje
- **WHEN** se activa movimiento reducido, se oculta la portada o se cierra su pestaña
- **THEN** queda una composición estática o se suspende el render, sin un bucle de animación activo para contenido oculto ni listeners retenidos tras desmontar

#### Scenario: Cadencia y reanudación
- **WHEN** el navegador entrega callbacks a distintas frecuencias o la animación se reanuda tras una pausa
- **THEN** se dibuja como máximo a120 FPS, el desplazamiento depende del tiempo activo y no de la cantidad de cuadros, y la pausa no produce saltos de recuperación

#### Scenario: Escena ambiental estable
- **WHEN** se dibuja o redimensiona la portada en cualquiera de los temas
- **THEN** tiburones y detalle ambiental conservan geometría finita y caracteres preparados con contraste del tema, el historial se reinicia al cambiar dimensiones y no se añaden requests ni escrituras del workspace

#### Scenario: Diversidad y textura de la referencia
- **WHEN** se observa la portada en movimiento o como composición estática
- **THEN** se distinguen las siluetas de tiburón, pez compacto y pez alargado, con sus propias aletas/colas y nado, mientras celdas/letras mantienen sombreado y acentos estables sin usar fotogramas como recurso

#### Scenario: Nado curvo y referencias de día y noche
- **WHEN** los animales giran, se cruzan o se alterna el tema durante el nado
- **THEN** cuerpo, aletas y cola forman una curva continua sin estiramientos laterales de cola ni giros rígidos, con sombreado que acompaña la postura y paleta del tema; la animación sigue siendo procedural y local

#### Scenario: Recorridos más dispersos
- **WHEN** se observa la portada durante distintos tramos del nado, también en ancho estrecho
- **THEN** los tiburones conservan recorridos separados por el ancho y los peces pequeños alternan zonas altas/bajas con posiciones cambiantes, reduciendo agrupaciones prolongadas y conservando el estilo, nado y pausas existentes

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

### Requirement: Tasks con composición de la referencia

Tasks SHALL presentar encabezado, acción New Task, pestañas con iconos, filas compactas y grupos plegables por proyecto con icono real, nombre y conteo de página. MUST alinear relaciones y fechas, conservar apertura/creación protegidas y ofrecer cobertura/reglas accesibles sin convertir cada fila en una tabla técnica.

#### Scenario: Vista agrupada y recursos locales
- **WHEN** se elige una vista agrupada de Tasks en claro u oscuro
- **THEN** los grupos y filas comparten iconos de sus proyectos, las fechas mantienen su valor civil y abrir/crear usa los originales y guardias existentes sin recursos remotos

#### Scenario: Cobertura error y ancho estrecho
- **WHEN** hay paginación, error o viewport estrecho
- **THEN** la cobertura distingue página de total, el error no simula ausencia y títulos/controles permanecen alcanzables con teclado sin desbordar la página

### Requirement: Vistas adicionales de tareas locales

El núcleo SHALL ofrecer Week, Month, Scheduled, No Due y tareas pendientes de proyectos activos/en progreso en home.query. MUST filtrar antes de paginar, conservar cursores/caps/snapshot y distinguir metadatos inválidos; Today, All y Done conservan sus reglas e Inbox se deriva de la ausencia de relación Project local. Las vistas sin soporte no se presentan como acciones habilitadas.

#### Scenario: Fechas y proyectos

- **WHEN** se consulta una semana ISO, mes civil, fechas asignadas/ausentes o proyectos activos
- **THEN** Rust devuelve sólo tareas pendientes admisibles del alcance solicitado, con grupos de proyectos elegibles y conteos anteriores a la página; no asigna fechas ni cambia estados

#### Scenario: Cursor o metadatos inválidos

- **WHEN** se reutiliza un cursor de otra vista o hay metadatos no interpretables
- **THEN** se rechaza el cursor ajeno y se identifica la cobertura no disponible, sin reparar originales ni atribuir filas viejas a la vista nueva

### Requirement: Arte de entidad como adjunto local

La interfaz SHALL presentar iconos personalizados de imagen desde adjuntos del workspace, además de Lucide. MUST usar referencias locales explícitas, no inferir arte por títulos, limitar cache/lecturas y mostrar fallback si el adjunto falta o no es una imagen compatible. El cambio de icono conserva contenido, relaciones y propiedades ajenas.

#### Scenario: Importación explícita de arte
- **WHEN** se recupera un icono autorizado y se asocia a una entidad con revisión esperada
- **THEN** se guarda el original como adjunto, la entidad conserva una referencia local y el mismo arte aparece en Home, relaciones, detalle, búsqueda y pestañas sin URL remota en runtime

#### Scenario: Falta de fuente o cambio de workspace
- **WHEN** el adjunto no está disponible, es excesivo/incompatible o termina una lectura de otro workspace
- **THEN** aparece un icono alternativo, no se ejecuta SVG/HTML y la respuesta anterior no muestra imágenes ajenas ni modifica originales

### Requirement: Recuperación explícita de vínculos importados

La recuperación de relaciones omitidas SHALL usar referencias explícitas de la fuente autorizada y revisiones esperadas. MUST conservar originales y relaciones existentes, respaldar antes de aplicar y omitir coincidencias ambiguas sin elegir por posición aislada o inventar proyectos.

#### Scenario: Vínculo confirmado y título duplicado
- **WHEN** se recuperan relaciones de la referencia y una tarea tiene más de una coincidencia local con el mismo título/fecha sin evidencia que resuelva su identidad
- **THEN** las coincidencias únicas se vinculan a sus proyectos reales y el duplicado queda pendiente en un informe externo, sin modificar fechas, estados, documentos o entidades ajenas

### Requirement: Notes con composición de la referencia

Notes SHALL presentar encabezado, New Note, pestañas con iconos y filas compactas con arte, relaciones, tipo y antigüedad disponibles. MUST conservar apertura y creación protegidas, accesibilidad por teclado, temas, errores y cobertura de página sin cargar imágenes remotas.

#### Scenario: Notas y referencias reales
- **WHEN** se abre Notes en claro u oscuro
- **THEN** las filas muestran originales y vínculos locales, tipos como chips y días compactos con procedencia accesible; una imagen ausente usa fallback

#### Scenario: Lista estrecha o lectura fallida
- **WHEN** se reduce el viewport o falla una consulta
- **THEN** título y controles siguen accesibles sin overflow global, el error ofrece reintento y no se muestra como lista vacía

### Requirement: Vistas diferenciadas de Notes

Rust SHALL calcular Notes, Fav., Clips, Voice, Journal y Meetings además de Inbox y All. MUST filtrar antes de paginar, separar la biblioteca de otras páginas importadas mediante procedencia explícita y conservar caps, cursores y descarte tardío. All mantiene acceso a todas las notas locales activas.

#### Scenario: Biblioteca y categorías
- **WHEN** hay notas locales y páginas importadas de varias colecciones
- **THEN** Notes incluye notas locales y notas generales de la colección Notes no archivadas en origen; las categorías usan tipo/favorito explícitos y All conserva todas las páginas activas

#### Scenario: Paginación y metadata inválida
- **WHEN** se pagina o hay metadata no interpretable
- **THEN** cada vista conserva orden/desempate estable y cobertura no disponible; un cursor de otra vista se rechaza y las respuestas anteriores no reemplazan la vista actual

### Requirement: Recuperación explícita de metadata de notas

La recuperación de metadata y arte SHALL usar el export/manifiesto o referencias explícitas autorizadas, con backup y revisión esperada. MUST omitir coincidencias ambiguas, conservar documentos, relaciones existentes y contenido sensible fuera de Git, sin abrir cuerpos de notas ni inventar iconos por título.

#### Scenario: Fuente e identidad inequívocas
- **WHEN** una nota coincide con el manifiesto original y las propiedades exportadas
- **THEN** se recuperan sólo los campos acreditados y vínculos visibles inequívocos, guardando informe externo y los documentos originales intactos

### Requirement: Projects con tablero compacto de referencia

Home Projects SHALL abrir Board y presentar Active, Planned y Board, encabezado, New Project y cinco columnas Planned/Doing/Ongoing/Done/On Hold. MUST mostrar tarjetas compactas con iconos originales locales, título, progreso disponible y contexto navegable, conservar captura protegida, teclado, temas y desplazamiento horizontal contenido.

#### Scenario: Tablero con columnas vacías
- **WHEN** se abre Projects con datos locales
- **THEN** aparecen cinco columnas en el orden indicado, incluso vacías, y New page prepara un proyecto del estado de esa columna sin guardarlo automáticamente

#### Scenario: Pantalla estrecha o error de lectura
- **WHEN** el viewport es estrecho o falla una consulta
- **THEN** el tablero permite desplazamiento interno sin overflow global y la sección conserva error, reintento y cobertura accesibles

### Requirement: Proyección real de proyectos y contexto importado

Rust SHALL filtrar procedencia archivada antes de paginar Projects y calcular total, completas, pendientes, vencidas y progreso en décimas desde tareas locales relacionadas únicas. MUST conservar caps, cursores y snapshots; proyectos sin tareas no equivalen a completos y datos inválidos producen resumen no disponible. Goals tipados y páginas importadas explícitamente de Goals MAY aparecer como referencias de contexto, conservando su página original; los Goals tipados abren su sección canónica.

#### Scenario: Relaciones completas y estados locales
- **WHEN** un proyecto tiene tareas relacionadas activas con estados válidos
- **THEN** su progreso es completas/total redondeado a una décima, las vencidas son pendientes con fecha anterior al día civil del workspace y los enlaces duplicados no incrementan los contadores

#### Scenario: Fuente archivada y consulta de tareas por proyecto
- **WHEN** un proyecto está archivado sólo en la procedencia
- **THEN** se excluye de las vistas Projects y de la pertenencia a proyectos activos de Home, conservando la entidad local y sus documentos

### Requirement: Recuperación acreditada de relaciones de proyectos

La recuperación SHALL vincular identidades mediante manifiesto, secuencia completa verificada y propiedades originales inequívocas o referencias explícitas autorizadas. MUST usar backup, CAS e informe privado incremental, conservar documentos, IDs, estados y relaciones existentes, omitir ambigüedades y recuperar sólo arte observado sin solicitudes remotas en Home.

#### Scenario: Recuperación con edición concurrente
- **WHEN** se aplica metadata acreditada o relaciones y cambia una revisión local
- **THEN** se usa la revisión vigente esperada y se rechaza un conflicto, dejando informe de lo completado y sin reemplazar documentos ni forzar escrituras

### Requirement: Tags con grupos y filas de referencia

Home Tags SHALL abrir Types y presentar Recent/Fav./A-Z/Types, encabezado, controles reales y grupos Area/Resource/Entity con colores, conteos y filas compactas. MUST mostrar iconos locales, última nota navegable y contador, conservar teclado, temas, errores, cobertura y captura con clasificación contextual. Topic conserva un grupo independiente cuando hay registros de ese tipo.

#### Scenario: Grupos e iconos locales
- **WHEN** se abre Tags en Types
- **THEN** aparecen los tres grupos de referencia incluso vacíos, los registros se ordenan por título en su tipo y las filas muestran la última nota y cantidad calculadas con recursos locales

#### Scenario: Ventana estrecha o fallo
- **WHEN** se reduce el viewport o falla la lectura
- **THEN** las filas se adaptan sin overflow global y el error permite reintentar sin confundirse con una lista vacía

### Requirement: Vistas y resúmenes acreditados de tags

Rust SHALL filtrar archivo de procedencia y favoritos antes de paginar y conservar A-Z/Types. MUST contar notas relacionadas activas únicas, excluir archivo de origen y elegir la última por modificación de fuente acreditada o local si falta, con desempate estable. Recent SHALL combinar actividad importada y fechas de notas relacionadas con fallback local, conservando caps, snapshots y cursores.

#### Scenario: Contador y última nota
- **WHEN** un tag tiene notas relacionadas válidas
- **THEN** el contador refleja esas notas únicas y la última nota se determina con timestamps interpretados como instantes, sin que el orden de recuperación de iconos altere el resultado

#### Scenario: Actividad reciente acreditada
- **WHEN** se consulta Recent con actividad de fuente y notas relacionadas
- **THEN** se combina su actividad explícita con fechas acreditadas de notas, usando modificación local si falta; la procedencia conserva su carácter de snapshot sin prometer sincronización continua

#### Scenario: Favoritos, archivo y datos inválidos
- **WHEN** hay favoritos, tags archivados en origen o metadatos inválidos
- **THEN** Fav. filtra la marca explícita, Home excluye los archivados sin archivar sus entidades locales y los datos no interpretables se informan; se mantienen caps, snapshots y cursores por vista

### Requirement: Recuperación explícita de tags y notas relacionadas

La recuperación SHALL acreditar tags con manifiesto/CSV y propiedades originales y notas por su ID original importado. MUST conservar documentos, IDs, metadatos elegidos y relaciones existentes, respaldar antes de aplicar, usar CAS e informe privado incremental y omitir identidades ambiguas. El arte sólo se recupera de referencias observadas y Home no realiza solicitudes remotas.

#### Scenario: Datos y revisión concurrente
- **WHEN** se recuperan relaciones y arte acreditados
- **THEN** sólo se agregan vínculos inequívocos, se registra la revisión vigente y un conflicto rechaza la escritura sin reemplazar cuerpos ni forzar el guardado

### Requirement: Movimiento de tarjetas de Projects Board

Projects Board SHALL permitir arrastrar una tarjeta antes de otra o al final de una columna, tanto dentro de su estado como entre estados. MUST ofrecer alternativa de teclado con elección de estado/posición, indicadores y estado de guardado/error, sin iniciar un movimiento desde datos de arrastre externos ni impedir abrir el proyecto o sus referencias.

#### Scenario: Orden y estado
- **WHEN** se suelta un proyecto en una posición válida de Board o se confirma Mover con teclado
- **THEN** se conserva ese orden tras recargar y cambiar de columna cambia el estado; soltarlo en su misma posición no escribe datos

#### Scenario: Cancelación o página incompleta
- **WHEN** se cancela el arrastre, se cierra el diálogo o el tablero es una página parcial/no interpretable
- **THEN** no se inicia una escritura y la limitación se informa; el resto de navegación conserva sus guardias

### Requirement: Persistencia atómica del orden de proyectos

El movimiento SHALL usar las operaciones de lote existentes, con payloads completos conservados, revisiones esperadas y receipt del preview. MUST contrastar el snapshot completo del tablero antes del apply, rechazar conflictos sin reintento mutante automático y preservar títulos, documentos, iconos, procedencia y relaciones. Los estados y órdenes se validan en Rust; el orden de presentación no excede los límites actuales.

#### Scenario: Cambio concurrente o error
- **WHEN** el tablero cambia durante el movimiento o falla la operación
- **THEN** el apply se rechaza o revierte de forma atómica, se informa el error y se actualiza la lectura; no se presentan cambios parciales como guardados

#### Scenario: Ámbito completo
- **WHEN** Board contiene hasta cien proyectos interpretables y está cargado completo
- **THEN** las tarjetas se pueden mover usando sólo sus revisiones/IDs reales, sin actualizar documentos ni modificar registros fuera del orden afectado

### Requirement: Catálogo de iconos SVG local

La interfaz SHALL usar un catálogo seleccionado de AnimateIcons en SVG estáticos
locales para sus iconos de sistema, conservando tamaño, color, etiquetas accesibles
y claves lógicas existentes. MUST preservar imágenes/logos importados, documentos
y payloads; nombres desconocidos conservan un fallback seguro. La generación
reproducible MUST fijar versión y conservar licencias, sin cargar catálogos completos,
red remota ni código de animación para mostrar iconos estáticos.

#### Scenario: Inicio y uso offline
- **WHEN** se inicia o navega por la aplicación sin pulsar el menú lateral
- **THEN** los iconos se presentan estáticos desde recursos locales, con tamaño/color y sin descargar iconos ni activar animaciones

#### Scenario: Datos conservados
- **WHEN** se muestra un logo importado o un nombre no reconocido
- **THEN** se mantiene el logo local o se usa el fallback, sin modificar la entidad

### Requirement: Animación explícita y breve del menú lateral

Únicamente pulsar un botón lateral de sección, Search o Collections SHALL activar
un ciclo breve del icono pulsado, con reproductor cargado a demanda y retorno exacto
al SVG estático. MUST conservar la navegación/guardias y no animar por hover,
selección de otra procedencia, render ni temporizadores recurrentes. Clics repetidos
reinician un único ciclo; desmontaje, pestaña oculta o movimiento reducido cancelan
la reproducción y liberan listeners/temporizadores.

#### Scenario: Clic y reposo
- **WHEN** se pulsa con ratón o teclado un botón lateral habilitado
- **THEN** sólo su icono reproduce brevemente y vuelve a reposo, sin animar el contenido de la pantalla

#### Scenario: Reducción o cancelación
- **WHEN** hay preferencia de movimiento reducido, se oculta la ventana o desaparece el botón durante la carga/reproducción
- **THEN** el icono permanece o vuelve a reposo y no queda una reproducción diferida activa

### Requirement: People con galería de referencia

Home People SHALL abrir Contacts y ofrecer Contacts/New/Updated/Check-Ins/Birthdays/Pipeline/Active Projects, encabezado y acciones reales. MUST presentar tarjetas compactas con foto original local y recorte explícito, icono, nombre, relaciones y empresa; conservar captura/apertura protegidas, teclado, temas, errores y cobertura. Load more SHALL acumular contactos únicos; una foto ausente usa espacio neutro sin atributos inventados ni solicitudes remotas.

#### Scenario: Galería offline y página siguiente
- **WHEN** se abre Contacts y se solicita otra página en claro u oscuro
- **THEN** las tarjetas mantienen identidad, orden acreditado y fotos locales, se añaden contactos únicos y la cobertura distingue los cargados del total

#### Scenario: Ancho estrecho o lectura fallida
- **WHEN** se estrecha el viewport o falla la consulta/lectura de foto
- **THEN** controles y tarjetas siguen accesibles sin overflow global, el error de consulta ofrece reintento y una imagen ausente no sustituye la identidad

### Requirement: Vistas locales diferenciadas de People

Rust SHALL ofrecer Contacts/New/Updated/Check-Ins/Birthdays/Pipeline/Active Projects con procedencia explícita, períodos civiles y relaciones locales. MUST filtrar antes de paginar, conservar caps/snapshots/cursores y distinguir metadata inválida. El listado general conserva personas activas de todas las colecciones.

#### Scenario: Biblioteca de contactos
- **WHEN** hay personas locales, contactos People y miembros WorkspacePeople importados
- **THEN** las vistas de contactos admiten personas sin procedencia o de People no archivadas en origen, y el listado general conserva también los miembros

#### Scenario: Períodos y calendario
- **WHEN** se consulta New/Updated, check-ins o un mes de cumpleaños
- **THEN** New/Updated usa timestamps de fuente o locales desde un mes civil antes inclusivo; Check-Ins admite fechas hasta un mes después, incluyendo atrasados; Birthdays muestra ocurrencias del mes elegido, con 29/2 el 28 en años no bisiestos sin modificar el original

#### Scenario: Proyectos y estados
- **WHEN** un proyecto Doing/Ongoing no archivado localmente ni en origen tiene vínculos a varias personas admisibles
- **THEN** aparece una vez como entidad y en cada grupo de persona, con resumen local y cobertura de apariciones; Pipeline agrupa por estado explícito y conserva grupos vacíos, sin inventar avance

#### Scenario: Metadata o cursor inválidos
- **WHEN** hay metadata no interpretable o un cursor de otra vista/día/workspace
- **THEN** se informa cobertura no disponible o se rechaza el cursor ajeno, sin reparar originales ni sustituir la vista actual con respuestas anteriores

### Requirement: Recuperación explícita de contactos

La recuperación SHALL usar el export/manifiesto y metadata/fotos observadas autorizadas, con backup nuevo, CAS e informe privado incremental. MUST conservar documentos, IDs, valores existentes y relaciones, separar las dos colecciones People, omitir identidades ambiguas y mantener datos personales fuera de Git.

#### Scenario: Identidad o revisión incierta
- **WHEN** una coincidencia no es única o una revisión cambia antes de escribir
- **THEN** se omite la ambigüedad o se rechaza el conflicto y queda informe de lo completado, sin reemplazar cuerpos ni forzar el guardado

### Requirement: Encabezado Home y actualización por teclado

Home SHALL omitir el desplegable Navegación y el botón Actualizar Home y ofrecer actualización con ⌘R en macOS, Ctrl+R en Windows/Linux o F5. MUST consultar sólo el Home activo mediante sus lectores, conservar vistas/pestañas/borradores y prevenir recarga nativa. Bloqueos, diálogos, composición y repetición no inician consultas; el listener se retira al desmontar.

#### Scenario: Actualización del Home activo
- **WHEN** se pulsa un atajo admitido con Home activo y disponible
- **THEN** sus secciones vuelven a leer los originales manteniendo sus vistas; no se recarga la aplicación ni se modifica un documento y los dos controles retirados no aparecen

#### Scenario: Diálogo o pestaña inactiva
- **WHEN** Home está inactivo o se pulsa el atajo con un diálogo/bloqueo, composición o repetición
- **THEN** no inicia una actualización; con Home activo se evita la recarga nativa y, tras desmontar, no quedan listeners de esa instancia

### Requirement: Página Tasks del menú lateral

Tasks SHALL abrir una página amplia con icono sobre título, navegación Inbox/Today/Week/Month, New Task y filas compactas con relaciones/fechas locales. Inbox MUST listar tareas pendientes sin relación Project local. MUST iniciar Today, acumular páginas únicas y mantener apertura/captura protegidas. MUST NOT ofrecer la vista Unclassified ni un control manual de Inbox. Recurring y Active Content se identifican como pendientes deshabilitadas en More views; la tira de vistas envuelve sin ocultar vistas ni usar scroll horizontal.

#### Scenario: Apertura y páginas independientes

- **WHEN** se navega a Tasks, se cambia de vista o se solicita Load more
- **THEN** aparece Today inicialmente, cada vista conserva su identidad de consulta y las páginas siguientes acumulan tareas únicas; una respuesta vieja no reaparece tras cambiar vista o abandonar la página

#### Scenario: Error y captura

- **WHEN** falla la lectura o se abre un original o New Task
- **THEN** el error permite reintentar sin fingir ausencia, y apertura/captura usan guardias y originales existentes; crear no asigna fechas ni guarda automáticamente

### Requirement: Projects integrado en la página Tasks

La página Tasks SHALL presentar Projects debajo con Active inicial y vistas Planned/Board/Deadlines/Completed. MUST usar listas compactas agrupadas para Active/Planned/Completed, fechas locales en Deadlines y tablero existente en Board, con referencias navegables, pendientes, vencidas y progreso real; conservar temas, teclado y ancho estrecho.

#### Scenario: Listas y tablero
- **WHEN** se abre Tasks y se alternan vistas de Projects
- **THEN** Active agrupa Doing/Ongoing, Planned muestra planificados, Completed muestra Done y Board ofrece sus cinco columnas con movimiento protegido; ambas secciones se leen y actualizan por separado

#### Scenario: Datos ausentes y actualización
- **WHEN** no hay fechas, tareas relacionadas, recursos locales o hay un diálogo abierto
- **THEN** se presenta ausencia explícita sin inventar porcentajes, fechas ni imágenes; actualizar con el atajo de plataforma o F5 conserva vistas y no recarga el WebView ni ejecuta cambios mientras existe un diálogo

### Requirement: Consultas de proyectos por fecha y completados

Rust SHALL ofrecer proyectos Done y proyectos no completados con end_date mediante home.query. MUST excluir archivo local/de origen, ordenar fechas civiles ascendentes con desempate estable, filtrar antes de paginar y conservar caps/snapshot/cursor. Fechas inválidas se señalan como metadatos no disponibles; leer no altera payloads ni documentos.

#### Scenario: Selección y fechas
- **WHEN** se consultan Completed o Deadlines con proyectos locales
- **THEN** Completed selecciona Done y Deadlines selecciona proyectos no Done con fecha de fin válida, ordenados por fin; ninguna vista toma fechas de tareas como plazo de proyecto

#### Scenario: Metadatos o cursor inválidos
- **WHEN** existe una fecha de fin inválida o se reutiliza un cursor de otra vista
- **THEN** la consulta identifica los metadatos no interpretables o rechaza el cursor ajeno, sin reparar datos ni aceptar una página de otro criterio

### Requirement: Breadcrumb jerárquico de secciones y registros

El escritorio SHALL mostrar Home → sección en las páginas del menú y Home → sección canónica → registro en sus detalles, con iconos propios y título actual. MUST omitir niveles externos ajenos, permitir navegar Home/sección mediante guardias existentes y conservar teclado, título accesible y ancho estrecho; el registro actual no es un enlace de guardado.

#### Scenario: Secciones y registro abierto
- **WHEN** se abre Tasks o el proyecto Proyecto de ejemplo
- **THEN** el breadcrumb muestra respectivamente Home → Tasks o Home → Projects → Proyecto de ejemplo, con los iconos locales de cada tramo y sin Volver a Home ni Databases

#### Scenario: Cambios pendientes y nombre largo
- **WHEN** se activa un antecesor con cambios pendientes o se reduce el ancho disponible
- **THEN** navegar respeta Guardar/Descartar/Cancelar sin perder el buffer; el título puede truncarse visualmente conservando su valor completo accesible y los antecesores siguen siendo alcanzables con teclado

### Requirement: Pies limpios y acciones de fila completas

Home y las páginas que reutilizan sus secciones SHALL omitir contadores/fechas de consulta al pie, Siguiente Tasks y el aviso técnico de Home. MUST ofrecer Load more y New page textuales como filas clickeables de ancho completo, conservar botones con borde, estados de carga y guardias de captura, y acumular páginas sin duplicar IDs. La cobertura sigue disponible en Opciones de vista.

#### Scenario: Cargar desde el extremo de la fila
- **WHEN** hay otra página y se activa Load more cerca del borde derecho
- **THEN** el botón ejecuta una única lectura y acumula elementos sin duplicarlos; el espacio no se reserva para un contador técnico y al terminar las páginas desaparece Load more

#### Scenario: Captura de ancho completo
- **WHEN** se activa New page sin borde desde cualquier punto de su fila
- **THEN** se prepara el borrador mediante las guardias existentes, sin guardarlo; las acciones de creación con borde mantienen tamaño y estilo

### Requirement: Página Notes del menú lateral

Notes SHALL abrir una página amplia con icono sobre el título, navegación Inbox/Fav./All/Tags, New Note y vistas Notes/Inbox/Fav./Clips/Voice/Journal/Meetings/All. Inbox MUST listar notas activas sin Project local; Unclassified conserva biblioteca sin Tag/Project/People. MUST iniciar Notes, mostrar filas con relaciones, tipo y antigüedad reales y mantener breadcrumb Home → Notes, captura/apertura protegidas y acciones textuales de ancho completo. All conserva acceso a todas las notas activas.

#### Scenario: Composición y vistas locales

- **WHEN** se abre Notes desde el menú lateral y se alternan sus vistas
- **THEN** aparece Notes inicialmente y se muestran sólo resultados de la consulta vigente, con iconos y relaciones locales, sin listado técnico paralelo ni recursos remotos

#### Scenario: Paginación y captura

- **WHEN** se activa Load more o New page desde el extremo derecho de su fila
- **THEN** se acumulan notas únicas con el cursor existente o se prepara un borrador protegido sin guardarlo; las categorías de creación conservan su tipo y favorito, e Inbox deja de seleccionarse con una marca manual

#### Scenario: Actualización y salida

- **WHEN** se actualiza mediante ⌘R/Ctrl+R/F5 o se abandona la página durante una lectura
- **THEN** se conservan las vistas sin recargar el WebView, un diálogo/bloqueo impide iniciar la actualización y las respuestas de una instancia anterior no reemplazan la pantalla actual ni sus borradores

### Requirement: Tags integrado en la página Notes

La página Notes SHALL presentar Tags debajo, con Recent inicial y vistas Fav./A-Z/Types. MUST mostrar filas con icono, última nota navegable y clasificación disponibles, mantener lecturas/páginas independientes de Notes y conservar acceso al listado general de Tags, captura protegida, errores, temas y ancho estrecho. La cobertura permanece en Opciones de vista y las filas textuales de creación/paginación ocupan el ancho disponible.

#### Scenario: Tags con contexto real
- **WHEN** se abre Notes y se cambia la vista de Tags
- **THEN** Recent se presenta inicialmente y cada vista usa los datos locales existentes; última nota y clasificación se muestran sólo si están disponibles, sin modificar la vista elegida de Notes

#### Scenario: Errores y ancho estrecho
- **WHEN** falla una sección o se reduce el ancho en cualquiera de los temas
- **THEN** la sección afectada ofrece reintento sin simular una lista vacía, la otra conserva sus datos y los títulos/controles siguen accesibles sin overflow global

### Requirement: Página Projects del menú lateral

Projects SHALL abrir una página amplia con icono propio sobre título, Nav, By Tag/By People, New Project y vistas Active/Planned/Board/Deadlines/Completed/On Hold. MUST iniciar Active con grupos Doing/Ongoing, filas con referencias y progreso locales, breadcrumb Home → Projects, apertura/captura protegidas y acciones textuales de ancho completo. Board conserva movimiento atómico existente; actualizar conserva vista sin recargar la aplicación.

#### Scenario: Composición y vistas reales
- **WHEN** se abre Projects y se alternan sus vistas en cualquiera de los temas
- **THEN** Active aparece inicialmente con los estados Doing/Ongoing, Planned/Completed/On Hold muestran sus estados, Deadlines conserva la lista local por fecha y Board sus cinco columnas; iconos, relaciones y porcentajes provienen de los originales

#### Scenario: Captura y ancho estrecho
- **WHEN** se activa New page en un grupo o se estrecha la página
- **THEN** se prepara un borrador del estado o relación elegidos sin guardarlo, el breadcrumb y controles siguen accesibles y no hay overflow global; las filas textuales responden en todo el ancho

#### Scenario: Lectura y salida protegidas
- **WHEN** falla una lectura, llega una respuesta después de salir o se actualiza con un diálogo/bloqueo
- **THEN** se ofrece reintento sin fingir ausencia, se descartan respuestas anteriores y el bloqueo no inicia actualización ni pierde borradores; el listener se retira al desmontar

### Requirement: Consultas de proyectos por pausa y relación

Rust SHALL ofrecer On Hold y proyectos agrupados por tag o persona en home.query. MUST excluir archivo local/de origen antes de paginar, conservar límites/cursores/snapshots y resúmenes vigentes, agrupar relaciones reales antes de truncar contexto y entregar identidad navegable de cada grupo. Un proyecto aparece una vez como entidad y una vez en cada grupo relacionado; los proyectos sin relación tienen un grupo explícito. Leer no modifica entidades, documentos ni vínculos.

#### Scenario: Estados y pertenencia
- **WHEN** se consulta On Hold, By Tag o By People
- **THEN** On Hold selecciona sólo on_hold y las agrupaciones admiten los cinco estados válidos, conservan relaciones múltiples y orden estable por grupo/estado/proyecto sin inferir vínculos por título ni limitar People a proyectos activos

#### Scenario: Página y metadata inválida
- **WHEN** se pagina, existe metadata no interpretable o se entrega un cursor de otra vista
- **THEN** los grupos cubren sólo los proyectos de la página con IDs únicos, el dato no interpretable se informa o el cursor se rechaza; una referencia ausente no se sustituye por otro registro y los límites existentes se conservan

### Requirement: Página Tags con navegación y vistas locales

El escritorio SHALL presentar Tags lateral con icono sobre título, breadcrumb
Home → Tags, Nav a Tasks/Notes/Projects, captura New Tag y Recent inicial. SHALL
ofrecer Recent, Fav., A-Z, Types y Table con identidades originales, última nota
relacionada y clasificación disponibles. New page y Load more SHALL ocupar su
fila completa, acumular páginas y conservar las guardias de captura/navegación.

#### Scenario: Entrada a Tags y enlaces locales
- **WHEN** se abre Tags con datos locales disponibles
- **THEN** Recent muestra hasta diez filas iniciales, última nota y tipo, el tag
  y la última nota abren sus registros originales y Nav permite abrir las tres secciones

#### Scenario: Creación y continuación protegidas
- **WHEN** se solicita New Tag, New page o Load more
- **THEN** las dos primeras preparan una captura sin guardar automáticamente y
  la última acumula la página siguiente; navegar con cambios pendientes conserva
  Guardar/Descartar/Cancelar

### Requirement: Tabla y actualización de la página Tags

Tags SHALL ofrecer Table como tabla semántica con tag, última nota y tipo,
usando el orden A-Z y la paginación locales. SHALL reutilizar filtros y validación
Rust, las guardias de respuestas tardías y reintento, y actualización activa con
⌘R/Ctrl+R/F5. La selección Table SHALL conservarse al actualizar o continuar.
La página SHALL funcionar en ambos temas y ventana estrecha sin overflow global,
sin alterar entidades/documentos ni las variantes de Tags en Home y Notes.

#### Scenario: Tabla local acumulable
- **WHEN** se selecciona Table, se actualiza o se continúa la lectura
- **THEN** se muestran columnas accesibles con datos locales disponibles, se
  mantiene Table y se respeta el cursor y desempate por identidad del orden A-Z

#### Scenario: Lectura fallida o sustituida
- **WHEN** una lectura falla o llega después de cambiar de vista o cerrar la página
- **THEN** se conserva el reintento o se descarta la respuesta obsoleta con las
  guardias compartidas; ningún campo inválido ni ausencia de relación se inventa

#### Scenario: Temas y preservación
- **WHEN** se abre la página sin conexión o con ancho estrecho en claro u oscuro
- **THEN** los datos y originales siguen siendo locales, las pestañas y tabla
  pueden desplazarse internamente y el documento no adquiere overflow horizontal

### Requirement: Goals lateral y página de referencia

El escritorio SHALL ubicar Goals entre Tags y People, con icono propio, entrada
de búsqueda y breadcrumb Home → Goals → registro. La página SHALL presentar
icono sobre título, Nav a Tasks/Notes/Projects/Tags, cuatro vistas, By Activity
inicial y grupos Active/Dream/Achieved con contexto, progreso y fecha disponibles.
By Tag SHALL presentar agrupación por tags y estados; Achieved su fecha de logro.
MUST reutilizar guardias, temas, actualización activa y filas de creación/paginación
de ancho completo, y reemplazar el aviso pendiente de Goals en Home con datos reales.

#### Scenario: Menú y página
- **WHEN** se abre Goals desde el lateral, buscador o Home
- **THEN** aparece en la posición solicitada, se consultan sus originales y las
  cuatro vistas reflejan reglas Rust con contextos navegables y vacíos explícitos

#### Scenario: Ventana estrecha y cambio de página
- **WHEN** se reduce el ancho en cualquiera de los temas o se navega con un borrador
- **THEN** no hay overflow global, se mantienen títulos/controles accesibles y
  Guardar/Descartar/Cancelar protege los cambios sin guardar automáticamente

### Requirement: Captura y detalle de Goal y Milestone

El escritorio SHALL preparar Goals con estado contextual o Dream por defecto,
editar estado/fechas y presentar sus hitos originales con creación vinculada.
Milestones SHALL aparecer debajo de Goals con List de completados, captura y
apertura originales; su detalle SHALL editar fecha completada, plazo y selección
de Goal por título. MUST mantener progreso derivado y la navegación canónica de
los contextos; ninguna lectura o selección de vista modifica entidades.

#### Scenario: Goal e hitos vinculados
- **WHEN** se abre un Goal y se solicita un hito nuevo
- **THEN** se muestran sus hitos cargados y se prepara una página Milestone con
  el vínculo al Goal, protegida por las guardias y validaciones existentes

#### Scenario: Edición de fechas y conservación
- **WHEN** se prepara o edita un Goal/Milestone y se descarta o falla su guardado
- **THEN** no se pierde el original ni un buffer por reemplazo silencioso;
  las fechas/progreso disponibles se presentan sin inventar metadatos ausentes

### Requirement: Página lateral People de referencia

El escritorio SHALL abrir People lateral con icono sobre título, Nav a Contacts,
Pipeline y calendario local, New Person y las siete vistas existentes; Contacts
SHALL ser inicial y usar galería con fotos originales, relaciones y empresa.
MUST reutilizar consultas, guardias, breadcrumb canónico, temas, reintento,
actualización activa y paginación acumulativa sin modificar datos al navegar.

#### Scenario: Contactos y navegación local
- **WHEN** se abre People y se activan Contacts, Pipeline, Cal o las otras vistas
- **THEN** se muestran sus datos locales y vacíos reales con las reglas Rust
  vigentes, y Cal presenta el calendario de cumpleaños sin depender de red o modelos

#### Scenario: Captura y ancho estrecho
- **WHEN** se crea o abre una persona, se carga otra página o se reduce el ancho
- **THEN** las identidades y fotos siguen siendo originales, la creación conserva
  Guardar/Descartar/Cancelar y los controles/títulos son accesibles en ambos temas
  sin overflow global ni reemplazo por lecturas obsoletas

### Requirement: Barra integrada de ventana por plataforma

Los controles del lateral, iconos/textos de pestañas, cierre de pestaña y nueva pestaña SHALL compartir un centro vertical en la barra. Noteka SHALL reunir el control del lateral, las pestañas de sesión y nueva pestaña en una sola barra superior. En macOS SHALL conservar los tres controles del sistema con espacio propio y sin un título nativo duplicado; en Windows SHALL ofrecer minimizar, maximizar/restaurar y cerrar en el extremo derecho. MUST permitir arrastrar la ventana desde espacios libres, mantener interactivos los botones y conservar las guardias de cierre de todas las pestañas. En web SHALL omitir controles y acciones de ventana nativa. MUST distinguir configuración implementada de aceptación nativa pendiente, sin atribuir a una previsualización comportamiento del sistema operativo.

#### Scenario: Barra de Mac y espacios libres
- **WHEN** se abre Noteka en macOS y se usan sus pestañas o un espacio libre de la barra
- **THEN** el control del lateral y las pestañas comparten altura con los botones del sistema sin superponerse, los botones siguen siendo clickeables y el espacio libre permite mover la ventana

#### Scenario: Controles de Windows y edición pendiente
- **WHEN** se minimiza, maximiza/restaura o solicita cerrar desde la barra de Windows
- **THEN** se aplica la acción de ventana correspondiente, el estado maximizado tiene nombre accesible y cerrar resuelve los cambios de todas las pestañas mediante Guardar/Descartar/Cancelar antes de terminar el host

#### Scenario: Navegador y ventana estrecha
- **WHEN** se usa la variante web o un ancho estrecho en cualquiera de los temas
- **THEN** no aparecen controles nativos falsos en web, las pestañas se desplazan internamente y lateral, nueva pestaña y cierre de pestañas conservan nombres accesibles sin overflow horizontal global

### Requirement: Lateral plegable sin reconstruir pestañas

Expandir y contraer SHALL usar una transición breve y reversible que desplace el lateral y ajuste el ancho del contenido en escritorio, y desplace el panel superpuesto en ancho estrecho, sin comprimir sus etiquetas. MUST omitir esa transición con movimiento reducido. El control superior SHALL expandir o contraer el lateral en escritorio y abrirlo/cerrarlo como panel superpuesto en ancho estrecho. MUST reflejar su estado accesible, retirar del foco la navegación oculta y conservar las instancias, buffers, selección y scroll de todas las pestañas. Plegar el lateral o cambiar el ancho SHALL ser una operación de presentación sin lecturas ni escrituras de entidades ni reinicio de la aplicación.

#### Scenario: Plegar con una edición pendiente
- **WHEN** se alterna el lateral con una página o borrador abierto
- **THEN** el área principal aprovecha el ancho liberado y el mismo panel conserva su contenido, revisión y estado sin guardar, descartar ni recrear la pestaña

#### Scenario: Panel estrecho y teclado
- **WHEN** se abre la navegación en ancho estrecho y se cierra con su control o Escape
- **THEN** la navegación puede usarse por teclado, el foco vuelve al control al cerrar con Escape y los controles ocultos dejan de participar del orden de tabulación

#### Scenario: Transición bidireccional e inversión
- **WHEN** se expande o contrae el lateral, o se invierte la acción antes de terminar
- **THEN** el movimiento continúa desde la posición visible sin saltos al destino, conserva paneles/buffers y retira inmediatamente del teclado la navegación que se cierra

#### Scenario: Movimiento reducido
- **WHEN** se alterna el lateral con preferencia de movimiento reducido
- **THEN** el estado final se aplica sin transición y mantiene el contenido y las mismas reglas de foco

### Requirement: Detalle de Task con propiedades y cuerpo editable

El escritorio SHALL presentar una Task como página con título editable, Status, Project y Due principales, relaciones originales y cuerpo enriquecido debajo. Inbox, prioridad y planificación SHALL seguir editables en View details. MUST conservar breadcrumb canónico, icono personal opcional, temas, ancho estrecho, apertura de proyectos protegida y acceso a metadatos/opciones existentes. Las acciones de subtareas, Content, comentarios, recurrencia o seguimiento de tiempo sin capacidad local no se presentan como funciones disponibles por copiar la referencia.

#### Scenario: Composición de la tarea original
- **WHEN** se abre una Task desde Home, Tasks o búsqueda
- **THEN** su detalle muestra propiedades reales y una entrada visible al cuerpo sobre la misma identidad; el proyecto abre su original mediante las guardias y los campos secundarios conservan sus valores en View details

#### Scenario: Formato no reconocido y ventana estrecha
- **WHEN** una Task tiene un cuerpo desconocido o se reduce el ancho en cualquier tema
- **THEN** el original desconocido se ofrece como JSON recuperable sin editor que lo reemplace; con cuerpo compatible, título, propiedades y editor permanecen alcanzables sin overflow horizontal global

### Requirement: Detalle de Note con propiedades locales y cuerpo protagonista

Una Note normal SHALL presentar icono y título editables, Tag/URL/Favorite/Type compactos y relaciones locales antes del cuerpo. Tipo y favorito MUST editar el payload local; Tag y las relaciones MUST abrir los originales con guardias existentes. URL importada SHALL identificarse como lectura. Inbox, procedencia y snapshot completo SHALL quedar accesibles en View details sin duplicar snapshots como controles vigentes. Goal/Milestone y Task MUST conservar su composición propia, sin migraciones ni writes al abrir.

#### Scenario: Referencia importada y edición local
- **WHEN** se abre una Note importada y se cambia Type o Favorite
- **THEN** los controles representan el payload local pendiente, el snapshot original sigue en View details y navegar a un tag o proyecto exige resolver Guardar/Descartar/Cancelar

#### Scenario: Ausencias y variantes
- **WHEN** falta URL o una relación o se abre Goal/Milestone, Task o formato desconocido
- **THEN** la Note muestra ausencia real sin inventar destinos; las variantes conservan sus controles y los documentos desconocidos siguen recuperables sin conversión

#### Scenario: Temas y ancho estrecho
- **WHEN** se abre la Note en cualquiera de los temas o a 420 px
- **THEN** título, propiedades, relaciones y cuerpo permanecen accesibles sin overflow global y los bloques de código conservan scroll propio

#### Scenario: Propiedades repetidas del export
- **WHEN** una Note acreditada de la base Notes contiene el prefijo reconocido y un primer párrafo plano de metadata con al menos tres coincidencias exactas y sólo nombres admitidos
- **THEN** el párrafo se presenta en View details y se recompone íntegro al editar; nombres desconocidos, valores contradictorios, marcas o texto ambiguo permanecen en el cuerpo

### Requirement: Herramientas discretas por grupo de funciones

Los detalles SHALL reemplazar Más opciones por botones de icono que abren paneles, sin ejecutar acciones al abrir. Notes SHALL ofrecer Exportar/Recursos externos/Adjuntos; otros tipos sólo grupos aplicables, con disponibilidad derivada de las condiciones anteriores. MUST preservar acciones, controles, buffers de panel y operaciones iniciadas al cerrar. Los botones SHALL tener tooltip en hover/foco, nombre accesible, aria-controls/expanded, foco visible y Tab/Enter/Espacio/Escape con retorno al disparador; el contenido oculto no recibe foco.

#### Scenario: Abrir y cerrar sin operaciones
- **WHEN** se abre un grupo con clic o teclado y se cierra con Escape
- **THEN** sólo cambia la visibilidad, no se exporta/vincula/procesa nada, el foco vuelve al icono y reabrir conserva los campos del panel

#### Scenario: Disponibilidad y contextos
- **WHEN** hay cambios pendientes, un borrador sin identidad o un documento desconocido
- **THEN** se mantienen las condiciones previas: exportar requiere documento guardado y sin cambios, Markdown requiere formato editable, recursos requieren identidad para abrir y sus escrituras siguen protegidas; Adjuntos aparece sólo para tipo note y Project/Tag/Person no reciben grupos ficticios

### Requirement: Cabecera de Project basada en la referencia local

El detalle Project SHALL presentar portada decorativa local, icono/título editables y Status/Target Deadline/Progress compactos. MUST usar estado/end_date locales y progreso devuelto por Rust, con décimas, carga/error/ausencia reales sin copiar porcentajes de Notion ni calcular reglas de dominio en la UI. Inicio/propósito/procedencia SHALL quedar en View details y las relaciones de contexto abrirán sus originales con guardias existentes.

#### Scenario: Propiedades y progreso acreditado
- **WHEN** se abre un Project con resumen disponible en la consulta local
- **THEN** el porcentaje y barra reflejan el resumen Rust y editar estado/plazo sólo prepara el payload pendiente, conservando revisión y documentos

#### Scenario: Resumen ausente o fallido
- **WHEN** el Project no está en la página de resumen disponible o falla su lectura
- **THEN** se distingue no disponible de cero y de error, sin inventar progreso; la lectura fallida permite reintentar

### Requirement: Contenidos relacionados del Project separados por tipo

Project SHALL presentar Nav a Projects/Dashboard y secciones Tasks/Notes de los originales obtenidos por workflow/context. Tasks SHALL agrupar por estado local reconocido y Notes conservar títulos/tipos reales. MUST conservar paginación acumulativa con cobertura explícita, carga/error/reintento, apertura protegida y captura contextual existente sin writes al preparar. Relaciones de cabecera no duplicarán todas las Tasks/Notes y el editor de relaciones completo seguirá accesible. Temas y 420 px SHALL evitar overflow global.

#### Scenario: Páginas y captura contextual
- **WHEN** se continúa la lectura o se prepara una Task/Note desde el Project
- **THEN** se acumulan originales únicos con el cursor del núcleo o se prepara la captura vinculada sin guardarla; una página parcial no afirma ausencia total de un tipo

#### Scenario: Cambios pendientes y respuesta vieja
- **WHEN** se intenta abrir un registro con cambios pendientes o llega una lectura después de abandonar el Project
- **THEN** se conserva Guardar/Descartar/Cancelar o se descarta la respuesta anterior sin reemplazar el buffer de otra página

### Requirement: Cabecera de Tag con propiedades y contador canónico

El detalle Tag SHALL presentar portada decorativa local, icono/título, Type/Favorite editables y Note Count desde el resumen Rust. MUST conservar las cuatro clasificaciones Area/Resource/Entity/Topic sin convertirlas, mostrar carga/error/ausencia sin inventar cero, y mantener procedencia/actividad de lectura en View details. El contador SHALL permitir llegar a Notes sin alterar el borrador. PageTools y sus condiciones/teclado MUST conservarse; sin cambios de datos al abrir.

#### Scenario: Datos locales y borrador
- **WHEN** se abre un Tag con resumen disponible y se modifica Type o Favorite
- **THEN** el contador refleja la consulta canónica y los controles preparan sólo el payload pendiente; navegar mantiene Guardar/Descartar/Cancelar y no altera documentos ni relaciones

#### Scenario: Resumen y tipos no reconocidos
- **WHEN** falla el resumen, falta el Tag en la página consultada o se encuentra una clasificación no reconocida
- **THEN** se ofrece reintento o ausencia explícita sin un contador ficticio y el valor desconocido se conserva hasta una selección explícita válida

### Requirement: Contenidos del Tag con contexto preservado

Tag SHALL presentar Nav a Notes/Tags y secciones Notes/Clips de workflow/context, distinguiendo Clips sólo por el tipo local web_clip y conservando títulos/identidades. MUST mantener tareas relacionadas y su captura, otras relaciones en cabecera y el editor completo de relaciones. Las listas SHALL acumular IDs únicos con su cursor común, distinguir cobertura parcial/carga/error y usar captura contextual/apertura protegidas. Temas, teclado y 420 px SHALL conservar accesibilidad sin overflow global ni controles de jerarquía/remotos inventados.

#### Scenario: Captura y navegación
- **WHEN** se abre un registro relacionado o se prepara Note/Clip/Task desde el Tag
- **THEN** se abre su original con guardias o se prepara la captura vinculada sin escribir; Clip sólo preselecciona el tipo existente, sin descargar ni extraer contenido

#### Scenario: Páginas y errores
- **WHEN** se continúa una página, falla la lectura o llega una respuesta tras salir
- **THEN** se acumulan originales únicos, se ofrece reintento sin afirmar ausencia total o se descarta la respuesta anterior; los paneles de herramientas conservan disponibilidad y teclado

### Requirement: Detalle Goal con cabecera y hitos canónicos

Goal SHALL presentar Status/Tag/Progress compactos, fechas editables en View details, navegación protegida y sus hitos con Target Deadline y Date Completed. El progreso MUST proceder del resumen Rust existente con carga/error/reintento/ausencia explícitos, sin inventar cero cuando falta o no hay hitos. MUST conservar el editor de documento, todos sus bloques/prefijo, CRUD/CAS, acciones de hitos, proyectos originales y condiciones de PageTools, sin escribir al abrir. Overview, Additional Details y Journal SHALL usar bloques locales del mismo documento y conservar contenido anterior.

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

### Requirement: Popup Settings desde el lateral

El lateral SHALL sustituir el selector de tema, ruta y acción de cambio de
workspace por un botón Settings con icono de engranaje a la izquierda. MUST
retirar «En tu equipo» y el punto azul junto a la marca. Settings SHALL abrir un
popup modal con esos tres controles, conservando el tema local, sus errores
recuperables y la ruta original como lectura. Abrir/cerrar o cambiar tema MUST
conservar entidades, buffers, pestañas y posiciones sin escrituras de dominio.

Settings SHALL organizarse en navegación y contenido por secciones: Preferencias
para apariencia y General del workspace para ruta/cambio. La búsqueda local MUST
filtrar los ajustes disponibles, llevar a su sección, mostrar un vacío explícito
y permitir limpiar la búsqueda. La composición SHALL adaptarse al ancho estrecho
y permitir añadir categorías sin presentar controles futuros inexistentes.

El popup SHALL ofrecer cierre accesible por botón y Escape, contener el foco y
devolverlo al acceso anterior al cerrarse. MUST funcionar offline en ambos temas
y ancho estrecho, impedir atajos de navegación detrás del popup y cerrar antes
de iniciar las guardias existentes de cambio de workspace para todas las pestañas.

#### Scenario: Acceso y contenido agrupados
- **WHEN** se abre un workspace y se pulsa Settings en el pie del lateral
- **THEN** aparece un popup con Tema, ruta y Cambiar workspace; el pie contiene sólo Settings con su engranaje y la marca no tiene punto azul ni el texto retirado

#### Scenario: Tema local y recuperación de preferencia
- **WHEN** se cambia tema desde Settings, incluso con edición pendiente o almacenamiento de preferencias inaccesible
- **THEN** el tema se aplica sin escribir ni descartar la edición; se conserva su persistencia local y se muestra el error recuperable si no pudo guardarse

#### Scenario: Secciones y búsqueda local de ajustes
- **WHEN** se navega entre Preferencias y General o se busca un ajuste existente o desconocido
- **THEN** se muestran únicamente la sección y controles que coinciden, se puede limpiar un resultado vacío y no se consultan proveedores ni se modifican datos o buffers

#### Scenario: Cambio de workspace protegido
- **WHEN** se pulsa Cambiar workspace con cambios en una pestaña activa o inactiva
- **THEN** Settings se cierra y el flujo existente considera todas las pestañas; cancelar o fallar un guardado conserva workspace y contenido pendiente

#### Scenario: Teclado y ancho estrecho
- **WHEN** se usa Settings en cualquier tema o ancho estrecho y se cierra por botón o Escape
- **THEN** el foco permanece dentro mientras está abierto, los atajos no actúan sobre pestañas detrás del modal, el contenido no desborda y el foco retorna al acceso anterior disponible

### Requirement: Iconografía primaria coherente

Home, Tasks, Notes, Projects, Tags, Goals y People SHALL compartir la forma del
icono de su breadcrumb en el lateral, pestañas de sección, encabezados de Home,
páginas y accesos de creación de sección. MUST compartir el acento vigente,
conservar trazos sin rellenos contradictorios y mantener la animación sólo al
clic lateral. Los iconos y colores personalizados de registros MUST conservarse.

#### Scenario: Siete destinos y sus contextos
- **WHEN** se recorren los siete destinos y los encabezados integrados de Home, Tasks y Notes
- **THEN** cada destino usa su misma forma y color canónicos en todos esos accesos, sin cambiar los iconos originales de los registros

#### Scenario: Estado animado y reposo
- **WHEN** se pulsa un destino lateral o se pide movimiento reducido
- **THEN** el ciclo lateral retorna al mismo icono estático del breadcrumb o permanece estático, sin animar encabezados ni datos

### Requirement: Color primario local configurable

Settings → Preferences SHALL ofrecer color primario azul inicial, sugerencias
simples y una última opción personalizada con rueda circular de tono, saturación,
luminosidad y valor hexadecimal. MUST aplicarlo inmediatamente a iconos primarios,
acciones, selección, enlaces y foco en ambos temas y todas las pestañas sin
alterar buffers o datos. MUST adaptar tonos para contraste legible, persistir
solamente valores hexadecimales válidos aparte del workspace y recuperar azul
ante un valor inválido o inaccesible; un fallo al guardar conserva el color
aplicado con un aviso recuperable en inglés.

#### Scenario: Sugerencia y selección precisa
- **WHEN** se elige un color sugerido o se ajusta la rueda por puntero/teclado o un hexadecimal válido
- **THEN** la interfaz usa el nuevo acento de inmediato y Settings muestra la selección, sin consultas externas ni guardado de entidades

#### Scenario: Persistencia y entrada inválida
- **WHEN** se reabre con un valor válido, inválido o almacenamiento inaccesible, o se ingresa un hexadecimal incompleto
- **THEN** se restaura el color válido o el azul inicial; una entrada incompleta no reemplaza la selección vigente y el fallo de guardado es recuperable

#### Scenario: Temas, foco y ancho estrecho
- **WHEN** se cambia tema con una elección muy clara/oscura o se opera el selector a 420 px
- **THEN** controles y enlaces mantienen contraste, el foco es visible, la rueda admite teclado y no aparece desbordamiento global

### Requirement: Textos de interfaz en inglés

El escritorio SHALL presentar sus etiquetas, botones, nombres accesibles,
estados vacíos, ayuda y mensajes propios en inglés. MUST preservar contenidos,
títulos, valores importados e identidades del workspace en su idioma original;
la traducción de presentación no convierte valores persistentes ni altera
contratos del núcleo. Los mensajes diagnósticos originales del núcleo se
conservan como detalle cuando no hay una traducción acreditada.

#### Scenario: Interfaz y contenido en distintos idiomas
- **WHEN** se abren Home, secciones, buscadores, Settings, editor y herramientas sobre datos con títulos en español
- **THEN** los controles propios se muestran en inglés y los títulos/contenidos conservan sus valores originales

#### Scenario: Guardias conservadas
- **WHEN** una navegación o cierre requiere resolver un borrador o falla una operación
- **THEN** las acciones Save and continue, Discard y Cancel mantienen las guardias y el buffer, con estado de interfaz en inglés y diagnóstico disponible

### Requirement: Tema de apariencia del sistema

Settings SHALL ofrecer System como primera opción de Theme, junto a Dark y Light. Al seleccionar System, Noteka MUST aplicar la apariencia clara u oscura informada por el dispositivo y actualizarla mientras la aplicación esté abierta. Las opciones fijas MUST ignorar cambios del sistema. La elección SHALL persistir localmente sin cambiar datos o descartar buffers; ante fallos de almacenamiento se conserva la elección de la sesión y se informa el error. Valores anteriores de Light y Dark MUST conservarse; ante valor ausente, inválido o inaccesible se conserva el fallback oscuro.

#### Scenario: Sistema cambia en tiempo real
- **WHEN** está seleccionado System y el dispositivo cambia entre claro y oscuro
- **THEN** la interfaz y su paleta de acento cambian al tema correspondiente sin reiniciar ni alterar ediciones

#### Scenario: Tema fijo y reapertura
- **WHEN** se selecciona Light o Dark y cambia la apariencia del dispositivo o se reabre Noteka
- **THEN** se conserva el tema seleccionado independientemente del sistema

#### Scenario: Persistencia y fallo
- **WHEN** se selecciona System y se reabre la aplicación, o no se puede guardar la preferencia
- **THEN** se restaura System si se guardó; ante fallo se aplica durante la sesión y se informa el error recuperable

### Requirement: Selector local de relaciones por nombre

El escritorio SHALL permitir seleccionar proyectos, etiquetas y personas compatibles mediante un menú buscable por nombre, sin requerir IDs. MUST identificar vínculos existentes, mostrar resultados progresivos y estados de carga, vacío y error, y permitir continuar la búsqueda sin afirmar que se agotó si quedan páginas. MUST conservar el acceso a las páginas originales y permitir desvincular relaciones existentes, incluidas las entrantes, sin eliminar entidades. Los vínculos archivados SHALL identificarse. Los controles ya representados como propiedades no se duplicarán en Relations.

#### Scenario: Agregar y quitar por nombre
- **WHEN** se busca un destino compatible y se selecciona o deselecciona
- **THEN** se aplica la operación local correspondiente, se actualiza la página y se preservan las entidades

#### Scenario: Búsqueda y cobertura
- **WHEN** hay más páginas, un resultado tardío o un error de lectura
- **THEN** el menú muestra resultados y cobertura progresivos, permite continuar o reintentar y no presenta una ausencia parcial como definitiva

#### Scenario: Borrador y bloqueo
- **WHEN** una página existente tiene edición pendiente y se agrega o quita una relación, o la página está bloqueada
- **THEN** la relación se guarda independientemente sin guardar ni descartar el borrador; la página bloqueada conserva su protección

#### Scenario: Guardado posterior y conflicto de revisión
- **WHEN** se cambia una relación con un borrador pendiente y luego se guarda el contenido, o una revisión externa impide modificar la relación
- **THEN** el guardado posterior conserva la relación usando la revisión confirmada; ante conflicto se conserva el borrador y se informa el error sin sobrescribir cambios externos

#### Scenario: Teclado, conflicto y preservación
- **WHEN** se usa teclado, se cierra el selector, hay cambio de revisión o un vínculo entrante/archivado
- **THEN** el foco y los mensajes permiten operar o recuperarse, se respeta el control de revisión y no se descartan buffers ni se elimina la entidad vinculada

### Requirement: Desplazamiento suave de rueda de mouse

El escritorio SHALL suavizar los pasos verticales reconocibles de la rueda de mouse y conservar su dirección y distancia aproximada, con una preferencia local activada por defecto que permite regresar al comportamiento nativo. MUST respetar movimiento reducido y no interferir con zoom, desplazamiento horizontal, controles que administran rueda ni propagación en límites. Los eventos sin señal suficiente SHALL continuar con el comportamiento nativo; la detección no se presenta como identificación infalible del dispositivo.

#### Scenario: Pasos y reversión
- **WHEN** se desplaza una rueda reconocible o se invierte su dirección sobre un panel desplazable
- **THEN** el contenido se mueve suavemente en la dirección indicada, acumulando pasos sin saltar fuera del panel

#### Scenario: Eventos nativos y límites
- **WHEN** se usa entrada precisa no identificada, zoom, desplazamiento horizontal, un control especial o se llega al límite
- **THEN** se conserva la acción nativa y el panel exterior puede recibir el desplazamiento según sus reglas

#### Scenario: Preferencia y movimiento reducido
- **WHEN** se desactiva la opción o el sistema solicita movimiento reducido
- **THEN** el desplazamiento usa el comportamiento nativo; la preferencia local se conserva y un error de guardado se informa sin descartar la elección de sesión

### Requirement: Margen compacto de contenido de Home

Home SHALL reservar un margen reducido junto a su indicador de secciones para aprovechar el ancho disponible. El indicador, scrollbar, contenido y borde ajustable del Side peek MUST permanecer accesibles y sin superponerse a controles, también en ventana estrecha y ambos temas.

#### Scenario: Home junto a un Side peek
- **WHEN** se muestra contenido de Home con un Side peek abierto o se modifica su ancho
- **THEN** el margen lateral es más estrecho, el contenido utiliza el espacio recuperado y las marcas, scrollbar y borde siguen operables

#### Scenario: Vista estrecha y temas
- **WHEN** se reduce el ancho de la ventana o se cambia entre temas claro y oscuro
- **THEN** el contenido y el indicador conservan separación, sin overflow global ni pérdida de acceso por teclado

### Requirement: Animación visible junto al Side peek

La portada ASCII de Home SHALL continuar animándose cuando Home esté visible junto a un Side peek, aunque el foco esté en el detalle. MUST conservar la pausa cuando la portada quede fuera de vista, la pestaña o ventana estén ocultas y la composición estática con movimiento reducido. El foco, atajos y borradores de ambos paneles MUST permanecer independientes.

#### Scenario: Side peek con foco
- **WHEN** se abre y enfoca un Side peek mientras la portada ASCII de Home permanece visible
- **THEN** la animación continúa y el foco y atajos pertenecen al panel enfocado

#### Scenario: Portada o pestaña oculta
- **WHEN** la portada sale del área visible o se cambia a otra pestaña
- **THEN** se suspende la animación y vuelve a avanzar cuando Home y su portada vuelven a estar visibles

#### Scenario: Movimiento reducido y detalle cubierto
- **WHEN** el sistema pide movimiento reducido, Center peek recibe el foco o Home queda cubierto por una presentación Full
- **THEN** la portada permanece estática o suspendida, sin modificar documentos ni buffers

### Requirement: Menú de acciones compartido en detalles

Los detalles y borradores de Note, Task, Project, Tag, Person, Goal y Milestone SHALL presentar sus herramientas y acciones disponibles en un único menú `…` de cabecera. MUST conservar las capacidades por tipo, disponibilidad y guardias de borrador, sin duplicar el menú o ejecutar operaciones al abrirlo. Lock page SHALL limitarse a Note normal hasta que exista soporte en otros tipos.

#### Scenario: Páginas y herramientas
- **WHEN** se abre cualquier detalle o borrador y se despliega el menú
- **THEN** aparecen las herramientas aplicables con las mismas condiciones de disponibilidad, y Archive o Restore para entidades guardadas; no se repiten las herramientas al pie

#### Scenario: Teclado, tamaño y modos de detalle
- **WHEN** se usa el menú en un panel estrecho, Side, Center o Full y se navega con teclado
- **THEN** sus opciones y paneles se mantienen dentro del viewport y permiten apertura, navegación y cierre con retorno de foco

#### Scenario: Acciones con borrador pendiente
- **WHEN** se elige Archive o Restore con cambios pendientes
- **THEN** se conserva la guardia para guardar, descartar o cancelar y la revisión esperada de la entidad; abrir o cerrar el menú no modifica datos

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

### Requirement: Anchos ajustables y persistentes de paneles

El menú izquierdo y Side peek SHALL permitir ajustar su ancho desde el borde con puntero y teclado, conservar el ancho local elegido y recuperarlo en la próxima apertura o inicio. MUST limitar el menú a 35% y Side peek a 65% del ancho de la ventana, también al redimensionarla. Preferencias inválidas usan defaults seguros; fallos de escritura SHALL ser recuperables sin modificar documentos.

#### Scenario: Ajuste y reapertura
- **WHEN** se arrastra un borde o se ajusta por teclado, se cierra y vuelve a abrir el panel o se reinicia la aplicación
- **THEN** se recupera el ancho confirmado dentro de sus límites, independientemente del otro panel y sin crear revisiones de entidades

#### Scenario: Límites y ventana menor
- **WHEN** se arrastra más allá del máximo o se reduce la ventana con una preferencia mayor guardada
- **THEN** el menú no supera 35% y Side peek no supera 65%; el ancho preferido permanece recuperable al ampliar la ventana y no hay overflow global

#### Scenario: Cancelación o preferencia inaccesible
- **WHEN** se cancela el arrastre con Escape, se pierde el puntero o falla el almacenamiento local
- **THEN** cancelar recupera las dimensiones anteriores; un valor inválido usa fallback, un fallo de escritura se informa y la sesión sigue operable con sus borradores intactos

### Requirement: Exclusión de navegación y Side peek ancho

Un Side peek visible con ancho mayor al 45% de la ventana MUST ocultar el menú izquierdo. Abrir el menú en ese estado SHALL ocultar el peek sin cerrar, guardar, descartar ni recrear su editor. Reabrir ese mismo detalle SHALL recuperar el buffer y cerrar el menú. Abrir otro destino o abandonar la sesión MUST incluir los peeks ocultos en las guardias existentes.

#### Scenario: Cruzar el umbral
- **WHEN** el peek se amplía desde un ancho menor o igual a 45% hasta uno mayor con el menú abierto
- **THEN** el menú se contrae; a 45% exacto se permite coexistencia y reducir después no abre el menú automáticamente

#### Scenario: Alternancia conservadora
- **WHEN** se abre el menú con un peek ancho visible y luego se elige ese mismo detalle desde el origen
- **THEN** el peek se oculta y reaparece con el mismo editor, ancho, buffer e historial; al reaparecer se oculta el menú y cada panel conserva su scroll

#### Scenario: Otro destino y cierre agregado
- **WHEN** un peek oculto tiene ediciones y se elige otra entidad o se solicita cerrar pestaña, workspace o host
- **THEN** su guardia se hace visible antes de continuar; cancelar o fallar guardar conserva el buffer y no confirma el cierre

#### Scenario: Ventana o pestaña cambian
- **WHEN** cambia el viewport o se activa una pestaña con Side peek ancho
- **THEN** se vuelven a aplicar los límites y la exclusión sin intercambiar buffers ni persistir ocultamiento como contenido

### Requirement: Movimiento reversible de Side peek

Side peek SHALL expandirse y contraerse con transición breve y reversible equivalente al lateral, ajustando el espacio del origen sin escalar texto. Ocultar o animar MUST conservar la instancia del editor y retirar inmediatamente foco/atajos del contenido que se cierra. MUST omitir transición con movimiento reducido y durante el ajuste por arrastre.

#### Scenario: Abrir, ocultar e invertir
- **WHEN** se abre u oculta el peek o se invierte esa acción antes de finalizar
- **THEN** el movimiento continúa desde la posición visible, el origen se ajusta y contenido/historial/revisiones permanecen sin writes ni lecturas por animar

#### Scenario: Cierre explícito
- **WHEN** se confirma guardar o descartar y cerrar el peek
- **THEN** primero se resuelve la guardia, después se contrae y finalmente se libera su instancia; Cancel conserva el detalle visible

#### Scenario: Movimiento reducido y arrastre
- **WHEN** se usa movimiento reducido o se arrastra un borde
- **THEN** el layout sigue el destino directamente sin interpolación ni pérdida de foco, selección o contenido

### Requirement: Barras de desplazamiento discretas

El escritorio SHALL presentar barras de desplazamiento finas y discretas, compatibles con los temas claro y oscuro, sin impedir el desplazamiento ni mostrar barras ocultas deliberadamente.

#### Scenario: Superficie desplazable
- **WHEN** se desplaza una vista vertical u horizontal con contenido desbordado
- **THEN** la barra conserva funcionamiento nativo, con tirador vertical de 7 px de tinta, un margen transparente derecho de 2 px dentro de un canal de 10 px y canal horizontal de 8 px en motores compatibles, pista transparente y contraste adaptado al tema

#### Scenario: Barra oculta intencionalmente
- **WHEN** una superficie oculta su barra para presentar otra navegación
- **THEN** la regla general no vuelve a mostrarla

### Requirement: Actualización de vistas coordinada

El escritorio SHALL actualizar las vistas visibles al recuperar foco, al activarlas o ante una solicitud manual. MUST agrupar eventos repetidos por consulta, limitar a dos las lecturas simultáneas de actualización por workspace y conservar el cupo hasta que una operación iniciada termine realmente. Los cambios confirmados por otro cliente SHALL reflejarse al siguiente refresco, sin polling continuo.

#### Scenario: Ráfaga durante una lectura
- **WHEN** llegan veinte eventos equivalentes mientras una consulta está pendiente o en curso
- **THEN** las solicitudes previas a su inicio se agrupan y como máximo queda una consulta posterior para cambios ocurridos durante la operación; no se exceden dos lecturas físicas simultáneas por workspace

#### Scenario: Navegación durante una lectura
- **WHEN** una consulta queda en espera o termina después de cambiar vista, fecha, entidad, workspace o desmontar el panel
- **THEN** la lectura en espera se descarta y la respuesta tardía no publica filas, estado ni error en la identidad nueva

#### Scenario: Cambio confirmado por otro cliente
- **WHEN** CLI o MCP confirma un cambio y luego se recupera el foco o se solicita actualizar
- **THEN** la consulta vuelve a leer los originales confirmados sin escribir ni alterar revisiones

### Requirement: Resultados anteriores identificados durante refrescos

Home y las listas compatibles SHALL conservar sus resultados mientras se actualiza la misma consulta. Si falla MUST indicar que son anteriores y ofrecer reintento; al cambiar la identidad MUST limpiar los resultados previos. Una actualización exitosa SHALL volver a la primera página y avisar si se habían cargado páginas adicionales. MUST deshabilitar el cursor anterior durante la actualización y mientras los resultados estén sin verificar.

#### Scenario: Actualización exitosa con páginas adicionales
- **WHEN** una lista con varias páginas cargadas se actualiza por foco o por solicitud manual
- **THEN** mantiene las filas visibles mientras consulta, publica la primera página nueva y avisa que se pueden cargar las demás nuevamente

#### Scenario: Actualización fallida
- **WHEN** falla la lectura de la misma consulta
- **THEN** mantiene las filas anteriores identificadas como no actualizadas, presenta el error y el reintento, y no permite continuar con el cursor anterior

#### Scenario: Consulta diferente
- **WHEN** se cambia vista o fecha seleccionada mientras hay resultados o una lectura pendiente
- **THEN** no se muestran los resultados de la identidad anterior como si correspondieran a la nueva

### Requirement: Edición y tablero protegidos durante actualizaciones

El escritorio MUST conservar borradores, selección e historial de edición mientras se actualiza. Si una revisión externa difiere de un borrador MUST conservar el buffer y mostrar conflicto. Projects Board MUST deshabilitar movimientos mientras sus datos estén actualizándose o no verificados, y permitirlos nuevamente sólo cuando el tablero vigente cumpla las condiciones de integridad y cobertura existentes.

#### Scenario: Conflicto de edición externo
- **WHEN** otro cliente modifica una entidad con cambios locales pendientes y la ventana recupera foco
- **THEN** el borrador permanece intacto y se informa el conflicto sin sobrescribir ninguna revisión

#### Scenario: Tablero pendiente o con error
- **WHEN** el tablero se actualiza o falla su lectura
- **THEN** las tarjetas anteriores pueden seguir visibles identificadas como anteriores, pero sus movimientos permanecen deshabilitados hasta obtener datos vigentes y completos

### Requirement: Reapertura automática del workspace

La interfaz MUST abrir automáticamente al iniciar el último workspace usado y SHALL conservar su ruta como preferencia local. Si no hay preferencia o la ruta guardada ya no contiene un workspace, MUST intentar el directorio de datos predeterminado. MUST mostrar la selección explícita sólo cuando no exista ningún workspace, la apertura falle o el usuario pida Change workspace.

#### Scenario: Apertura automática

- **WHEN** se inicia la aplicación y existe un workspace guardado o el del directorio de datos predeterminado
- **THEN** la interfaz abre ese workspace sin pedir una ruta y muestra Home con su contenido

#### Scenario: Preferencia persistente

- **WHEN** se crea o abre un workspace con éxito
- **THEN** su ruta se guarda como preferencia local y el siguiente arranque la reutiliza

#### Scenario: Ruta guardada no disponible

- **WHEN** la preferencia apunta a una ruta que ya no contiene un workspace
- **THEN** se intenta el directorio de datos predeterminado, sin presentar datos de otro workspace como propios

#### Scenario: Primer uso

- **WHEN** se inicia la aplicación y no existe ningún workspace en la preferencia ni en el directorio de datos
- **THEN** ofrece crear o abrir uno y muestra su ruta después de seleccionarlo

#### Scenario: Cambio explícito de workspace

- **WHEN** el usuario elige Change workspace en Settings
- **THEN** vuelve a la pantalla de selección y su elección se guarda para el siguiente arranque
