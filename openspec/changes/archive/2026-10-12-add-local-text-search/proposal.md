# Proposal

## Why

Fases 0–2 ya permiten capturar y organizar, pero encontrar texto todavía exige navegar listas. Se adelanta el primer tramo independiente de 5a, permitido por la hoja de ruta, para ofrecer búsqueda local sin modelos antes de colecciones/IA.

## What Changes

- FTS5 de títulos y texto de bloques conocidos, con revisión, entidad y bloque de origen.
- Actualización transaccional del índice y reconstrucción explícita sin alterar originales.
- Filtros de tipo, archivo y contexto, búsqueda literal acotada y paginación.
- CLI/Tauri/UI comparten search.query/search.rebuild; resultados abren la entidad original y señalan el bloque.
- SQLite v3 con backup previo y recuperación de índice. Este cambio NO completa 5a: cola asíncrona, extracción PDF/Excel/OCR/adjuntos siguen en otro cambio; tampoco embeddings, IA, ranking semántico o cuentas.

## Capabilities

### New Capabilities
- `local-text-search`: búsqueda textual, provenance y reconstrucción local del índice derivado.

### Modified Capabilities
Ninguna: se extienden versiones soportadas y operaciones sin retirar comportamientos existentes.

## Impact

Migración numerada v2→v3; Rust mantiene el índice junto a operaciones de entidad/revisión. SQLite y originales siguen autoritativos; FTS es regenerable. UI añade sólo búsqueda funcional, no placeholders de IA. Se medirán consultas release sobre 10.000 notas sintéticas en M1 Max, sin presentar benchmarks no ejecutados.
