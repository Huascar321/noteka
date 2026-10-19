# local-extraction Specification

## Purpose

Procesar originales administrados mediante trabajos locales separados de la edición, conservando derivados regenerables con procedencia y sin reemplazar el contenido de usuario.

## Requirements

### Requirement: Selección explícita de PDF textual acotado

El extractor pdf_text v1 SHALL admitir originales administrados PDF 1.0–1.7 .pdf no cifrados, de hasta 1 MiB y 1–100 páginas, con texto directo Type1 latino estándar en Standard/WinAnsi. MUST rechazar fuentes, mappings o estructuras textuales no soportados sin conversión con pérdida ni éxito parcial. Encolar no parsea ni ejecuta; importar, abrir o leer no inicia el helper.

#### Scenario: PDF seleccionado offline
- **WHEN** se importa un PDF sintético elegible, se encola dos veces con pdf_text, se habilita la cola y se ejecuta con red denegada
- **THEN** hay un único trabajo y un único intento explícito, con texto por página y sin UI, modelos, llamadas externas ni cambios del original

#### Scenario: Lecturas y cola pausada
- **WHEN** se abre, importa, encola o consulta el PDF y se pide run con la cola pausada
- **THEN** no se parsea ni inicia un helper; se conserva queued y no aumentan intentos por leer o por run pausado

#### Scenario: Nombre firma o tamaño no elegibles
- **WHEN** el nombre/MIME no es elegible o el original excede 1 MiB, o durante la ejecución no tiene cabecera PDF admitida o está truncado
- **THEN** los metadatos inválidos se rechazan antes de encolar y el fallo de contenido se identifica en el intento ejecutado, sin fragmentos publicados ni cambio del original

#### Scenario: Cifrado y conversión no admitida
- **WHEN** el PDF está cifrado incluso con contraseña vacía, usa una fuente/encoding no admitidos, contiene glyphs sin mapping o exige recuperación con pérdida
- **THEN** el intento falla con motivo explícito, sin pedir contraseña, usar fallback ni publicar texto parcial como completo

#### Scenario: Contenido activo es dato no autorización
- **WHEN** un PDF contiene enlaces, JavaScript, acciones, instrucciones o referencias a archivos externos
- **THEN** no se evalúan, abren, ejecutan ni descargan; sólo se procesa el subconjunto textual declarado y el contenido no autoriza herramientas

### Requirement: Cobertura PDF y localizadores de texto derivado

La generación PDF SHALL conservar un manifiesto de todas las páginas físicas 1-based, su estado text/no_text, bytes y hash del texto derivado. Cada fragmento MUST localizar un rango UTF-8 dentro del texto de su página, sin cruzar páginas ni presentarlo como offset del PDF. Complete significa procesar todas las páginas del subconjunto declarado; MUST explicitar ausencia de OCR, orden visual, geometría y tablas estructuradas.

#### Scenario: Páginas Unicode y fragmentación
- **WHEN** un PDF admitido contiene texto WinAnsi multibyte al decodificar, saltos y una página mayor que 4.096 bytes
- **THEN** sus fragmentos reconstruyen el texto derivado de cada página sin cortar caracteres, con rangos continuos por página y ordinales globales deterministas

#### Scenario: Páginas vacías escaneadas y mixtas
- **WHEN** páginas válidas no tienen texto directo, sólo imágenes, o combinan texto admitido e imágenes
- **THEN** el manifiesto registra todas las páginas, identifica no_text sin inventar texto y explicita que imágenes no fueron transcritas; cero fragmentos puede ser un éxito de cobertura del extractor, no prueba de documento visual vacío

#### Scenario: Estructura textual omitible
- **WHEN** una página requiere texto de Form XObjects, formularios, annotations o una estructura textual fuera del subconjunto soportado
- **THEN** no se omite silenciosamente para certificar transcripción completa; el trabajo falla con motivo de soporte, conservando originales y una generación previa

#### Scenario: Fuente y resultado son medidas diferentes
- **WHEN** el tamaño del texto extraído difiere del tamaño del PDF
- **THEN** source_bytes/covered_bytes describen la fuente íntegra verificada y el manifiesto expone extracted_bytes del texto; los offsets y hashes de página corresponden sólo al texto derivado

### Requirement: Contención del parser y conservación ante interrupción

La extracción PDF MUST aislar fallos del parser del cliente y acotar vida, heap y comunicación del helper. SHALL respetar ownership/deadline existentes, terminarlo ante pausa, cancelación o pérdida de ownership y no dejarlo residente tras caída del padre. Crash, exceso o resultado inválido impiden publicación; se conservan resultados previos. No genera core dumps ni expone objetos PDF en errores o logs.

#### Scenario: Crash o agotamiento de recursos
- **WHEN** el parser aborta, supera 128 MiB de heap vivo, un límite de descompresión o el presupuesto de ejecución de 5 segundos
- **THEN** el cliente sobrevive y devuelve un fallo identificable; no aparecen core dumps, una generación parcial ni un helper persistente

#### Scenario: Resultado de helper inválido
- **WHEN** el helper produce protocolo/version/hash incorrectos, páginas/rangos incoherentes, stdout mayor a 8 MiB o stderr mayor a 8 KiB
- **THEN** el núcleo rechaza el resultado completo y termina/recoge el proceso sin propagar contenido del parser a stdout público o al error persistido

#### Scenario: Pausa cancelación y fencing durante parsing
- **WHEN** otro cliente pausa, cancela, cambia control o invalida ownership mientras se procesa PDF
- **THEN** el ejecutor detiene el helper, no publica con token anterior y conserva queued/cancelled o el estado más reciente pertinente y la generación previa

#### Scenario: Caída del padre y recuperación explícita
- **WHEN** el cliente ejecutor muere durante parsing y luego se solicita recuperación con lease vencido o desde otra ruta
- **THEN** el helper termina por su propio presupuesto, el intento heredado se recupera sin retry automático y ningún resultado tardío se confirma; reintentar exige revisión vigente

#### Scenario: Publicación PDF cercada y atómica
- **WHEN** otra conexión lee durante publicación válida, o cambia el hash del original o falla persistencia antes del commit
- **THEN** se observa una generación anterior o una nueva completa con manifiesto y fragmentos coherentes; el fallo/alteración impide reemplazar resultados previos y no cambia entidades, FTS o historial

### Requirement: Lectura PDF verificable sin reprocesamiento

La lectura PDF SHALL devolver generación, manifiesto completo, localizadores y fragmentos paginados con límite 1–64, default 32 y cursor scoped a operación/workspace/ruta/trabajo/generación/límite. MUST revalidar la fuente y la integridad del derivado antes de exponer texto vigente, sin lanzar parser ni modificar datos. Un manifiesto completo no convierte una página parcial de fragmentos en cobertura completa.

#### Scenario: Paginación y regeneración
- **WHEN** se recorren varias páginas de fragmentos y una regeneración reemplaza la generación antes de continuar
- **THEN** cada respuesta distingue manifiesto completo de covers_generation parcial y el cursor viejo falla sin mezclar generaciones

#### Scenario: Fuente o derivado alterados
- **WHEN** el original desaparece/cambia o texto, rangos, hashes o manifiesto persistidos son incoherentes
- **THEN** no se devuelve ese texto como vigente; se identifica input_stale o integridad inválida sin repararlo, parsearlo ni borrar resultados

#### Scenario: Cursor ajeno y lecturas puras
- **WHEN** se usa un cursor de otra operación/ruta/workspace/job o se consulta PDF tras restore
- **THEN** el cursor ajeno falla y una lectura válida conserva revisiones, estados, intentos, control, ownership, originales e historial, sin ejecutar el helper

### Requirement: Selección local explícita y ejecución separada

El sistema SHALL encolar extracción sólo por selección explícita de un adjunto administrado y un extractor soportado. MUST separar encolado de ejecución, comenzar pausado y funcionar offline sin modelo, cuenta ni ventana. Importar, abrir el workspace o consultar trabajos MUST NOT ejecutar extracción ni iniciar un proceso residente.

#### Scenario: Importación sin procesamiento implícito
- **WHEN** se importa un adjunto, se abre el workspace y se consulta la cola sin solicitar extracción
- **THEN** no aparece un trabajo nuevo ni se extrae contenido o inicia un worker

#### Scenario: Trabajo seleccionado sin red
- **WHEN** un cliente encola un adjunto elegible, habilita la cola y solicita ejecutar con red denegada
- **THEN** el trabajo se procesa localmente sin iniciar la UI ni usar modelos o proveedores

### Requirement: Identidad incremental y trazabilidad del trabajo

Cada trabajo SHALL registrar fuente/hash, extractor/configuración versionados, estado, revisión, intentos, timestamps y error identificable; no tendrá perfil de modelo. MUST deduplicar una fuente/configuración iguales sin volver a ejecutar ni perder resultados. Estados conocidos: queued, running, succeeded, failed, cancelled y stale.

#### Scenario: Encolado repetido
- **WHEN** se encola dos veces la misma fuente y configuración, incluso desde clientes concurrentes
- **THEN** se devuelve un único ID de trabajo sin duplicar intentos, resultados o revisiones

#### Scenario: Regeneración explícita
- **WHEN** se reintenta un trabajo terminado con revisión vigente
- **THEN** queda queued con el mismo ID, conserva el resultado previo hasta otra publicación válida y sólo incrementa intentos al comenzar otra ejecución

#### Scenario: Configuración persistida futura
- **WHEN** un trabajo contiene una versión/configuración que el cliente no soporta
- **THEN** la consulta identifica su incompatibilidad conservando los datos crudos; ejecutarlo o reintentarlo no los transforma ni descarta

### Requirement: Extracción UTF-8 literal y localizada

El extractor utf8_text v1 SHALL admitir originales `.txt` y `.md` de hasta 1 MiB, con UTF-8 válido y sin NUL, excluyendo medios binarios reconocidos. MUST conservar bytes/texto literal, producir fragmentos sin cortar caracteres y exponer offsets y líneas del original. No interpreta marcado, URLs, código o instrucciones del contenido, ni selecciona PDF automáticamente.

#### Scenario: Unicode y saltos de línea
- **WHEN** se procesa texto con caracteres multibyte, BOM, CRLF y una línea mayor que un fragmento
- **THEN** los fragmentos reconstruyen exactamente el texto y sus rangos de bytes/líneas apuntan al original sin normalizarlo

#### Scenario: Contenido que parece una instrucción
- **WHEN** el texto contiene HTML, enlaces remotos, comandos o instrucciones para un agente
- **THEN** permanece texto literal; no se evalúa, ejecuta ni descarga contenido

#### Scenario: Formato o codificación no admitidos
- **WHEN** se solicita utf8_text sobre PDF, imagen, Excel, un nombre no elegible, UTF-8 inválido o texto con NUL
- **THEN** se informa el motivo explícito sin OCR, conversión con pérdida, fragmentos parciales ni modificación del original

#### Scenario: Original vacío
- **WHEN** se procesa un archivo elegible de cero bytes
- **THEN** termina succeeded con cero fragmentos y cobertura vacía explícita, no como un fallo ni texto inventado

### Requirement: Ejecución cooperativa con ownership acotado

La ejecución SHALL procesar como máximo un trabajo por llamada, liberar el lock de escritura durante lectura/extracción y utilizar ownership exclusivo, scope y vencimiento verificables. MUST acotar contención y ejecución; un worker vencido, cancelado o de otra ruta no puede publicar. Recuperar un trabajo interrumpido no lo reintenta automáticamente.

#### Scenario: Dos ejecutores independientes
- **WHEN** dos procesos intentan reclamar trabajo simultáneamente
- **THEN** sólo uno obtiene ownership activo; el otro no ejecuta ni publica el mismo trabajo, y otro cliente puede seguir editando mientras se extrae

#### Scenario: Worker interrumpido y publicación tardía
- **WHEN** un worker muere o vence y una ejecución explícita recupera su estado
- **THEN** queda failed con motivo recuperable; un token anterior no puede confirmar resultados y se requiere retry explícito

#### Scenario: Deadline o contención
- **WHEN** se agota la ejecución cooperativa o otro escritor supera 2.000 ms de bloqueo
- **THEN** termina con error recuperable, no se publica una generación parcial y no se ignoran límites para confirmar

### Requirement: Pausa cancelación y reintento con revisión

El sistema SHALL permitir pausar/reanudar la cola y cancelar o reintentar trabajos con revisión esperada. MUST rechazar revisiones obsoletas sin rebase. Pausar impide nuevos claims y detiene cooperativamente el activo sin perder su resultado previo; cancelar invalida ownership y retry no crea resultados hasta ejecutar nuevamente.

#### Scenario: Cola pausada
- **WHEN** se pausa con revisión vigente y luego se solicita ejecución
- **THEN** no se reclama un trabajo nuevo; un activo observa la pausa, se retira sin publicar y queda queued para una ejecución explícita posterior

#### Scenario: Cancelación durante extracción
- **WHEN** otro cliente cancela un queued o running con revisión vigente
- **THEN** queda cancelled, el worker anterior no publica y los originales y una generación previa válida permanecen disponibles

#### Scenario: Revisión antigua
- **WHEN** otro cliente ya cambió la revisión del trabajo o del control de cola
- **THEN** la modificación con revisión vieja devuelve revision_conflict sin sobrescritura ni reintento automático

#### Scenario: Reintento luego de un fallo
- **WHEN** el cliente reintenta failed, cancelled, stale o succeeded con revisión vigente
- **THEN** queda queued; un running no admite retry y una cancelación ya vigente no genera una nueva revisión

### Requirement: Publicación atómica de una generación

El sistema MUST confirmar juntos fragmentos, cobertura, generación y estado succeeded, después de revalidar fuente, configuración, revisión y ownership. Cualquier fallo SHALL impedir una publicación parcial y conservar una generación previa válida. Los derivados no modifican entidades, revisiones de notas, referencias, FTS textual ni historial de entidades.

#### Scenario: Generación completa observable
- **WHEN** termina un trabajo válido mientras otra conexión consulta sus fragmentos
- **THEN** el lector ve una generación completa anterior o la nueva, nunca fragmentos mezclados con un estado succeeded anticipado

#### Scenario: Falla antes de publicar
- **WHEN** falla un fragmento, validación o persistencia antes del commit
- **THEN** no se publican fragmentos parciales ni un éxito; el error/estado recuperable puede consultarse sin alterar el original

#### Scenario: Regeneración fallida
- **WHEN** falla una nueva ejecución de un trabajo que tenía resultados válidos
- **THEN** no se sustituye la generación previa por resultados incompletos; su antigüedad y condición de resultado previo quedan visibles

### Requirement: Lectura verificable y paginada de derivados

La lectura SHALL devolver fuente/hash, extractor/configuración, generación y localizador por fragmento, con paginación limitada y cursor ligado a workspace/ruta/trabajo/generación. MUST revalidar el original antes de presentar texto como vigente; una fuente alterada se identifica como stale sin reescribirla. No atribuye el fragmento a una nota que ya no lo referencia.

#### Scenario: Fuente alterada o ausente
- **WHEN** el original desaparece, cambia su hash/tamaño o se sustituye por una ruta no admitida
- **THEN** no se devuelve texto derivado como vigente y una ejecución detecta stale o fallo de almacenamiento sin reparar el original

#### Scenario: Página y cambio de generación
- **WHEN** se consulta más de una página y una regeneración publica antes de continuar
- **THEN** el cursor antiguo se rechaza; no mezcla generaciones ni afirma cobertura completa de una página parcial

#### Scenario: Ruta ajena y lecturas sin efectos
- **WHEN** se usa un cursor de otra ruta/trabajo o se consultan metadatos/fragmentos sin ejecución
- **THEN** el cursor ajeno falla y la consulta válida no cambia estado, intentos, control, entidades ni historial

### Requirement: Presupuestos explícitos y errores útiles

La cola SHALL publicar y verificar límites de fuentes, jobs, fragmentos, almacenamiento derivado, páginas, respuestas y ejecución. MUST rechazar exceso antes de publicar, sin truncar un éxito como completo ni exponer contenido íntegro en errores. No sube los límites vigentes de importación, transporte o batches para acomodar derivados. Los 32 MiB suman texto UTF-8/PDF y JSON canónico XLSX/OCR.

#### Scenario: Fronteras y backpressure
- **WHEN** se alcanza un límite de 1 MiB de fuente, 1.000 queued/running, 10.000 trabajos o 32 MiB de texto derivado retenido
- **THEN** la frontera permitida se acepta y el exceso produce un motivo explícito, sin crear trabajos o resultados parciales

#### Scenario: Respuesta limitada
- **WHEN** una lectura pide un límite inválido o una respuesta completa supera 2 MiB
- **THEN** se rechaza o pagina conforme al contrato, sin ocultar cobertura, cursores, texto omitido o errores

#### Scenario: Límites PDF y presupuesto mixto
- **WHEN** un PDF excede 100 páginas, 1 MiB de texto derivado, 512 fragmentos, 4.096 bytes por fragmento o el presupuesto retenido junto con texto UTF-8
- **THEN** la frontera válida se acepta y el exceso rechaza la generación sin purga ni truncamiento; la regeneración descuenta sólo su resultado previo reemplazable y no usa el tamaño comprimido del PDF para contar texto

### Requirement: Lector XLSX independiente y sin activación implícita

El sistema SHALL ofrecer un lector local aislado de XLSX Transitional mediante un protocolo privado acotado de bytes, sin rutas ni acceso al workspace. MUST separar ese helper de las operaciones del core: importar, abrir, leer, encolar, indexar, buscar o migrar no inicia parsing; sólo `extraction.run` explícito sobre un trabajo admitido lo ejecuta. Index/search usan derivados ya confirmados, no el contenedor XLSX.

#### Scenario: Lectura explícita offline
- **WHEN** se invoca el helper con un XLSX sintético admitido y red denegada
- **THEN** devuelve una representación determinista completa del subconjunto declarado, sin modificar archivos ni iniciar UI, modelos o conexiones

#### Scenario: Producto existente conservado
- **WHEN** se consulta el catálogo, abre un workspace actual o importa/encola/lee/indexa/busca XLSX sin ejecutar run
- **THEN** el catálogo y los contratos vigentes declarados en core-operations, incluido extraction.xlsx_cells, se conservan; index/search sólo evolucionan mediante cambios verificados y UTF-8/PDF/Home mantienen su comportamiento sin iniciar un helper

### Requirement: Contenedor XLSX verificable y cerrado

El lector MUST verificar ZIP, CRC, nombres únicos, estructura XML y relaciones internas antes de emitir éxito. SHALL rechazar cifrado, macros, rutas ambiguas, ZIP64/multidisco, métodos distintos de Stored/Deflate, XML con DTD y OOXML fuera del subconjunto declarado. No extrae entradas a disco ni resuelve entidades o relaciones externas.

#### Scenario: Contenedor malformado o ambiguo
- **WHEN** hay truncamiento, CRC incorrecto, entradas duplicadas/solapadas, rutas con traversal, encabezados inconsistentes o partes requeridas ausentes
- **THEN** falla de forma identificable sin representación parcial ni reparación tolerante

#### Scenario: Contenido activo o externo
- **WHEN** el paquete declara macros, cifrado, conexiones, enlaces externos o relaciones con TargetMode externo
- **THEN** se rechaza sin ejecutar, abrir ni descargar su contenido y sin devolver datos del archivo en errores

#### Scenario: XML hostil o variante no admitida
- **WHEN** hay DTD, entidades personalizadas, codificación no UTF-8, namespace Strict o contenido alternativo que podría sustituir celdas
- **THEN** se identifica formato no admitido o XML inválido, sin fallback con pérdida

### Requirement: Celdas XLSX tipadas sin pérdida numérica

El lector SHALL conservar hojas en orden, nombres/visibilidad, coordenadas dispersas, rangos declarados/observados, merges, sistema de fechas y estilos numéricos. MUST separar ausencia de valor, texto, número léxico, booleano, error y fecha ISO; conservar Unicode/espacios sin convertir números a f64, seriales a fechas ni inferir encabezados.

#### Scenario: Números fechas y valores vacíos
- **WHEN** hay decimales largos, exponentes, seriales de fecha, booleanos, errores, celdas sin valor y cadenas vacías
- **THEN** los lexemas numéricos permanecen exactos y cada tipo/ausencia se distingue con su estilo y sistema de fechas; no se presenta un serial como fecha calculada

#### Scenario: Cadenas y hojas dispersas
- **WHEN** hay shared strings, texto inline/rich-text, Unicode, hojas ocultas, merges y una celda XFD1048576
- **THEN** se conserva el texto lógico y los localizadores sin expandir la grilla o los merges; el rango observado se calcula de las celdas, no de la dimensión declarada

#### Scenario: Celdas o referencias incoherentes
- **WHEN** existen coordenadas duplicadas/desordenadas, filas incompatibles, índices de strings/estilos inválidos o tipos/contenido de celda no soportados
- **THEN** el libro se rechaza sin omitir esas celdas ni inventar valores

### Requirement: Fórmulas XLSX y caché diferenciadas

El lector MUST conservar separadamente la fórmula literal, su tipo normal/shared/array, referencia e índice compartido y el valor guardado opcional. SHALL validar grupos compartidos sin expandirlos ni evaluar fórmulas. El resultado explicita que la caché puede faltar o estar obsoleta y no reproduce cálculo ni formato visual de Excel.

#### Scenario: Fórmula con caché y sin caché
- **WHEN** una celda contiene fórmula con resultado guardado, sin resultado, o una fórmula array
- **THEN** conserva fórmula/metadatos y caché tipada opcional sin afirmar que esté recalculada o sea vigente

#### Scenario: Fórmula compartida
- **WHEN** un grupo tiene ancla y seguidores válidos o carece de ancla/rango coherente
- **THEN** el caso válido retiene metadatos originales sin fabricar fórmulas expandidas y el incoherente falla completo

#### Scenario: Fórmula no admitida
- **WHEN** se encuentra fórmula data-table u otro tipo no soportado
- **THEN** se devuelve una limitación explícita sin omitirla, ejecutarla o reemplazarla por su caché

### Requirement: Presupuestos del lector XLSX

El lector SHALL limitar fuente a 1 MiB, ZIP a 256 entradas/4 MiB por entrada/16 MiB expandido, XML a 100.000 nodos/profundidad 64, hojas a 32 y celdas totales a 10.000. MUST limitar cada texto/fórmula a 131.072 bytes, representación a 1 MiB y stdout a 8 MiB, sin truncar éxito. Otros límites estructurales se documentan y verifican.

#### Scenario: Fronteras y expansión
- **WHEN** una entrada, representación, estructura o conteo alcanza su límite o lo excede, incluido un ZIP comprimido pequeño de expansión excesiva
- **THEN** la frontera admitida se conserva y el exceso falla sin asignación proporcional a dimensiones declaradas ni resultado parcial

#### Scenario: Exceso de respuesta
- **WHEN** la representación completa excede el presupuesto o una respuesta no respeta versión/hash/esquema/límites
- **THEN** no se emite ni acepta como éxito completo y el motivo no contiene valores ni fórmulas del archivo

### Requirement: Vida acotada del helper de lectura

El helper MUST terminar tras una lectura, fallo, panic o exceso; limitar heap vivo solicitado a 128 MiB, desactivar core dumps y acotar CPU a 4 segundos y vida a 5 segundos desde inicio, incluso bloqueado en pipes o sin padre. SHALL permitir acortar, nunca extender, el presupuesto inicial; PDF conserva su contención y protocolo previos.

#### Scenario: Pipes y caída del consumidor
- **WHEN** stdin queda abierto/incompleto, stdout se bloquea o desaparece el consumidor
- **THEN** el helper termina dentro del presupuesto sin quedar residente ni emitir datos del archivo en stderr

#### Scenario: Fallos de memoria panic y presupuesto
- **WHEN** se excede heap/CPU/deadline o hay panic
- **THEN** el proceso termina con código identificable, sin core dump, log sensible ni afectar al cliente

#### Scenario: Lecturas concurrentes y regresión PDF
- **WHEN** se ejecutan lectores independientes simultáneos o se procesa el corpus PDF previo tras compartir el runtime
- **THEN** cada proceso conserva su presupuesto/resultado y PDF mantiene admisión, salida, límites y contención existentes

### Requirement: Selección explícita de XLSX administrado

El extractor `xlsx_cells` v1 SHALL admitir sólo originales administrados `.xlsx` hasta 1 MiB con MIME OOXML de workbook u octet-stream y configuración v1 predeterminada. MUST usar el lector Transitional verificado, sin fallback a texto/PDF ni conversión numérica, y conservar pausa inicial, deduplicación, cancelación, retry y generaciones anteriores de la cola.

#### Scenario: Extracción XLSX offline
- **WHEN** se importa un libro sintético admitido, encola dos veces con xlsx_cells, habilita la cola y ejecuta un trabajo con red denegada
- **THEN** hay un único job y un intento explícito con representación completa tipada, original idéntico y sin modelos, UI o conexiones

#### Scenario: Admisión y parser rechazados
- **WHEN** la extensión/MIME/configuración no son admitidos o el lector rechaza ZIP/XML/celdas/fórmulas/límites
- **THEN** se devuelve un motivo identificable, no se publica éxito parcial y los errores no contienen valores ni fórmulas del libro

#### Scenario: Cola pausada y regeneración fallida
- **WHEN** se pide run pausado o se regenera un resultado y falla el intento
- **THEN** la pausa no inicia el helper y la falla conserva la generación previa identificada como resultado anterior, sin purgar originales u otros derivados

### Requirement: Publicación XLSX cercada y presupuesto compartido

El worker MUST verificar ownership, lease, revisión de control/job, configuración y hash fuente antes y durante publicación atómica. SHALL ejecutar el helper fuera del lock de escritor, acotar ejecución a 5 s, stdout/stderr y heap/CPU del helper sin ampliar límites, y matar/recolectar el hijo ante deadline, cancelación o pérdida de fence. Los derivados UTF-8/PDF/XLSX suman al mismo límite 32 MiB, descontando sólo el resultado del mismo job que se reemplaza.

#### Scenario: Pause cancel y fuente cambiada
- **WHEN** se pausa/cancela el job, cambia la fuente/configuración/ownership o vence el lease tras claim y antes de commit
- **THEN** no se confirma la nueva generación ni se pierde la anterior y no queda helper residente

#### Scenario: Respuesta y proceso no confiables
- **WHEN** el hijo cuelga, inunda pipes, cae, devuelve hash/versión/esquema inválidos o desaparece su ejecutable
- **THEN** el intento termina con error acotado y saneado, sin publicar y sin bloquear lecturas existentes

#### Scenario: Frontera del presupuesto agregado
- **WHEN** UTF-8, PDF y representación XLSX retenidos alcanzan 32 MiB o lo exceden en un byte
- **THEN** la frontera válida se publica y el exceso falla sin purga ni truncamiento; el tamaño XLSX contado es JSON canónico, no ZIP ni número de celdas

### Requirement: Lectura paginada de celdas XLSX con procedencia

`extraction.xlsx_cells` SHALL verificar manifiesto, chunks, hash, representación y original completos antes de paginar 1–64 celdas (32 por defecto), en <=2 MiB. MUST incluir procedencia, parser/limitaciones, sistema de fechas, estilos y hojas ordenadas con visibilidad/rangos/merges/conteos, ordinal global y localizador hoja/A1 por celda. Cursores se ligan a scope/operación/job/generación/límite; chunks no son celdas ni offsets del ZIP.

#### Scenario: Valores fórmulas y celdas largas
- **WHEN** se pagina un libro disperso con decimales largos, texto Unicode mayor a 4 KiB, shared/array formulas y caché ausente/obsoleta
- **THEN** cada celda sale completa y tipada, sin convertir números a float/fechas ni recalcular, expandir grillas o inventar encabezados; la concatenación de páginas coincide con el golden independiente

#### Scenario: Libro sin celdas y metadatos
- **WHEN** el libro admitido tiene hojas vacías/ocultas o rangos/merges sin celdas
- **THEN** el lector devuelve el manifiesto completo con cero items y sin cursor, distinguiendo ausencia de valor de cadenas vacías

#### Scenario: Cursor inválido o generación reemplazada
- **WHEN** se reutiliza un cursor con otro workspace/job/límite/operación o después de reemplazar su generación
- **THEN** se rechaza sin mezclar páginas ni devolver datos como actuales

#### Scenario: Corrupción fuera de la página
- **WHEN** se altera un chunk/ordinal/hash/conteo/metadato/propietario o el original, incluso fuera de la página pedida
- **THEN** la lectura falla completa y backup/integridad rechazan derivados inconsistentes sin repararlos automáticamente

### Requirement: XLSX excluido de indexación textual implícita

El sistema MUST excluir selección XLSX implícita al importar, abrir, migrar, extraer o leer. SHALL admitir put explícito de xlsx_cells v1 íntegro dentro de cuotas, mediante campos tipados sin interpretarlo como UTF-8/PDF. Extractores/configuraciones futuros se rechazan sin descartarlos. Lectores, representación, búsqueda de notas/UTF-8/PDF, validación completa de postings/copias/fuentes, caps y target query p95 <=300 ms sobre los corpus fijados se conservan.

#### Scenario: Intento de indexar XLSX
- **WHEN** se intenta indexar explícitamente una generación XLSX existente junto a un índice UTF-8/PDF válido
- **THEN** el caso íntegro y dentro de cuotas añade sólo su selección; un caso incompatible, corrupto o excesivo falla sin cambiar el índice, búsqueda u originales previos ni ejecutar helpers

#### Scenario: Regresión de búsqueda y contratos
- **WHEN** se repiten el corpus/gate fijo y la extracción/lectura UTF-8/PDF/XLSX después de habilitar búsqueda XLSX
- **THEN** pasan validaciones completas y target sin subir caps u omitir fuentes; sólo los enums/DTOs de index/search y el descriptor de capacidades evolucionan de forma documentada, sin cambiar los otros 50 contratos ni los tres lectores

### Requirement: Selección de adjunto administrado desde la nota

El escritorio SHALL ofrecer preparación separada de Search desde la nota actual, con referencias de archivo/imagen e ingreso explícito de ID administrado. MUST distinguir rótulos del bloque de metadatos de archivo y no importar, consultar, descargar originales ni procesar al montar o cambiar selección.

#### Scenario: Referencias compartidas y selección manual
- **WHEN** una nota referencia dos veces un adjunto o se ingresa un ID de un adjunto sin referencia
- **THEN** se puede elegir el ID sin inventar dueño/nombre y sin duplicar la referencia ni alterar el documento

#### Scenario: Apertura y cambio sin efectos
- **WHEN** se abre preparación, se cambia el adjunto o se alterna Search
- **THEN** no se encola, ejecuta, indexa, repara ni descarga el original y el buffer se conserva

### Requirement: Consulta explícita de procesamiento en escritorio

El panel SHALL consultar trabajos filtrados por adjunto en páginas reemplazables de 20, y consultar por ID el trabajo elegido, control global y selección del índice. MUST mostrar estados, revisión, intentos, soporte, hashes, configuración, generación anterior y errores del núcleo como snapshots separados, sin polling ni nombres inferidos.

#### Scenario: Página de trabajos y trabajo elegido
- **WHEN** se consultan más de veinte trabajos y se elige uno
- **THEN** siguiente reemplaza la página y la inspección muestra identidad/procedencia, soporte y resultado previo del trabajo, control e índice sin afirmar un snapshot conjunto

#### Scenario: Incompatible vacío o error
- **WHEN** no hay trabajos, hay configuración futura o falla una consulta
- **THEN** se distingue vacío de error/incompatibilidad, se conserva razón/código y no se representa información anterior como consulta vigente ni se modifica configuración

### Requirement: Acciones de cola independientes en escritorio

El panel SHALL encolar sólo con adjunto y extractor elegidos, ejecutar sólo el trabajo elegido y ofrecer pausa/reanudación global, cancelación y reencolado como acciones independientes. MUST reutilizar operaciones Rust y sus revisiones observadas, mostrar deduplicación/outcome/razón/recuperaciones y no ejecutar después de encolar, reanudar o reintentar.

#### Scenario: Extracción offline paso a paso
- **WHEN** se encola UTF-8, PDF o XLSX admitido, se reanuda y luego se ejecuta explícitamente su ID
- **THEN** cada paso tiene su propio resultado y sólo ejecutar aumenta intentos; el original y las entidades permanecen iguales y la generación no se selecciona para búsqueda automáticamente

#### Scenario: Pausa cancelación y reencolado
- **WHEN** se pausa/reanuda, cancela o reencola con revisión observada
- **THEN** se aplica sólo esa operación, resume no ejecuta y retry conserva generación previa; run informa que puede recuperar intentos vencidos según el núcleo

#### Scenario: Conflicto y resultado incierto
- **WHEN** otro cliente cambia una revisión o falla el transporte de una escritura
- **THEN** no hay rebase/repetición automática; se exige consultar antes de una nueva decisión y se aclara que el estado puede haber cambiado

### Requirement: Selección de generación observada en escritorio

El panel SHALL permitir seleccionar explícitamente una generación compatible observada, usando job_id, expected_generation_id y revisión observada del índice. MUST mostrar identidad/cobertura/selección anterior y conservar validación completa, cuotas y errores Rust, sin reset, parsing ni encadenamiento con búsqueda.

#### Scenario: Selección compatible sin ejecutar
- **WHEN** el usuario consulta un trabajo con generación compatible y la selecciona
- **THEN** put usa ambos CAS, muestra changed/revisión/entrada y no ejecuta helpers ni busca; cero fragmentos se distingue de ausencia de generación

#### Scenario: Generación o índice concurrentes
- **WHEN** otro cliente reemplaza la generación o cambia el índice antes de put
- **THEN** el conflicto no selecciona una versión distinta ni reintenta y pide nueva consulta explícita

### Requirement: Lifetime y cupo físico de preparación

El escritorio MUST descartar resultados tardíos al cambiar adjunto, trabajo, nota o workspace y acotar a una operación física de preparación simultánea entre montajes. SHALL mantener el cupo hasta completar la llamada iniciada, cancelar sólo esperas en cola y nunca convertir desmontar en cancelación del worker ni repetir una mutación.

#### Scenario: Lectura tardía y cambio de contexto
- **WHEN** una consulta finaliza después de cambiar identidad o desmontar el panel
- **THEN** no publica datos/errores viejos ni modifica el buffer y conserva el cupo físico hasta terminar

#### Scenario: Escritura tardía o encolada
- **WHEN** se abandona el panel durante una escritura o una espera todavía no iniciada
- **THEN** la escritura iniciada puede terminar sin repeticiones, la espera cancelada no ejecuta y al volver se consulta explícitamente el estado real

### Requirement: Preparación legible y acotada sin alterar edición

El panel SHALL presentar controles por teclado, foco visible, texto no confiable escapado y límites reales UTF-8/PDF/XLSX/cola/índice en ambos temas. MUST conservar buffers, guardias y contratos sin almacenar contenido en el navegador, activar remotos, modificar Home ni presentar pruebas headless como aceptación nativa.

#### Scenario: Temas teclado y texto hostil
- **WHEN** se usa el flujo por teclado en ambos temas con rótulos/configuración/errores que parecen HTML o instrucciones
- **THEN** se muestran como datos, sin navegación remota/ejecución y con controles accesibles en viewport estrecho

#### Scenario: Buffer y navegación protegida
- **WHEN** se opera preparación con nota pendiente y se intenta navegar con cancelación o fallo de guardado
- **THEN** se conservan documento, revisión capturada y guardias; las mutaciones derivadas no guardan ni reemplazan la nota

### Requirement: Lectura explícita de generaciones en escritorio

El escritorio SHALL ofrecer lectura por ID de trabajo y tipo UTF-8/PDF/XLSX elegido explícitamente, sin requerir una selección del índice. MUST llamar sólo al lector compartido correspondiente al solicitar leer, sin importar, procesar, indexar, reparar o descargar originales al montar, editar criterios o abrir detalles.

#### Scenario: Generación sin seleccionar para búsqueda
- **WHEN** se solicita leer una generación existente no seleccionada para Search
- **THEN** el lector comparte la validación Rust y devuelve el derivado con fuente/generación, sin cambiar el índice ni ejecutar helpers

#### Scenario: Abrir editar y refrescar sin consulta
- **WHEN** se abre el panel, cambia el ID/tipo o se activa/refresca la ventana
- **THEN** no se realiza lectura ni procesamiento implícito y se retira cualquier página atribuida a los criterios anteriores

### Requirement: Páginas de lectura con generación y cobertura

El lector de escritorio SHALL presentar una página reemplazable de hasta veinte items, con siguiente/inicio explícitos, generación/procedencia, resultado anterior y covers_generation del núcleo. MUST distinguir página vacía de error y rechazar/desactivar continuación obsoleta sin mezclar generaciones ni afirmar cobertura completa de una página parcial.

#### Scenario: Recorrido completo y vacío
- **WHEN** se recorren más de veinte fragmentos/celdas o una generación sin texto/celdas
- **THEN** las páginas conservan orden/localizadores sin acumulación, cursor nulo termina el recorrido y vacío no se interpreta como OCR o documento visual vacío

#### Scenario: Fuente derivado o cursor inválidos
- **WHEN** la fuente cambia, el derivado es incoherente, el ID/tipo no corresponde o se reemplaza la generación antes de siguiente
- **THEN** el error conserva código/razón y retira texto/cursor anteriores sin reparar, reiniciar ni omitir evidencia automáticamente

### Requirement: Presentación fiel de derivados tipados

El lector SHALL conservar texto canónico y localizadores UTF-8/PDF; PDF muestra su manifiesto de páginas/scope/limitaciones. Para XLSX MUST mostrar hoja/A1/ordinal, tipo/valor léxico, fórmula y caché separados, además del manifiesto de hojas/visibilidad/rangos/merges/estilos/fechas. No infiere encabezados, expande grillas/merges, recalcula ni atribuye offsets del original a texto derivado.

#### Scenario: Texto PDF y UTF-8 literal
- **WHEN** se leen Unicode, espacios/saltos, páginas no_text y fragmentos mayores que el viewport
- **THEN** el texto no se interpreta ni recorta y la procedencia distingue bytes/líneas originales de página/bytes derivados y los límites PDF existentes

#### Scenario: Celdas y metadatos XLSX
- **WHEN** se leen números largos, cadena vacía/ausencia, fórmulas con/sin caché y hojas ocultas/dispersas
- **THEN** se conservan tipos/lexemas/campos completos y metadatos del manifiesto sin redondear, ejecutar, inferir valores o esconder hojas

### Requirement: Lectura de escritorio aislada y sin pérdida de edición

El lector MUST descartar éxito/error tardíos al cambiar ID/tipo/nota/workspace, mantener un cupo físico entre montajes hasta terminar IPC y evitar llamadas todavía en cola al cancelar su espera. SHALL preservar buffers/guardias y ofrecer controles por teclado/foco/texto escapado en ambos temas, sin logs/persistencia de contenido ni remotos.

#### Scenario: Respuesta vieja y remount
- **WHEN** termina una lectura abandonada o se remonta el panel mientras sigue una llamada iniciada
- **THEN** no se publica en el nuevo contexto ni se libera prematuramente el cupo; sólo una lectura física de este panel está activa y una espera cancelada no inicia

#### Scenario: Temas teclado y buffer
- **WHEN** se lee/pagina por teclado con contenido hostil y nota pendiente, o se cancela/falla una navegación protegida
- **THEN** los controles y texto son legibles en ambos temas y viewport estrecho, no se ejecuta contenido y el documento/revisión/buffer quedan conservados

### Requirement: Lector OCR local independiente acotado

Un lector descartable SHALL reconocer explícitamente una imagen PNG/JPEG hasta 1 MiB, dimensiones positivas de hasta 4.096 por eje y cuatro millones de píxeles. MUST usar sólo el perfil macOS revision 3/fast/es-ES/en-US sin corrección, detección automática, fallback, descargas o proveedor remoto; otro formato o perfil no disponible falla explícitamente.

#### Scenario: Imagen admitida sin red
- **WHEN** se envía una imagen sintética PNG o JPEG admitida con el perfil local disponible y red denegada
- **THEN** devuelve regiones reconocidas y procedencia del motor sin instalar pesos, abrir ventanas ni contactar un proveedor remoto

#### Scenario: Formato dimensiones o secuencia no admitidos
- **WHEN** se envía contenido inválido, otro formato, más de una imagen, dimensiones o bytes excesivos
- **THEN** rechaza la lectura completa con razón tipada, sin resultado parcial o conversión automática

#### Scenario: Perfil no disponible
- **WHEN** el sistema no dispone de la revisión o idiomas del perfil solicitado
- **THEN** informa indisponibilidad sin elegir otro modelo, perfil accurate, idioma o proveedor

### Requirement: Contrato OCR privado validado en Rust

El consumidor Rust MUST validar framing/versiones, hash fuente esperado, motor, perfil, metadatos, regiones y cuotas de la respuesta antes de admitirla. SHALL rechazar campos desconocidos, identidad incoherente, texto inválido y geometría/confianza no finitas o fuera de rango; los errores sólo contienen razones tipadas sin original o contenido reconocido.

#### Scenario: Respuesta y origen ligados
- **WHEN** se recibe una respuesta válida ligada al SHA-256 de los bytes enviados
- **THEN** Rust admite sus metadatos y regiones completas sin confiar sólo en un exit code exitoso

#### Scenario: Respuesta adulterada
- **WHEN** cambian hash, versión, perfil, campos, ordinales, conteos, dimensiones o geometría/confianza
- **THEN** la validación rechaza toda la respuesta, sin truncar, inferir campos o conservar texto parcial

#### Scenario: Framing o error inválidos
- **WHEN** entrada/salida tienen framing incompleto, bytes adicionales, JSON inválido o error ajeno al protocolo
- **THEN** no se admite contenido y se informa fallo de protocolo sin reflejar texto de la fuente en el error

### Requirement: Evidencia OCR inferida por región

El lector SHALL conservar texto reconocido, orden de observaciones, confianza y rectángulos normalizados con origen inferior izquierdo del marco orientado, junto con dimensiones/orientación fuente y motor/OS. MUST identificarlo como inferencia falible, sin prometer orden visual, texto literal, tablas o cobertura de todos los caracteres; cero regiones no demuestra imagen vacía.

#### Scenario: Texto orientación y geometría
- **WHEN** se reconoce texto con acentos, saltos o contenido parecido a instrucciones y orientación declarada
- **THEN** se devuelve como datos con localizadores del marco orientado y campos completos, sin ejecutar, corregir ni convertirlo en instrucciones

#### Scenario: Imagen sin detecciones
- **WHEN** el motor no devuelve regiones para una imagen válida
- **THEN** el resultado válido conserva dimensiones/procedencia y limitaciones, sin afirmar ausencia universal de texto o información visual

### Requirement: Cuotas OCR sin truncamiento

El resultado OCR SHALL tener hasta 512 regiones, 131.072 bytes UTF-8 por región y 1 MiB de representación JSON; stdout y stderr se acotan a 8 MiB y 8 KiB. MUST rechazar exceso completo sin purga, reducción de imagen, omisión de regiones o corte de texto. Los límites de fuente, derivados e índices existentes permanecen intactos.

#### Scenario: Frontera y exceso del resultado
- **WHEN** conteos/texto/representación alcanzan la frontera válida o la exceden
- **THEN** la frontera pasa validación y el exceso rechaza la respuesta completa sin un éxito parcial

#### Scenario: Salida excesiva o desconocida
- **WHEN** un helper produce stdout/stderr por encima del límite o una respuesta no reconocida
- **THEN** el consumidor de prueba la rechaza y termina/recolecta al proceso sin bloquear pipes o aceptar texto parcial

### Requirement: Contención OCR descartable y presupuesto

El helper MUST desactivar core dumps, limitar CPU a cuatro segundos y vida a cinco desde su inicio aun bloqueado en pipes; el presupuesto recibido sólo se acorta. SHALL terminar si el máximo RSS observado supera 128 MiB, documentando que el muestreo no equivale al allocator Rust ni a un límite duro de RSS/GPU; PDF/XLSX conservan su contención previa.

#### Scenario: Deadline bloqueo y presupuesto menor
- **WHEN** la entrada/salida se bloquea, el trabajo supera su presupuesto o recibe un plazo menor
- **THEN** el watchdog termina el proceso dentro de la tolerancia de prueba sin extender cinco segundos; el consumidor lo recolecta y no deja trabajo residente

#### Scenario: CPU memoria y fallo interno
- **WHEN** ocurre exceso de CPU/RSS observado o un fallo interno
- **THEN** el helper termina sin core dump ni texto en diagnósticos y el consumidor distingue el fallo del resultado válido

### Requirement: OCR aislado de fuentes y operaciones persistentes

El lector independiente MUST recibir bytes y no abrir workspaces ni recibir rutas originales, credenciales o configuración de proveedores. SHALL preservar formatos persistentes, catálogo/MCP vigentes, operaciones, fuentes, cuotas/validación/gates y Home, sin integrar procesamiento, indexación o reparación al montar o buscar; la integración persistente exige otro cambio verificado.

#### Scenario: Lectura sin workspace
- **WHEN** se ejecuta el lector sobre bytes sintéticos sin cuenta ni workspace
- **THEN** funciona de forma independiente sin crear tablas, jobs, índices o modificaciones de originales

#### Scenario: Regresión de contratos y Home
- **WHEN** se incorporan el protocolo/helper y se verifican las operaciones previas
- **THEN** catálogo, formatos persistentes, whitelist batch, lectores, búsqueda y guardias/Home conservan sus contratos y evidencia delimitada sin atribuir a la adición del lector una integración persistente nueva ni toda 5a completa

### Requirement: Protocolo OCR decimal estable y compatible

El helper SHALL admitir un protocolo privado v2 explícito con bbox/confianza como cadenas decimales finitas, conservando v1 sin reinterpretarlo. Rust MUST validar, convertir con su parser estándar y canonizar esos escalares sin pérdida del valor f64, sin cambiar features JSON globales, texto reconocido o contratos previos.

#### Scenario: Escalares estables
- **WHEN** se transmiten valores f64 válidos, incluidos extremos/negativo cero y decimales sensibles al parseo JSON
- **THEN** sus valores se conservan al canonizar/guardar/leer repetidamente, mientras NaN, infinito, fuera de rango o cadenas inválidas se rechazan completos

#### Scenario: Versiones privadas separadas
- **WHEN** se llama al lector independiente v1 o al nuevo flujo v2
- **THEN** cada header/respuesta conserva su contrato y cuota; versiones, magic/reservados o campos cruzados se rechazan sin fallback

### Requirement: Extracción OCR administrada explícita

El extractor image_ocr v1 SHALL admitir sólo PNG/JPEG administrados elegibles hasta 1 MiB con configuración v1 predeterminada y perfil local fijo. MUST conservar cola pausada por defecto, deduplicación, cancelación, retry, ejecución explícita por trabajo y generaciones anteriores, sin OCR al importar, abrir, leer o buscar.

#### Scenario: Flujo offline y deduplicación
- **WHEN** se importa una imagen admitida, encola dos veces, reanuda la cola y ejecuta el trabajo elegido con red denegada
- **THEN** se conserva un job deduplicado y se publica su generación sin modificar original/entidades ni seleccionarla para Search

#### Scenario: Fuente o perfil no admitidos
- **WHEN** extensión/MIME/contenido, tamaño, frames, dimensiones o perfil no cumplen el contrato
- **THEN** se informa razón tipada sin reducción, conversión, otro modelo, red, descarga o resultado parcial

#### Scenario: Reintento y resultado anterior
- **WHEN** se cancela/reencola o falla un intento posterior con una generación previa
- **THEN** se conservan la generación anterior y su marca de intento previo hasta una publicación nueva válida, sin ejecución o selección implícitas

### Requirement: Representación OCR persistida e íntegra

La generación OCR SHALL guardar una sola representación canónica v1 de imagen/motor/OS/regiones, con escalares decimales, SHA-256 y chunks UTF-8 <=4 KiB/512, total <=1 MiB. MUST ligar job/generación/fuente, validar identidad/conteos/partición/hash completos y sumar sus bytes al presupuesto retenido compartido de 32 MiB sin duplicar su proyección pública.

#### Scenario: Publicación y lectura fiel
- **WHEN** se publica y reabre una imagen con regiones, Unicode o sin detecciones
- **THEN** se recuperan exactamente texto, valores canónicos, orientación/dimensiones/procedencia y limitaciones; fragment_count cuenta chunks internos, no regiones

#### Scenario: Corrupción y cruce de formato
- **WHEN** hay cambios de hash/chunks/ordinales/owner/manifiesto, bytes incoherentes, campos desconocidos o mezcla de filas UTF-8/PDF/XLSX/OCR
- **THEN** se rechaza la representación completa antes de usarla, sin reparación, inferencia o descarte automático

#### Scenario: Cuota mixta y frontera
- **WHEN** el resultado o la suma retenida con UTF-8/PDF/XLSX alcanza el límite o lo excede
- **THEN** la frontera válida se admite y el exceso rechaza toda publicación, descontando sólo la generación del mismo job reemplazable y sin purgar originales/resultados ajenos

### Requirement: Publicación OCR cercada y contención compartida

El worker MUST ejecutar el helper fuera del lock y verificar fuente, ownership, lease y revisiones de control/job antes y durante publicación atómica. SHALL conservar presupuesto cooperativo 5 s, lease 15 s, límites de pipes y contención nativa declarada; pérdida de fence/deadline mata y recolecta al hijo, sin publicar datos parciales.

#### Scenario: Control cancelación y otro worker
- **WHEN** cambia control/job/ownership, vence lease o otro cliente cancela mientras corre OCR
- **THEN** el intento viejo no publica y el estado recuperable sigue las reglas existentes, sin sobrescribir el trabajo confirmado por otro cliente

#### Scenario: Fuente cambia al procesar
- **WHEN** el original o su metadata/hash cambia después del claim o antes del commit
- **THEN** el intento falla con motivo explícito y conserva originales, resultado previo e índice confirmado

#### Scenario: Helper ausente o incorrecto
- **WHEN** el helper falta, se bloquea, excede recursos/pipes o responde con protocolo/perfil/hash inválidos
- **THEN** run falla de forma recuperable y recolecta el proceso; describir, leer o restaurar resultados previos no necesita el helper ni hace fallback

### Requirement: Lectura OCR completa por páginas

extraction.ocr_regions SHALL leer 1–64 regiones (32 por defecto), con manifiesto, generación/fuente, intento previo, ordinal y bbox/confianza decimales del marco orientado, en <=2 MiB. MUST revalidar original y representación completos antes de paginar, ligar cursor a scope/job/generación/operación/límite y no ejecutar OCR o indexación.

#### Scenario: Páginas y vacío
- **WHEN** se recorren más de veinte regiones o una generación sin detecciones
- **THEN** se conservan orden, texto y campos completos, siguiente termina en null y covers_generation describe sólo la página; vacío no demuestra ausencia de texto visual

#### Scenario: Cursor fuente o derivado inválidos
- **WHEN** cambia la generación/fuente, el cursor es ajeno o el tipo/resultado es incompatible
- **THEN** se devuelve error tipado sin mezclar generaciones, reiniciar, transcribir o reparar automáticamente

### Requirement: Proyección OCR explícita para búsqueda

El índice SHALL seleccionar OCR sólo con put explícito y CAS de generación/revisión, proyectando texto reconocido por región en fragmentos UTF-8 <=4 KiB, hasta 512/1 MiB por generación. MUST conservar bbox/confianza, proyección y rango de texto reconocido sin inventar bytes de imagen, filtrar por confianza o indexar JSON/metadata; las cuotas globales previas se conservan.

#### Scenario: Selección búsqueda y cero texto
- **WHEN** se selecciona una generación OCR íntegra, incluida una sin detecciones
- **THEN** sólo su texto reconocido entra en FTS con localizador de región y cobertura explícitos; extraer/leer por sí solo no incorpora hits

#### Scenario: CAS y regeneración
- **WHEN** otro cliente modifica índice/generación entre consulta y put, o se regenera una selección vigente
- **THEN** CAS obsoleto se rechaza; una publicación nueva invalida la selección anterior sin seleccionar otra ni repetir escrituras

#### Scenario: Proyección excesiva
- **WHEN** una representación admitida produce más fragmentos/bytes de los permitidos para indexarla
- **THEN** el put completo se rechaza y conserva generación/índice anteriores, sin cortar regiones, omitir evidencia o relajar cuotas

### Requirement: Verificación OCR íntegra en índice y consulta

Índice/búsqueda MUST validar manifiesto, representación, proyección, copias y postings completos de las selecciones OCR, incluyendo toda su metadata en la huella. SHALL conservar validación de candidatos fuente, cobertura y rechazo completo de corrupción/futuros no admitidos, sin cambiar hashes o semántica UTF-8/PDF/XLSX ni los gates fijados.

#### Scenario: Derivado o postings alterados
- **WHEN** se altera bbox/confianza/texto, su huella, copias, postings, conteos o representación de una selección
- **THEN** get/search/put rechazan el estado inválido sin éxito parcial, caché que omita validación, reset o reconstrucción automáticos

#### Scenario: Consulta mixta y fuentes
- **WHEN** se consulta un índice mixto o se altera el original de un candidato
- **THEN** se conservan filtros/cursores/cobertura y verificación fuente del contrato existente; los corpus/gates previos mantienen query p95 <=300 ms y lock p95 <=150 ms

### Requirement: Preparación y lectura OCR explícitas en escritorio

La interfaz SHALL permitir elegir image_ocr y leer sus regiones por acciones explícitas independientes de Search, mostrando perfil/procedencia, inferencia, revisiones, límites y errores reales. MUST conservar páginas de veinte, texto/manifiesto completos, sin consultas/procesamiento al montar/editar/activar el lector ni reinterpretar un tipo nuevo como XLSX.

#### Scenario: Preparar seleccionar y leer
- **WHEN** se prepara una imagen administrada y se solicita leer su generación desde una nota
- **THEN** cada paso usa su operación/CAS y recibo, el lector muestra regiones/metadata sin selección obligatoria y ningún paso posterior se ejecuta solo

#### Scenario: Presentación inferida y límites
- **WHEN** se muestran regiones, error, resultado previo o cero detecciones
- **THEN** bbox/confianza y rangos se identifican correctamente, confianza no es certeza y complete/covers_generation no prometen transcripción de todos los caracteres

### Requirement: Lifetime y edición protegidos durante OCR

Los paneles MUST descartar éxito/error tardíos y esperas canceladas al cambiar contexto, sin liberar cupos de IPC iniciado. SHALL conservar buffers/guardias, teclado y ambos temas, sin guardar notas desde OCR, registrar contenido persistente en el navegador o modificar composición/navegación Home.

#### Scenario: Respuesta vieja y cambio de nota
- **WHEN** termina una lectura/mutación OCR abandonada o se remonta el panel con IPC activo
- **THEN** no publica datos/errores en el nuevo contexto, no repite la acción y mantiene el cupo físico hasta terminar la llamada iniciada

#### Scenario: Buffer temas y teclado
- **WHEN** se extrae/lee/busca con contenido hostil y una nota pendiente, o se cancela/falla la navegación protegida
- **THEN** el contenido se escapa, los controles funcionan en ambos temas/viewport estrecho y buffer/revisión/guardias permanecen intactos

### Requirement: Configuraciones OCR futuras conservadas

El sistema MUST conservar crudos los jobs/configuraciones no soportados sin ejecutarlos, convertirlos, repararlos o borrar resultados. SHALL distinguir soporte del contrato de disponibilidad real del helper/perfil y mantener las operaciones antiguas offline sin instalar recursos para resolver indisponibilidad OCR.

#### Scenario: Metadata futura sin ejecución
- **WHEN** se consulta un job con configuración/extractor futuro o se intenta ejecutarlo/indexarlo
- **THEN** sus datos se conservan, se identifica incompatibilidad y ejecución/selección se rechazan sin reinterpretación o purga

#### Scenario: Núcleo sin motor OCR disponible
- **WHEN** el helper/perfil no está disponible y se usan notas, lectores existentes o resultados OCR persistidos
- **THEN** esas operaciones siguen independientes del motor; sólo una ejecución OCR reclamada falla con razón explícita y sin activar alternativas

### Requirement: Lector PDF nativo independiente y explícito

Un lector descartable SHALL ofrecer el perfil local apple_pdfkit_text_v1 mediante bytes y un protocolo privado propio. MUST devolver páginas de texto derivado con hash fuente y procedencia OS/build sin modificar originales, abrir workspaces o activar operaciones del núcleo. Su incorporación no sustituye pdf_text ni inicia parsing al importar, leer, buscar o abrir la interfaz.

#### Scenario: Texto moderno offline
- **WHEN** se envía un PDF propio admitido con texto seleccionable Unicode y el perfil disponible, con red denegada
- **THEN** devuelve todas sus páginas en orden de documento y texto del API conservado, sin UI, modelos, proveedores o cambios de archivos

#### Scenario: Página sin texto seleccionable
- **WHEN** una página no devuelve texto, incluida una página escaneada
- **THEN** se conserva su posición y estado no_text con texto vacío, sin OCR ni prueba de ausencia visual

### Requirement: Admisión nativa y límites de fidelidad declarados

El lector MUST verificar framing PDF de cabecera/EOF, rechazar cifrado reconocido incluso desbloqueado y conservar estrictamente el texto devuelto por el API al convertir UTF-16 a UTF-8, sin normalización propia. SHALL declarar parser de sistema tolerante, sin validación completa de PDF, orden visual, OCR, tablas o geometría; no repara el original ni pide contraseñas.

#### Scenario: Cabecera EOF y cifrado
- **WHEN** falta framing admitido, el backend no reconoce el documento o isEncrypted/isLocked son verdaderos
- **THEN** falla con razón tipada sin texto parcial ni intento propio de desbloqueo, incluso si el sistema abrió una contraseña vacía

#### Scenario: Codificación y contenido como datos
- **WHEN** el texto del API tiene UTF-16 no convertible íntegramente, NUL, enlaces o contenido semejante a instrucciones
- **THEN** la codificación no válida se rechaza sin reemplazos; el contenido válido permanece dato sin ejecutar acciones ni abrir enlaces

#### Scenario: Tolerancia del sistema sin falsa validación
- **WHEN** el backend admite anomalías internas que el framing no detecta o el orden de texto difiere del visual
- **THEN** el resultado identifica esas limitaciones y el perfil/OS reales, sin afirmar integridad completa o modificar bytes fuente

### Requirement: Protocolo PDF nativo validado y acotado

El consumidor Rust SHALL verificar versión/magic/reservados, hash esperado, perfil, metadatos acotados y todas las páginas/estados/conteos de una respuesta. MUST limitar fuente y texto UTF-8 total a 1 MiB, páginas a 1–100, stdout a 8 MiB y stderr a 8 KiB, rechazando campos desconocidos, incoherencia o exceso completos sin fallback, omisión o truncamiento.

#### Scenario: Respuesta fuente y páginas ligadas
- **WHEN** la respuesta válida corresponde al hash de entrada y respeta límites
- **THEN** se admiten páginas consecutivas 1-based, text/no_text coherentes, bytes totales exactos y metadatos/procedencia completos

#### Scenario: Alteración y frontera
- **WHEN** hay versión/hash/perfil/metadata/ordinal/estado/count ajenos o se alcanza/excede una frontera de fuente/texto/páginas
- **THEN** la frontera válida se conserva y el caso inválido falla completo sin confiar en un exit code exitoso ni devolver datos parciales

#### Scenario: Pipes o envelope no admitidos
- **WHEN** hay JSON/framing inválido, bytes adicionales, protocolo de otro helper o pipes excesivos
- **THEN** se rechaza el resultado, termina/recolecta el proceso y no se reflejan datos de la fuente en el error

### Requirement: Contención nativa de PDF descartable

El helper MUST desactivar core dumps, limitar CPU a 4 s y vida a 5 s desde inicio aun bloqueado o sin consumidor; un plazo recibido sólo se acorta. SHALL terminar al observar máximo RSS mayor a 128 MiB, con muestreo de 5 ms y comprobaciones al publicar, declarando que no es límite duro RSS/GPU ni allocator Rust. Diagnósticos del framework no contaminan el transporte.

#### Scenario: Bloqueo y pérdida del consumidor
- **WHEN** stdin/stdout quedan bloqueados o desaparece el padre
- **THEN** el helper termina dentro del plazo/tolerancia declarados sin quedar residente ni producir core dump

#### Scenario: CPU memoria fallo y plazo menor
- **WHEN** se excede CPU/RSS observado, ocurre abort o se recibe un presupuesto menor
- **THEN** termina con estado identificable y sin extender el presupuesto, publicar texto parcial o afectar al cliente

#### Scenario: Controles separados del binario normal
- **WHEN** se ejecutan probes de contención o se envían sus flags al binario normal
- **THEN** los controles existen sólo en el build de pruebas y el normal los rechaza sin ejecutar sus acciones

### Requirement: Entrega nativa aislada y compatibilidad preservada

El lector SHALL prepararse por target/perfil macOS con SDK instalado, sin descargas ni staging automático del host. MUST conservar núcleo/UI, catálogo y formatos vigentes, lectores previos, cuotas/gates y Home; no añade perfil público o persistencia. Integrar resultados exige otro cambio con recuperación diseñada antes de alterar formatos.

#### Scenario: Preparación y disponibilidad
- **WHEN** se prepara para un target/perfil admitido o falta el SDK/plataforma
- **THEN** se obtiene el helper propio correspondiente o un error explícito, sin instalar recursos, fallback o ventana nativa

#### Scenario: Producto y evidencia previos
- **WHEN** se verifican las operaciones y artefactos existentes tras incorporar el lector independiente
- **THEN** conservan contratos y comportamiento medidos con procedencia de binarios real; no se declara PDF integrado o toda 5a completa por pruebas del lector
