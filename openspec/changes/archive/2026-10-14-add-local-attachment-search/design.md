# Design

## Context

Motivación y alcance: [proposal.md](proposal.md). **19/19 tareas verificadas, sincronizado y archivado el 14/10/2026.** Baseline vigente archivado: SQLite v10/API v1/51, con regresión/revisión, query <=300 ms sobre el corpus fijado y gate lock aprobados; scratch físico medido. PDF archivado 21/21 y Home activo 9/17 sin tocar. No hay commit inicial: la planificación comparó 284 archivos; la implementación conserva su propio baseline externo de 291 archivos y CLI v9 independiente. [Verificación y cierre](../../../../docs/verification-attachment-search.md), [guía](../../../../docs/attachment-search.md) y [ADR 0013](../../../../docs/decisions/0013-explicit-attachment-index.md). La evidencia de planificación al final es histórica, no prueba runtime.

Evidencia inspeccionada en el código actual:

| Área | Hecho y punto de extensión |
| --- | --- |
| `crates/noteka-core/src/search.rs:5–65,67–164` | `search_fts` sólo contiene títulos/bloques. Hay reindex transaccional y cursor de offset no ligado a generación. No cambiar esa semántica ni sus DTOs. |
| `extraction_contracts.rs:89–180`, `extraction.rs:175–257`, `pdf_storage.rs:181–237` | Lectores separados; generación/ordinal/localizador ya existen. UTF-8 reconstruye bytes y hash; PDF valida manifiesto/fragmentos y hash del archivo sin parser. |
| `extraction_worker.rs:192–198,257–328` | Validación PDF pesada fuera del escritor; publicación cercada elimina generación anterior e inserta la nueva en una transacción. Ahí debe invalidarse la selección anterior, sin insertar la nueva automáticamente. |
| `extraction_storage.rs:73–90,190–209,262–275,326–385` | Configuración compatible estricta, scope workspace/ruta, fuente segura <=1 MiB y resultado de intento anterior. Reutilizar validaciones, no recovery ni claim. |
| `documents.rs:209–236` | Las referencias a adjuntos pueden aparecer anidadas y ser compartidas. No existe una entidad dueña única que pueda ponerse en el hit sin otra decisión de producto. |
| `workspace.rs:73–130,161–208,270–315` | Preflight privado valida versión/columnas; `Workspace::open` puede migrar/configurar escritura. Nuevas lecturas deben despacharse antes de ese camino. |
| `batch_storage.rs:18–36,54–103,139–173,226–266` | Snapshot de 64 MiB, registry completo, huella base.v2 de originales y presupuesto reutilizables. Su apertura read-only activa además query_only: no sirve sin adaptar para un verificador con TEMP. Derivado no significa excluido del snapshot. |

Los paths abreviados de la tabla pertenecen a `crates/noteka-core/src/`. Esos hechos no prueban las operaciones nuevas ni su rendimiento.

## Goals / Non-Goals

**Goals:** índice seleccionado, verificable y reconstruible; una generación por escritura; consultas puras y paginación que falla explícitamente si cambia el índice; reutilizar FTS5 sin otro motor/runtime. La única dependencia adicional del lock, sha2-asm 0.6.4, habilita el backend existente de SHA-256 en aarch64 con compatibilidad comprobada. Separar autoridad de originales, publicación de extracción y selección para búsqueda.

**Non-Goals:** ver lista en la propuesta. En particular, no fusionar rankings de notas/adjuntos, no mantener relaciones de origen de entidades y no reutilizar los filtros archive/context de notas con otra semántica. Sin scheduler, UI nueva, MCP ni modelos. No implementar todavía un gestor de índices intercambiables o tablas vectoriales vacías.

## Decisions

### 1. Independent FTS and explicit selection

Usar otro FTS5 normal, `extraction_search_fts`, con texto copiado y columnas de identidad no indexadas (`job_id`, `generation_id`, `ordinal`). Sólo `text` participa del ranking. Tokenizer fijo `unicode61`, `detail=full`, sin prefijos ni tokenizer configurable por el solicitante. El perfil persistente v1 identifica exactamente esas opciones y la versión del formato; no es un perfil de embeddings.

Una selección identifica un job y su generación **actual publicada**, incluso si procede de un intento anterior mientras el job está queued/running/failed/cancelled. No exige estado succeeded si hay generación conservada válida. Una generación sin texto también puede seleccionarse y aporta cero filas FTS. No se deduplica entre trabajos distintos ni se infiere dueño por las referencias de notas.

Alternativas:
- Mezclar con `search_fts` obligaría a cambiar hits/filtros y compatibilidad de la búsqueda de notas; descartado.
- Indexar automáticamente al publicar contradice la selección explícita y aumenta el trabajo bajo el lock del extractor; descartado.
- FTS external-content sobre una vista UTF-8/PDF evita la copia, pero complica reparación, correspondencia de rowids y validación de dos localizadores. Contentless tampoco conserva texto consultable. Se elige copia pequeña y comprobada contra canónicos; no es fuente de verdad.

### 2. Additive persistence and lifecycle

`010.sql` propuesta añade únicamente:

| Objeto | Estado propuesto |
| --- | --- |
| `extraction_search_control` | Singleton: versión 1, revisión positiva u32 inicialmente 1, perfil fijo y timestamp. No es `extraction_control` ni modifica pausa. |
| `extraction_search_entries` | Slot entero único 1–1.024, job único, generación única, conteo/bytes reales, timestamp y hash versionado de identidad/configuración/manifiesto/localizadores/textos canónicos. FK compuesta `(job_id,generation_id)` a `extraction_generations(job_id,id)` con CASCADE. |
| `extraction_search_fts` y shadows | Una fila por fragmento. Rowid determinista `(slot−1)*512 + ordinal+1`; el slot no es identidad pública. Copia exacta de texto, sin snippets persistentes. |
| Trigger de borrado de entry | Elimina sus postings/copias e incrementa revisión del índice en la misma transacción. Incluye el borrado por CASCADE al reemplazar la generación. |

Validar versión/perfil, singleton, FKs, IDs, hashes, límites, pertenencia job↔generación y rowids. El hash de entry usa un namespace nuevo de contenido indexado y excluye el estado/attempts/revisión dinámicos del job; incluye el intento de la generación publicada. No reemplaza el hash del original ni constituye una firma.

Una publicación exitosa invalida la selección antigua sólo si existía. Queued/running/cancel/retry/falla no la retiran. El CASCADE/trigger no reconstruye ni hashea todo el índice bajo el escritor. Revisiones tienen incremento checked; overflow aborta la transacción, no hace wrap/ABA. No se usa un contador de revisiones de entidades como sustituto.

Alternativa de limpieza posterior asíncrona: dejaría un estado confirmado con postings huérfanos y cursores aparentemente válidos; descartada. La elección SQL del trigger frente a repetir limpieza por caller reduce rutas omitidas y debe probarse también con rollback de publicación.

### 3. Public contract and coverage

Dos variantes nuevas del catálogo, sin modificar las otras 49:

| Operación/acción | Input propuesto | Output propuesto |
| --- | --- | --- |
| `extraction.index`, `action:get` | Filtros opcionales `job_id`, `attachment_id`, `extractor`; `limit` default 32/max 64; `cursor` | Perfil/revisión, cobertura, entries paginadas por job/generación/slot, next_cursor. No contiene todos los textos. |
| `extraction.index`, `action:put` | `job_id`, `expected_generation_id`, `expected_revision`, todos requeridos | Revisión, entry confirmada y `changed`; misma generación íntegra con revisión actual devuelve changed=false. |
| `extraction.index`, `action:reset` | Sólo `expected_revision` | Revisión nueva y conteos eliminados. Vacía todo el índice, no una selección individual. |
| `extraction.search` | `query`; mismos tres filtros; `limit` default 20/max 64; `cursor` | Perfil/revisión, cobertura, hits tipados, next_cursor. |

Cada input niega campos desconocidos, mezclas de acciones y null donde un campo es requerido. Filtros UUID canónicos y extractor enum `utf8_text/pdf_text`; combinaciones sin coincidencias dan cero, no se aflojan. No se aceptan `kind`, `archived`, `related_id`, configuración FTS libre, paths de archivos, helper, SQL, prompt ni proveedor.

La cobertura de get/search se calcula en el mismo snapshot y con los mismos filtros: `scope=selected_generations`, conteos de `indexed_generations`, `unindexed_supported_generations`, `unsupported_generations`, `jobs_without_generation`, fragmentos/bytes seleccionados. Las tres primeras particionan generaciones actuales; la cuarta cuenta jobs sin generación. **No** es un conteo de todos los adjuntos del workspace. La selección sin texto cuenta como indexada con cero fragmentos.

Declarar `index_validation=canonical_text_and_postings`; `source_validation=not_checked` en get y `returned_candidates` en search. La salud estructural del índice no afirma haber releído todos los originales. Un job con configuración futura se cuenta sin alterarlo y no se selecciona; un perfil futuro del **índice** impide operar ese índice, incluso reset. No usar un booleano ambiguo `complete` que prometa OCR o cobertura universal.

Cada hit incluye IDs/hash/versión/config_hash de la generación, ordinal, score finito, `result_from_previous_attempt` y texto completo del fragmento (<=4.096 bytes), no texto de la copia como evidencia autónoma. El localizador discriminado conserva:
- `utf8_bytes`: byte_start/end y line_start/end existentes, rangos de bytes del original con extremo final exclusivo.
- `pdf_page_text`: page_number/text_byte_start/end existentes, sobre el texto derivado de esa página, junto con scope/limitations PDF actuales. No coordenadas visuales, offsets del archivo, OCR ni entidad falsa.

No se modifica `ExtractionGeneration`, `SearchHit` ni los lectores previos para añadir campos opcionales ajenos. Los nombres de DTOs/módulos nuevos se resuelven en apply sin cambiar este contrato.

### 4. Literal query and cursor semantics

Conservar la política literal de notas sin cambiar su implementación: split por blancos, 1–16 términos y <=256 **bytes** UTF-8; rechazar controles no blancos, citar cada término escapando comillas y unir por AND. MATCH recibe parámetro, no SQL interpolado. Unicode61 decide tokenización, mayúsculas y diacríticos; «literal» significa sin operadores del usuario, no substring ni igualdad byte a byte. Puntuación sin tokens puede dar cero; no construir expresiones de recuperación automática. Todos los términos deben coincidir dentro del mismo fragmento, sin joins de texto entre páginas o chunks.

Orden `bm25` ascendente y desempate binario `(job_id,generation_id,ordinal)`. No se compara score con el de notas o embeddings. Filtros restringen candidatos antes de LIMIT; estadísticas BM25 siguen siendo las del índice seleccionado completo, no un corpus recalculado por filtro.

Cursor JSON opaco v1, <=2.048 bytes, con scope hash workspace ID+ruta canónica, operación/acción, hash de query exacta, filtros normalizados, limit, perfil/revisión y offset positivo acotado a 16.384. No expone ruta ni query en crudo; no es secreto ni permiso. Rechazar campos/versión inválidos, límites desplazados y offsets fuera de rango. Con shape correcto pero revisión diferente: `revision_conflict/index_revision_changed`; otros scopes inválidos: `validation_error/cursor_scope_mismatch`. Get y search no intercambian cursores.

Una modificación válida del índice afecta ranking global y vence todos sus cursores, incluso con filtros disjuntos. El no-op put no los vence; reset siempre sí, incluso vacío. Cambios de estado del job o nuevas generaciones **no seleccionadas** pueden cambiar conteos/flags entre páginas, pero no el orden del conjunto indexado. No se ofrece un snapshot persistente entre solicitudes. Archivar/editar notas no cambia este índice independiente.

### 5. Pure reads and full logical index verification

Despachar las dos operaciones antes del `Workspace::open` genérico. Reutilizar preflight privado/versionado y la política de permisos/versiones de `batch_storage`, no su open read-only tal cual: activa query_only y bloquearía TEMP. Una apertura específica para estas operaciones abre get/search main READ_ONLY sin query_only, y put/reset READ_WRITE, siempre exigiendo versión actual. No debilitar query_only de los lotes ni exigir su whitelist al endpoint nuevo. Sólo el flujo explícito existente workspace.open/migración crea el backup y migra v9. No configurar journal_mode ni hacer recovery por consultar.

Get/search realizan una transacción de lectura: validar control/estructura y todos los entries/fragmentos seleccionados, independientemente del filtro; comparar copia FTS e identidades con los canónicos. Para UTF-8 reconstruir/hash a partir de fragmentos; para PDF reutilizar manifiesto/load, sin parser ni releer fuentes no devueltas. Comparar el hash canónico guardado al seleccionar para detectar cambios posteriores de texto/localizadores/manifiesto.

**Las filas FTS no bastan:** pueden faltar postings aun si la copia del texto está bien. Crear un FTS esperado sólo en TEMP, con el mismo esquema/tokenizer y los rowids deterministas reconstruidos desde canónicos. `fts5vocab(instance)` permite comparar bidireccionalmente `(term,doc,col,offset)` del FTS main y TEMP, además de la igualdad de filas/conteos/columnas. La implementación compara primero secuencias exactas con memoria constante: igualdad completa implica igualdad del multiset, sin asumir un orden público. Ante cualquier diferencia usa el fallback de conjuntos E ⊆ A y cardinalidades iguales (E canónico es único); no es un digest aproximado ni validación parcial. Detectar filas y postings faltantes o extra; error SQLite o diferencia da `storage_error/invalid_attachment_index`. No usar `INSERT ... integrity-check` en la base principal durante lectura. Temporales privados se descartan y no entran en backups/registry/base.v2. Usar almacenamiento temporal privado de SQLite; no prometer un cap RSS por limitar bytes lógicos.

El camino necesita TEMP escribible aunque main sea read-only. Esto sigue siendo una lectura pura **de datos persistentes**; SQLite puede usar archivos operativos SHM/temporales. La apertura read-only no equivale a `PRAGMA query_only=ON`, que bloquearía también TEMP. No prometer cero bytes operativos ni sandbox ante otro proceso privilegiado. Comprobar la propiedad con la SQLite bundled real y una transacción de lectura, no sólo con un mock.

Optimización de implementación: la conexión dedicada usa NO_MUTEX, igual que el default de rusqlite, porque no es Sync ni se comparte entre threads; conserva todos los locks de base/CAS/busy timeout. Las copias completas se comparan ordenadas explícitamente por rowid único, incluyendo sus tipos, en lugar de dos EXCEPT de texto. Esto no cambia el esquema, el hash ni la verificación completa de postings, cuyo orden público sigue sin suponerse. No se alteran aperturas de lotes/workspace/lectores.

Para repeticiones dentro del snapshot se permite reutilizar sólo tokenización, con clasificación por igualdad completa de bytes: textos únicos <=25% del total. El FTS esperado completo sigue disponible para consultas/scores. Un FTS temporal adicional con el mismo perfil obtiene el mapa completo posición/término por texto único; cada posting real verifica documento, término, columna y posición/tipos. Un bitset por documento rechaza duplicados; A ⊆ E, unicidad de A y cardinalidad igual implican A=E. No se asume orden público ni se guarda confianza entre llamadas. Sin reutilización suficiente, o si no hay igualdad, se conserva el verificador anterior completo. Memoria acotada por texto/fragmentos/rowids, no constante, con máximo 2 MiB de textos únicos elegibles y posiciones acotadas por sus bytes. Se mide RSS/scratch y se prueban duplicados/faltantes/extras, tipos, Unicode, orden y fuentes sin rebajar el objetivo.

El hash v1 usa vistas prestadas con orden de claves explícito, mismos bytes JSON/namespace/prefijos y prueba de compatibilidad con el código anterior/generaciones persistidas: no se cambia formato. Compilación aarch64 activa `sha2/asm` para habilitar el backend ARM con detección runtime/fallback portable; lock suma sólo sha2-asm 0.6.4. El backend también se aplica a hashes previos, con los mismos bits. Se declara esta modificación acotada de dependencias y se repite la regresión completa; no se agrega FFI/unsafe propio, ni se sustituye SHA-256, ni se actualiza SQLite.

Después de validar el índice, search lee n+1 candidatos ordenados y contrasta sus IDs/orden/scores con la misma consulta sobre el esperado TEMP: estadísticas BM25 dañadas también deben fallar, sin interpretar blobs internos privados de SQLite. Luego carga textos/localizadores canónicos y verifica metadatos/hash del original de cada generación distinta de esos candidatos, incluido lookahead. Fuente insegura/alterada da `storage_error/input_stale` reutilizando la IO segura actual. No omitir hits dañados como si no existieran, reparar ni hacer refill silencioso. La comprobación corresponde al instante de lectura; no garantiza inmutabilidad del filesystem después de responder.

Alternativas descartadas: validar sólo hits no detecta falsos ceros; `integrity-check` sobre main requiere escritura; invalidación por timestamps no verifica bytes. No se pretende autenticidad frente a un atacante que reescriba coherentemente toda la base y sus hashes.

### 6. Prepare outside the writer and commit with fencing

Put: validar input/perfil/revisión y preparar una generación en snapshot de lectura. Verificar fuente segura, configuración, canónicos y hash de entry; verificar índice existente fuera del lock de escritor. Liberar snapshot antes de `BEGIN IMMEDIATE`, nunca promover una lectura larga al escritor. Dentro del lock revalidar revisión global, identidad de generación/configuración, hash canónico de la generación elegida, fuente y cuotas reales; insertar entry/FTS, comprobar tamaños/respuesta/revisión y confirmar juntos. Un cambio de generación provoca conflicto aun si no había selección y por ello no cambió la revisión del índice.

La verificación completa del índice de hasta 8 MiB queda fuera del lock. Los escritores autorizados siempre modifican contenido/selección con revisión; el fence detecta esos commits entre preparación y publicación. Cambios fuera de las operaciones del núcleo no son una vía de concurrencia soportada. Revalidar bajo lock sólo la generación elegida (<=1 MiB/512 fragmentos), no todas las fuentes.

No-op: exige primero ambos CAS y validación completa; no encubre corrupción como una repetición exitosa. Reset: no necesita validar postings dañados para borrarlos. Con versión/perfil/control/esquema propios reconocibles, CAS de revisión, desactivar temporalmente su trigger de limpieza, vaciar entries, recrear únicamente su FTS con DDL canónica, reinstalar trigger e incrementar revisión una vez; todo dentro de una transacción. Un error revierte también el esquema. Verificar que el registry acepta el resultado exacto. Si está dañado el esquema/base física o no puede leerse el control de forma segura, fallar sin reparación general y recurrir al backup/recuperación existente. No resetear formatos futuros.

Errores conservan códigos existentes: `validation_error` (acción/campos/cuotas/cursor/config), `not_found` (job o generación ausente), `revision_conflict` (expected_revision_mismatch, generation_changed, index_revision_changed), `workspace_busy` (storage_contention/execution_deadline), `storage_error` (input_stale/invalid_attachment_index), `unsupported_workspace_version` (migración/versión futura). Los errores de índice recuperable incluyen current_revision; no necesitan devolver texto ni rutas sensibles para permitir reset. Contadores desbordados y errores de serialización se comprueban antes de commit.

### 7. Independent budgets and measured acceptance

| Recurso | Límite propuesto y relación |
| --- | --- |
| Selección por put | Una generación existente <=1 MiB, <=512 fragmentos; no cambiar admisión del extractor. |
| Índice seleccionado completo | 1.024 entries, 16.384 fragmentos, 8 MiB de texto copiado real (`length(CAST(text AS BLOB))`), también seleccionables sin texto. Sin eviction automática. |
| Extracción retenida | Sigue el límite independiente de 32 MiB reales UTF-8+PDF; el índice no descuenta/borra sus fragmentos ni duplica ese presupuesto como si fueran canónicos. |
| Snapshot | Siguen 64 MiB para SQLite completo. Put comprueba `page_count*page_size` e impone además un max_page_count de conexión hasta confirmar/rollback, incluyendo asignaciones FTS tardías en commit, shadows y freelist; no sólo 8 MiB de texto. Si la base ya supera el cap tampoco agrega selecciones. |
| Petición/respuesta | Validación global previa existente; respuesta API serializada <=2 MiB, cursor <=2 KiB. Sin truncar evidencia para forzar tamaño. |
| Ejecución | Presupuesto cooperativo de 5 s después de admisión/preflight, progress handler SQLite y checks en loops/IO; busy timeout actual 2 s. No es watchdog duro ni límite de tiempo para syscalls del SO. |

El guard de páginas usa el menor límite entre el ya existente y `floor(64 MiB/page_size)`, verifica el valor devuelto y lo restaura al terminar la conexión/operación. No confiar sólo en page_count antes de que FTS sincronice postings, ni aumentar un límite previo más estricto. Probar saturación en el propio COMMIT y rollback ante SQLITE_FULL con la SQLite bundled. No confundir límite de tamaño lógico de SQLite con bytes de WAL o RSS.

Reset sigue disponible para limpiar selecciones aun si el tamaño actual ya impide put/preview. No promete compactar el archivo ni devolver páginas al SO; no añadir VACUUM implícito. Los límites de bytes no son cap de RSS, de postings físicos ni de scratch; medir esos recursos por separado.

Benchmark de release reproducible en M1 Max/64 GiB: corpus sintético fijo, cinco warmups/veinte mediciones, consulta con/sin resultados, filtros y páginas; put/no-op/reset e invalidación de generación, ediciones concurrentes y saturación de cada cap por separado. Medir wall time, lock adquirido→commit/rollback, bloqueo observado por otro escritor, tamaño SQLite/WAL/scratch y RSS del proceso. Target de consulta p95 <=300 ms como referencia del plan, no resultado existente; gate de lock p95 <=150 ms y presupuesto acotado en casos admitidos. Comparar búsquedas/ediciones de notas con baseline. Si falla un target/gate, registrar baseline fallido, diagnosticar y ajustar implementación o revisar explícitamente el diseño; no aumentar caps, retirar validación ni declarar cierre por pasar tests funcionales.

### 8. Batches and source ownership remain unchanged

Añadir 010 al registry de esquema completo, creación, migración, preflight e integridad de backups/restores. Incluir FTS y shadows/triggers en snapshots; no agregar estas tablas a `ORIGINAL_TABLES`. La huella base.v2 después de migrar no cambia al indexar/resetear/invalidate: sigue incluyendo `extraction_control` original, no este control derivado. La versión v10 sí cambia el entorno/huella y rechaza recibos v9.

No ampliar whitelist/checks de `changes.preview/apply`. Preview ejecuta sólo lo admitido sobre snapshot; apply sigue mutando el workspace actual, nunca reemplazándolo por la copia. Probar que un put/reset concurrente al período preview→apply no se pierde ni genera conflicto falso de dominio. El historial de entidades no registra indexación como edición de nota. No modificar documentos ni adjuntos, ni codificar relaciones de notas en FTS.

## Risks / Trade-offs

- [Validación completa de postings en cada lectura cuesta CPU/temporales] → Caps independientes y benchmark real; no confundir el probe SQL con rendimiento del producto. No introducir cache global con invalidación incompleta para ocultarlo.
- [La copia FTS ocupa más que sus bytes de texto] → Medición física/snapshot antes de commit; cuotas rechazan sin eviction. Un reset puede dejar freelist grande.
- [El conjunto seleccionado puede ser parcial y fragmentos cortan frases] → Cobertura y semántica explícitas. No usar cero hits como prueba de ausencia universal; híbrida/chunking contextual serán otro cambio.
- [Una fuente no devuelta puede haberse alterado] → get informa que no verifica originales; search verifica sólo candidatos de la página. No prometer auditoría universal del filesystem.
- [Índice global cambiante vence cursores de filtros ajenos] → Conflicto explícito, preferido a páginas silenciosamente mezcladas; no sesión persistente de consultas en este recorte.
- [Nuevo trigger afecta publicación PDF/UTF-8] → Tests de cada punto de rollback, carreras y lock p95, preservando fences/lease 15 s/ejecución 5 s y helper existentes.
- [Reset no repara SQLite físicamente corrupto o esquema desconocido] → Límite de recuperación documentado, backup fiel y error sin descartar formatos futuros.
- [Más operaciones no significan UI/modelos listos] → API v1/51 verificada, sincronizada y archivada en este recorte. Home nativo y resto de 5a/5b siguen pendientes.

## Migration Plan

1. Preservar baseline v9 externo y CLI v9 construido desde esa copia para pruebas reales; nunca usar una base fabricada por el futuro CLI como única evidencia de migración. No tocar 001–009.
2. Añadir 010 vacía y cadena/registry/integridad v10. Abrir/migrar por el flujo existente crea backup antes del lock y confirma versión+esquema juntos, sin indexar ni recuperar jobs. Probar v9 poblado y v9 running/con resultado previo/configuración futura.
3. Publicar contratos aditivos y operaciones después de pruebas; get/search no migran automáticamente. Un cliente anterior rechaza v10 antes de escribir.
4. Backup/restauración copia un estado consistente completo y valida el índice en staging. Comprobar restauración sin helper y rechazo de cursor de ruta anterior, incluso con misma identidad de workspace.
5. Rollback de migración fallida: v9 intacto con backup. Rollback de versión del software después de migración confirmada: usar el backup previo en destino nuevo; **no** bajar user_version, editar migraciones antiguas o sobrescribir cambios posteriores silenciosamente. Reset sólo reconstruye el derivado en v10, no revierte esquema.
6. Completar matriz de los 15 requisitos/40 escenarios nuevos y tests/benchmark antes de sincronizar únicamente estos tres deltas y archivar. El cierre reconcilia además el requisito heredado de migración y sus cinco escenarios: v10 ya no es una versión futura, v7/v8 recorren la cadena actual sin ejecutar extracción y el índice añadido queda vacío. No cambia código ni migraciones previas; la regresión focalizada debug/release y el CLI v8 independiente lo comprueban. Home queda activo 9/17; ningún test headless/no-bundle satisface su aceptación nativa.

## Research and Validation Status

Implementación: [verificación actual](../../../../docs/verification-attachment-search.md) y `tasks.md`, **19/19 tareas**. Regresión fresca de 212 tests Rust por perfil, migraciones/20 rondas CLI por perfil, PDF parser/CLI/contención, contratos y 606 checks headless aprobados; 15 requisitos/40 escenarios nuevos trazados y un requisito/cinco escenarios heredados reconciliados. Dos benchmarks finales consecutivos sobre el corpus/hash de 8 MiB: query p95 **214,20–231,07 ms** (<300 ms), lock máximo p95 **17,66 ms** (<150 ms), scratch físico **21.254.144 bytes** por sonda externa con self-test/copia/clon/SHM. Se conservan baselines fallidos y experimentos descartados; no se cambiaron targets/caps ni se retiró validación. No garantiza ese p95 para textos sin repetición, caché fría o todos los caps simultáneos. Tres deltas sincronizados y contrastados con main; archivado como `2026-10-14-add-local-attachment-search`, sin alcanzar a Home ni cerrar toda 5a/5b.

Consulta oficial del 14/10/2026: [SQLite FTS5](https://www.sqlite.org/fts5.html), secciones external/contentless tables, BM25, integrity-check y fts5vocab; [PRAGMA max_page_count](https://www.sqlite.org/pragma.html#pragma_max_page_count). La documentación describe las funciones, no garantiza latencia en Noteka.

Probe SQL descartable fuera de Git con Node 22.22.3/SQLite 3.51.3: FTS normal, MATCH literal y borrado por trigger/CASCADE; luego main read-only + FTS esperado y vocabularios sólo en TEMP. Diferencia de tokens 0 con copias iguales y 6 al cambiar el esperado; main rechazó DELETE y conservó hash del archivo. Un primer intento con otro esquema adjunto `:memory:` bajo apertura read-only falló al escribir: se descartó esa variante y se comprobó TEMP. Otro probe impuso 16 páginas a una conexión con FTS: SQLITE_FULL revirtió la inserción completa (cero filas/seis páginas) y al reabrir no persistió el límite reducido. Es evidencia de viabilidad, **no** una prueba de la SQLite bundled de Rust, una transacción concurrente ni un benchmark.

Evidencia externa de planificación: `noteka-attachment-search-plan-B5eCmv/` bajo el temporal aprobado de OpenCode, con baseline/manifiesto e informes de probes. Validación OpenSpec estricta del cambio y global aprobada: **14/14** elementos. Revisión externa: 284→291 archivos, tres documentos/contextos existentes modificados, siete artefactos nuevos, 281 archivos previos idénticos y ninguno eliminado; los 222 archivos protegidos de código/tests/specs vigentes/archivos históricos/Home conservan sus hashes. Enlaces locales y 19 checks sin marcar comprobados. Ver `planning-diff-review.json`, `planning-manifest.json`, `planning-baseline.diff` y logs de validación externos. No se modificó código ni se ejecutaron suites de aplicación en esta fase; estas comprobaciones no acreditan ninguna tarea de implementación.
