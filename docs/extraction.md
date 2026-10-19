# Extracción local de adjuntos

Estado vigente: [búsqueda XLSX](xlsx-search.md) entregada en v12/API v1/52;
los cierres v8/v11 de abajo son etapas históricas. Extraer nunca selecciona para búsqueda.

Cambio [`add-local-extraction-queue`](../openspec/changes/archive/2026-10-13-add-local-extraction-queue/proposal.md) archivado, **20/20 tareas**, baseline histórico SQLite v8/API v1/48 operaciones. [Evidencia histórica](verification-extraction.md), [ADR 0011](decisions/0011-explicit-local-extraction.md). Se añadieron después [PDF textual acotado](pdf-extraction.md), [índice explícito UTF-8/PDF](attachment-search.md) y [MCP local](mcp.md), sin UI nueva ni worker automático. La [integración XLSX verificada](xlsx-extraction.md) evoluciona el código a v11/52; [aceptación](verification-xlsx-extraction.md). PDF amplio/Excel genérico/OCR, embeddings/modelos/harness/remotos y resto de 5a siguen pendientes.

## Contrato y límites

Selección explícita de attachment_id y extractor `utf8_text`; configuración v1 `{version:1,chunk_bytes:4096}`. El encolado no ejecuta. Fuentes administradas `.txt`/`.md` de hasta 1 MiB, UTF-8 estricto sin NUL; ningún modelo, red, HTML/Markdown interpretado ni comando del contenido.

DTOs de entrada niegan campos desconocidos. Control usa `{action:"get"}` sin escritura o `{action:"set",paused,expected_revision}`. Cancel/retry requieren `{id,expected_revision}`. Run admite `{id?}`. Jobs tienen revisión, intentos, fuente/hash, configuración versionada, timestamps, estado/error y disponibilidad de ownership; los tokens permanecen internos. Estados queued/running/succeeded/failed/cancelled/stale.

Jobs: páginas 1–100/default 50. Fragmentos: páginas 1–64/default 32, cada fragmento <=4096 bytes; máximo 512/generación. Entrada/respuesta completas <=2 MiB; cursores JSON v1 <=2 KiB ligados a workspace/ruta/filtros/límite y a generación para fragmentos. Revisión obsoleta nunca se rebasa. Máximo 1.000 queued/running, 10.000 jobs y 32 MiB de texto derivado retenido, sin purga automática. Errores no incluyen contenido íntegro ni token.

Presupuesto cooperativo 5 s, lease 15 s, contención 2.000 ms; no se promete tiempo real estricto frente a syscalls. Preflight/apertura, espera inicial y cleanup separado no son una garantía wall-clock de 5 s. Home y aceptación nativa permanecen separados.

## Operaciones y transiciones

| Operación (`noteka extraction <acción>`) | Input / efecto |
| --- | --- |
| enqueue | `{attachment_id,extractor:"utf8_text",config?}` → `{job,deduplicated}`. Dedupe antes de caps por adjunto/hash/extractor/version/config canónica, también terminal y concurrente. Otro adjunto con mismos bytes tiene otro job. No valida UTF-8 hasta run. |
| get | `{id}` → job sin texto/token; snapshot de lectura, soporte, ownership_active/recoverable calculados. |
| list | `{attachment_id?,state?,limit?,cursor?}` → `{items,next_cursor}`; keyset created_at/ID asc, snapshot por llamada, no snapshot global entre páginas. |
| run | `{id?}` → idle/succeeded/interrupted, reason/job/recovered_count/incompatible_count. Un queued compatible por llamada; sin ID omite incompatibles y cuenta. ID explícito exige existencia/soporte/estado válido aun pausado. Pausada/vacía/sin compatibles devuelve idle. Worker activo: workspace_busy. |
| cancel | `{id,expected_revision}`; queued/running → cancelled. Cancelled vigente no-op. Terminal restante invalid_state. |
| retry | `{id,expected_revision}`; failed/cancelled/stale/succeeded → queued tras cap, sin attempts ni borrar generación. queued/running invalid_state; futuro incompatible se rechaza. |
| control | `{action:"get"}` o `{action:"set",paused,expected_revision}`. Valor idéntico vigente no-op sin recuperación escondida; revisión vieja falla. Cambiar pausa aumenta revisión. Resume no ejecuta. |
| fragments | `{id,limit?,cursor?}` → generación/procedencia/cobertura, página literal y covers_generation/next_cursor/result_from_previous_attempt. Revalida original, no modifica job. |
| pdf-fragments | `{id,limit?,cursor?}` → manifest completo y página de fragmentos PDF; rangos del texto derivado por página, nunca bytes del original. |

Todos `batch_mutation=false/batch_check=false`: no se incorporan a las nueve mutaciones/un check existentes. Importar/abrir/consultar no encola ni ejecuta. Jobs no cambian notas, revisiones de entidad, referencias, FTS ni historial. No se atribuye una nota ficticia a un fragmento cuando se retira su referencia.

Claim confirmado: queued→running, revision+1/attempts+1/token privado. Lectura/hash y fragmentación fuera del lock de escritura; otras conexiones pueden editar. Publicación relee fuente/hash/config/token/revisiones/scope/lease, confirma fragmentos/generación/succeeded/revision+1 y libera reserva en una transacción. Respuesta limitada antes del commit, vencimiento/deadline justo antes de confirmar.

Pausa running vigente de esta ruta → queued/revision+1 e invalida ownership, conservando generación/attempts. Ownership vencido/ajeno → failed, no queued; requiere retry. Cancel invalida token. Un worker viejo jamás pisa pausa/cancel/reclaim o una generación nueva. Fallos tras claim no revierten su intento confirmado; `details` informa job_id/state/revision/attempt. Si no pudo persistir cleanup: `cleanup_pending:true`, no se afirma failed confirmado. Proceso muerto queda running hasta acción explícita; open/get/list no recuperan. Lease calculada inactiva queda visible como recoverable.

## Localizadores, lectura y límites

Extensión case-insensitive; MIME conocido de PDF/imagen u otro medio no textual se rechaza, application/octet-stream permitido. Archivo/directorio administrado regular sin symlinks, metadata/hash exactos, lectura hasta límite+1. UTF-8 estricto/no NUL, sin trim/normalización/HTML/Markdown ejecutado. BOM/CRLF se conservan; se permite frontera entre CR/LF. Líneas cuentan LF: `[byte_start,byte_end)` desde 0 y line_start/end inclusivos desde 1 (último byte del fragmento).

Concatenar ordinales reconstruye bytes exactos. Vacío → succeeded/complete con 0 fragmentos. `covered_bytes=source_bytes`, complete describe generación; covers_generation sólo si llamada empezó sin cursor y devolvió todo (incluido vacío). Última página no acredita anteriores. Una nueva generación invalida cursor viejo; restore/ruta/job/filtros/límite/operación ajenos también. Cursores JSON estrictos no son permisos ni firmas.

Una generación por job; retry conserva anterior con result_from_previous_attempt=true. Se descuenta la anterior del cap al reemplazar; falla conserva resultado completo. Fragments valida original antes de devolver texto; fuente alterada/ausente/insegura devuelve storage_error/input_stale sin mutar, run con ownership pasa a stale. Resultado config futura conserva metadata pero no certifica lectura textual.

Errores: forma/config/codificación/caps → validation_error (2); ID/generación ausente → not_found (3); revisión → revision_conflict (4); worker/lock/deadline → workspace_busy (4); IO/fuente/persistencia → storage_error (5); SQLite futuro → unsupported_workspace_version (6). `details.reason` distingue invalid_utf8, nul_in_text, unsupported_source_format, source_size_limit, pending_count_limit, job_count_limit, derived_size_limit, cursor_scope_mismatch, input_stale, worker_interrupted, scope_mismatch, etc. Config cruda <=8 KiB; error <=4 KiB/mensaje <=512 caracteres. Sin contenido íntegro/token/stacktrace.

32 MiB es texto lógico, no archivo SQLite: índices, overhead/páginas libres cuentan además para snapshot batch de 64 MiB. Saturación rechaza, sin purga/VACUUM/elevación automática. [Batches](batches.md) vinculan control original con base.v2; actividad operativa/derivada no vence recibo, schema/tamaño completos siguen validados.

## Migración, backup y restore

Open explícito migra versiones anteriores a la actual con backup previo y transacción atómica; una cola nueva empieza pausada/vacía y una existente conserva control/jobs/ownership/configuración/resultados sin recuperación implícita. La integración XLSX añade sólo 011 a 001–010, sin reescribir formatos/IDs/revisiones originales; falla conserva la versión anterior y las futuras se rechazan sin repair. Backup manifiesto v2 incluye SQLite completo/originales verificados y consistencia de cada extractor según versión, sin parser. Backups anteriores v1/v2 siguen restaurables; apertura compatible migra sin ejecutar. [Recuperación v11 y límite de compatibilidad con v10](xlsx-extraction.md#persistencia-presupuestos-y-recuperación).

Restore en destino nuevo conserva control/jobs/revisiones/resultados/config futura. Ownership registrado de origen se reporta inactivo por scope distinto, sin writes en lecturas; sólo run/control/cancel explícito recupera. Fuente sigue independiente. Rollback de software restaura backup pre-migración en otra ruta: no incluye escrituras posteriores, nunca se descartan automáticamente.

## Ejemplo reproducible offline

Construir CLI con [entorno](development.md), luego:

```sh
TEMP_ROOT="${NOTEKA_TEST_TMP:-${TMPDIR}}"
BASE=$(mktemp -d "${TEMP_ROOT%/}/noteka-extraction-XXXXXX")
sandbox-exec -p '(version 1)(allow default)(deny network*)' \
  node scripts/extraction-fixture.mjs "$BASE/workspace"
# Release: export NOTEKA_CLI=target/release/noteka y repetir con destino nuevo.
```

Fixture sintética prueba import→enqueue→habilitar→run→páginas byte-exactas→retry→cancel sin entidades/historial ni ejecución de URLs/comandos. `noteka --json operations describe` publica shapes/errores/defaults/caps. stdout un envelope JSON por llamada, stderr separado; --input y stdin como operaciones anteriores. Ejecución sin ventana/modelos/red, no autorización remota.

## Escritorio al 17 de octubre de 2026

La [preparación explícita](attachment-preparation-ui.md) permite seleccionar un
adjunto administrado, consultar y ejecutar extracción y seleccionar generación
para Search, con las revisiones del núcleo. El [lector de generaciones](attachment-reader-ui.md)
recorre todo el derivado por páginas sin exigir indexación. La
[consulta agrupada](grouped-search.md) usa ambos motores sin fusionar cursores o
scores. [Aceptación técnica conjunta](verification-local-preparation-closure.md).
Los límites de formato/capacidad del núcleo no se amplían con estas UIs; abrir o
buscar no prepara ni repara. Los hitos históricos anteriores conservan su alcance.
