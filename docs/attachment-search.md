# Búsqueda local de adjuntos

Implementación de [`add-local-attachment-search`](../openspec/changes/archive/2026-10-14-add-local-attachment-search/proposal.md) con **19/19 tareas verificadas**, tres deltas sincronizados y cambio archivado el 14/10/2026. [Evidencia](verification-attachment-search.md): dos corridas con p95 query 214,20–231,07 ms sobre el corpus fijado de 8 MiB frente a <=300 ms; lock aprobado y scratch físico medido. No se modificaron target/caps ni se retiró validación; no es una garantía universal para cualquier corpus/caché. Sin UI nueva ni modelos/proveedores.

## Persistencia y recuperación

Esta guía conserva el cierre histórico UTF-8/PDF v10. El producto actual es
v12/API v1/52: [XLSX amplía index/search](xlsx-search.md), no el perfil FTS ni los
hashes UTF-8/PDF. Creá/migrá explícitamente el workspace antes de usar index/search;
los ejemplos requieren la versión vigente.

La migración aditiva `010.sql` lleva SQLite v9 a v10, con backup previo por el flujo de apertura/migración existente. No modifica 001–009, originales, documentos, cola ni generaciones UTF-8/PDF. Añade control derivado v1/revisión1/perfil `literal_unicode61_v1`, selecciones por trabajo/generación y FTS5 independiente con sus shadows. El índice inicia vacío: extraer no selecciona.

Una selección usa un slot interno 1–1.024. Cada fragmento tiene rowid `(slot−1)*512+ordinal+1`; no es un ID público. La FK a la generación borra la selección al reemplazarla; su trigger elimina filas FTS e incrementa revisión en la misma transacción. No se genera una edición de nota ni se toca `search_fts`.

Una migración fallida revierte íntegramente y conserva backup. Volver al software v9 después de migrar requiere restaurar el backup previo en un destino nuevo, no bajar manualmente la versión ni sobrescribir cambios posteriores. Un reset del índice no revierte la migración ni repara corrupción física de SQLite.

## API v1 y uso explícito

El CLI añade `extraction index` y `extraction search` (operaciones `extraction.index/search`). Las 49 entradas/salidas anteriores permanecen iguales; no hay whitelist batch nueva. Pasá JSON por stdin con `--workspace RUTA --json`. Ejemplos de payloads, en orden:

```json
{"action":"get"}
{"action":"put","job_id":"UUID","expected_generation_id":"UUID","expected_revision":1}
{"query":"contrato café","extractor":"pdf_text","limit":20}
{"action":"reset","expected_revision":2}
```

Cada objeto es un input distinto, no un archivo de cuatro líneas para una sola llamada. En un workspace existente actual (v12), con `WORKSPACE` definido y archivos JSON externos `PUT_INPUT`/`RESET_INPUT` que contengan los IDs/revisiones elegidos:

```sh
printf '%s\n' '{"action":"get"}' | ./target/debug/noteka --workspace "$WORKSPACE" --json extraction index
./target/debug/noteka --workspace "$WORKSPACE" --json --input "$PUT_INPUT" extraction index
printf '%s\n' '{"query":"contrato café","limit":20}' | ./target/debug/noteka --workspace "$WORKSPACE" --json extraction search
./target/debug/noteka --workspace "$WORKSPACE" --json --input "$RESET_INPUT" extraction index
```

La revisión de los ejemplos 1/2 sólo vale para un índice recién creado, sin otra escritura. Ante conflicto, releé `index get` y `extraction get` para la revisión/generación actuales; decidí explícitamente si todavía querés seleccionar ese resultado y enviá un put nuevo. No repitas una escritura a ciegas ni reutilices cursores vencidos. Reset requiere la revisión actual y borra **todas** las selecciones; no es el reintento automático de un put fallido.

Get y search aceptan filtros AND opcionales `job_id`, `attachment_id`, `extractor` (`utf8_text`/`pdf_text`), `limit` y `cursor`. Defaults: get 32, search 20, máximo 64/mínimo 1. Put exige trabajo/generación UUID canónicos y revisión positiva; reset sólo revisión. Campos ajenos, acciones mezcladas y null requerido son inválidos. No se aceptan rutas de fuentes, helper/proveedor, configuración FTS, filtros por entidad/archivo/contexto ni consultas SQL. No hay extracción implícita, recovery, daemon ni envío remoto.

Los endpoints exigen SQLite v10: un v9 devuelve `unsupported_workspace_version` sin migración; sólo `workspace open` usa el flujo existente de migración y backup. Get/search abren main READ_ONLY, con TEMP privado escribible para verificar; no usan ni debilitan `query_only` de los lotes. Put/reset usan READ_WRITE sin cambiar journal_mode.

Errores y razones: `validation_error` para formas, cuotas, configuración no admitida y cursor inválido; `not_found` para job/generación ausente; `revision_conflict` para `expected_revision_mismatch`, `generation_changed` o `index_revision_changed`; `workspace_busy` para `storage_contention`/`execution_deadline`; `storage_error` para `input_stale`/`invalid_attachment_index`; `unsupported_workspace_version` para migración requerida/futuro. Un error lógico recuperable del índice informa `current_revision`, utilizable en reset; no entrega texto ni rutas como requisito para repararlo.

## Identidad, lectura y cobertura

Cada selección corresponde a la generación actual publicada del job, aun mientras un intento posterior está queued/running/failed/cancelled. El hit tiene IDs/hash fuente/versión/config_hash, ordinal, score finito, fragmento canónico completo y `result_from_previous_attempt`. No inventa una entidad propietaria: un adjunto puede estar compartido, archivado por alguna referencia o sin referencias.

Localizador `utf8_bytes`: byte_start/end y line_start/end del lector literal, extremo final exclusivo, sin normalizar CRLF/BOM/multibyte. `pdf_page_text`: página 1-based y rango de bytes del texto derivado de esa página, scope `page_text_objects` y limitaciones PDF existentes. No son offsets del binario, coordenadas, render ni OCR. No se modifican los dos lectores anteriores.

Cobertura filtrada: `indexed_generations`, `unindexed_supported_generations`, `unsupported_generations` particionan las generaciones actuales; `jobs_without_generation` cuenta jobs sin resultado. PDF `no_text` seleccionado cuenta como indexado, con cero fragmentos. `scope=selected_generations` no representa todos los adjuntos. Configuraciones futuras se cuentan sin reserializarlas ni ejecutarlas.

Get/search verifican todos los entries/canónicos/copias y postings bidireccionalmente con tokenización esperada en TEMP y enumeración exacta de instancias, antes de responder incluso cero hits. `index_validation=canonical_text_and_postings`. Get declara `source_validation=not_checked`: no relee archivos. Search compara también candidatos/scores con TEMP y verifica los originales de las generaciones distintas de la página, incluido lookahead, declarando `returned_candidates`. Si una fuente falla no omite/refill hits, ni repara/reextrae. No garantiza archivos ajenos a la página ni su inmutabilidad después de responder.

Para texto repetido, la optimización comparte solamente el mapa completo de tokenización de textos byte a byte iguales **dentro de la misma lectura**. Verifica cada posting real y usa un bitset por documento para rechazar duplicados; no omite documentos ni usa un digest aproximado ni mantiene confianza entre llamadas. Sin suficiente repetición conserva la comparación completa anterior y su fallback exacto. La mejora depende del corpus: las mediciones del corpus fijado no son una garantía universal sobre cualquier texto de 8 MiB. [Prueba de igualdad, memoria y mediciones](verification-attachment-search.md).

En la validación MCP se añadió un recorrido exacto por la API auxiliar pública FTS5: todos los términos reales y cada documento/columna/offset con los mismos bitsets y cardinalidad, sin omitir checks de fuentes/copias/estadísticas. Vocabularios de más de 256 términos o streams independientes conservan `fts5vocab(instance)` completo; el umbral elige algoritmo, no es un cap de entrada. [ADR 0015](decisions/0015-exact-fts5-instance-traversal.md) y [gates actuales](verification-mcp.md#recorrido-exacto-fts5--gates-finales-aprobados): dos rondas con query p95 **134,96–204,81 ms**, sin cambiar corpus/target/caps. La evidencia de scratch físico del cierre anterior sigue siendo histórica, no una medición nueva.

## Consultas y cursores

Consulta de 1–16 términos separados por blancos y hasta 256 bytes UTF-8. Se rechazan controles no blancos, se escapan comillas y se unen términos citados por AND parametrizado. OR/NEAR y SQL aparente son texto; Unicode61 decide tokens, case y diacríticos. No es substring: puntuación sin tokens puede dar cero; todos los términos deben coincidir en un mismo fragmento. Orden BM25 ascendente, desempate binario job/generación/ordinal, estadísticas del índice seleccionado completo aun con filtros. No comparar score con notas ni embeddings.

Cursor propio JSON <=2 KiB ligado a workspace/ruta canónica, operación, hash de consulta exacta, filtros, límite, perfil/revisión y offset. No contiene ruta/consulta en crudo ni es permiso. Put con cambios, reset aun vacío e invalidación vencen todos los cursores; put no-op no. Ante conflicto, el cliente reinicia explícitamente, nunca mezcla revisiones. Estado de cola/cobertura/flag puede cambiar sin modificar orden del índice. Una restauración en otra ruta rechaza los cursores anteriores.

## Escrituras y límites

Put prepara canónicos/fuente/índice íntegro fuera del escritor. Bajo BEGIN IMMEDIATE revalida revisión y generación/fuente elegida, cuotas y respuesta; confirma entry/FTS/revisión juntos. No-op también exige ambos CAS e integridad. Reemplazo exitoso de generación invalida sólo su selección por CASCADE/trigger en la misma transacción; la nueva no se selecciona. Falla, retry, pausa o cancelación conservan el índice previo con flag anterior coherente.

Caps independientes: 1.024 selecciones, 16.384 fragmentos y 8 MiB de texto copiado real; generación <=1 MiB/512 fragmentos; extracción retenida sigue 32 MiB; snapshot completo 64 MiB; respuesta 2 MiB; cursor 2 KiB. Sin eviction/truncado de evidencia. Presupuesto cooperativo 5 s tras preflight/progress handler/checks IO; espera escritor 2 s. No son watchdog duro, cap RSS, WAL o scratch.

Put mantiene max_page_count hasta COMMIT/rollback, tomando el menor límite entre el previo y 64 MiB/page_size, y lo restaura al salir. Así incluye asignaciones FTS tardías, shadows/freelist. Reset queda disponible si put no cabe, pero no compacta ni ejecuta VACUUM.

## Reparación deliberada

Si main sigue legible y control/versión/perfil/schema propio son reconocibles, reset con CAS elimina entries y recrea sólo FTS/trigger canónicos transaccionalmente, sin validar antes postings dañados. Después ejecutá puts de generaciones retenidas; no hace falta helper ni modelos. Una falla revierte DDL/revisión/datos íntegramente. Formato futuro o schema no reconocido se rechaza sin descartarlo; corrupción física exige el flujo de backup/recuperación, no reset.

## Evidencia de admisión y migración

Baseline externo de implementación: `noteka-attachment-search-apply-qLSCJI/` en el temporal aprobado, 291 archivos y CLI v9 independiente compilado offline/release desde su copia. `scripts/test-attachment-search.py --baseline-cli "$AUDIT/noteka-v9"` comprueba v9 poblado UTF-8/PDF y v9 running con resultado previo/configuración futura, todas las filas/archivos, recibo v9 rechazado, cliente anterior sin escrituras y restauración del backup previo con el cliente v9. La inyección Rust de falla de migración deja v9/backup íntegros. Regresión debug/release, 606 checks UI headless y matriz funcional documentadas; el target query se comprueba con dos benchmarks finales y scratch físico con sonda/self-test separado, no se infieren de esos tests funcionales.

## Escritorio al 17 de octubre de 2026

La [preparación explícita](attachment-preparation-ui.md) permite seleccionar un
adjunto administrado, consultar y ejecutar extracción y seleccionar generación
para Search, con las revisiones del núcleo. El [lector de generaciones](attachment-reader-ui.md)
recorre todo el derivado por páginas sin exigir indexación. La
[consulta agrupada](grouped-search.md) usa ambos motores sin fusionar cursores o
scores. [Aceptación técnica conjunta](verification-local-preparation-closure.md).
Los límites de formato/capacidad del núcleo no se amplían con estas UIs; abrir o
buscar no prepara ni repara. Los hitos históricos anteriores conservan su alcance.
