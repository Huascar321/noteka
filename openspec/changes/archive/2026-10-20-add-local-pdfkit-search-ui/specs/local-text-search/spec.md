# Spec Delta

## MODIFIED Requirements

### Requirement: Selección explícita de generaciones extraídas

El sistema SHALL indexar mediante `extraction.index put` una generación confirmada UTF-8, PDF, XLSX, OCR o PDFKit compatible por llamada, con trabajo y generación esperada. MUST verificar fuente, configuración y fragmentos/proyección antes de publicarla. Importar, extraer, abrir, migrar y buscar no seleccionan generaciones automáticamente; el índice empieza vacío.

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

Cada hit SHALL devolver trabajo, adjunto, generación, hash fuente, extractor/versión, hash de configuración, ordinal, texto canónico y localizador tipado. MUST validar la fuente y distinguir rangos/procedencia UTF-8, PDF/PDFKit, XLSX y OCR; identificar intentos previos sin inventar dueño, geometría o certeza OCR. PDFKit conserva perfil/motor/limitaciones; cachés XLSX no implican recálculo.

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

#### Scenario: Página PDFKit con perfil real
- **WHEN** se muestra un hit pdfkit_page_text de una página textual dividida
- **THEN** muestra perfil/proyección, página, bytes totales/rango UTF-8 y motor/OS/build/limitaciones reales, sin tratarlo como PDF estricto, OCR, geometría o una tabla


## ADDED Requirements

### Requirement: Evidencia PDFKit seleccionada en búsqueda individual y agrupada

Search SHALL incluir páginas PDFKit sólo tras selección explícita, con texto/procedencia/localizador íntegros, filtro de extractor y grupos/cursores/cobertura existentes. MUST conservar contenido escapado, errores/lifetimes y buffers; consultar o inspeccionar no ejecuta extracción, lectura adicional, indexación o reparación.

#### Scenario: Consulta mixta y páginas canónicas
- **WHEN** coinciden notas y generaciones seleccionadas de los cinco formatos con texto Unicode o contenido activo
- **THEN** Search mantiene motores/grupos separados y los hits PDFKit coinciden byte a byte con el rango del lector; sólo se muestra texto como datos, con límites del perfil y sin inferir dueño o scores conjuntos

#### Scenario: Vacío sin selección y error parcial
- **WHEN** hay PDFKit extraído sin seleccionar o falla/vencen sus criterios entre páginas
- **THEN** el primero no aporta hits y la cobertura lo distingue; el error retira sólo la página/grupo afectado sin autoindexar, reintentar, mostrar evidencia vieja o convertirlo en ausencia universal
