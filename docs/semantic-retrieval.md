# Recuperación semántica local

Fase 5b agrega un índice vectorial derivado sobre los fragmentos de extracción de 5a y una búsqueda híbrida con fuentes. SQLite y los originales siguen siendo la fuente de verdad; el índice se puede reconstruir sin tocar notas, documentos ni adjuntos. [ADR 0029](decisions/0029-semantic-retrieval.md).

## Operaciones

### `embedding.index`

```json
{"action":"status"}
{"action":"build","profile":{...},"endpoint":"http://127.0.0.1:8080/v1/embeddings",
 "job_id":null,"attachment_id":null,"extractor":null}
{"action":"reset","expected_revision":3}
```

- `status`: generación activa/construcción/retirada, revisión de control y cobertura (chunks, fuentes obsoletas, generaciones no indexadas).
- `build`: selecciona jobs de extracción con generación, valida originales y fragmentos, embebe y activa atómicamente. Si el digest de fuentes y la huella coinciden con la activa, reutiliza sin duplicar.
- `reset`: elimina sólo el índice derivado; exige la revisión actual.

Límites: 1024 fuentes, 8192 chunks, 8 MiB de texto, 4096 bytes por fragmento, dimensión 1–1024, perfil ≤8 KiB. Una construcción reciente bloquea otra; una abandonada se descarta tras 15 minutos.

### `retrieval.search`

```json
{"query":"¿qué acordamos sobre CRE?","mode":"hybrid","limit":10,
 "budget_bytes":16384,"job_id":null,"attachment_id":null,
 "extractor":null,"include_stale":false}
```

Modos: `hybrid` (default), `semantic`, `lexical`. Filtros antes de puntuar. La respuesta incluye `mode_requested`, `mode_used`, `degraded`, generación/huella, cobertura, presupuesto usado y por hit: job, adjunto, generación de extracción, extractor/versión, ordinal, texto, localizador, estado de fuente, puntuaciones/rangos semántico y léxico y score fusionado.

La fusión es reciprocal rank fusion (k=60). Se deduplica el mismo chunk y el texto idéntico del mismo adjunto. El presupuesto corta antes de excederse; no trunca una cita.

## Perfil y huella

El perfil registra: proveedor, runtime, modelo, revisión de pesos, dimensión, normalización, tarea, prompts query/document, tokenizer, chunking y cuantización. La huella SHA-256 cubre todos esos factores. Cambiar cualquiera crea un espacio distinto y exige reconstruir; no se comparan vectores de huellas diferentes.

Perfil documentado para Jina v5 Text Small (no instalado ni ejecutado):

```json
{
  "version":1,
  "provider":"local_http",
  "runtime":"openai_embeddings_v1",
  "model":"jina-embeddings-v5-text-small-retrieval",
  "revision":"<revisión de pesos>",
  "dimension":1024,
  "normalization":"l2",
  "task":"retrieval",
  "query_prefix":"Query: ",
  "document_prefix":"Document: ",
  "tokenizer":"qwen3-0.6b-base",
  "chunking":"extraction_fragment_v1",
  "quantization":"f32"
}
```

La ficha oficial (21/10/2026) indica 677M parámetros, 32K tokens, 1024 dimensiones Matryoshka y licencia CC BY-NC 4.0. Para retrieval usa `task=retrieval` y prompts `query`/`document`; la guía de best practice recomienda `Query:`/`Document:` con el adaptador de retrieval. El adaptador aplica esos prefijos; **no** se asume que un endpoint genérico los aplique. Usá un runtime que tenga el adaptador de retrieval fusionado.

## Endpoint local

Sólo `http://127.0.0.1[:puerto]/ruta` o `http://[::1][:puerto]/ruta`. Se rechazan HTTPS, hostnames, userinfo, query, redirecciones, respuestas >8 MiB y cuerpos chunked fuera de límite. Sin credenciales ni proxy. El endpoint se registra en la generación; no forma parte de la huella.

## Degradación

Si no hay generación activa o el endpoint falla, `retrieval.search` usa FTS y lo informa en `degraded` (`no_active_generation`, `query_embedding_failed`, `no_lexical_index`, `query_not_lexical`, `semantic_search_deadline`). No inventa resultados. Si tampoco hay FTS, devuelve página vacía con cobertura y razones.

## Backend

Se adopta escaneo exacto en Rust con producto punto sobre vectores normalizados, filtros SQL y deadline de 5 s. No se adopta sqlite-vec/HNSW: no hay verificación local ni empaquetado. Benchmark reproducible:

```sh
cargo test --release -p noteka-core --lib semantic_backend_benchmark -- --ignored --nocapture
```

Mide 1, 4096 y 8192 chunks de 1024 dimensiones (p50/p95 y bytes de vectores). No mide calidad de Jina ni latencia del modelo.

## Verificación y límites

Ver [verificación](verification-semantic-retrieval.md). Las fixtures con vectores sintéticos acreditan contratos, filtros, activación, presupuesto y degradación; no calidad semántica. Ejecutar Jina real requiere autorización de pesos y un corpus representativo.
