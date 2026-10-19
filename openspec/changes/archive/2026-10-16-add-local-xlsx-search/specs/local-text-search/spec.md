# Spec Delta

## MODIFIED Requirements

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

## ADDED Requirements

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
