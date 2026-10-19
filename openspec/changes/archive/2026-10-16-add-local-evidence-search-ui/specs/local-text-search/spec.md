# Spec Delta

## MODIFIED Requirements

### Requirement: Búsqueda de escritorio offline

La interfaz SHALL buscar notas/entidades y adjuntos en ámbitos explícitos separados, sin fusionar scores ni cursores. MUST abrir entidades con su localizador, conservar buffers pendientes al navegar y mostrar evidencia y límites de fuente de los adjuntos sin cargar servicios remotos ni ejecutar extracción o indexación implícitas.

#### Scenario: Buscar y abrir sin red
- **WHEN** se busca una nota enriquecida sin red ni modelos
- **THEN** se abre la entidad original con referencia de bloque/revisión y se conserva la protección de edición pendiente

#### Scenario: Cambiar de ámbito sin cambiar el editor
- **WHEN** se cambia entre notas/entidades y adjuntos mientras hay un buffer pendiente
- **THEN** el buffer permanece intacto, los filtros de tipo/archivo/contexto sólo se aplican a entidades y cada ámbito usa su propio contrato sin mezclar resultados ni cursores

#### Scenario: Consultar adjuntos no es procesarlos
- **WHEN** se entra al ámbito de adjuntos, se busca o se inspecciona un hit
- **THEN** no se importa, encola, ejecuta, selecciona, resetea ni reconstruye contenido o índices; los adjuntos sin selección explícita no se incorporan a la búsqueda

## ADDED Requirements

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

La interfaz SHALL distinguir bytes/líneas UTF-8, página y bytes del texto PDF derivado, y hoja/A1/celda/campo/rango XLSX. MUST mostrar límites del extractor y diferenciar fórmula de caché; no presentar offsets como geometría/ZIP ni cachés como recálculo vigente.

#### Scenario: Rangos UTF-8 del original
- **WHEN** se muestra un hit utf8_bytes
- **THEN** se identifican byte_start/end con final exclusivo y line_start/end del original sin confundirlos con índices de caracteres JavaScript ni con offsets del extracto visual

#### Scenario: Página PDF sin geometría
- **WHEN** se muestra un hit pdf_page_text
- **THEN** se ven página, rango de bytes del texto derivado, scope y limitaciones no_ocr/no_visual_order/no_geometry/no_structured_tables devueltas; no se afirma posición visual, lectura de escaneo o tabla reconstruida

#### Scenario: Campo XLSX parcial y caché
- **WHEN** se muestra un hit xlsx_cell_text de caché o fórmula, incluso de hojas ocultas o campo dividido
- **THEN** se ven proyección, sheet_id, A1, ordinal global de celda, campo tipado, bytes totales y rango UTF-8 con final exclusivo; fórmula y caché permanecen separadas, se advierte ausencia de recálculo y no se inventa nombre/visibilidad de hoja, encabezado ni campo completo a partir de una porción

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
