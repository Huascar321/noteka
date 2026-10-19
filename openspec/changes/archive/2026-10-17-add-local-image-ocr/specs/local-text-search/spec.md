# Spec Delta

## MODIFIED Requirements

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


## ADDED Requirements

### Requirement: Evidencia OCR seleccionada en ámbitos de búsqueda

Search individual y agrupada SHALL presentar OCR sólo de generaciones seleccionadas explícitamente, con páginas/cobertura/errores existentes y texto escapado. MUST conservar filtros/orden/cursores de cada motor, sin reextraer, corregir, fusionar scores, inferir propiedad del archivo o crear una ausencia universal a partir de cero detecciones/hits.

#### Scenario: Consulta mixta y evidencia inferida
- **WHEN** notas y adjuntos UTF-8/PDF/XLSX/OCR coinciden con una consulta local
- **THEN** los ámbitos conservan sus contratos y el grupo OCR muestra su evidencia/localizador/límites sin ejecutar contenido ni preparar fuentes

#### Scenario: Fuente sin seleccionar y error
- **WHEN** existe OCR extraído sin put o cambia índice/fuente durante búsqueda/paginación
- **THEN** la generación no seleccionada no produce hits y el error sigue las reglas del ámbito/grupo afectado, sin indexación, reparación o retry automáticos
