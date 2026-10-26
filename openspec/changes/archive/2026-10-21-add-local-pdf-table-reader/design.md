# Design

## Context

Ver proposal. Baseline verificado16/API56; biblioteca independiente noteka-pdf-table
cerrada6/6, real PDF y benchmark. Core ahora la consume mediante API57, bajo
verificación conjunta antes de sincronizar o archivar este cambio.
Fuentes/catálogos y clientes56 conservados en evidencia externa21/10.

## Goals / Non-Goals

**Goals:** lectura de tabla verificable, operación compartida y UI explícita.
**Non-Goals:** persistencia de layouts, extractor nuevo, detección genérica, cambios
en PDFKit/Vision, modelos, Home o activación implícita de preparación/búsqueda.

## Decisions

1. DTO ReadPdfOcrTable `{id,page_number,x_edges,y_min,y_max,header_row?,limit?,cursor?}`.
API57 aditiva con CLI/scoped/MCP; actual IndexedExtractor sigue pdf_ocr (no crear
extractor pdf_table). Core depende de noteka-pdf-table y consume su Input interno;
los DTOs públicos del núcleo son propios y JsonSchema no exige publicar toda la
biblioteca. Raíces Contract se agregan al final para conservar Page<T> previos.
2. Transacción Deferred: job/generación/config admitida, original completo vía
source/hash y pdf_ocr_storage::load íntegro antes de seleccionar página. Construir
Input con reconocimiento ya validado y hash real de generación; no aceptar evidence
ni source_sha256 del cliente. Proyectar todas las celdas antes de paginar. No helper,
I/O externo del proyector, reparación ni modificación de canónico/FTS.
3. Output PdfOcrTablePage: job/generation/result_from_previous_attempt; manifiesto
completo PDF OCR y descriptor raster/motor físico; perfil/grid/header, row_count,
column_count/cell_count, referencias exteriores locales/globales y limitaciones.
Items celda tienen ordinal row-major0-based, row/column1-based y regiones completas
con PdfOcrRegion (global ordinal=conteos previos+ordinal local). Celdas vacías se
conservan. Header sólo referencia fila; no copiar textos ni quitar celdas.
4. covers_table sólo true en una única página completa; covers_generation false:
la proyección no acredita devolver el derivado original. Exteriores se preservan
como referencias visibles al lector pdf_ocr existente, no se descartan del canónico.
No autoload de todas las regiones o reconstrucción de tablas desde Search.
5. Cursor wrapper <=2KiB: Cursor común y hash de elección (namespace específico
v1 sobre perfil/página/x_edges/y_min/y_max/header); comparar scope/job/generación,
límite y grid íntegros. Posición es ordinal de celda,>0 y<cell_count. Response<=2MiB
incluye envelope; mantener rechazo explícito sin duplicar sourceRecognition completa
por cada celda. Máximos de biblioteca no se amplían.
6. UI amplía sólo lector/controlador existente con opción pdf_ocr_table (extractor
real pdf_ocr), página, bordes columna y rectángulo normalizados y header opcional.
Puede usar JSON estricto para bordes; Rust decide semántica. Una página20 celdas,
regiones/valores escapados, sin acumular, y manifiesto/exteriores diferenciados.
Criterios completos retenidos para siguiente; edición/salida invalida lifetime pero
cupo físico sigue hasta IPC real. Tests con PDF real propio, late/error/capacity.
7. Search/índice existentes no cambian: regiones OCR ya son evidencia textual,
layout es elección efímera. Contrastar búsqueda individual/agrupada con lector,
ordinales/bbox/procedencia y ausencia de nuevos duplicados/indexación automática.
No se exige SQLite17 para una lectura sin persistencia; decidir por nuevo cambio
si alguna necesidad real exigiera guardar grids, antes de escribir DDL.

## Risks / Trade-offs

- OCR puede unir columnas o cometer errores → conservar texto y rechazar cruces.
- Exteriores no muestran texto en cada página de tabla → refs completas y lector
original accesible explícitamente; no confundir cobertura de tabla con generación.
- Más DTOs/catalog → comparar56 contratos completos y roots/errores/batch9, sin
cambiar hashes históricos para aceptar extensiones ajenas.
- No promesa de tabla genérica → perfil/limitaciones claros; evaluar pendientes
reales de5a antes de empezar5b, no cerrar sólo por completar esta tarea.

## Migration Plan

Sin DDL, original/canónico/índice/snapshots/retención intactos. Preservar cliente56
y fixture real mixta antes de integrar, probar lecturas/backup/restore y admisión
scoped. No instalar pesos/proveedores. Query/lock300/150ms y gates originales
intactos, sin compilación paralela; investigar fallos y preservar evidencia.
