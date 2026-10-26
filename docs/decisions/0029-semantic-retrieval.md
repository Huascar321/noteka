# ADR 0029: Recuperación semántica local con escaneo exacto

- Fecha: 21 de octubre de 2026
- Estado: adoptado para fase 5b
- Ámbito: perfiles de embeddings, índice generacional, backend vectorial y búsqueda híbrida

## Contexto

La búsqueda de 5a es literal (FTS5) y no recupera paráfrasis. El plan pide un adaptador de embeddings independiente del LLM, un perfil reproducible, reindexación generacional con activación atómica y degradación a SQL+FTS. No hay autorización para instalar pesos ni activar proveedores; el producto debe funcionar offline sin índice semántico.

## Decisión

1. **Perfil explícito y huella SHA-256** sobre proveedor, runtime, modelo, revisión, dimensión, normalización, tarea, prompts query/document, tokenizer, chunking y cuantización. La huella identifica el espacio vectorial; el endpoint no forma parte de la huella pero se registra en la generación.
2. **Adaptador HTTP loopback** (`127.0.0.1`/`::1`, sin TLS ni redirecciones) compatible con `/v1/embeddings`. El adaptador aplica los prefijos del perfil; no asume que un endpoint genérico aplique los prompts de Jina.
3. **Chunking por fragmento de extracción** (`extraction_fragment_v1`): se reutilizan los fragmentos canónicos de 5a, sin truncar. Límites: 1024 fuentes, 8192 chunks, 8 MiB de texto y 4096 bytes por fragmento.
4. **Generaciones**: `building` → verificación → `active` en una transacción; la anterior pasa a `retired` y sólo se conserva una. La búsqueda usa una única generación y nunca mezcla huellas.
5. **Backend exacto en Rust**: producto punto sobre vectores normalizados, filtros SQL previos, deadline de 5 s y orden determinista. Se adopta por ser local, determinista, cancelable y no requerir extensión nativa.
6. **Fusión híbrida RRF** con FTS existente, deduplicación por chunk/texto exacto y presupuesto de texto. Sin generación o proveedor, se degrada explícitamente a FTS.

## Jina v5 Text Small

La ficha oficial consultada el 21/10/2026 reporta 677M parámetros, 1024 dimensiones (Matryoshka 32–1024), 32K tokens, licencia CC BY-NC 4.0 y adaptadores por tarea. Para retrieval el modelo usa `task=retrieval` y prompts `query`/`document`; la guía de best practice indica prefijos `Query:` y `Document:` con el adaptador de retrieval. El perfil inicial documentado usa:

- modelo `jina-embeddings-v5-text-small-retrieval` (adapter fusionado) o base con `task=retrieval`;
- dimensión 1024, normalización L2, cuantización f32;
- prefijos `Query: ` y `Document: `;
- tokenizer Qwen3-0.6B-Base;
- chunking `extraction_fragment_v1`;
- revisión de pesos obligatoria (no `latest`).

**No se instalaron pesos ni se ejecutó Jina.** Las pruebas usan un servidor loopback simulado y no acreditan calidad de recuperación. El endpoint OpenAI-compatible debe usar el adaptador de retrieval; un endpoint que ignore los prefijos no reproduce este perfil.

## Alternativas evaluadas

- **sqlite-vec**: no adoptado. Requiere extensión nativa, empaquetado por plataforma y verificación de binario/licencia que no se realizaron; no se declara medición.
- **HNSW/FAISS/servicio**: descartado por memoria, empaquetado y operación para el corpus acotado.
- **Archivo de vectores separado**: descartado porque rompe backup consistente y activación transaccional.
- **Búsqueda solo semántica**: descartada; el fallback FTS es obligatorio y los estados/cálculos no dependen de embeddings.

## Consecuencias

- El coste de consulta crece linealmente con chunks × dimensión. El benchmark reproducible mide 1, 4096 y 8192 chunks de 1024 dimensiones; el límite de 8192 es una decisión de producto, no una garantía para corpus mayores.
- El snapshot de lotes incluye el índice semántico; el límite de 64 MiB puede rechazar previews con índices grandes. No se eleva en este cambio.
- Cambiar cualquier factor de la huella exige reconstruir; no hay comparación entre espacios distintos.
- La calidad real de Jina queda pendiente de una ejecución autorizada con pesos y corpus representativo.

## Evidencia

`docs/verification-semantic-retrieval.md` conserva comandos, pruebas, benchmark y limitaciones. El benchmark del backend se ejecuta con `cargo test --release -p noteka-core --lib semantic_backend_benchmark -- --ignored --nocapture`.
