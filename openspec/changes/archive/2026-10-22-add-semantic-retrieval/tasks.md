# Tasks

## 1. Contratos y perfil

- [x] 1.1 Definir `EmbeddingProfile`, `EmbeddingIndexInput/Result`, `RetrievalInput/Hit/Page`, enums de modo/estado y límites publicados en `semantic-retrieval` y `core-operations`.
- [x] 1.2 Implementar validación estricta, canonicalización y huella SHA-256 del perfil; probar estabilidad y sensibilidad a cada factor.
- [x] 1.3 Verificar documentación vigente de Jina v5 Text Small, tarea retrieval y distinción query/document; registrar el perfil documentado sin instalar pesos.

## 2. Persistencia y migración

- [x] 2.1 Diseñar y escribir la migración aditiva 017 con perfiles, generaciones, chunks, control e índices; conservar 001–016 sin cambios.
- [x] 2.2 Integrar 017 en creación, apertura, migración, `schema_check`, integridad, backup/restauración y registro de esquema de lotes.
- [x] 2.3 Probar migración desde 16 con datos reales, fallo transaccional, versión futura, backup/restore y preservación de documentos/bloques/propiedades desconocidos.

## 3. Adaptador local

- [x] 3.1 Implementar cliente HTTP loopback acotado (sólo `127.0.0.1`/`::1`, sin TLS, timeout, tope de respuesta, sin redirecciones) y adaptador OpenAI-compatible de embeddings.
- [x] 3.2 Implementar lotes, prefijos query/document, normalización L2, validación de dimensión/finitud/norma y errores estructurados.
- [x] 3.3 Probar con servidor loopback simulado: éxito, prefijos, timeout, dimensión errónea, NaN, vector nulo, cuerpo grande y rechazo de endpoint remoto.

## 4. Índice generacional

- [x] 4.1 Implementar selección de fuentes con filtros, `canonical`/`source`, límites y digest reproducible.
- [x] 4.2 Implementar construcción `building` incremental, verificación final y activación atómica con retiro/poda de generaciones.
- [x] 4.3 Implementar `status` y `reset` con revisión esperada, cobertura, fuentes obsoletas y limpieza de construcciones abandonadas.
- [x] 4.4 Probar reutilización por digest, fallo que conserva la activa, concurrencia de builds, reset y ausencia de mezcla de huellas.

## 5. Búsqueda híbrida

- [x] 5.1 Implementar búsqueda semántica exacta con filtros SQL, deadline, orden determinista y verificación de vigencia por fuente.
- [x] 5.2 Implementar fusión RRF con léxica, deduplicación exacta, presupuesto de texto, límites y metadatos por componente.
- [x] 5.3 Implementar degradación explícita a SQL+FTS y página vacía con razones cuando no hay índices.
- [x] 5.4 Probar filtros, deduplicación, presupuesto agotado, fuentes obsoletas, modo léxico/semántico/híbrido y fallback sin endpoint.

## 6. Superficies y documentación

- [x] 6.1 Registrar operaciones en catálogo, CLI, scopes y MCP; actualizar conteos y pruebas de acciones puras.
- [x] 6.2 Actualizar `docs/semantic-retrieval.md`, ADR 0029, plan, configuración OpenSpec y guía de desarrollo.
- [x] 6.3 Ejecutar benchmark del backend y documentar p50/p95/RSS sin atribuir calidad de modelo.

## 7. Verificación y cierre

- [x] 7.1 Ejecutar pruebas Rust por perfil, contratos, CLI/MCP, fmt, Clippy, `git diff --check` y OpenSpec estricto.
- [x] 7.2 Revisar diff/diseño/escenarios, conservar fallos previos y documentar limitaciones reales.
- [x] 7.3 Sincronizar únicamente los deltas terminados y archivar `add-semantic-retrieval`; preservar Home 9/17 y no iniciar fase 6.
