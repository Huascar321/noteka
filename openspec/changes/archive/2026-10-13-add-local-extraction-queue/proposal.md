# Proposal

## Why

Al proponer el cambio, los adjuntos se conservaban con hash sin trabajos persistentes ni fragmentos extraídos; FTS sólo buscaba títulos y bloques. Después de fase 4, se recorta 5a para una base local verificable antes de parsers complejos, embeddings o modelos. Implementación y aceptación se registran en [la verificación](../../../../docs/verification-extraction.md), no se infieren de esta propuesta.

## What Changes

- Crear una cola persistente de extracción, desacoplada de la importación: selección explícita de adjuntos, deduplicación por fuente/configuración, estados, revisiones, intentos y errores. Pausada por defecto; procesar un trabajo por llamada explícita, sin daemon ni ejecución automática al abrir la app.
- Incorporar un primer extractor determinista de texto UTF-8 para originales administrados `.txt` y `.md`, hasta 1 MiB. Conservar texto literal, offsets de bytes y líneas; no interpretar Markdown, HTML, enlaces o instrucciones del contenido. Formatos no admitidos fallan explícitamente.
- Publicar fragmentos y generación completa en una única transacción, validando nuevamente fuente y ownership del trabajo. Cancelación, vencimiento o un worker anterior no pueden publicar; fallos conservan una generación previa válida y los originales.
- Exponer ocho operaciones compartidas `extraction.enqueue/get/list/run/cancel/retry/control/fragments`, catálogo y CLI JSON. No son pasos de lote; el catálogo pasa de 40 a 48 operaciones sólo al implementarlas.
- Proponer SQLite **v8**, con migración y backup previo, recuperación de trabajos interrumpidos y restauración en otra ruta sin revivir leases. Actualizar compatibilidad de batches: su snapshot incluye todo SQLite, pero la huella distingue configuración original de trabajos/resultados derivados.
- Verificar límites, concurrencia, fallos antes de publicación, regeneración, recuperación y costos release sobre fixtures sintéticas, sin red ni pantalla del usuario.

## Non-goals

No completar toda 5a: PDF, Excel, OCR e indexación/búsqueda de adjuntos requieren cambios posteriores sobre esta base. No embeddings, LLM, resúmenes, inferencia, descargas de modelos, MCP, Drive, proveedores remotos, UI nueva, scheduler residente, autoextracción al importar, edición/importación de Notion ni aceptación nativa de Home. No reemplazar notas, adjuntos, FTS existente ni introducir operaciones de extracción dentro de changes.preview/apply.

## Capabilities

### New Capabilities

- `local-extraction`: trabajos locales persistentes y acotados, ejecución separada, publicación atómica de derivados y fragmentos con procedencia verificable.

### Modified Capabilities

- `core-operations`: catálogo/CLI de extracción, paridad y compatibilidad de lotes frente a configuración original y datos operativos/derivados.
- `local-workspace`: migración/backup/restauración preservan cola y resultados sin reconocer como activo un worker de otra ruta o de un lease vencido.

## Impact

Nuevos módulos Rust de contratos, cola y extractor; migración `008.sql`, dispatcher/catálogo, CLI y contratos generados. Adaptaciones explícitas en workspace y batch_storage, más integración sobre SQLite real. Sin dependencias nuevas: UTF-8 usa Rust estándar; rusqlite/sha2/uuid existentes. Nuevas guías/verificación y ADR 0011 durante implementación, no como evidencia de adopción por crear este plan.

Decisiones del usuario: offline, Rust compartido, originales y pruebas sin pantalla. Recorte UTF-8/ocho comandos/v8/leases: decisiones técnicas adoptadas y verificadas en este cambio; [ADR 0011](../../../../docs/decisions/0011-explicit-local-extraction.md). No hay dependencia externa bloqueante. Home permanece independiente, 9/17 tareas verificadas.
