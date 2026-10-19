# Proposal

## Why

Noteka ya conserva fragmentos UTF-8 y PDF verificables, pero `search.query` sólo encuentra títulos y bloques de entidades. El siguiente recorte independiente de 5a permite encontrar esos adjuntos con evidencia local antes de incorporar embeddings o agentes.

## What Changes

- Añadir `extraction.index` con acciones estrictas `get/put/reset`: indexación FTS5 explícita de una generación confirmada por llamada, revisión esperada del índice y generación esperada. Reset borra sólo el índice derivado; importar, extraer, abrir y buscar no indexan automáticamente.
- Añadir `extraction.search`: términos literales acotados, filtros de trabajo/adjunto/extractor, ranking textual y cursor ligado a la revisión del índice. Cada hit devuelve el fragmento canónico verificado, identidad/hash de generación y localizador literal o de texto PDF por página, sin confundir ambas medidas.
- Exponer cobertura del **conjunto seleccionado para indexar**, generaciones disponibles sin indexar y configuraciones incompatibles. Cero hits no afirma haber buscado todos los adjuntos ni transcrito imágenes. Los adjuntos son fuentes independientes: no se infiere una nota propietaria ni se aplican filtros de archivo/contexto de entidades a este endpoint.
- Invalidar transaccionalmente entradas cuando se reemplaza su generación; preservar índice/generación anteriores si el reemplazo falla. Consultar revalida fuentes y derivados de los hits sin parser, modelos, red ni escrituras de reparación.
- Proponer SQLite v10 mediante `010.sql` aditiva: índice inicialmente vacío, backup previo y preservación de 001–009/filas existentes. Mantener API v1 y los 49 contratos actuales; catálogo propuesto de 51 operaciones, sin ampliar la whitelist de lotes. Caps propios de 1.024 selecciones/16.384 fragmentos/8 MiB de texto, separados de los 32 MiB de extracción y 64 MiB de snapshot.

**Non-goals:** búsqueda unificada de notas/adjuntos, UI o navegación PDF, resolver relaciones de un adjunto compartido con entidades, filtros por archivo/contexto de notas, extracción nueva, OCR/Excel/fuentes PDF adicionales, indexación residente, embeddings/Jina, modelos/harness/MCP, proveedores y sincronización. No se modifica Home ni se necesita aceptación nativa para este recorte. No completa toda 5a ni habilita por sí solo 5b.

## Capabilities

### New Capabilities

Ninguna: se amplían búsqueda textual, operaciones compartidas y recuperación existentes.

### Modified Capabilities

- `local-text-search`: índice seleccionado de generaciones extraídas, búsquedas/localizadores tipados, vigencia, cursores y cobertura explícita; separación de la búsqueda de notas.
- `core-operations`: dos operaciones aditivas por núcleo/CLI, DTOs/errores estrictos, límites y compatibilidad de lotes sin ejecutar indexación en preview.
- `local-workspace`: migración recuperable v9→v10, integridad y backup/restauración de índices confirmados sin parser ni recuperación de jobs.

## Impact

Módulo Rust de búsqueda de adjuntos separado, `extraction*`/`pdf_storage`, publicación de generaciones, contratos/catálogo/CLI, migraciones/registry/backup y tests. Se reutilizan SQLite FTS5/rusqlite existentes; sin motor o runtime nuevos. La optimización medida habilita `sha2/asm` sólo en aarch64 y añade únicamente sha2-asm 0.6.4 al lock, con detección runtime/fallback portable y hashes persistentes idénticos; [ADR 0013](../../../../docs/decisions/0013-explicit-attachment-index.md). `search.rs` y sus operaciones query/rebuild, lectores literal/PDF, formatos documentales, originales y reglas de extracción permanecen intactos.

**Decisiones conservadas:** Rust concentra reglas; SQLite/originales son autoridad; consentimiento remoto separado; pruebas UI sólo headless. Índice separado con copia verificable del texto, selección por generación y CAS global; detalles, alternativas y gates en [diseño](design.md). Estado: **19/19 tareas verificadas, sincronizado y archivado el 14/10/2026**, baseline vigente v10/51. Regresión/revisión aprobadas y dos benchmarks consecutivos de query p95 214,20–231,07 ms sobre el corpus fijado (<300 ms), lock aprobado y scratch físico medido. Sin rebajar targets/caps ni retirar validación; no garantiza p95 universal. [Evidencia y cierre](../../../../docs/verification-attachment-search.md): tres deltas contrastados íntegramente con main, 15 requisitos añadidos/40 escenarios nuevos y un requisito de migración reconciliado con sus cinco escenarios heredados. Home sigue activo 9/17, sin tocar.
