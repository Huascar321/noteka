# Verificación de recuperación semántica (5b)

Estado: implementación y aceptación técnica local aprobadas el 21/10/2026
(America/La_Paz). No se instalaron pesos de Jina ni se ejecutó un modelo real.
La calidad semántica queda pendiente de una ejecución autorizada con corpus
representativo; esta evidencia sólo acredita contratos, persistencia, activación,
filtros, presupuesto, degradación y el backend exacto.

## Cambio y superficie

`add-semantic-retrieval` agrega el perfil de embeddings, el adaptador HTTP
loopback, la migración aditiva 017, las generaciones de índice, `embedding.index`
y `retrieval.search`. SQLite pasa de 16 a 17; API v1 de 57 a 59 operaciones en
este recorte (62 al cerrar 5c). Las nueve mutaciones batchables no cambian.

## Evidencia ejecutada

- Rust: **407 pruebas aprobadas y 14 ignoradas** por perfil debug/release
  (`logs/phase5bc-rust-debug.log`, `logs/phase5bc-rust-release.log`). Incluye 13
  pruebas semánticas: huella, vectores, activación, reutilización, fallo,
  obsolescencia, híbrida, presupuesto, reset, migración 017 y adaptador HTTP.
- SDK MCP: **28 pruebas por perfil** (`npm run test:mcp`), sin fallos.
- Frontend: **94 pruebas Node** (editor, preparación, lector, puente, lotes,
  búsqueda de adjuntos) y `svelte-check` con **0 errores/0 warnings**.
- Headless: **5.120 checks** sin solicitudes externas: núcleo 592+14 teclado,
  evidencia 1.248, preparación 154, lector 990, agrupada 238, OCR 326, PDF OCR
  450, tabla PDF 286 y PDFKit 822. No se abrió ventana nativa.
- Contratos: `scripts/check-semantic-retrieval-contracts.py` contra
  `phase5a-accepted/catalog57.json`: 57 operaciones previas byte-idénticas,
  exactamente 5 adiciones (`embedding.index`, `retrieval.search`, `agent.ask`,
  `agent.cancel`, `agent.trace`), `attachment_index` y batch9 intactos.
- Migración real: copia de `restored-by-accepted57` (v16) abierta con el cliente
  nuevo → schema 18, 1 entidad, 6 jobs; backup/restore v18 verificados. El
  cliente v16 independiente rechaza el workspace v18 como versión futura y no
  modifica el SHA-256 del archivo.
- Smoke CLI debug/release (`scripts/test-semantic-agent-smoke.py`): índice
  semántico de 1 chunk, búsqueda `semantic`, luego `lexical` con
  `query_embedding_failed` al apagar el servidor loopback; harness con cita
  verificada y 3 pasos; sin red externa.
- Gates originales intactos (corrida final): attachment query p95 máximo
  **134,020834 ms**, lock máximo **22,925292 ms**; XLSX mixto query p95 máximo
  **189,807041 ms**, lock máximo **16,415833 ms**. Límites 300/150 sin cambios; corpus/caps sin
  relajar.
- `cargo fmt --check`, Clippy workspace con `-D warnings`, `git diff --check`,
  `npm run contracts:check`, `npm run check` y host no-bundle sin ventana
  aprobados.

## Benchmark del backend

`cargo test --release -p noteka-core --lib semantic_backend_benchmark -- --ignored --nocapture`:

| Chunks | Dimensión | Bytes de vectores | p50 | p95 |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 1024 | 4.096 | 0,001000 ms | 0,001167 ms |
| 4.096 | 1024 | 16.777.216 | 3,800417 ms | 5,065917 ms |
| 8.192 | 1024 | 33.554.432 | 7,644292 ms | 7,902042 ms |

Mide el escaneo exacto en Rust, no la latencia de Jina ni la calidad de
recuperación. El debug fue más lento (hasta 120 ms p95 a 8.192 chunks); se
conserva como referencia, no como resultado de release.

## Perfil Jina y decisión

La ficha oficial consultada el 21/10/2026 reporta 677M parámetros, 1024
dimensiones, 32K tokens, licencia CC BY-NC 4.0 y adaptadores por tarea. El perfil
documentado usa `retrieval`, prefijos `Query:`/`Document:`, normalización L2,
chunking `extraction_fragment_v1` y revisión obligatoria. **No se descargaron
pesos ni se ejecutó Jina.** Un endpoint genérico no aplica automáticamente esos
prompts; el adaptador los aplica y el perfil los registra.

Se adopta escaneo exacto ([ADR 0029](../docs/decisions/0029-semantic-retrieval.md));
sqlite-vec/HNSW no se adoptan sin verificación local. El coste crece linealmente
y el límite de 8.192 chunks es una decisión del recorte, no una garantía para
corpus mayores. El snapshot de lotes incluye el índice y puede alcanzar el tope
de 64 MiB; no se elevó.

## Incidencias conservadas

La primera corrida de pruebas encontró que `prepare-evidence-search-fixture.py`
y `prepare-attachment-reader-fixture.py` fijaban schema 16 y `target/` en vez de
`NOTEKA_CLI`; el puente del lector usaba `target/debug/noteka`. Se corrigió el
arnés para usar el binario aislado y aceptar el schema actual; las suites
volvieron a pasar. No fue un defecto del producto ni se cambió una aserción para
ocultar un fallo. Los logs de la primera corrida se conservan en la evidencia
externa.
