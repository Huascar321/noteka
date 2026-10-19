# ADR 0013: índice FTS explícito e independiente de adjuntos

Fecha: 14/10/2026. Estado: adoptada y verificada, **19/19 tareas**, tres deltas sincronizados y cambio archivado. Cambio: [`add-local-attachment-search`](../../openspec/changes/archive/2026-10-14-add-local-attachment-search/design.md).

## Contexto

SQLite y los originales siguen siendo autoridad. Las generaciones UTF-8/PDF tienen lectores y localizadores diferentes. Indexar automáticamente o mezclar con FTS de notas modificaría selección, ranking y contratos anteriores. No existe una entidad dueña única de un adjunto compartido.

## Decisión

SQLite v10 aditiva (`010.sql`) añade control derivado, selecciones y FTS5 normal/Unicode61 independiente. API v1 suma `extraction.index` y `extraction.search` sin modificar los 49 contratos previos. Put explícito usa CAS revisión/generación y preparación fuera del escritor; CASCADE/trigger invalida selecciones en la transacción de reemplazo de generación. Reset explícito recrea sólo el FTS/trigger reconocidos, no formatos futuros ni SQLite físicamente dañado.

Get/search abren main READ_ONLY con TEMP escribible. El verificador reconstruye FTS esperado desde canónicos y compara copias/postings, aun para cero hits. La comprobación exacta de postings implementa E ⊆ A y |E|=|A|: E es único por Unicode61 y `(term,doc,col,offset)`, por lo que implica igualdad, incluso si A tuviese duplicados. No es un checksum aproximado ni comparación sólo de conteos. Candidatos/orden/scores se contrastan además con TEMP; sólo los originales de candidatos de la página, incluido lookahead, se releen. No se debilita query_only de los lotes.

Se conservan caps de selección/texto/fragmentos, extracción y snapshot independientes, presupuesto cooperativo, ausencia de helper/modelos en lectura/indexación y nueve mutaciones batch. Un guard de max_page_count vive hasta COMMIT/rollback para incluir asignaciones FTS tardías.

La apertura dedicada usa NO_MUTEX como el default de rusqlite: conexión local a una operación, no Sync ni compartida entre threads. No elimina exclusión de escritor, busy timeout o fencing; las aperturas anteriores no cambian. Las copias se comparan completas con ORDER BY rowid explícito (único por FTS), incluyendo tipos, para evitar árboles EXCEPT de texto. No se interpretan blobs privados de SQLite ni se reduce la comparación de postings.

Para texto repetido dentro del snapshot, una clasificación por igualdad completa de bytes reutiliza tokenización Unicode61, no confianza entre llamadas. Se conservan todas las filas del FTS esperado; un segundo FTS temporal tokeniza textos distintos sólo cuando suman <=25% del total. Se verifica cada posting real contra el mapa completo posición/término de su texto, con identidad de documento, columna/tipos y un bitset por documento que rechaza duplicados. A ⊆ E, unicidad de A y cardinalidad igual prueban igualdad exacta; no se supone orden público. Cualquier falta de igualdad conserva el fallback anterior E ⊆ A/cardinalidad; sin repetición se mantiene la secuencia completa. No es el camino de memoria constante: memoria acotada por texto/fragmentos/rowids, medida sin elevar caps. Se mantienen pruebas adversariales de ambos caminos.

El hash de contenido v1 serializa vistas prestadas, ordenando claves explícitamente y conservando bytes JSON/namespace/prefijos de longitud. Pruebas contra el código anterior y 16 generaciones persistidas comprueban compatibilidad: no hay formato ni migración nuevos. En aarch64 se activa la feature `asm` existente de sha2 0.10.9 para habilitar su backend ARM con detección runtime/fallback portable; Cargo.lock suma sólo sha2-asm 0.6.4. Es una modificación acotada de compilación, también para hashes previos, no una sustitución del algoritmo ni un hash aproximado. No se actualiza SQLite ni se añade unsafe/FFI propio al verificador.

El camino sin suficiente repetición compara primero ambas secuencias de tuplas con memoria constante: si son idénticas y terminan juntas, sus multisets son idénticos. No presupone orden público de fts5vocab: cualquier diferencia usa el fallback exacto E ⊆ A / cardinalidad. Así un orden distinto no genera un falso error ni permite omitir validación; faltantes, extras y duplicados siguen siendo detectables. Sus benchmarks históricos insuficientes se conservan, no se reemplazan por los favorables del camino más reciente.

## Alternativas descartadas

FTS combinado con notas; selección automática; FTS external-content/contentless; validar sólo hits o sólo copias; integrity-check con escritura en main durante lecturas; reparación o eviction automáticas. No se elimina validación ni se eleva un cap para mejorar un benchmark.

## Consecuencias

La validación completa consume CPU/TEMP en cada lectura. Reutilizar tokenización por sí solo pasó una vez y falló en la réplica (282,97–344,71 ms); no se eligió sólo la favorable. Con backend ARM/vistas prestadas, **dos benchmarks consecutivos** verifican query p95 **214,20–231,07 ms** (<300 ms), lock máximo p95 **17,66 ms** (<150 ms), y la sonda con self-test mide scratch físico **21.254.144 bytes**. [Evidencia](../verification-attachment-search.md) conserva baselines y experimentos descartados. Se cumplió la decisión de **optimizar sin cambiar el objetivo**, no una excepción. La ventaja de reutilizar depende de repetición, no garantiza 300 ms para todos los corpora/cachés. No se promete un cap RSS/WAL/scratch ni autenticidad ante reescritura coherente de toda la base. La búsqueda unificada/UI, formatos adicionales, embeddings, harness y remotos quedan fuera; Home sigue 9/17.
