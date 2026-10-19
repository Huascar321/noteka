# local-text-search Specification

## Purpose
Encontrar títulos y texto local sin modelos, manteniendo referencias verificables al contenido original y un índice derivado reconstruible.

## Requirements

### Requirement: Búsqueda textual acotada con fuentes

El sistema SHALL buscar términos literales en títulos y bloques admitidos, devolver entidad/revisión/bloque de origen y extracto de texto, filtrar tipo/archivo/contexto y paginar. MUST rechazar consultas vacías o excesivas sin interpretar SQL, shell ni instrucciones contenidas en documentos.

#### Scenario: Bloque enriquecido y filtros
- **WHEN** se busca texto de un bloque anidado dentro de un contexto
- **THEN** se obtiene el ID/revisión del original y sólo resultados que cumplen filtros

#### Scenario: Consulta inválida
- **WHEN** la consulta está vacía, supera límites o contiene texto parecido a SQL/operadores
- **THEN** se rechaza el input inválido o se busca literalmente sin ejecutar instrucciones

### Requirement: Índice consistente y reconstruible

El sistema MUST actualizar el índice junto a las escrituras de dominio, no devolver revisiones desactualizadas como actuales y preservar originales desconocidos sin convertirlos para indexar. SHALL reconstruir FTS desde SQLite sin modificar entidades, revisiones ni adjuntos y migrar con backup previo.

#### Scenario: Actualizar y archivar
- **WHEN** cambia el texto o se archiva una entidad
- **THEN** la búsqueda vigente deja de presentar texto/revisión anterior y respeta el estado de archivo

#### Scenario: Reconstrucción y formato futuro
- **WHEN** se reconstruye el índice con un documento desconocido almacenado
- **THEN** sus originales y revisión quedan intactos; sólo el título reconocido es indexable

### Requirement: Búsqueda de escritorio offline

La interfaz SHALL buscar notas/entidades y adjuntos en ámbitos explícitos separados y ofrecer consulta conjunta por grupos independientes, sin fusionar scores ni cursores. MUST abrir entidades con su localizador, conservar buffers pendientes al navegar y mostrar evidencia y límites de adjuntos sin servicios remotos ni extracción o indexación implícitas.

#### Scenario: Buscar y abrir sin red
- **WHEN** se busca una nota enriquecida sin red ni modelos
- **THEN** se abre la entidad original con referencia de bloque/revisión y se conserva la protección de edición pendiente

#### Scenario: Cambiar de ámbito sin cambiar el editor
- **WHEN** se cambia entre notas/entidades y adjuntos mientras hay un buffer pendiente
- **THEN** el buffer permanece intacto, los filtros de tipo/archivo/contexto sólo se aplican a entidades y cada ámbito usa su propio contrato sin mezclar resultados ni cursores

#### Scenario: Consultar adjuntos no es procesarlos
- **WHEN** se entra al ámbito de adjuntos, se busca o se inspecciona un hit
- **THEN** no se importa, encola, ejecuta, selecciona, resetea ni reconstruye contenido o índices; los adjuntos sin selección explícita no se incorporan a la búsqueda

#### Scenario: Consulta local conjunta explícita
- **WHEN** se solicita una consulta conjunta de texto
- **THEN** la misma entrada consulta entidades activas y adjuntos seleccionados, con dos grupos/coberturas independientes sin total o snapshot global inventados; los ámbitos individuales conservan sus filtros

### Requirement: Selección explícita de generaciones extraídas

El sistema SHALL indexar mediante `extraction.index put` una generación confirmada UTF-8, PDF o XLSX compatible por llamada, con trabajo y generación esperada. MUST verificar fuente, configuración y fragmentos/proyección antes de publicarla. Importar, extraer, abrir, migrar y buscar no seleccionan generaciones automáticamente; el índice empieza vacío.

#### Scenario: Extracción no implica indexación
- **WHEN** se importa un adjunto y se confirma su extracción sin llamar a index put
- **THEN** sus datos siguen disponibles por el lector correspondiente, pero no aparecen en extraction.search y la cobertura los cuenta sin indexar

#### Scenario: Indexación local de ambos extractores
- **WHEN** se selecciona explícitamente una generación vigente utf8_text o pdf_text compatible, sin red, modelos ni helper PDF instalado
- **THEN** se indexan todos sus fragmentos de forma atómica sin ejecutar extracción y sin cambiar el original, el trabajo ni su generación

#### Scenario: Fuente o generación inadmisible
- **WHEN** falta el trabajo/generación, la configuración no es compatible o la fuente está ausente, alterada o fuera del almacenamiento seguro
- **THEN** se devuelve un error identificable y no se publica una selección parcial ni se modifica lo previamente indexado

#### Scenario: XLSX persistido sin helper
- **WHEN** se selecciona una generación xlsx_cells v1 íntegra cuya proyección cabe en los límites, con los helpers ausentes
- **THEN** se publican todos sus campos indexables en una selección atómica, sin interpretar ZIP/XML ni cambiar Workbook, chunks, estado, intentos o revisiones de entidades; los chequeos de fuente leen el original sólo para verificar su hash

### Requirement: Escrituras del índice con revisión esperada

El sistema MUST exigir revisión esperada para put/reset y generación esperada para put. Una revisión o generación vieja SHALL producir conflicto sin sobrescritura. Repetir put sobre la misma generación íntegra no duplica filas ni cambia la revisión. Reset explícito vacía sólo este índice e incrementa su revisión, incluso si estaba vacío.

#### Scenario: Dos indexadores compiten
- **WHEN** dos procesos intentan indexar con la misma revisión y uno confirma primero
- **THEN** el otro recibe revision_conflict y conserva íntegros sus originales y el índice confirmado por el primero

#### Scenario: Repetición sin duplicados
- **WHEN** se repite put de una generación ya indexada con la revisión actual
- **THEN** se informa que no hubo cambios, con las mismas filas, ranking y revisión; una revisión vieja sigue siendo conflicto

#### Scenario: Reset deliberado
- **WHEN** se ejecuta reset con la revisión actual
- **THEN** desaparecen selecciones, copias y postings de adjuntos, cambia la revisión y siguen intactos entidades, search.query/rebuild, originales, cola y fragmentos

### Requirement: Consulta literal acotada de fragmentos

El sistema SHALL buscar términos literales en fragmentos indexados, con filtros combinables de trabajo, adjunto y extractor. MUST aceptar 1–16 términos y hasta 256 bytes, rechazar controles no blancos y paginar con límite 1–64. Operadores aparentes no ejecutan SQL, shell ni instrucciones; todos los términos deben coincidir en un mismo fragmento según el tokenizer documentado.

#### Scenario: Filtros y orden reproducible
- **WHEN** se busca con filtros de adjunto y extractor sobre varias generaciones seleccionadas
- **THEN** sólo se devuelven coincidencias de esos filtros, ordenadas por ranking textual y desempate estable de trabajo/generación/ordinal

#### Scenario: Operadores como texto
- **WHEN** una consulta admisible contiene OR, NEAR, comillas o texto parecido a SQL y un fragmento contiene instrucciones
- **THEN** la consulta se parametriza como términos literales y el contenido se devuelve como datos, sin ampliar operadores ni ejecutar herramientas

#### Scenario: Límites y frontera de fragmentos
- **WHEN** la consulta está vacía, excede límites, usa un filtro/límite inválido o sus términos sólo coinciden repartidos en dos fragmentos
- **THEN** se rechaza el input inválido o no se devuelve esa coincidencia repartida; no se inventa una frase que cruce fragmentos

### Requirement: Hits con fragmento canónico y localizador tipado

Cada hit SHALL devolver trabajo, adjunto, generación, hash fuente, extractor/versión, hash de configuración, ordinal, texto canónico y localizador tipado. MUST verificar la fuente de los hits y distinguir bytes/líneas UTF-8, página/rango del texto PDF derivado y hoja/A1/celda/campo/rango del texto XLSX proyectado. Un resultado de un intento anterior se identifica; no se inventa una entidad propietaria, coordenada visual o cita OCR. La caché XLSX no se presenta como cálculo actualizado.

#### Scenario: Evidencia literal
- **WHEN** un hit corresponde a un fragmento UTF-8 con caracteres multibyte y saltos de línea
- **THEN** su texto coincide con el lector literal y su localizador conserva los rangos de bytes/líneas del original, no offsets de un snippet normalizado

#### Scenario: Evidencia PDF sin renderizador
- **WHEN** un hit corresponde a texto PDF confirmado y el helper no está disponible
- **THEN** su texto/localizador coincide con pdf_fragments, identifica página y rango del texto derivado y expone el alcance y limitaciones PDF, sin fingir offsets del archivo ni geometría

#### Scenario: Original cambiado después de indexar
- **WHEN** un candidato de la página o su lookahead procede de un archivo eliminado, alterado o sustituido por un enlace inseguro
- **THEN** la consulta falla con razón identificable, sin devolver esa página como evidencia vigente ni reparar, reextraer o borrar datos

#### Scenario: Evidencia XLSX verificable
- **WHEN** un hit corresponde a una porción de valor guardado o fórmula XLSX
- **THEN** identifica proyección xlsx_cell_fields_v1, ordinal global de celda, sheet_id, referencia A1, campo tipado, bytes totales del campo y rango UTF-8 exclusivo al final; su texto coincide exactamente con ese rango del campo derivado de extraction.xlsx_cells, no con offsets del ZIP o del JSON

### Requirement: Invalidación atómica al reemplazar generaciones

El sistema MUST retirar selección y postings de una generación reemplazada en la misma transacción que confirma la nueva generación, e incrementar la revisión del índice si había selección. La nueva generación no se indexa automáticamente. Un reemplazo fallido, pausa, cancelación o retry SHALL conservar la generación e índice anteriores disponibles y señalar su procedencia anterior cuando corresponda.

#### Scenario: Reemplazo confirmado
- **WHEN** se publica una generación nueva de un trabajo previamente indexado
- **THEN** no queda un estado confirmado con postings viejos o mezcla de generaciones; la nueva figura sin indexar y el cursor anterior queda vencido

#### Scenario: Falla o cancelación del reemplazo
- **WHEN** un retry falla o se cancela antes de confirmar otra generación
- **THEN** la generación y selección anteriores siguen consultables si su fuente es válida, con result_from_previous_attempt coherente y sin reindexación automática

#### Scenario: Publicación compite con indexación
- **WHEN** una publicación reemplaza la generación entre la preparación y el commit de put
- **THEN** put revalida la generación bajo exclusión de escritor y falla por conflicto, sin reintroducir el resultado reemplazado

### Requirement: Cursores de adjuntos ligados al índice

El sistema SHALL ligar cada cursor a workspace/ruta canónica, operación, consulta/filtros/límite, perfil y revisión del índice. MUST rechazar cursores inválidos, ajenos, excesivos o vencidos, sin reiniciar silenciosamente la búsqueda. Sin cambios del índice, la paginación mantiene orden estable sin duplicados; los cursores existentes de notas permanecen intactos.

#### Scenario: Páginas estables
- **WHEN** se recorren varias páginas sin modificar el índice
- **THEN** se obtiene cada hit una sola vez, con el mismo ranking y orden, y next_cursor termina en null

#### Scenario: Cambio de revisión entre páginas
- **WHEN** después de la primera página se confirma put, reset o invalidación de una selección
- **THEN** la página siguiente devuelve conflicto de revisión del índice, no resultados mezclados con la revisión anterior

#### Scenario: Cursor ajeno o manipulado
- **WHEN** se reutiliza un cursor en otra ruta/restauración, operación, consulta, filtro o límite, o con versión/campos/offset no admitidos
- **THEN** se devuelve validation_error sin revelar rutas internas ni interpretar el cursor como autorización

### Requirement: Cobertura explícita del conjunto seleccionado

El sistema SHALL informar cobertura filtrada por generaciones: seleccionadas, compatibles sin indexar, incompatibles y trabajos sin generación. MUST declarar que busca sólo el conjunto seleccionado y distinguir validación del índice de verificación de originales. No deriva archivo/contexto de entidades ni promete buscar imágenes, adjuntos sin extracción o toda la base; cero hits no significa evidencia universalmente ausente.

#### Scenario: Cobertura parcial y texto ausente
- **WHEN** hay generaciones sin seleccionar, un PDF confirmado sin texto y trabajos aún sin generación
- **THEN** get/search distinguen esas categorías; una selección PDF sin texto es válida con cero fragmentos y no se presenta como OCR o cobertura de imágenes

#### Scenario: Configuración futura conservada
- **WHEN** existe una generación de configuración no compatible junto a generaciones compatibles seleccionadas
- **THEN** se cuenta como incompatible sin indexarla, ejecutarla, descartarla ni reserializar su configuración persistida; las compatibles siguen consultables

#### Scenario: Adjunto compartido y entidad archivada
- **WHEN** un adjunto está referenciado por varias entidades, alguna archivada, o por ninguna
- **THEN** se devuelve el hit independiente del adjunto una sola vez por fragmento/trabajo, sin inventar dueño; filtros de entidad, archivo o contexto no admitidos se rechazan

#### Scenario: XLSX vacío y rechazo por cuota
- **WHEN** se selecciona un XLSX sin campos no vacíos o se rechaza por exceso una selección de extractor compatible
- **THEN** el primero cuenta como indexado con cero fragmentos/bytes y el segundo permanece compatible sin indexar: compatibilidad se determina por extractor/configuración y no garantiza caber en cuotas, incluso si excede 512 fragmentos; no se afirma cobertura textual de metadatos, encabezados inferidos o celdas omitidas

### Requirement: Índice verificable y reparación explícita

El sistema MUST comprobar coherencia de selecciones, copias y postings con los fragmentos confirmados antes de responder get/search; un índice inconsistente produce error, no cero hits confiables. Las lecturas SHALL ser puras respecto de datos persistentes y no recuperar jobs. Reset y puts explícitos permiten reconstruir un índice derivado dañado recuperable, sin reextraer ni tocar fuentes; versiones futuras se preservan y rechazan.

#### Scenario: Copia o posting inconsistente
- **WHEN** se elimina/agrega una fila, se cambia su identidad/texto o faltan/sobran postings conservando un SQLite legible
- **THEN** get/search devuelve storage_error de índice, incluso ante una consulta que habría devuelto cero hits, sin reparación automática

#### Scenario: Reconstrucción sin parser
- **WHEN** se obtiene la revisión desde el error de índice recuperable y se ejecuta reset seguido de puts de generaciones conservadas
- **THEN** se reconstruye un índice coherente sin helper, modelos ni nuevas extracciones; un fallo de reset revierte íntegramente la transacción

#### Scenario: Lectura durante trabajo interrumpido
- **WHEN** get/search se ejecuta con un job vencido o running y una generación anterior disponible
- **THEN** sólo lee el snapshot, sin recuperar ownership, incrementar intentos/revisiones, ejecutar trabajo ni cambiar pausa o selecciones

### Requirement: Recursos acotados sin ampliar la extracción

El sistema MUST acotar cada índice a 1.024 selecciones, 16.384 fragmentos y 8 MiB reales de texto copiado, cada respuesta a 2 MiB y cursor a 2 KiB. SHALL aplicar presupuesto cooperativo de 5 s y espera de escritor acotada; exceso o interrupción revierte escrituras completas. Se conservan los límites separados de extracción y snapshots, sin eviction implícita de selecciones.

#### Scenario: Saturación del índice
- **WHEN** put excede un límite por cantidad o bytes UTF-8 reales
- **THEN** falla sin expulsar otras selecciones ni modificar la generación; las generaciones no seleccionadas siguen disponibles y se informan en cobertura

#### Scenario: Deadline, contención o respuesta excesiva
- **WHEN** se agota presupuesto/espera o una respuesta excede su cap serializado
- **THEN** se devuelve un error identificable sin resultados parciales ni escrituras confirmadas, sin aumentar automáticamente los límites

### Requirement: Proyección textual XLSX determinista por campos

El sistema SHALL proyectar caché y fórmula separadas, en orden del lector, incluyendo hojas ocultas, con los seis campos y representación exacta de los escenarios. MUST omitir ausencia/vacío y dividir por UTF-8 hasta 4096 bytes sin mezclar, normalizar, recalcular ni expandir fórmulas. Sólo indexa campos persistidos, no etiquetas, nombres, metadatos o JSON; el resultado no acredita recálculo ni libro visualmente vacío.

#### Scenario: Tipos léxicos y fórmulas separadas
- **WHEN** hay decimales/exponentes largos, fechas ISO/seriales, booleanos, errores y fórmulas con caché ausente u obsoleta
- **THEN** recorre hojas visibles/hidden/very_hidden y celdas en orden del lector, por celda caché antes de fórmula, distinguiendo cached_text, cached_number, cached_boolean, cached_error, cached_date_iso y formula_text; texto/números/fechas/errores conservan strings exactos y booleanos true/false del DTO, no lexemas XML; no inserta etiquetas, signo igual, nombres de hoja o JSON ni expande shared/array, mezcla campos o presenta caché como recálculo; campos ausentes/vacíos no generan fragmentos ni acreditan libro visualmente vacío

#### Scenario: Campos largos y Unicode
- **WHEN** un campo tiene más de 4096 bytes con Unicode multibyte, saltos o espacios
- **THEN** se divide en rangos consecutivos no vacíos de hasta 4096 bytes sin cortar caracteres, cuyos textos concatenados reconstruyen el campo; el ordinal de fragmento es único y distinto del ordinal de celda o chunk persistido

#### Scenario: Frontera de selección completa
- **WHEN** una proyección produce exactamente 512 fragmentos o requeriría 513, o se excede otra cuota vigente
- **THEN** la frontera válida se admite y el exceso se rechaza completo sin purgar selecciones ni truncar campos/celdas; extraction.xlsx_cells conserva su capacidad de leer libros que no caben en el índice

#### Scenario: Coincidencia no cruza campos ni fragmentos
- **WHEN** los términos sólo coinciden repartidos entre fórmula/caché, dos celdas o dos porciones de un campo, o un token cruza un corte
- **THEN** no se inventa un hit conjunto ni se promete coincidencia a través del corte; rigen la semántica literal por fragmento y el tokenizer existentes, sin alterar el texto para unir términos

### Requirement: Integridad completa del índice mixto XLSX

Get/search SHALL validar íntegramente selecciones, representaciones XLSX, proyecciones, copias, postings y estadísticas antes de responder, aun fuera del filtro o sin hits. Search MUST confrontar candidatos/score y originales de página/lookahead. No se usan caches de confianza entre llamadas. Se conservan source_validation, CAS, retención, invalidación, presupuestos y rendimiento de los escenarios; formatos/perfiles futuros se rechazan sin reset.

#### Scenario: Corrupción ajena a filtro o página
- **WHEN** se altera un chunk/manifiesto/propietario/rango/copia/posting de una selección XLSX fuera del filtro o página, o se conserva texto mientras cambia metadato canónico
- **THEN** get/search fallan completos con error saneado e integridad recuperable cuando corresponde, sin devolver cero hits como índice válido ni repararlo automáticamente; get conserva source_validation=not_checked y search returned_candidates, sin prometer validación de originales ajenos a página/lookahead

#### Scenario: No-op y reemplazo concurrentes
- **WHEN** se repite put íntegro con revisión vigente o publicación/reset/otro put compiten antes de su commit
- **THEN** no-op preserva filas/revisión/hash y la competencia revalida CAS/fuente bajo escritor; sólo se confirma un estado completo, el reemplazo retira sólo su selección atómicamente e invalida el cursor sin autoindexar la nueva generación, mientras falla/retry/cancel conservan el resultado anterior; reset sólo reconoce formato/perfil actuales y no descarta futuros

#### Scenario: Presupuestos y rendimiento sin excepciones
- **WHEN** se ejecutan saturación/fronteras, lecturas mixtas y los gates release fijados UTF-8/PDF y mixto XLSX
- **THEN** se conservan 512 fragmentos/1 MiB por selección, 1024 selecciones, 16384 fragmentos/8 MiB totales, snapshots 64 MiB, respuesta 2 MiB, cursor 2 KiB, ejecución 5 s y espera 2 s; query p95 es <=300 ms y lock p95 <=150 ms sin omitir validación o ampliar caps

### Requirement: Consulta de adjuntos desde escritorio con alcance explícito

El escritorio SHALL consultar fragmentos seleccionados con texto y filtros combinables opcionales de extractor, trabajo y adjunto, usando el contrato compartido. MUST distinguir estado inicial, carga, cero coincidencias y fallo; conservar la validación Rust y mostrar errores identificables sin ejecutar instrucciones de consulta o contenido.

#### Scenario: Paridad de consulta y filtros
- **WHEN** se consulta texto con filtros de extractor, trabajo y adjunto por escritorio y CLI sobre el mismo índice sin cambios
- **THEN** la página conserva texto, IDs, orden y cobertura del núcleo; no envía filtros de entidades ni construye otro ranking o una búsqueda por subcadenas

#### Scenario: Consulta o filtro inválido
- **WHEN** se envía una consulta que excede 256 bytes UTF-8 o 16 términos, controles no admitidos o IDs no válidos
- **THEN** aparece el error de validación sin publicar una página parcial, truncar la consulta para hacerla válida ni cambiar datos; un campo opcional vacío no se envía como filtro

#### Scenario: Sin consulta y sin coincidencias son estados distintos
- **WHEN** se abre el ámbito sin consultar o termina una consulta válida sin hits
- **THEN** el primero invita a buscar sin afirmar resultados y el segundo muestra cero coincidencias con la cobertura devuelta; un fallo nunca se presenta como ausencia de evidencia

### Requirement: Evidencia de adjuntos legible sin ejecución

Cada hit SHALL mostrar su texto canónico y permitir inspeccionar IDs de trabajo/adjunto/generación, hash fuente, extractor/versión, hash de configuración, ordinal y localizador tipado. MUST presentar texto no confiable como datos escapados, identificar intentos anteriores y no inventar propietario, nombre de archivo ni vigencia posterior a la respuesta.

#### Scenario: Texto Unicode y procedencia completa
- **WHEN** se inspecciona un hit con saltos, caracteres multibyte, espacios y result_from_previous_attempt verdadero
- **THEN** se preserva el texto sin recortar ni normalizar, se ven identidad y hashes completos y se advierte que corresponde a un intento anterior; el ordinal de fragmento se distingue de los otros localizadores

#### Scenario: Contenido activo es sólo texto
- **WHEN** un fragmento contiene etiquetas HTML, scripts, enlaces, fórmulas o instrucciones para ejecutar herramientas
- **THEN** se muestra como texto, sin HTML interpretado, navegación o recursos automáticos, shell, evaluación de fórmulas ni llamadas a proveedores

### Requirement: Localizadores tipados de adjuntos en escritorio

La interfaz SHALL distinguir bytes/líneas UTF-8, página y bytes del texto PDF derivado, hoja/A1/celda/campo/rango XLSX y región/bbox/confianza/rango de texto inferido OCR. MUST mostrar límites del extractor y diferenciar fórmula de caché; no presentar offsets como geometría/ZIP, cachés como recálculo vigente o confianza como certeza.

#### Scenario: Rangos UTF-8 del original
- **WHEN** se muestra un hit utf8_bytes
- **THEN** se identifican byte_start/end con final exclusivo y line_start/end del original sin confundirlos con índices de caracteres JavaScript ni con offsets del extracto visual

#### Scenario: Página PDF sin geometría
- **WHEN** se muestra un hit pdf_page_text
- **THEN** se ven página, rango de bytes del texto derivado, scope y limitaciones no_ocr/no_visual_order/no_geometry/no_structured_tables devueltas; no se afirma posición visual, lectura de escaneo o tabla reconstruida

#### Scenario: Campo XLSX parcial y caché
- **WHEN** se muestra un hit xlsx_cell_text de caché o fórmula, incluso de hojas ocultas o campo dividido
- **THEN** se ven proyección, sheet_id, A1, ordinal global de celda, campo tipado, bytes totales y rango UTF-8 con final exclusivo; fórmula y caché permanecen separadas, se advierte ausencia de recálculo y no se inventa nombre/visibilidad de hoja, encabezado ni campo completo a partir de una porción

#### Scenario: Región OCR inferida
- **WHEN** se muestra un hit image_ocr_region_text, incluido un campo dividido o un resultado de intento previo
- **THEN** texto, fuente, proyección, región, bbox/confianza decimales, marco orientado y rango derivado permanecen completos; se identifica inferencia falible y no se inventan bytes originales, nombre/propiedad de nota, certeza o cobertura universal

### Requirement: Cobertura de adjuntos visible en escritorio

Cada página SHALL mostrar la cobertura filtrada de generaciones seleccionadas, compatibles sin indexar, incompatibles y trabajos sin generación, con fragmentos/bytes, perfil/revisión y alcance de verificación. MUST declarar que cero hits no prueba ausencia en toda la base ni verificación de todos los originales.

#### Scenario: Cobertura parcial sin recolección automática
- **WHEN** hay selecciones, generaciones compatibles sin seleccionar, incompatibles y trabajos sin generación
- **THEN** se muestran los contadores del núcleo y el alcance selected_generations; la interfaz indica que importar/extraer no indexa y que buscar no amplía ese conjunto

#### Scenario: Selección vacía o fuentes fuera de página
- **WHEN** una consulta devuelve cero hits con selecciones sin texto o hay originales ajenos a la página y su lookahead
- **THEN** se distingue índice canónico/postings verificados de originales de candidatos devueltos; no se afirma OCR, libro visualmente vacío o vigencia de todos los archivos, ni se confunde cantidad de fragmentos con total de coincidencias

### Requirement: Paginación de adjuntos acotada y reinicio deliberado

La interfaz SHALL mostrar una página acotada por vez, recorrer el cursor opaco sin mezclar consultas y ofrecer reinicio explícito. MUST invalidar resultados y cursor al cambiar texto/filtros, ámbito o workspace; ante cursor vencido o error retirar la evidencia vigente, mostrar la razón y no reiniciar, reparar ni ampliar límites automáticamente.

#### Scenario: Páginas sin acumulación
- **WHEN** se recorren páginas de un índice estable con más coincidencias que el límite visible
- **THEN** cada página reemplaza la anterior, conserva orden/identidad del núcleo y ofrece siguiente sólo con next_cursor; volver al inicio requiere una acción explícita y no se acumula toda la búsqueda en memoria o DOM

#### Scenario: Cambio de índice o fuente inválida entre páginas
- **WHEN** un put/reset/reemplazo vence el cursor o la fuente de un candidato es inválida
- **THEN** la siguiente lectura muestra el conflicto o error de fuente sin mezclar revisiones, omitir hits para rellenar la página ni presentar la anterior como vigente; buscar de nuevo es una decisión explícita

#### Scenario: Editar criterios invalida página anterior
- **WHEN** se cambia texto o cualquier filtro después de obtener una página o durante su lectura
- **THEN** se retiran página, detalle y cursor anteriores y no se ejecuta una nueva consulta hasta una acción explícita; una respuesta del criterio anterior no se publica bajo el nuevo

### Requirement: Aislamiento de lecturas de evidencia en escritorio

La interfaz MUST descartar éxito, error y actualizaciones tardías de lecturas invalidadas al cambiar criterios, ámbito, sección o workspace. SHALL preservar editor y guardias existentes, y ofrecer consulta, paginación e inspección con controles nombrados, foco visible y teclado en ambos temas, sin modificar Home.

#### Scenario: Respuesta tardía de otro workspace o ámbito
- **WHEN** una lectura queda pendiente y se navega a Home, notas u otro workspace antes de completarse
- **THEN** su resolución o rechazo no publica texto, cobertura, error ni estado de carga en la vista nueva ni dispara otra operación; tampoco recupera la evidencia al volver al ámbito

#### Scenario: Buffer y guardia preservados
- **WHEN** se busca o inspecciona evidencia con una nota editada y luego se intenta abrir otra nota o cambiar de sección/workspace
- **THEN** la consulta no altera el buffer; la navegación conserva guardar/descartar/cancelar, y cancelar conserva contenido, selección y foco de la guardia existente

#### Scenario: Teclado y temas sin cambio de Home
- **WHEN** se consulta, pagina e inspecciona un fragmento mediante teclado en claro y oscuro
- **THEN** los controles tienen nombres, foco visible, carga/error identificables y texto largo legible sin ejecutar contenido; Home conserva composición, navegación, posición del clima y secciones sin que estos checks acrediten IPC/foco/cierre nativos

### Requirement: Páginas independientes de consulta conjunta

La consulta conjunta SHALL mostrar hasta veinte hits por grupo con siguiente/inicio independientes, respetando contratos y orden del núcleo. MUST enviar la consulta literal sin truncarla, retirar ambos grupos al editar y no consultar al montar/cambiar texto. El alcance visible identifica entidades activas y generaciones seleccionadas, sin filtros cruzados ni suma de scores/conteos de cobertura como hits.

#### Scenario: Paginación por grupo
- **WHEN** hay más de veinte hits de notas y adjuntos y se pagina uno de los grupos
- **THEN** se reemplaza sólo su página, conserva el otro grupo y usa únicamente el cursor de su operación; inicio es explícito y no acumula todos los resultados

#### Scenario: Consulta excesiva y alcance
- **WHEN** se ingresa texto que excede un límite Rust, se edita la consulta o hay generaciones sin seleccionar
- **THEN** no se recorta la entrada para hacerla válida, los errores se distinguen por grupo y editar retira páginas/cursores sin consultar; el alcance no incorpora originales sin selección

### Requirement: Errores parciales y evidencia de consulta conjunta

Cada grupo SHALL mostrar carga, vacío, error y página propios; conservar fuente/revisión/bloque de notas y texto/procedencia/localizador/cobertura de adjuntos. MUST presentar contenido escapado y resultados anteriores como tales. Un fallo o cursor vencido retira sólo evidencia de ese grupo y no convierte la consulta parcial en ausencia universal ni repara/reinicia automáticamente.

#### Scenario: Fallo de una fuente
- **WHEN** falla la consulta de adjuntos o notas mientras el otro grupo responde
- **THEN** se conserva el grupo válido y se identifica el grupo fallido con código/razón y reintento explícito de esa lectura, sin repetir la otra consulta ni hacer writes

#### Scenario: Índice cambiado entre páginas
- **WHEN** un cambio del índice vence el cursor de adjuntos después de obtener ambos grupos
- **THEN** sólo adjuntos retira página/cursor y muestra conflicto, mientras notas conserva su snapshot sin etiquetarse como snapshot conjunto vigente

### Requirement: Lifetime y apertura protegida en consulta conjunta

La interfaz MUST descartar éxito/error tardíos de consultas invalidadas al cambiar texto/ámbito/sección/workspace o desmontar, manteniendo cupos físicos hasta terminar IPC y evitando esperas canceladas en cola. SHALL conservar buffers/guardias, apertura de notas por ID/localizador y controles de teclado/foco en ambos temas, sin alterar Home ni persistir contenido en el navegador.

#### Scenario: Criterios nuevos y llamadas tardías
- **WHEN** una consulta anterior termina después de editar o desmontar y se inicia otra
- **THEN** no publica datos/error en el nuevo contexto ni libera prematuramente el cupo; sólo las lecturas físicas admitidas por sus colas se ejecutan

#### Scenario: Abrir nota con buffer y usar teclado
- **WHEN** se consulta/pagina/inspecciona en ambos temas con nota pendiente y se abre un hit de entidad
- **THEN** el hit abre el original mediante guardar/descartar/cancelar y conserva el localizador/revisión; cancelar o fallo de guardado conserva buffer y los controles siguen legibles por teclado en viewport estrecho

### Requirement: Evidencia OCR seleccionada en ámbitos de búsqueda

Search individual y agrupada SHALL presentar OCR sólo de generaciones seleccionadas explícitamente, con páginas/cobertura/errores existentes y texto escapado. MUST conservar filtros/orden/cursores de cada motor, sin reextraer, corregir, fusionar scores, inferir propiedad del archivo o crear una ausencia universal a partir de cero detecciones/hits.

#### Scenario: Consulta mixta y evidencia inferida
- **WHEN** notas y adjuntos UTF-8/PDF/XLSX/OCR coinciden con una consulta local
- **THEN** los ámbitos conservan sus contratos y el grupo OCR muestra su evidencia/localizador/límites sin ejecutar contenido ni preparar fuentes

#### Scenario: Fuente sin seleccionar y error
- **WHEN** existe OCR extraído sin put o cambia índice/fuente durante búsqueda/paginación
- **THEN** la generación no seleccionada no produce hits y el error sigue las reglas del ámbito/grupo afectado, sin indexación, reparación o retry automáticos
