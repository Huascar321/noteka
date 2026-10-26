# Design

## Contexto y alcance

Fase 5b agrega recuperación semántica sobre los fragmentos de extracción ya verificados por 5a. No reemplaza la búsqueda literal, no introduce un LLM y no activa Jina ni ningún proveedor remoto. El producto debe seguir funcionando sin índice semántico y degradar a SQL+FTS con una razón visible.

## Ownership y modelo de datos

- **Originales y extracción**: `attachments`, `extraction_*` y FTS siguen siendo la fuente de verdad. El índice semántico es derivado y regenerable.
- **Perfil**: DTO inmutable `EmbeddingProfile`; su huella SHA-256 canonicaliza todos los factores que cambian el espacio vectorial. El endpoint local no forma parte de la huella porque no cambia el espacio, pero sí se registra en la generación para reproducir la ejecución.
- **Generación**: fila en `semantic_generations` con estado `building|active|retired|failed`, huella, endpoint, digest de fuentes, conteos y fechas. Sólo una puede estar `active`; una restricción parcial permite una sola `building`.
- **Chunk**: copia derivada del texto del fragmento (≤4096 bytes) con job/adjunto/generación de extracción, hash de contenido, extractor/versión, ordinal, hash de texto, localizador JSON y vector f32 LE normalizado.
- **Control**: `semantic_control` singleton con revisión y `active_generation_id`. La revisión se incrementa sólo al activar o resetear.

La migración 017 es aditiva. No toca tablas previas ni crea FK hacia `extraction_generations`: un chunk huérfano debe poder marcarse obsoleto, no desaparecer en cascada. El backup SQLite incluye el esquema semántico; la restauración reutiliza la validación de integridad existente.

## Flujo de construcción

1. Validar perfil, endpoint loopback y filtros. Leer revisión de control.
2. Seleccionar jobs con generación y aplicar filtros; usar `attachment_search_storage::canonical` para obtener fragmentos y localizadores ya validados. Verificar el hash del original con `index::source`.
3. Rechazar antes de embeber si se superan 1024 fuentes, 8192 chunks, 8 MiB de texto o 4096 bytes por fragmento.
4. Calcular digest de fuentes (job, generación, content_hash) y comparar con la generación activa de la misma huella. Si coincide, devolver reutilización sin duplicar vectores.
5. Insertar la generación `building` en una transacción corta. Embeber en lotes (16 textos) con prefijo documental y deadline total. Insertar chunks por lotes en transacciones cortas; la búsqueda no ve `building`.
6. Verificación final: conteos, bytes, dimensión, finitud/norma y digest. Transacción única: `active` + retiro de la anterior + control. Podar generaciones retiradas más allá de la inmediata anterior.
7. Ante cualquier error: borrar chunks parciales y marcar `failed`; la activa no se toca. Una construcción abandonada se puede descartar tras 15 minutos.

La cancelación es cooperativa: deadline total, timeout por solicitud, `check_shutdown` del scope y sondeo de una bandera de cancelación en la fila de generación (no se expone una operación de cancelación de embeddings en este recorte; reset explícito permite descartar).

## Búsqueda híbrida

- Filtros SQL antes de puntuar: job, adjunto y extractor.
- Semántica: embeber la consulta con prefijo query, recorrer los chunks activos y calcular producto punto (vectores unitarios). Deadline de 5 s, tope de candidatos y orden determinista por score descendente y claves estables.
- Léxica: reutilizar `attachment_search::search` con los mismos filtros y su verificación canónica.
- Fusión: reciprocal rank fusion (k=60) sobre listas de hasta `limit*4` candidatos. Deduplicar por referencia de chunk y, para texto exactamente idéntico del mismo adjunto, conservar la mejor entrada. Ordenar por score fusionado y claves estables.
- Presupuesto: sumar bytes UTF-8 de textos completos; detenerse antes de exceder. `budget_exhausted` es explícito; no se trunca una cita.
- Vigencia: por fuente única, comprobar job→generación, hash del original y hash de entrada FTS si existe. Los obsoletos se excluyen por defecto y se cuentan.

## Perfil Jina v5 Text Small

La ficha oficial consultada el 21/10/2026 indica 677M parámetros, 1024 dimensiones (Matryoshka 32–1024), 32K tokens, licencia CC BY-NC 4.0 y adaptadores por tarea. Para retrieval el modelo usa `task=retrieval` y prompts `query`/`document`; la guía de best practice indica prefijos `Query:` y `Document:` con el adaptador de retrieval.

El perfil inicial documentado usa `jina-embeddings-v5-text-small-retrieval` (adapter fusionado) o el modelo base con `task=retrieval`, dimensión 1024, normalización L2, prefijos `Query: ` y `Document: `, tokenizer Qwen3-0.6B-Base, chunking `extraction_fragment_v1` y cuantización f32. **No se instalan pesos ni se ejecuta Jina en este cambio.** Un endpoint OpenAI-compatible genérico no aplica automáticamente esos prompts; el adaptador los aplica en el texto y el perfil los registra. La revisión de pesos es obligatoria y no se acepta `latest`.

El endpoint admitido es `http://127.0.0.1:puerto/...` o `http://[::1]:puerto/...`, HTTP/1.1 sin TLS, con `Connection: close`, `Content-Length`, timeout y tope de respuesta. Se rechazan hostnames, userinfo, HTTPS, redirecciones y cuerpos chunked fuera de límite.

## Backend vectorial

Alternativas evaluadas:

- **sqlite-vec**: no se adopta en este recorte. Requiere una extensión nativa adicional, empaquetado por plataforma y verificación de licencia/binario que no se hizo; no se declara medida.
- **HNSW/FAISS/service**: descartado por memoria, empaquetado y superficie operativa para el corpus acotado.
- **Escaneo exacto en Rust**: adoptado. Con ≤8192 chunks de 1024 dimensiones, el máximo es ~32 MiB de vectores; el recorrido es determinista, cancelable y no añade dependencias. El benchmark reproducible mide 1/4096/8192 chunks y publica p50/p95 y RSS en el equipo local.

El ADR 0029 registra la decisión y sus límites. El benchmark no mide calidad de Jina ni latencia de inferencia; sólo el backend.

## Alternativas descartadas

- Guardar vectores en un archivo aparte: rompe backup consistente y la transacción de activación.
- Indexar notas y adjuntos en un único espacio sin filtros: impide deduplicar fuentes y mezcla vigencias.
- Activar automáticamente un modelo local al abrir: viola el núcleo offline sin modelos y el requisito de selección explícita.
- Usar un endpoint remoto como fallback: envía contenido sin autorización; prohibido.

## Riesgos y límites

- La calidad real de Jina no está probada; las fixtures sólo prueban contratos.
- El escaneo exacto escala linealmente; por encima de 8192 chunks se requiere otro cambio/ADR.
- Un endpoint local mal configurado produce degradación a FTS, no resultados inventados.
- El snapshot de lotes incluye el índice semántico; el límite de 64 MiB puede rechazar previews con índices grandes. Se documenta y no se eleva en este cambio.
- La generación retirada se conserva sólo una; no hay rollback automático, pero rebuild es explícito y reproducible.

## Verificación

- Unitarias de perfil/huella, validación de vectores, chunking, digest, estados y migración 017 (incluido fallo y versión futura).
- Integración con servidor HTTP loopback simulado: éxito, timeout, dimensión inválida, NaN, respuesta grande y endpoint remoto.
- Búsqueda híbrida con filtros, deduplicación, presupuesto, obsolescencia y fallback sin endpoint.
- Backup/restore con generación activa; preservación de documentos/desconocidos.
- CLI/MCP: catálogo, scopes, acciones puras y errores.
- Benchmark del backend y `fmt`/Clippy/contratos.
