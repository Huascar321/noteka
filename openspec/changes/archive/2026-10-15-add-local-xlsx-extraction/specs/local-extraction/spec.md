# Spec Delta

## MODIFIED Requirements

### Requirement: Lector XLSX independiente y sin activación implícita

El sistema SHALL ofrecer un lector local aislado de XLSX Transitional mediante un protocolo privado acotado de bytes, sin rutas ni acceso al workspace. MUST separar ese helper de las operaciones del core: importar, abrir, leer, encolar o migrar no inicia parsing; sólo `extraction.run` explícito sobre un trabajo admitido lo ejecuta.

#### Scenario: Lectura explícita offline
- **WHEN** se invoca el helper con un XLSX sintético admitido y red denegada
- **THEN** devuelve una representación determinista completa del subconjunto declarado, sin modificar archivos ni iniciar UI, modelos o conexiones

#### Scenario: Producto existente conservado
- **WHEN** se consulta el catálogo, abre un workspace actual o importa/encola XLSX sin ejecutar run
- **THEN** API v1 conserva sus 51 operaciones previas y añade sólo el lector XLSX; UTF-8/PDF/Home mantienen su comportamiento y no se inicia ningún helper

## ADDED Requirements

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

El sistema MUST rechazar `extraction.index` para `xlsx_cells` con limitación explícita antes de reemplazar el índice; no interpreta XLSX como UTF-8/PDF. SHALL conservar índice/búsqueda de notas y adjuntos UTF-8/PDF, validación completa de postings/copias/fuentes, caps existentes y target query p95 <=300 ms sobre el corpus de aceptación fijado.

#### Scenario: Intento de indexar XLSX
- **WHEN** se intenta indexar una generación XLSX existente junto a un índice UTF-8/PDF válido
- **THEN** se rechaza como extractor no indexable y el índice, búsqueda y originales previos quedan iguales

#### Scenario: Regresión de búsqueda y contratos
- **WHEN** se repite el corpus/gate fijo y se procesan fuentes UTF-8/PDF anteriores después de integrar XLSX
- **THEN** pasan las validaciones completas y el target sin subir caps ni omitir fuentes; las 51 operaciones anteriores conservan sus esquemas y semántica
