# Spec Delta

## ADDED Requirements

### Requirement: Lectura explícita de tabla PDF OCR vigente

`extraction.pdf_ocr_table` SHALL proyectar una página de generación PDF OCR confirmada con grid/encabezado elegidos, usando el proyector verificado sin OCR nuevo. MUST validar original y representación completos antes de devolver celdas, manifiesto, motor/raster/limitaciones y referencias exteriores, sin persistir grids o modificar fuente/FTS.

#### Scenario: Tabla desde derivado confirmado
- **WHEN** se elige página/grid válido sobre una generación existente sin helper instalado
- **THEN** devuelve celdas completas, vacías incluidas, con fila/columna1-based y regiones textuales/ordinals locales/globales exactos; el manifiesto distingue cobertura de tabla/página/generación

#### Scenario: Fuente corrupción y proyección ambigua
- **WHEN** cambia original/canónico/configuración o una región cruza grid/filas ambiguas, incluso fuera de la página pedida
- **THEN** falla completo con razón útil, conserva generación previa y desconocidos sin reparación, truncamiento, extracción o writes

### Requirement: Páginas de tabla PDF ligadas a elección y generación

El lector SHALL paginar1–64 celdas/default32 en<=2MiB con cursor<=2KiB ligado a workspace/ruta/operación/job/generación/grid/página/header/límite. MUST conservar límites512 regiones/512 celdas/32 columnas/64 filas del proyector y referencias exteriores, sin convertir una página final en cobertura total ni duplicar índice textual.

#### Scenario: Cursores cobertura y región exterior
- **WHEN** se recorren celdas en varias páginas y hay regiones fuera del rectángulo
- **THEN** una página reemplaza la anterior; cada celda mantiene ordinal/localizadores y las referencias exteriores completas permiten volver al lector OCR original sin descartes o cobertura ficticia

#### Scenario: Criterios obsoletos y pureza
- **WHEN** cambia cualquier criterio/revisión de generación o se consulta tras restore en otra ruta
- **THEN** el cursor ajeno falla y una lectura nueva funciona sin modificar datos, fuente, selección FTS o contenidos desconocidos
