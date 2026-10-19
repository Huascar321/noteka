# Spec Delta

## ADDED Requirements

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

## MODIFIED Requirements

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

### Requirement: Presupuestos explícitos y errores útiles

La cola SHALL publicar y verificar límites de fuentes, jobs, fragmentos, almacenamiento derivado, páginas, respuestas y ejecución. MUST rechazar exceso antes de publicar, sin truncar un éxito como completo ni exponer contenido íntegro en errores. No sube los límites vigentes de importación, transporte o batches para acomodar derivados. Los 32 MiB compartidos se computan sobre texto retenido real de ambos extractores.

#### Scenario: Fronteras y backpressure
- **WHEN** se alcanza un límite de 1 MiB de fuente, 1.000 queued/running, 10.000 trabajos o 32 MiB de texto derivado retenido
- **THEN** la frontera permitida se acepta y el exceso produce un motivo explícito, sin crear trabajos o resultados parciales

#### Scenario: Respuesta limitada
- **WHEN** una lectura pide un límite inválido o una respuesta completa supera 2 MiB
- **THEN** se rechaza o pagina conforme al contrato, sin ocultar cobertura, cursores, texto omitido o errores

#### Scenario: Límites PDF y presupuesto mixto
- **WHEN** un PDF excede 100 páginas, 1 MiB de texto derivado, 512 fragmentos, 4.096 bytes por fragmento o el presupuesto retenido junto con texto UTF-8
- **THEN** la frontera válida se acepta y el exceso rechaza la generación sin purga ni truncamiento; la regeneración descuenta sólo su resultado previo reemplazable y no usa el tamaño comprimido del PDF para contar texto
