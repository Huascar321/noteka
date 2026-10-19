# Tasks

Estado: **19/19 tareas verificadas, sincronizado y archivado el 14/10/2026**, SQLite v10/API v1/51. **6.1 aprobada:** dos benchmarks consecutivos con query p95 **214,20–231,07 ms** (<300 ms) en el corpus/hash fijado de 8 MiB, lock máximo p95 **17,66 ms** (<150 ms), scratch físico **21.254.144 bytes** por sonda con self-test. Regresión completa fresca: 212 tests Rust por perfil, 20 rondas CLI por perfil y 606 checks headless. Se conservan fallos y experimentos, sin cambiar target/caps ni retirar validación; el resultado no es universal para cualquier corpus/caché. Baseline/CLI v9 y evidencia externa en `noteka-attachment-search-apply-qLSCJI/`; [verificación y cierre](../../../../docs/verification-attachment-search.md). Home 9/17 intacto, sin pantalla/modelos/proveedores.

Trazabilidad abreviada, en el orden de los requisitos de cada delta:

| ID | Capability / Requirement |
| --- | --- |
| S1 | local-text-search / Selección explícita de generaciones extraídas |
| S2 | local-text-search / Escrituras del índice con revisión esperada |
| S3 | local-text-search / Consulta literal acotada de fragmentos |
| S4 | local-text-search / Hits con fragmento canónico y localizador tipado |
| S5 | local-text-search / Invalidación atómica al reemplazar generaciones |
| S6 | local-text-search / Cursores de adjuntos ligados al índice |
| S7 | local-text-search / Cobertura explícita del conjunto seleccionado |
| S8 | local-text-search / Índice verificable y reparación explícita |
| S9 | local-text-search / Recursos acotados sin ampliar la extracción |
| O1 | core-operations / Operaciones aditivas de indexación y búsqueda de adjuntos |
| O2 | core-operations / Admisión y errores sin ejecución implícita |
| O3 | core-operations / Indexación fuera de la whitelist de lotes |
| W1 | local-workspace / Migración recuperable para índice de adjuntos |
| W2 | local-workspace / Backup y restauración con selecciones confirmadas |
| W3 | local-workspace / Snapshots y huellas preservan el índice sin volverlo original |
| W4 | local-workspace / Evolución recuperable a cola local — reconciliación de cinco escenarios heredados, sin ampliar alcance |

## 1. Persistence and admission

- [x] 1.1 Añadir únicamente `010.sql`, creación/migración v10, schema checks y registry completo del nuevo índice/control/entries/FTS/shadows/trigger, inicialmente vacío; preservar 001–009 y filas anteriores. Verificar creación/migración/rollback/FKs/SQL registry con SQLite real y hashes de migraciones previas; documentar esquema y límites de rollback en la nueva guía `docs/attachment-search.md`. [W1, W3, S1, S5]
- [x] 1.2 Añadir DTOs estrictos separados, localizador discriminado, perfiles/límites y apertura específica current-version/read-only-main con TEMP permitida, sin cambiar query_only de lotes. Verificar serde/schema, campos requeridos/desconocidos/null/IDs/versiones y negación de escritura main dentro de una transacción, sin recovery/autómigración; documentar inputs/defaults y razones de error en la guía. [O1, O2, S4, S8, S9]
- [x] 1.3 Construir un CLI v9 independiente desde baseline externo y probar migración real de v9 poblado y v9 running/con generación previa/configuración futura; comparar todas las filas/archivos salvo versión, restaurar backup previo e inyectar falla. Verificar que cliente v9 rechaza v10 sin escribir y que la admisión Rust nueva rechaza v9 sin migrar (su exposición CLI se prueba en 5.1); registrar comandos/manifiestos y requisitos de recuperación en la guía. [W1, O2]

## 2. Canonical data and pure validation

- [x] 2.1 Implementar carga/hash canónico de la generación seleccionada reutilizando validaciones UTF-8/PDF sin parser, preservando ambos lectores y shapes. Verificar multibyte/CRLF, fronteras/ordinales, PDF por página/no_text, fuente/configuración inválidas y equivalencia exacta con los lectores existentes; documentar identidad/localizadores y alcance PDF sin coordenadas u OCR. [S1, S4, S7, S8]
- [x] 2.2 Implementar verificador completo de entries/copias/postings con FTS esperado y fts5vocab sólo en TEMP sobre main read-only y snapshot real de la SQLite bundled. Probar faltantes/extras de filas y postings, identidad/texto/hash/locator adulterados, consulta potencialmente sin resultados, cleanup/deadline y ausencia de mutaciones main/cola; comprobar que una falla recuperable expone revisión segura. Documentar diferencia índice íntegro/originales no releídos y límites de corrupción recuperable. [S8, S9, O2]
- [x] 2.3 Implementar index get/cobertura filtrada y entries paginadas con cursor propio. Verificar partición de generaciones, jobs sin generación, selección vacía, configuración futura conservada byte a byte, adjunto compartido/sin dueño y estados running/vencidos sin recovery; tests de scopes/límites/orden de get. Documentar conteos y por qué no representan todos los adjuntos ni heredan archivo/contexto. [S6, S7, S8, O1]

## 3. Atomic writes and extraction lifecycle

- [x] 3.1 Implementar put de una generación con preparación fuera del escritor, CAS de revisión/generación y revalidación bajo lock, filas/entry/revisión/respuesta atómicas. Verificar UTF-8/PDF/no_text, generación previa disponible, no-op sin duplicados/bump, revisión vieja aun en no-op, falla en cada escritura y dos indexadores reales con un único ganador; documentar selección explícita y comandos/métodos de reintento. [S1, S2, S5]
- [x] 3.2 Implementar reset transaccional con CAS, recreación únicamente de FTS y restauración exacta del trigger/registry. Verificar reparación de índice lógico inconsistente sin validar primero sus postings, rollback de DDL/fallas, reset vacío con bump y rechazo sin descarte de formato futuro/esquema no reconocido; documentar reset→puts sin helper y cuándo hace falta backup. [S2, S8, O2]
- [x] 3.3 Integrar/probar invalidación por CASCADE en publicación de nuevas generaciones UTF-8/PDF, sin selección automática. Verificar rollback en cada falla de publicación, cancel/pause/retry/fallo con índice previo disponible y flag anterior, carreras put/publicación/reset/edición y backup durante worker; usar procesos reales además de hooks deterministas y comprobar locks/ownership/fences sin cambiar lease/ejecución del extractor. Documentar ciclo de vida. [S5, S6, S8, W2]
- [x] 3.4 Aplicar cuotas reales de 1.024 entries/16.384 fragmentos/8 MiB, respuesta 2 MiB/cursor 2 KiB, presupuesto cooperativo y guard max_page_count para snapshot 64 MiB hasta COMMIT. Verificar cada borde/saturación por separado, bytes vs caracteres, contador overflow, contención/deadline, asignación tardía FTS/SQLITE_FULL y restauración del límite previo; probar ausencia de eviction y reset disponible sin compactación implícita. Documentar caps independientes y qué no es un cap RSS/WAL. [S9, W3, O2]

## 4. Search and provenance

- [x] 4.1 Implementar consulta literal parametrizada AND/Unicode61, filtros y ranking/desempate estables del índice independiente, contrastando candidatos/scores con el esperado TEMP. Verificar espacios/comillas/OR/NEAR/SQL aparente, controles, multibyte/acentos/puntuación sin tokens, filtros, estadísticas BM25 dañadas y términos repartidos entre chunks/páginas sin hit inventado; comparar que search.query/rebuild de notas no cambia. Documentar semántica de términos/tokenizer/ranking y no equivalencia a substring. [S3, S7, S8]
- [x] 4.2 Construir hits desde canónicos con identidad/hash/ordinal/localizador tipado, estado de intento previo y alcance PDF, validando archivos de candidatos distintos incluido lookahead. Verificar igualdad con lectores, helper ausente, archivos cambiados/eliminados/symlinks/metadatos y falla de página completa sin omitidos/refill/reparación; dejar fuente ajena a la página sin afirmación falsa de vigencia. Documentar el alcance returned_candidates. [S4, S7, S8, O2]
- [x] 4.3 Ligar cursor de search a workspace/ruta/query/filtros/límite/perfil/revisión y separar get/search. Verificar recorrido completo con empates sin duplicados, no-op estable, revisión vencida por put/reset/invalidation, campos/versiones/offset inválidos, restauración/ruta distinta y estado de cola cambiante sin mezclar conjuntos; documentar conflictos y reinicio explícito del cliente. [S6, O1, W2]

## 5. Public integration and recovery

- [x] 5.1 Conectar catálogo/dispatch temprano/CLI para las dos operaciones y regenerar contratos TS/JSON sin cambiar los 49 anteriores. Añadir harness CLI real sobre fixtures sintéticas con golden independiente de IDs/texto/localizadores/ranking/cobertura, inputs/errores/códigos de salida y funcionamiento sin helper/red; verificar contratos contra baseline y comandos reproducibles de la guía (`get→put→search→reset`), sin abrir host/UI. [O1, O2, S1–S9]
- [x] 5.2 Integrar verificación del índice en backup/restauración staging y ejecutar roundtrip completo sin helper/modelos, preservando revisión/perfil/ranking/estados/configuración futura. Verificar backups durante put/reset/publicación con snapshot entero anterior/posterior, backup adulterado rechazado sin destino parcial y original intacto, y cursor viejo rechazado por ruta; documentar recuperación y límites físicos. [W2, S5, S6, S8]
- [x] 5.3 Verificar base.v2 estable tras put/reset/invalidation, snapshot completo de todas las tablas/shadows/triggers y cap 64 MiB; pruebas reales preview→put/reset→apply sin pérdida del índice, recibo v9 rechazado y nuevas operaciones rechazadas por whitelist/checks. Conservar nueve mutaciones y contratos; actualizar `docs/batches.md`/guía con versión vigente y exclusión de derivados, sin atribuirles permiso o idempotencia. [W3, O3]

## 6. Integrated verification and review

- [x] 6.1 Ejecutar benchmark reproducible de release sobre M1 Max/64 GiB con corpus fijado, cinco warmups/veinte muestras por consulta/put/no-op/reset/invalidation, filtros/páginas y saturación separada de caps. Medir lock p95 <=150 ms, target query p95 <=300 ms, deadline, bloqueo de editor, RSS y tamaños SQLite/WAL/scratch; comparar notas con baseline y preservar baselines fallidos/diagnóstico sin subir caps o quitar validaciones. Publicar resultados y condiciones en `docs/verification-attachment-search.md`. [S3, S5, S9, W3]
- [x] 6.2 Ejecutar regresión integrada fresca debug/release: Rust workspace, corpus/harness PDF y extracción, migraciones/CLI nuevos, fmt/check/Clippy con warnings denegados, contratos, adaptador/cola/tema/puente/suscripción, build frontend y host no-bundle sin lanzarlo. Ejecutar los 606 checks UI headless existentes con temporales aislados/sin perfil personal ni requests externos, sin suprimir asserts ni usar pantalla; registrar comandos, versiones, resultados y fallas en la verificación. [S1–S9, O1–O3, W1–W3]
- [x] 6.3 Contrastar diff externo con baseline porque no hay commit inicial; completar matriz individual de 15 requisitos/40 escenarios nuevos, más la reconciliación de migración W4/cinco escenarios heredados, con archivo/test/log y limitaciones. Verificar Home, UI/host, lectores/49 contratos previos, originales y migraciones 001–009 preservados salvo adiciones generadas autorizadas; reconciliar guía/README/desarrollo/roadmap/contexto y ADR de la decisión implementada. Validar OpenSpec estricto sin tratarlo como test de aplicación; dejar Home 9/17 y demás 5a/5b pendientes explícitos. [S1–S9, O1–O3, W1–W4]

## Workflow follow-up

- Completado el 14/10/2026: tres deltas sincronizados y contrastados íntegramente con main; 15 requisitos añadidos/40 escenarios nuevos y un requisito reconciliado/cinco escenarios heredados, preservando requisitos ajenos y las otras nueve capabilities.
- Archivado como `2026-10-14-add-local-attachment-search` sólo después de esa verificación/sincronización. Home no se sincronizó ni archivó; el archivo PDF previo permanece intacto. Auditoría externa de cierre: `noteka-attachment-search-close-EKtnFG/`.
- Búsqueda unificada/UI, PDF amplio/Excel/OCR y resto de 5a, embeddings/Jina/5b y harness/5c son cambios separados; no iniciar descargas o activación remota al terminar este recorte.
