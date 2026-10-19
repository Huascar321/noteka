# ADR 0004: índice textual transaccional

- Fecha: 12 de octubre de 2026.
- Estado: adoptada para el tramo textual de 5a.

## Decisión

FTS5 se aloja dentro de SQLite v3 y es regenerable desde entidades/documentos. Los localizadores conservan entidad, revisión y bloque; documentos desconocidos sólo exponen título reconocido. Se separa una fila de título de las filas de texto de hojas para evitar duplicar matches de contenedores.

Las operaciones de dominio reindexan en su transacción: indexar fallido revierte la edición y el historial. Consulta exige revisión actual y filtra archivo/tipo/contexto. Rebuild explícito es atómico y no modifica originales. La migración v2→v3 conserva backup previo e índices se reconstruyen antes de confirmar.

La API expone términos literales escapados, no expresiones SQL/FTS arbitrarias. Extractos son texto plano. Orden bm25 + IDs; paginación por offset acotado sin snapshot multipágina. Se conserva SQLite y originales como autoridad, sin convertir Markdown/embeddings en almacenamiento.

## Alternativas y consecuencias

- Índice en archivo separado: añade coordinación/crash consistency, sin necesidad actual; descartado.
- Cola para textos pequeños: introduce staleness en operaciones básicas; se reserva para extracción de archivos/modelos costosos.
- HTML marcado de snippets: riesgo de interpretar contenido y mayor adaptación; extracto plano elegido.
- Resultado por entidad solamente: pierde localizador exacto; se presentan fuentes por bloque/título.

El costo al guardar crece con el documento. El benchmark release en 10.000 notas mide consulta, no ese costo. Cola, extracción de PDF/Excel/OCR y embeddings permanecen en cambios posteriores de 5a/5b.
