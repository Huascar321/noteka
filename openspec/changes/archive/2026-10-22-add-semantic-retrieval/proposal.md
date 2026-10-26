# Proposal

## Why

La búsqueda actual es literal y depende de términos exactos. La fase 5b necesita recuperar fragmentos por significado, mantener el índice reproducible cuando cambie el modelo y conservar una degradación local a SQL/FTS cuando el runtime de embeddings no esté disponible. Sin un perfil de espacio vectorial explícito y una activación generacional atómica, sería imposible distinguir vectores incompatibles o fuentes obsoletas.

## What Changes

- Nuevo adaptador de embeddings independiente del proveedor LLM, con perfil completo y huella SHA-256 reproducible (proveedor, runtime, modelo, revisión, dimensión, normalización, tarea retrieval, prompts query/document, tokenizer, chunking y cuantización).
- Nueva migración aditiva 017 con perfiles, generaciones de índice semántico, chunks derivados y control de generación activa. No reemplaza originales, documentos, bloques, propiedades desconocidas ni extraction_*.
- Reindexación generacional: construir aparte, validar cobertura/consistencia y activar en una única transacción; la generación anterior queda retirada y la búsqueda nunca mezcla huellas.
- Búsqueda híbrida con filtros (job, adjunto, extractor), fuentes/localizadores, deduplicación exacta, presupuesto de contexto y fallback SQL+FTS. Si el runtime de embeddings falla, la respuesta se degrada explícitamente en lugar de inventar resultados.
- Evaluación del backend vectorial con benchmark reproducible y ADR. Se implementa un escaneo exacto acotado; sqlite-vec/HNSW no se adoptan sin verificación local.
- Operaciones nuevas `embedding.index` y `retrieval.search`, expuestas por CLI y MCP con las mismas reglas.
- Endpoint local de embeddings exclusivamente loopback (HTTP sin TLS), sin credenciales ni envío remoto. La preferencia Jina v5 Text Small se registra como perfil documentado, no se instala ni ejecuta en este cambio.

## Capabilities

### New Capabilities

- `semantic-retrieval`: perfiles de embeddings, generaciones de índice, activación atómica, búsqueda híbrida con filtros/fuentes/deduplicación/presupuesto y degradación SQL+FTS.

### Modified Capabilities

- `core-operations`: catálogo/CLI/contratos para `embedding.index` y `retrieval.search`, con validación y errores estructurados.
- `local-workspace`: migración aditiva 017, backup/restauración y rechazo de versiones futuras sin descartar datos desconocidos.

## Impact

- Rust: `noteka-core` (nuevos módulos de contratos, almacenamiento, proveedor local HTTP y búsqueda), `noteka-cli`, `noteka-mcp`, `batch_storage` (registro de esquema) y migración 017.
- SQLite pasa de 16 a 17; API v1 de 57 a 59 operaciones; MCP de 29/49/50 a 32/53/55; las nueve mutaciones batchables no cambian.
- No se activan modelos, proveedores remotos, OAuth, pagos ni publicación. Las pruebas usan servidores/fixtures locales sintéticos y no acreditan calidad real de Jina.
- Home 9/17 permanece intacto. Fase 6 no se inicia.
