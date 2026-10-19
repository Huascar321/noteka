# Design

## Context

Fases 0–2 archivadas. Se adelanta FTS básico de 5a por valor offline y menor dependencia que colecciones/embeddings. No se incluye procesamiento de adjuntos ni se declara terminada la fase 5a completa.

## Goals / Non-Goals

**Goals:** búsqueda textual con localizadores, consistencia, filtros, rebuild y evidencia de rendimiento.

**Non-Goals:** cola/extracción/OCR, vectores/reranking, respuesta LLM, parsear HTML, sincronización, índices ajenos a SQLite.

## Decisions

FTS5 dentro de SQLite v3: entity_id/block_id/revision UNINDEXED, title/text indexados, unicode61 remove_diacritics 2. Un registro de título por entidad y registros por hojas con texto; contenedores no duplican texto de hijos. No indexar JSON desconocido suponiendo forma: sólo título; originales intactos. Actualizar filas en las mismas transacciones de creación/update/bump de revisiones. Archivo conserva filas con filtro por entities.archived_at; restauración mantiene IDs. Query verifica revision = entities.revision antes del ranking.

search.query: texto <=256 bytes, hasta 16 términos separados por whitespace, AND de frases escapadas literales; sin sintaxis FTS arbitraria ni prefijos. Filtros tipo, archived (false activos / true sólo archivo), related_id (proyecto/tag/persona). Orden bm25, entity_id, block_id; cursor offset acotado 0–100.000, límite 1–100. No se garantiza snapshot entre páginas concurrentes. Resultados con extracto plano (sin HTML), entity_id/title/kind/revision/block_id y score. La UI mantiene esos localizadores y abre con guardia de buffer.

search.rebuild usa una transacción inmediata y reindexa todos los originales reconocidos; rollback ante error. No modifica entidades/historial/archivos. Migración 003 crea FTS, reconstruye y valida antes de commit, con backup SQLite+originales v2 previo. Backups v3 validan esquema requerido y archivos; no se confunde índice con autoridad. Una pérdida de filas puede repararse con rebuild; corrupción estructural del archivo SQLite usa recuperación/backup, no se oculta como búsqueda vacía.

## Risks / Trade-offs

Consistencia sin cola evita resultados stale y añade costo de indexar al guardado. FTS inicial opera títulos/bloques, no archivos ni propiedades genéricas futuras. Los filtros de tareas siguen workflow.query; búsqueda no inventa estados. Rebuild síncrono puede tardar con corpus grandes; será explícito, no al abrir cada workspace. Se medirá release y se documentará si no alcanza objetivo p95 <300 ms en 10.000 notas, sin maquillar el método de acceso.
