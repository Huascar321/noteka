# Design

## Context

Motivación y recorte en [proposal.md](proposal.md); comportamiento exigido en los tres deltas de `specs/`. **Baseline previo a implementación, 13/10/2026:** SQLite v7/API v1/40 operaciones, sin cola ni extractor. Las referencias de líneas de la tabla siguiente corresponden a ese baseline, no al código final. **Implementación actual:** SQLite v8/API v1/48 operaciones; [guía](../../../../docs/extraction.md), [verificación](../../../../docs/verification-extraction.md) y [ADR 0011](../../../../docs/decisions/0011-explicit-local-extraction.md). Este diseño conserva las decisiones del cambio; la evidencia está separada.

Fronteras verificadas en el código:

| Área | Evidencia y consecuencia |
| --- | --- |
| Originales | `crates/noteka-core/src/attachments.rs:17–88,106–188`: metadatos, archivo regular, SHA-256 y máximo de importación de 64 MiB; la lectura pública devuelve base64. Extracción necesita lectura interna limitada, no cargar/base64-encodear un adjunto de 64 MiB. |
| Procedencia actual | `search.rs:5–67,109–163`: FTS de título/bloques y revisión de entidad. No indexa archivos; este cambio no altera ese contrato ni declara búsquedas de adjuntos. |
| Persistencia | `workspace.rs:73–109,188–229,245–289,292–437`: creación, migración con backup, snapshot SQLite y manifiesto v1/v2. La evolución exige cubrir creación, apertura, validación de backup y fixtures anteriores, no sólo agregar SQL. |
| Lotes | `batch_storage.rs:19–98,162–187,279–322`: schema cerrado, copia de toda SQLite y huella de trece tablas originales. Nuevas tablas requieren clasificación explícita; no pueden omitirse de la whitelist ni del tamaño del snapshot. |
| Transportes | `lib.rs:34–96,118–224` concentra catálogo/dispatcher; `crates/noteka-cli/src/main.rs:251–288,324–377` valida JSON, canales/códigos y igualdad de catálogo/enum/CLI. No hay un `catalog.rs` separado ni reglas de negocio en el CLI. |
| Recuperación existente | Tests internos de workspace, `tests/workspace_validation.rs`, `tests/block_editor.rs` y `src/batch_tests.rs` cubren fallos de migración, originales, extensiones, WAL, formato futuro, rollback y restore. Sus fixtures de downgrade actualmente conocen sólo v7. |

El workspace permanece offline; Home conserva su código/composición y 9/17 tareas verificadas. No se usa la pantalla ni se altera su aceptación nativa. No se inspeccionan notas personales ni se instala ningún modelo.

## Goals / Non-Goals

**Goals:** separar decisiones de usuario, intentos operativos y resultados regenerables; permitir edición concurrente durante extracción; demostrar procedencia exacta y recuperación ante proceso interrumpido; integrar el cambio con migración, backup y lotes existentes.

**Non-Goals:** convertir el CLI en servicio residente, mantener una transacción abierta durante todo el procesamiento, persistir resultados parciales, copiar originales para publicar, agregar reglas en TypeScript o prometer atomicidad frente a modificaciones externas del filesystem. Las exclusiones funcionales están en proposal.

## Decisions

### 1. Worker explícito compartido, no scheduler implícito

Módulos dentro de noteka-core: `extraction_contracts.rs` para DTOs, `extraction.rs` para operaciones, `extraction_storage.rs` para persistencia/lectura/integridad, `extraction_worker.rs` para fencing/claim/publicación y `text_extractor.rs` para UTF-8 puro. La cola es dueña de transacciones/transiciones; el extractor recibe bytes verificados y callback de presupuesto, produce fragmentos en memoria sin SQLite/shell/red/UI. DTOs reexportados desde contracts, ocho variantes Operation/Contract/catálogo/dispatcher. Tauri transporta operaciones genéricas sin pantalla ni reglas nuevas. La separación en dos módulos auxiliares conserva las fronteras previstas; hooks/reloj sólo cfg(test).

`extraction.run` bloquea **su llamada**, pero el trabajo está desacoplado del encolado/edición y otra conexión puede operar. Procesa como máximo un trabajo. Un cliente puede repetir run explícitamente para drenar la cola; no se agrega un comando loop, hilo permanente, polling o ejecución al abrir/importar. La primera versión permite **un único ownership activo por workspace**, aunque tenga muchos jobs; no sólo uno por job.

**Alternativas:** extraer al importar hace impredecible su latencia y mete procesamiento dentro de la copia del original; daemon/Tokio añaden lifecycle sin necesidad probada; mantener una Immediate durante extracción bloquea ediciones. Un worker limitado por llamada permite verificar separación, interrupción y uso con ventana cerrada usando dependencias actuales.

### 2. SQLite v8 y cinco tablas con ownership distinto

La migración aditiva `008.sql` implementa:

| Tabla | Datos | Autoridad / política |
| --- | --- | --- |
| `extraction_control` | Singleton: versión de configuración, paused, revisión, updated_at | Configuración original; inicial paused=true/revisión 1. Entra en base_hash. |
| `extraction_runtime` | Singleton: job activo, token, scope, revisión de control al claim, lease hasta | Operativo, token privado; excluido de base_hash. Es la reserva global, sin filas de trabajo ficticias. |
| `extraction_jobs` | ID, adjunto/hash/tamaño, extractor/version/config JSON/huella, state/revision, attempts, timestamps, error, generación publicada | Operativo; identidad estable. Selección explícita deduplicada por adjunto + hash de entrada + extractor/version + configuración canónica. |
| `extraction_generations` | Job/generación, fuente/config conocidas, cobertura, bytes, fragment_count y published_at | Derivado regenerable; sólo generación publicada por job. No es una nota. |
| `extraction_fragments` | Job/generación/ordinal, texto literal, byte_start/end y line_start/end | Derivado; orden por ordinal y clave compuesta, sin FTS nuevo. |

FKs e índices cubren fuente, job/generación, selección queued y paginación; las restricciones de integridad evitan ownership sin job y fragmentos de generación ajena. Estado inicial conocido: queued/running/succeeded/failed/cancelled/stale. Revisiones e intentos usan incrementos comprobados; overflow se rechaza, no se reinicia.

No hay definición editable de extractor, perfil de modelo vacío ni tablas genéricas para fases posteriores. La configuración versionada del job es inmutable; cambiar extractor/configuración crea otra identidad de trabajo, no parchea resultados viejos. Adjuntos diferentes con bytes iguales siguen teniendo jobs distintos: la procedencia se refiere al ID original, no sólo al hash.

Persistir el JSON de configuración de forma conservadora permite get/list y backup de versiones futuras: devolver crudo + `supported:false`/motivo, sin convertirlo. Ejecutar o retry sobre un formato no soportado se rechaza. En selección automática se omiten incompatibles y se informa su cantidad; no se bloquea silenciosamente toda la cola. Datos estructuralmente corruptos se identifican como storage_error, no como configuración válida.

**Alternativas:** meter fragmentos en documentos rompe autoridad/revisiones; usar un manifest externo añade atomicidad entre dos almacenes; tratar control, jobs y resultados como una sola tabla impide diferenciar configuración original de actividad derivada en recibos.

### 3. Ocho operaciones estrictas y resultados que no ocultan intentos

API v1 continúa; nombres y subcomandos son `extraction <acción>`. Todos los inputs niegan campos desconocidos; no aceptan rutas arbitrarias, contenido, SQL, reloj, token o presupuesto del cliente.

| Operación | Entrada propuesta | Resultado / efectos |
| --- | --- | --- |
| enqueue | `{attachment_id, extractor, config?}`; extractor obligatorio `utf8_text`; config omitida usa `{version:1,chunk_bytes:4096}` | `{job, deduplicated}`. Validar ID/metadata, elegibilidad y fuente de hasta 1 MiB antes de crear; el contenido se valida al ejecutar. Buscar dedupe antes de aplicar caps de admisión. |
| get | `{id}` | Job, generación previa si existe, soporte y disponibilidad de ownership; sin texto ni token, no ejecuta ni cambia estado. |
| list | `{attachment_id?, state?, limit?, cursor?}` | Página de jobs sin texto, con `next_cursor`; default 50, máximo 100. Orden created_at/ID asc, snapshot por llamada. |
| run | `{id?}` | Con ID valida existencia/soporte/estado incluso pausado y procesa sólo ese queued compatible; sin ID reclama el compatible más antiguo. Devuelve `{outcome, reason?, job?, recovered_count, incompatible_count}`; no retorna texto completo. |
| cancel | `{id, expected_revision}` | queued/running → cancelled; cancelled vigente es no-op. Otros terminales rechazan invalid_state. Invalida reserva/token sin borrar una generación válida. |
| retry | `{id, expected_revision}` | failed/cancelled/stale/succeeded → queued, después de admitir capacidad. queued/running rechazan invalid_state; no incrementa attempts ni borra resultado previo. |
| control | Unión estricta `{action:"get"}` o `{action:"set", paused, expected_revision}` | Configuración; get es sólo lectura. Set idéntico con revisión vigente es no-op; cambiar paused incrementa revisión. No encola/ejecuta. |
| fragments | `{id, limit?, cursor?}` | Generación completa identificada, procedencia/cobertura y **una página** de fragmentos; default 32, máximo 64. No resultado → not_found con reason específico. |

Enqueue repetido devuelve el job actual, incluso terminal, sin claim/retry implícitos. Las modificaciones verifican revisión **antes** de considerar un no-op: una revisión vieja falla también con el valor deseado ya aplicado.

Run con cola pausada/vacía/sin compatibles devuelve outcome=idle con motivo, no éxito ficticio de extracción. Otro ownership activo da workspace_busy sin ejecutar. Una publicación válida devuelve succeeded; pausa/cancelación ya confirmadas dan interrupted con estado actual. Un fallo de contenido/IO/deadline devuelve el envelope de error con job_id/estado/revisión/intento si existen; no se presenta como succeeded. El claim ya confirmado **no se revierte** por ese envelope: un intento failed o recuperable es observable.

Mantener CoreError/códigos vigentes: entrada/config/formato/UTF-8/cap → validation_error (2); ID/resultado ausente → not_found (3); revisión → revision_conflict (4); lock/worker/deadline → workspace_busy (4); fuente alterada/IO/persistencia → storage_error (5); esquema futuro → unsupported_workspace_version (6). `details.reason` distingue casos, sin stacktrace, token, contenido del adjunto ni input íntegro. Error operativo retenido se acota a 4 KiB, mensaje a 512 caracteres y configuración cruda a 8 KiB. Si el almacenamiento impide registrar el fallo, informar `cleanup_pending:true`; no afirmar que se confirmó failed.

Catálogo final esperado: 48 operaciones; todas las extraction.* con batch_mutation=false y batch_check=false. No cambia la whitelist de nueve mutaciones ni el único check database.query. Comparar semánticamente inputs/outputs de las 40 operaciones anteriores, además de regenerar esquema y TypeScript.

### 4. Claim, procesamiento y publicación con fencing

1. **Adquisición corta / Immediate:** leer control/scope y validar integridad operativa. Un run explícito recupera un ownership vencido o de otra ruta como failed (`worker_interrupted` o `scope_mismatch`), incrementa revisión y libera reserva. No reintenta ese job. Si era el ID solicitado, devuelve su fallo recuperado; sin ID puede reclamar otro queued. Un ownership todavía válido rechaza otro claim. Elegir job compatible, cambiar queued→running, aumentar revisión/attempts y reservar token aleatorio con scope y lease; serializar/limitar resultado acotado antes de ese commit.
2. **Trabajo sin lock de escritura:** leer bytes del adjunto con límite+1 y verificación de ID/ruta regular/tamaño/hash; no usar attachment.read/base64. Decodificar y fragmentar en memoria. Comprobar presupuesto y la reserva/control entre bloques de lectura y fragmentos. No mantener snapshot SQLite largo que oculte pausa/cancelación confirmadas.
3. **Publicación corta / Immediate:** releer control, revisión del job, token, scope y lease; revalidar metadata/hash del original mediante lectura interna limitada. Confirmar configuración soportada y cupos. Sustituir generación previa por la completa preparada, registrar nueva generación UUID/fragmentos/cobertura, marcar succeeded e invalidar reserva juntos. Serializar la respuesta y comprobar nuevamente vencimiento y presupuesto inmediatamente antes del commit.
4. **Salida sin publicación:** para fallo, hacer transición condicionada a ese token/revisión; jamás sobrescribir cancelación/pausa/recuperación ajena. La pausa confirmada set(true) devuelve un running con ownership vigente de esta ruta a queued e invalida reserva en su propia transacción; el worker se retira cooperativamente. Si el ownership ya venció o es ajeno, cambiar control lo recupera a failed con motivo, no a queued: requiere retry explícito. Un set idéntico sigue siendo no-op, sin recuperación encubierta. Si una limpieza falla por IO/lock, queda ownership recuperable por vencimiento, sin fragmentos publicados. Cancelar invalida ownership a cancelled. Reanudar no arranca un job.

Token + revisión del job + revisión del control + scope forman el fencing. Comparar sólo state=running no alcanza: un retry/reclaim puede volver a ese mismo estado. Guardar la revisión de control del claim evita el ABA pausa→reanudar entre dos comprobaciones, aunque el worker no haya observado paused=true. El token jamás se expone como permiso o autenticación: sólo impide publicación tardía accidental.

Scope = workspace ID + SHA-256 de ruta canónica, con namespace propio de extracción. El worker comprueba que su ruta siga resolviendo al scope reclamado. Restore en otro destino conserva el registro operativo, pero get/list informa ownership no activo y recoverable=true; no lo muta hasta run/control/cancel autorizado. Los timestamps de lease se guardan UTC; el presupuesto del proceso usa Instant. Lease fijo **15 s**, sin heartbeat para una ejecución cooperativa de **5 s** después del claim. Expiración se revalida antes del commit; salto de reloj hacia adelante falla conservadoramente. Un retroceso del reloj puede demorar recuperación tras crash; cancel con revisión vigente permite liberar sin adivinar liveness por PID.

Espera de escritor: 2.000 ms por adquisición; deadline no se reinicia entre lectura, extracción y publicación. Usar progress handler/guardia para SQL y checks fuera de SQL; limpieza best-effort tiene espera acotada separada. Abrir/preflight, espera inicial, syscall y cleanup no constituyen una garantía wall-clock de 5 s. No retry automático de mutaciones.

**Alternativas:** identificar por PID puede confundir procesos reutilizados y restores; un simple flag running no resiste publicación tardía; recuperar al abrir o al consultar hace lecturas con efectos y no puede distinguir otra conexión viva. Lease/fencing + recuperación explícita admiten pruebas deterministas sin servicio permanente.

### 5. Texto UTF-8 literal con localizadores exactos

Elegibilidad inicial: nombre persistido con extensión `.txt`/`.md` case-insensitive, no MIME reconocido como imagen/PDF u otro medio no textual; application/octet-stream es elegible porque el importador actual lo usa para texto. La extensión no demuestra por sí sola que el contenido sea texto: el worker exige UTF-8 estricto, sin NUL, y verifica bytes/hash. PDF o imagen renombrados no se aceptan por su nombre. No agregar detección perfecta de todo formato binario ni decodificación con pérdida.

Config v1 acepta solamente chunk_bytes=4096. Fragmentos contiguos, sin overlap, trim ni normalización: desde byte 0 tomar el mayor endpoint de carácter UTF-8 <=4096 bytes desde el inicio. Esto resuelve líneas largas sin generar fragmentos enormes. BOM y saltos CRLF se conservan; se permite una frontera entre CR/LF y se documenta que las líneas se cuentan por LF. Texto de cada fragmento es exactamente el slice original. La concatenación reconstruye el archivo; cero bytes produce una generación succeeded con cero fragmentos.

Localizador: `[byte_start, byte_end)` con offsets desde 0; `line_start/line_end` inclusivos desde 1. La línea de un byte es 1 + cantidad de LF anteriores a él; para fragmento no vacío, line_end es la línea de su último byte. No atribuir entidad/bloque: devolver attachment_id/hash y configuración/generación. Quitar una referencia de una nota no modifica el adjunto ni inventa una relación vigente con esa nota.

Cada generación registra source_bytes, covered_bytes, fragment_count y complete=true sólo después de comprobar cobertura exacta `[0, source_bytes)` y hash/config. Retener **una** generación publicada por job. Durante retry queda disponible la previa con `result_from_previous_attempt:true`; falla nueva no la elimina. Sustitución atómica borra sólo la generación derivada anterior de ese job, nunca el original. No se promete historial de todas las extracciones.

**Alternativas:** normalizar saltos/BOM o interpretar Markdown complica offsets y modifica texto; fragmentar sólo por líneas permite memoria/respuestas sin límite; indexar todo en FTS ahora cambia la búsqueda antes de contar con parsers/procedencia estabilizados.

### 6. Lecturas verificables, cursores y backpressure

Get/list/control-get usan snapshot de lectura y no reparan estados. Fragments fija job/generación en una lectura, verifica el original con el mismo límite interno antes de retornar texto y devuelve antigüedad/origen del resultado. Si faltó, cambió o es inseguro, retorna storage_error con reason=input_stale sin presentar texto como vigente ni modificar el job; un run que detecta fuente inválida transiciona a stale si aún tiene ownership. Configuración de resultado no soportada impide lectura textual certificada, conservando metadatos crudos. Fallos generales de lectura/persistencia no se disfrazan de fuente alterada.

Cursores nuevos de JSON v1, máximo 2 KiB, estrictos y scoped a workspace/ruta/operación/filtros/límite. List usa keyset created_at/ID y no promete snapshot global frente a escrituras. Fragments usa job/generación y siguiente ordinal; una nueva publicación invalida el cursor anterior. Campo covers_generation sólo es true si la llamada empezó sin cursor y devolvió todos los fragmentos, incluso el caso vacío; una última página no certifica las omitidas. La cobertura completa de la **generación** se distingue de cobertura de la **respuesta**. Rechazar cursores ajenos y los de entity/home/database. No son firmas ni barrera de seguridad.

| Presupuesto | Política propuesta |
| --- | --- |
| Original extraíble | <=1 MiB; importación sigue <=64 MiB. Verificar metadata y lectura límite+1; no truncar un éxito. |
| Pendientes / total | <=1.000 queued+running y <=10.000 jobs por workspace; enqueue/retry calculan admisión en la misma Immediate. Dedupe no consume cupo nuevo. |
| Fragmentos por generación | <=512, cada uno <=4096 bytes UTF-8. |
| Texto derivado retenido | <=32 MiB de bytes UTF-8, suma de generaciones publicadas. En reemplazo descontar la anterior del mismo job antes de admitir nueva; verificar concurrentemente en publicación. |
| Páginas | Jobs 1–100/default50; fragmentos 1–64/default32. |
| Transporte | Entrada/respuesta completas <=2 MiB; límite antes de commit donde hay escritura, después de serializar donde hay lectura. Escapes JSON cuentan. |
| Contención / ejecución / lease | 2.000 ms / cooperativo 5 s / 15 s. |

32 MiB es texto lógico, **no** tamaño físico SQLite: índices, JSON, páginas libres y originales tienen otros costos. No hacer VACUUM, purgas ni elevación de 64 MiB de snapshot como efecto secundario. Al saturarse total/derivados se rechaza explícitamente; limpieza selectiva y políticas de retención requieren otro cambio. Esta primera versión no puede afirmar soporte de corpus ilimitado ni recuperación de espacio tras cada retry.

### 7. Lotes: registro completo, huella original y snapshot intacto

Separar en batch_storage la lista de tablas reconocidas de la lista de tablas cuyo **contenido** es original. Schema esperado se construye con migraciones 001–008. Tablas/virtual/shadows/triggers siguen cerrados; conservar política de índices ordinarios adicionales sólo sobre tablas originales conocidas. No habilitar SQL arbitrario para acomodar una cola.

Huella original: las trece tablas anteriores + extraction_control (incluye revisión/version). Excluir contenido de runtime/jobs/generations/fragments, igual que FTS derivado; incluir **sus definiciones SQL** en el schema hash. Usar namespace `noteka.batch.base.v2` para explicitar la nueva semántica y documentar en catálogo/guía. Shape de recibo/plan y efecto siguen v1; recibos previos a migración no son reaprovechables y exigen un preview nuevo. IDs deterministas/desempates de lotes no cambian.

Encolar/cancelar/reintentar/publicar no vuelve obsoleto un recibo por actividad operativa; cambiar control sí, incluso pausa→reanudar al valor original porque la revisión cambió. No confundir config del job inmutable/operativa con configuración original de cola. Preview incluye todas las tablas en copia/tamaño sin invocar worker ni recuperar leases. Apply reejecuta en el original bajo su propia Immediate y no publica la copia: conserva actividad ajena posterior al preview. Un exceso físico de snapshot o esquema ajeno sigue fallando aunque el original de dominio no cambió.

**Alternativas:** incluir todos los jobs en base_hash causa conflictos por cada extracción; excluir también control permite aplicar sobre configuración original no examinada; copiar sólo tablas originales reduce el presupuesto engañosamente y no es el snapshot definido en fase 4.

### 8. Verificación y fuentes

Integración sobre SQLite real y procesos CLI independientes. Barreras/test clocks/failpoints sólo bajo cfg(test) o harness de tests: detener antes de publicar, editar desde otra conexión, pausar/reanudar, cancelar/retry, vencer ownership y provocar error antes del commit. Evitar sleeps como evidencia primaria, mocks de persistencia o retries que oculten carreras. Probar todos los escenarios y caps en frontera/exceso, incluidos JSON escapable, vacíos y Unicode.

Medir release con fixtures sintéticas reproducibles: 1 MiB ASCII/multibyte/control escapable, cola de 1.000 pendientes/10.000 jobs y corpus derivado cercano a 32 MiB. Registrar lectura/hash/fragmentación/claim/publicación/lock hold, respuesta, RSS del CLI y tamaño físico SQLite; al menos cinco warmups y veinte repeticiones por caso de latencia. Objetivo orientativo p95 de cada lock corto <150 ms en M1 Max, no promesa ni aprobación antes de medir. Si no se cumple, registrar causa y resolver/documentar alcance antes del cierre, sin debilitar límites ni tests.

Regresión: core/CLI offline, suscripción real, fmt/Clippy, contratos, adaptador/cola/tema, puente, check/build, release y Tauri build --no-bundle **sin iniciar el host**. UI existente sólo headless. Estos checks no verifican aceptación nativa de Home; no marcar sus tareas.

Fuentes oficiales consultadas el 13/10/2026: [aislamiento de SQLite](https://www.sqlite.org/isolation.html), [transacciones rusqlite](https://docs.rs/rusqlite/0.38.0/rusqlite/struct.Transaction.html) y [UTF-8 estricto en Rust](https://doc.rust-lang.org/std/str/fn.from_utf8.html). La consulta Context7 confirma Immediate/rollback al abandonar y cambios afectados para CAS; no sustituye los tests sobre rusqlite 0.38 del proyecto. No hace falta seleccionar parser PDF, OCR, runtime o proveedor para este cambio.

## Risks / Trade-offs

- [Worker muerto queda running hasta recuperación] → disponibilidad calculada visible, lease, cancel con revisión y run explícito; no afirmar detección instantánea de crash.
- [Publicación tardía o ABA] → token/revisión/scope/control revalidados en commit y tests de pausas/cancelación/reclaim intercalados.
- [Modificación externa de original justo después del hash] → hash del buffer extraído + nueva validación antes de publicar/leer; originales administrados son inmutables por contrato. No prometer transacción filesystem/SQLite ni aislamiento contra un proceso con acceso arbitrario a esos archivos.
- [Lectura/hash final bajo lock excede objetivo en disco lento] → máximo 1 MiB, presupuesto cooperativo y medición específica; la extracción extensa permanece fuera del lock.
- [Backpressure sin comando de limpieza] → saturación explícita; retención/purga posterior, nunca borrar datos de usuario para que pasen tests o lotes.
- [Derivados rompen batches por tamaño físico] → mantener snapshot 64 MiB, medir overhead y advertir; excluir del hash no equivale a excluir del snapshot.
- [Configuración futura o fixtures anteriores incompatibles] → JSON conservado, soportes explícitos, tests de bytes y migraciones; actualizar downgrades eliminando sólo tablas v8 de fixtures sintéticas.
- [Primer UTF-8 confundido con 5a completa] → PDF/Excel/OCR e indexación permanecen en el plan posterior; no cerrar 5a ni desbloquear por nombre una dependencia no cumplida de 5b.

## Migration Plan

1. Añadir 008/SCHEMA_VERSION=8 en creación y migración, schema_check por versión y registro batch; no editar migraciones 001–007 ni formatos de documentos/adjuntos.
2. Workspace open explícito aplica backup previo consistente y migración transaccional, inicializando control pausado/runtime vacío sin jobs. Tests v0–v7 conservan ruta soportada a v8; fallo inyectado conserva íntegro v7/backup.
3. Backup sigue usando manifiesto v2 para v8: los datos nuevos están en SQLite, sin archivos/credenciales adicionales. Validar las cinco tablas/columnas requeridas en v8 y consistencia de job/generación/ownership, además de hashes/originales. Versiones anteriores no requieren tablas futuras. Configuración futura conservable no invalida por sí sola backup.
4. Restore conserva IDs/revisiones/control y derivados confirmados. No reescribe tokens o jobs al abrir; su scope de otra ruta impide reconocerlos como activos. Sólo acción explícita recupera estado operativo en la copia. Backup fuente y workspace origen no se modifican.
5. Rollback de software no hace downgrade destructivo: un cliente v7 rechaza v8. Para recuperar versión previa se restaura el backup pre-migración en ruta nueva usando software compatible; ese snapshot no contiene escrituras posteriores, que nunca se descartan automáticamente.
6. Publicar guía/ADR 0011 y evidencia **al implementar**. Sincronizar sólo estos deltas después de completar/verificar tareas; no sincronizar/archivar Home ni declarar implementado este diseño porque valide OpenSpec.

## Open Questions

Ninguna bloqueante para este recorte. Sólo quedan decisiones de cambios posteriores: parsers PDF/Excel/OCR/licencias, indexación de adjuntos, limpieza/retención, scheduler residente y recuperación semántica. No exigen acceso remoto, permisos de pantalla ni descargas para ejecutar las tareas de esta propuesta.
