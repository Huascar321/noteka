# Proposal

## Why

El proyector de tablas PDF ya está verificado como biblioteca independiente, pero el usuario aún no puede consultar sus resultados desde Noteka. Ésta es la integración pendiente de tablas de5a antes de evaluar su cierre y avanzar5b.

## What Changes

- Añadir lectura explícita `extraction.pdf_ocr_table` sobre una generación PDF OCR confirmada, con página/rectángulo/columnas/fila de encabezado elegidos y sin ejecutar OCR.
- Paginar celdas completas con regiones/procedencia y manifiesto de cobertura, conservando celdas vacías y referencias a regiones exteriores.
- Compartir API/CLI/MCP/UI; reutilizar lector/preparación existentes sin cambiar Home. Buscar sigue usando las regiones PDF OCR seleccionadas; no se crea otro índice ni autoridad para una elección transitoria de columnas.
- Verificar compatibilidad56 contratos, pureza, fuente/derivado completos, UI headless y gates300/150 antes de sincronizar/archivar.

## Non-goals

No extractor nuevo, DDL, persistencia de grids, cambios de PDFKit/OCR/FTS, detección genérica de tablas, texto dividido/spans, modelos, proveedores o fase6. No declarar integración a partir de biblioteca o planificación. Evaluar pendientes reales5a antes de5b/5c.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `local-extraction`: proyección paginada explícita de tablas de una generación PDF OCR validada.
- `core-operations`: operación aditiva APIv1/CLI con56 contratos anteriores íntegros y batch9.
- `local-mcp`: lectura por permisos existentes y catálogo29/49/50, sin helper o autoridad por contenido.
- `desktop-shell`: elección de grid/página y lectura paginada con cobertura/celdas/exteriores y buffers preservados.

## Impact

Rust core depende del crate noteka-pdf-table cerrado; loader PDF OCR completo, DTOs/scoped/dispatcher/CLI/bindings y lector UI. API57 implementada bajo verificación, desde baseline16/API56/MCP28/48/49, sin DDL ni representación nueva. Fuentes/catálogos/clientes56 independientes preservados fuera de Git. No recrear ni resincronizar los componentes independientes cerrados; sólo adaptar consumo. Home9/17 intacto. El corte horario de la planificación inicial quedó revocado; la aceptación requiere todos los checks y gates.


Autorización vigente21/10: el usuario levantó el límite de una hora; continuar hasta completar5a, luego5b y5c, y detenerse antes de fase6. Los cortes horarios citados arriba son históricos y están revocados. Se conservan todos los invariantes, Home9/17 y la prohibición de activar modelos/proveedores.
