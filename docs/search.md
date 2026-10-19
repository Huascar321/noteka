# Búsqueda textual local

`add-local-text-search` adelanta el tramo textual de 5a. **No completa extracción, cola, OCR, embeddings ni QA con modelos.**

## Uso y contrato

- UI: Search → texto → Buscar; tipo/archivo/contexto opcionales. Abrir un resultado conserva guardia de buffer y muestra entidad/bloque/revisión. Si cambió el original entre consulta y apertura, el localizador advierte la revisión vigente. Reconstruir índice no modifica originales.
- CLI `search query --json`: `{query,kind?,archived?,related_id?,limit?,cursor?}`. `archived:false` activos, `true` sólo archivados; contexto ID local de proyecto/tag/persona. Resultado `{items,next_cursor}`, cada fuente con entity_id/title/kind/revision/block_id/excerpt/score.
- Hasta 256 bytes UTF-8, 1–16 términos por whitespace, AND literal; sin sintaxis FTS pública, prefijos, ejecución SQL/shell ni interpretación de instrucciones contenidas en notas. Tokenización unicode61 insensible a mayúsculas y diacríticos latinos. No es búsqueda por subcadenas; puntuación se tokeniza según FTS5.
- Límite 1–100 (default 30). Cursor es offset JSON acotado hasta 100.000; orden bm25/entity_id/block_id. No garantiza snapshot entre páginas ante escrituras concurrentes.
- `search rebuild --json` recibe `{}` y devuelve `{entities,chunks}`. Repara filas ausentes/desactualizadas a partir de SQLite; rollback ante error. Una corrupción estructural de SQLite requiere recuperación/backup, no se oculta como una consulta vacía.

```sh
printf '%s' '{"query":"acuerdo CRE","kind":"note"}' | ./target/debug/noteka --workspace /ruta/al/workspace search query --json
printf '%s' '{}' | ./target/debug/noteka --workspace /ruta/al/workspace search rebuild --json
```

## Persistencia y alcance de fuentes

SQLite v3 agrega FTS5, migración 003 con backup previo y reconstrucción inicial. FTS es derivado: un registro por título y otro por cada bloque de texto conocido, incluyendo hijos anidados. No duplica texto de contenedores. Creación/edición/cambios de revisión reindexan en la misma transacción; una falla revierte dominio e índice juntos. Las consultas excluyen filas cuya revisión no coincide con entities.

Documentos desconocidos se conservan sin convertir y sólo su título se indexa. No se busca dentro de adjuntos, JSON arbitrario ni atributos de archivos. Un resultado es evidencia textual, no una respuesta generada. Originales y relaciones siguen autoritativos.

## Medición local reproducible

`cargo run --offline --release -p noteka-core --example search_benchmark`

Apple M1 Max, 64 GiB RAM, aarch64. Corpus temporal sintético: 10.000 notas con título y un párrafo ~400 bytes; 80 consultas por modo, mezcla de términos comunes, selectivos, conjunción y ausencia, caches calientes, limit 30. Se mantiene una conexión para medir query y se mide aparte execute_json con apertura/preflight reales (incluye copia de SQLite/WAL). No incluye UI, arranque frío ni modelos.

Resultado del 12/10/2026: **p95 16,69 ms** consulta directa; **p95 22,92 ms** operación completa con apertura/preflight. Objetivo propuesto <300 ms cumplido **para este corpus y método**, no una garantía universal. Archivo SQLite principal 19.357.696 bytes (excluye WAL). Evidencia temporal: `noteka-search-benchmark.json` en el directorio de pruebas fuera de Git. No implica mediciones de guardado/UI/adjuntos ni optimización de inferencia.
