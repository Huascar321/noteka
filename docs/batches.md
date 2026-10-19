# Cambios agrupados locales

`changes.preview` y `changes.apply` están implementados en Rust/CLI; **fase 4 verificada y [archivada](../openspec/changes/archive/2026-10-13-extend-agent-operations/proposal.md)**, 22/22. [Evidencia histórica v7/40](verification-phase-4.md). Último baseline archivado: SQLite v10/API v1/51 del [índice de adjuntos](attachment-search.md), **19/19 tareas verificadas**, tres deltas sincronizados y [cambio cerrado](../openspec/changes/archive/2026-10-14-add-local-attachment-search/proposal.md); regresión/revisión, target query y scratch físico comprobados. El cierre previo [PDF v9/49](verification-pdf-extraction.md) conserva su evidencia. Se conservan huella original v2 y plan/receipt v1. Sin ventana/cuenta/modelo/OAuth/red, MCP ni UI de aprobación. Home nativo sigue pendiente.

## Contrato y límites

```text
BatchPlan { version: 1, batch_id: UUID canónico, steps: [BatchStep], checks: [BatchCheck] }
BatchStep { key, operation, input: JSON previo a bindings, bindings: [Binding] }
Binding { kind: value | property_key, target: JSON Pointer, source: {step, export} }
BatchCheck { key, input: JSON de DatabaseQuery, bindings: [Binding],
             require_complete: true, require_no_calculation_errors: true }
BatchReceipt { version: 1, workspace_id, path_hash, plan_hash, base_hash, effect_hash }
changes.apply { plan, expected: BatchReceipt }
```

Todos los DTO de entrada rechazan campos desconocidos. `input` de cada paso no es un DTO válido hasta resolver sus placeholders; después se deserializa por el mismo DTO individual. El catálogo (`noteka --json operations describe`) distingue este input de `resolved_input_schema` y publica capacidades, exports, registry y límites. No acepta SQL, shell, operaciones de lote recursivas ni JSON Patch general.

| Límite | Valor |
| --- | --- |
| Steps / checks | 1–100 / 0–10 |
| Bindings totales | 512 |
| Key / pointer | 64 caracteres ASCII / 512 bytes |
| Entrada y respuesta completas | 2 MiB cada una, incluyendo envolturas |
| Snapshot SQLite | 64 MiB (`page_count * page_size`, incluye páginas libres) |
| Contención / ejecución | 2.000 ms / presupuesto cooperativo de 5 s |

Keys únicas entre pasos/checks: letra inicial ASCII seguida por letras, números, `-` o `_`. Versión 1 y UUID en minúsculas con guiones. Flags de check omitidos significan **true**, no aceptación parcial. El límite de respuesta se verifica antes del commit; no se trunca para aprobar.

## Mutaciones y exports

| Operación agrupable | Exports |
| --- | --- |
| entity.create / entity.update | entity_id, entity_revision |
| collection.create | collection_id, collection_revision, title_property_id |
| property.define / formula.set | property_id, collection_id, collection_revision |
| record.create / record.attach / record.update | entity_id, entity_revision, collection_id |
| view.save | view_id, view_revision, collection_id |

Sólo estas nueve mutaciones son pasos. `database.query` es check final, nunca intercalado. Las otras 39 operaciones no son pasos ni checks: archivos/workspace, importación, backups, exportación, reconstrucción FTS, archivo/restauración, relaciones built-in, templates y las nueve extraction.*. `property.define` obtiene revisión en la misma conexión; identidad de fórmula por ID existente o nombre validado, no posición del bundle.

`value` modifica únicamente un placeholder presente en un slot semántico de ID/revisión. Se admiten collection/entity/view y revisión correspondiente; extremos de contexto/membresía; propiedad destino de relación; IDs del AST reconocido (`property`, `rollup`, `allocate` con recorridos de `binary/not/if`); columnas, filtros, sorts, grupos y calendario de config; referencias collection_view en bloques/children reconocidos. Un array relation admite IDs de entidades sólo tras comprobar el tipo de su propiedad en SQLite. `title_property_id` sirve en un slot de propiedad; una revisión no sirve como ID ni se intercambian IDs de distintos dominios.

No se permiten bindings sobre título, texto, literal `text` del AST, moneda, payload arbitrario, attrs desconocidos ni strings que parezcan aliases. Un adjunto existente se referencia por su ID literal: esta whitelist no produce exports de adjuntos.

`property_key` renombra una clave en `/values/<placeholder>` o `/record/values/<placeholder>` y exige `property_id`. Primero se sustituyen todos los `value` sobre el input original; después se renombran claves. El tipo de un array relation se consulta mediante la identidad declarada en el binding de clave, incluso antes de renombrarla. Se rechazan claves destino existentes, renombres repetidos, bindings duplicados/solapados, fuentes futuras/checks, exports ausentes, rutas inexistentes y tipos incompatibles. Escapes RFC 6901: `~0` significa `~`, `~1` significa `/`; índices de array canónicos.

Ejemplo de property.define dentro de un plan:

```json
{"key":"cost","operation":"property.define",
 "input":{"collection_id":null,"expected_revision":null,
          "spec":{"name":"Costo","type":"money","currency":"BOB"}},
 "bindings":[
   {"kind":"value","target":"/collection_id","source":{"step":"subscriptions","export":"collection_id"}},
   {"kind":"value","target":"/expected_revision","source":{"step":"subscriptions","export":"collection_revision"}}
 ]}
```

El ejemplo ejecutable completo vive en `scripts/subscription-plan.mjs`; no contiene reglas monetarias del núcleo ni IDs de datos personales.

## Ownership transaccional

`MutationContext` se presta a helpers junto con una conexión y el root del workspace. Transporta origen/request, fuente de IDs, reloj y observación de efectos; no abre ni confirma transacciones ni usa estado global. Los wrappers individuales siguen siendo dueños de su transacción Immediate.

`entities::create_in_transaction` continúa siendo la única creación de entidad/documento/contexto/membresía/historial/FTS, compartida con registros, captura y templates. `update_in_transaction` conserva revisión esperada, patch conservador de payload, protección de documento no editable y reindexación. Cuando el payload se omite, conserva incluso su representación textual exacta: editar título/documento no lo reserializa. Los modos individuales generan IDs aleatorios como antes. Plantillas y relaciones individuales no se convierten en pasos de lote por adaptar su contexto.

El contexto de lote permite observar claves antes de escribir; una transacción abandonada revierte creación, actualización, relaciones, historial e índice. La prueba `borrowed_entity_helpers_share_rollback_and_preserve_unknown_documents` verifica esta frontera sobre SQLite real, incluido documento futuro no reemplazado. Es evidencia de ownership interno; las pruebas de preview/apply/CLI y recuperación están separadas en [verificación](verification-phase-4.md).

Colecciones y propiedades exponen helpers prestados; `formulas::set_in_transaction` conserva inferencia, revisión y grafo. `record_create_in_transaction` reutiliza la creación única, `record_update_in_transaction` une pertenencia/patch/revisión/historial y `view_save_in_transaction` conserva validación y revisión de vistas. `query_in_connection` usa la conexión del llamador; el wrapper individual todavía establece su snapshot. Ninguno confirma ni abre transacciones anidadas. Los cambios de esquema no inventan historial persistente: `changes` continúa referenciando solamente entidades.

`borrowed_schema_record_view_and_query_helpers_never_commit` encadena esquema, cálculo, registro, incorporación de una página original, actualización y vista dentro de una transacción real; consulta los cálculos, introduce un ciclo inválido y verifica rollback completo al abandonar la transacción. Las regresiones individuales de database, formulas, advanced_views y templates siguen comprobando reglas/errores anteriores.

## Snapshot y confirmación

Changes tiene apertura separada de `Workspace::open`: workspace existente v10, sin crear/migrar/cambiar journal_mode. V0–v9 requieren open explícito y preview nuevo; futuras/extensiones SQL no admitidas fallan sin reparación. Registry incluye TODAS las tablas de 001–010, también operativas/derivadas, PDF e índice de adjuntos con shadows/trigger; índices ordinarios adicionales sólo sobre originales conocidos. JSON futuros no se confunden con extensiones SQL ni se reemplazan.

`extraction.index/search` no son pasos ni checks. Put/reset/invalidation no cambian base.v2: su control/entries/FTS son derivados, pero se incluyen completos en snapshots. La migración v10 sí cambia schema/huella y vence recibos v9. Un preview→put/reset→apply conserva el índice confirmado actual porque apply nunca reemplaza el workspace por su copia. Los recibos siguen siendo comparación de estado, no permisos, firmas ni idempotencia persistente.

Preview usa READ_ONLY/query_only, inicia una lectura y fija identidad/versión antes del tamaño/Backup. SQLite Backup copia ese snapshot, incluido WAL confirmado, a una conexión **distinta en memoria** con page_size igual al original. Avanza por 256 páginas con deadline; la fuente se libera y la copia se descarta siempre. No hay mutaciones de dominio/base/WAL, migración, backups publicados ni importación de originales por preview. SQLite puede coordinar/crear SHM al leer WAL: no se promete inmutabilidad física de ese archivo. La prevalidación privada no es el snapshot definitivo ni el mecanismo de publicación. Si WAL desaparece durante esa copia preliminar porque cerró la última conexión y hubo checkpoint, se vuelve a leer la base checkpointada en privado; no se ignoran otros errores de IO ni se repiten escrituras.

Apply reejecuta helpers sobre el original bajo una única Immediate; nunca promueve la copia. Dentro del lock vuelve a comprobar esquema, identidad, tamaño y `base_hash`. Ejecuta pasos/checks, compara `effect_hash`, serializa/limita la respuesta y comprueba el presupuesto antes de un solo commit. Fallos revierten esquema, entidades/documentos/contexto/membresías/valores/vistas/cálculos, revisiones, FTS e historial. No reintenta, rebasa ni refresca revisiones automáticamente.

## Huellas, diff e identidad

Snapshot incluye jobs/runtime/generaciones/fragmentos y páginas libres en el cap de 64 MiB. Preview no ejecuta ni recupera workers vencidos/futuros; apply conserva actividad posterior al reejecutar en original. Sin purga ni elevación de límites. [ADR 0011](decisions/0011-explicit-local-extraction.md).

- `plan_hash`: SHA-256 con namespace `noteka.batch.plan.v1`, JSON interno canónico: objetos ordenados, arrays preservados y números serializados por serde_json. Incluye batch_id, bindings, flags y pasos, no whitespace del archivo.
- `path_hash`: namespace `noteka.batch.path.v1` y bytes de ruta canónica; `/var` y `/private/var` equivalentes funcionan, un restore en otro destino no comparte scope.
- `base_hash`: namespace `noteka.batch.base.v2`; schema ordenado sin rootpage y filas de workspace_meta, entities, documents, relations, changes, attachments, collections, properties, memberships, property_values, views, calculations, templates y extraction_control. Orden PK BINARY, tipos/longitudes delimitados; INTEGER i64 BE, REAL bits IEEE-754, NULL distinto de vacío, TEXT/BLOB exactos. JSON persistente no se parsea. Se excluye contenido de FTS/shadows y extraction_runtime/jobs/generations/fragments, no definiciones SQL. Enqueue/run/cancel/retry no vencen recibos por actividad derivada; control sí, también pausa→resume por revisión. Shape de recibo/plan/efecto sigue v1; recibos anteriores a migrar requieren preview nuevo.
- Diff consolida claves observadas antes/después: entity/document/relation/collection/property/membership/value/calculation/view. Presenta JSON reconocido como datos de dominio; formatos no reconocidos quedan crudos. No es SQL ejecutable ni copia de adjuntos. `history_summary` separado sólo refleja historial de entidades; no inventa historial persistente de esquema.
- `effect_hash`: namespace `noteka.batch.effect.v1`; diff, resultados, exports, checks e historial resumido. Sólo normaliza created_at/updated_at propios de Entity y valores de propiedades tipadas CreatedAt/UpdatedAt, incluidos grupos de esas propiedades. Historial resumido no incluye ID/request/origen/fecha. No oculta claves iguales dentro del payload/texto, fechas civiles, cálculos/errores, IDs de dominio, revisiones ni cobertura/orden. Una divergencia en una consulta causada por el reloj aborta y exige otro preview.

Cada ID nuevo en modo lote deriva de SHA-256 sobre cinco campos prefijados por su longitud u64 BE: `noteka.batch.ids.v1`, UUID batch_id (16 bytes), key del paso (UTF-8), rol y ordinal u64 BE. Se toman 16 bytes, bits de versión 8 y variante RFC. Roles entity/collection/property/relation/view/change; ordinal reiniciado por paso/rol. Vector: UUID nil, step `fixed`, rol `entity`, ordinal 0 → `30a8fcdc-5ad0-8c9c-beb8-2402e368f9e4`. Los IDs existentes y los IDs de bloques enviados por el cliente no se recalculan. Colisiones reservadas abortan, incluso antes de upsert de fórmula/vista.

Preview etiqueta su reloj como `evaluated_at`; apply usa su propio `executed_at`. No se acepta reloj del cliente ni se promete instante exacto del commit físico. IDs/centavos/referencias permanecen iguales; timestamps pueden diferir. Los wrappers individuales siguen generando IDs aleatorios.

**Los hashes no son permisos, firma, secretos ni aprobación humana.** La autorización local vive fuera del recibo. Tampoco son idempotencia persistente: reaplicar un recibo después de una mutación efectiva falla por base_hash. Para reconciliar una respuesta perdida, consultá los IDs publicados en preview; no hagas retries ciegos. No hay undo automático ni reservas/auditoría/tablas batch nuevas. Una actualización sin cambios de contenido conserva su bump de revisión anterior; no se inventa un no-op de dominio.

## Checks y errores

Checks sólo consultan el estado **final**. Complete exige cursor de entrada ausente, next_cursor ausente y matching_count igual a cantidad de filas. Una última página no acredita las filas salteadas. `require_no_calculation_errors` cubre esas filas/filtro, no todos los cálculos del workspace; cero checks no certifica ausencia global. Desactivar el flag no evita errores de filtro/sort que la consulta individual rechaza.

CoreError/exit codes vigentes: forma/bindings/límites/recibo incompatible → validation_error (2); scope/plan incompatible → validation_error; base/efecto/revisión/colisión reservada → revision_conflict (4); lock/backup/presupuesto → workspace_busy (4); almacenamiento → storage_error (5); formato de workspace/documento → errores vigentes (6). Details agregan phase, step_key/index, operation, binding_index y reason cuando corresponde, sin el input completo ni stacktrace. Un fallo no publica éxitos aparentes como aplicados ni un recibo aplicable.

Progress handler SQL (`rusqlite/hooks`) se instala con guardia y se retira al salir, también al desenrollar un pánico controlado. El presupuesto comprueba fases, pasos, filas y antes del commit; SQLITE_INTERRUPT causado por deadline se diferencia de otro fallo SQL. Es cooperativo, **no tiempo real estricto** frente a IO/OS; no suma preflight/espera del lock como una sola duración. Una referencia a un original valida existencia/hash en apply, sin prometer atomicidad contra cambios externos al filesystem.

## CLI real, sin ventana

```sh
noteka --json operations describe
noteka --json --workspace "$WORKSPACE" changes preview --input "$PLAN"
# Aplicar {"plan": <mismo plan>, "expected": <receipt exacto de preview>}:
noteka --json --workspace "$WORKSPACE" changes apply --input "$APPLY"
# stdin funciona también; --input - significa stdin explícito.
```

Construcción sintética probada con red denegada:

```sh
TEMP_ROOT="${NOTEKA_TEST_TMP:-${TMPDIR}}"
BASE=$(mktemp -d "${TEMP_ROOT%/}/noteka-subscription-XXXXXX")
sandbox-exec -p '(version 1)(allow default)(deny network*)' \
  node scripts/subscription-fixture.mjs "$BASE/workspace"
```

`workspace.create` es una operación individual previa: un fallo puede dejar workspace vacío, nunca colecciones/página parciales del lote. El script arma 16 pasos (2 colecciones + 5 propiedades + fórmula + 6 registros + vista + página v3), evalúa BigInt antes de aplicar el **mismo** plan/recibo y consulta resultados confirmados. Reparto: 14.569 centavos BOB = 48,57 + 4 × 24,28, seis asientos y cinco miembros. La página collection_view referencia IDs originales, sin materializar copias. Pruebas `npm run test:batches` verifican costo actualizado a 150 BOB, pago de un miembro, Pendientes con cuatro filas, reapertura y backup/restore.

Es una política sintética del allocate existente, no un sistema financiero autorizado: no infiere cargos por capacidad vacante, períodos, vencimientos ni sobreasignación. [ADR 0010](decisions/0010-atomic-local-change-plans.md), [verificación de fase 4](verification-phase-4.md). Home conserva sus ocho brechas nativas: estos recorridos de CLI no las certifican.
