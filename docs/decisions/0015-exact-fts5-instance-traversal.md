# ADR 0015 — Recorrido exacto de postings con la API pública FTS5

- Fecha: 15/10/2026
- Estado: adoptada y verificada; dos gates finales y regresión integrados aprobados
- Cambio: [add-local-mcp](../../openspec/changes/archive/2026-10-15-add-local-mcp/design.md)

## Contexto

El gate heredado de adjuntos no mantuvo suficiente margen: cuatro de seis mediciones superaron query p95 300 ms, sin cambiar corpus/caps. Un diagnóstico pareado encontró CPU semejante entre el CLI baseline y el actual; eso no permite ignorar el gate. En conexión real, la verificación de 1.199.240 postings repetidos consumió 130,40 ms de 208,96 ms totales. La proyección SQL que ahorraba una celda por fila no mejoró y fue retirada.

La decisión de [ADR 0013](0013-explicit-attachment-index.md) de comprobar **cada** posting, copias, fuentes, estadísticas y origen se mantiene; no se admite validación parcial ni caché de confianza entre snapshots.

## Decisión

Para el camino de reutilización exacta de tokenización, usar la API auxiliar **pública** FTS5 de SQLite ya incluido en rusqlite: `fts5_api.xCreateFunction`, `xInstCount`, `xInst` y `xRowid`. Enumerar todos los términos **reales** con fts5vocab(row), rechazar términos ajenos y recorrer todos los documentos/instancias de cada término con MATCH de una frase/token. Cada instancia debe tener phrase 0, columna text (3), rowid canónico, offset y término esperados, con bitset independiente por documento que rechaza duplicados. La cardinalidad final debe coincidir exactamente. Subconjunto sin duplicados más cardinalidad igual demuestra igualdad; no se sustituye la comparación por un hash.

Con más de 256 términos esperados, si no está disponible la API auxiliar o si el vocabulario de entrada no es exactamente la tabla virtual que apunta al FTS real, conservar el recorrido completo anterior de fts5vocab(instance). Esto evita ignorar un stream materializado/independiente aportado al verificador. El umbral elige algoritmo, **no** rechaza ni recorta entradas; la ruta sin reutilización de tokenización conserva asimismo su comparador completo. No se modifican estadísticas, validación de copias, originales, hashes, TEMP existente ni caps de memoria/fragmentos/tiempo. Los formatos persistentes, migraciones 001–010 y API v1/51 son idénticos.

## Seguridad y ciclo de vida

`attachment_postings.rs` concentra el FFI; usa bindings generados de libsqlite3-sys, sin biblioteca nueva, carga de extensiones ni parseo de páginas privadas. Un statement RAII queda ligado a la conexión y se finaliza antes del estado prestado; la expresión usa SQLITE_TRANSIENT. El puntero con tag se liga al statement, nunca al registro persistente de función, y ningún callback escapa ni ejecuta SQL reentrante. El registro conserva sólo código/metadatos de conexión, sin punteros a una verificación o sus resultados; se evita acumular registros por llamada. NULL/tag ajeno se rechaza. El callback no permite unwind por el ABI C. Deadline cooperativo cada 1.024 instancias y entre términos, además del progress handler existente. TEMP auxiliar se elimina en éxito y rechazo/error.

No se amplía la amenaza cubierta: misma conexión privada NO_MUTEX usada secuencialmente, DDL/tokenizer unicode61/detail=full validado antes de recorrer, sin acceso SQL arbitrario de MCP ni sandbox frente a un proceso hostil del mismo usuario. Un error SQLite sigue siendo error; no se convierte en éxito ni reparación implícita.

## Alternativas descartadas

- Omitir postings/copias/originales, validar una muestra o cachear entre consultas: debilita integridad.
- Aceptar 300 ms como excepción o reducir corpus/caps: contradice la decisión explícita del usuario.
- Proyectar CASE sobre col para ahorrar una celda FFI: no mejoró en el diagnóstico medido.
- Parsear bloques FTS privados o añadir una extensión nativa independiente: agrega mantenimiento/formato/ABI innecesarios.

## Verificación

Cinco tests adicionales contrastan contra el comparador histórico completo, sin tocar asserts anteriores: Unicode/vacíos/posiciones repetidas; postings ocultos, ausentes, extra, desplazados y rowids ajenos; llamadas reiteradas sin caché/puntero vivo; fallback con 300 términos; duplicados/offsets/deadline del callback. El test heredado de stream independiente/duplicados encontró inicialmente que se suponía un FTS real donde su contrato también admite un vocabulario materializado. Se corrigió la selección del algoritmo, **no** el test ni sus asserts. El corpus y el harness de benchmark heredados permanecen byte a byte iguales. Source final7 pasó **235 Rust por perfil**; dos gates consecutivos sobre el corpus intacto: query p95 **134,96–204,81 ms**, lock máximo p95 **19,80 ms**. [Evidencia, fallos conservados y límites](../verification-mcp.md); no constituye promesa universal sobre otros textos/cachés.

Fuentes oficiales consultadas el 15/10/2026: [extensión auxiliar y API FTS5](https://www.sqlite.org/fts5.html#extending_fts5), [punteros tipados SQLite](https://www.sqlite.org/bindptr.html), más la implementación/bindings de SQLite incluidos en libsqlite3-sys 0.36.0. No se promete compatibilidad de rendimiento con otras arquitecturas o distribuciones SQLite no probadas.
