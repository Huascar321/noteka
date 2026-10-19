# Spec Delta

## ADDED Requirements

### Requirement: Selección explícita de generaciones extraídas

El sistema SHALL indexar mediante `extraction.index put` una generación confirmada UTF-8 o PDF por llamada, con trabajo y generación esperada. MUST verificar fuente, configuración y fragmentos antes de publicarla. Importar, extraer, abrir y buscar no seleccionan generaciones automáticamente; el índice empieza vacío.

#### Scenario: Extracción no implica indexación
- **WHEN** se importa un adjunto y se confirma su extracción sin llamar a index put
- **THEN** sus fragmentos siguen disponibles por el lector correspondiente, pero no aparecen en extraction.search y la cobertura los cuenta sin indexar

#### Scenario: Indexación local de ambos extractores
- **WHEN** se selecciona explícitamente una generación vigente utf8_text o pdf_text compatible, sin red, modelos ni helper PDF instalado
- **THEN** se indexan todos sus fragmentos de forma atómica sin ejecutar extracción y sin cambiar el original, el trabajo ni su generación

#### Scenario: Fuente o generación inadmisible
- **WHEN** falta el trabajo/generación, la configuración no es compatible o la fuente está ausente, alterada o fuera del almacenamiento seguro
- **THEN** se devuelve un error identificable y no se publica una selección parcial ni se modifica lo previamente indexado

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

Cada hit SHALL devolver trabajo, adjunto, generación, hash fuente, extractor/versión, hash de configuración, ordinal, texto canónico y localizador tipado. MUST verificar la fuente de los hits y distinguir bytes/líneas UTF-8 de página/rango del texto PDF derivado. Un resultado de un intento anterior se identifica; no se inventa una entidad propietaria, coordenada visual o cita OCR.

#### Scenario: Evidencia literal
- **WHEN** un hit corresponde a un fragmento UTF-8 con caracteres multibyte y saltos de línea
- **THEN** su texto coincide con el lector literal y su localizador conserva los rangos de bytes/líneas del original, no offsets de un snippet normalizado

#### Scenario: Evidencia PDF sin renderizador
- **WHEN** un hit corresponde a texto PDF confirmado y el helper no está disponible
- **THEN** su texto/localizador coincide con pdf_fragments, identifica página y rango del texto derivado y expone el alcance y limitaciones PDF, sin fingir offsets del archivo ni geometría

#### Scenario: Original cambiado después de indexar
- **WHEN** un candidato de la página procede de un archivo eliminado, alterado o sustituido por un enlace inseguro
- **THEN** la consulta falla con razón identificable, sin devolver esa página como evidencia vigente ni reparar, reextraer o borrar datos

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
