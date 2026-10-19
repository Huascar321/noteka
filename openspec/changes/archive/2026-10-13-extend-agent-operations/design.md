# Design

## Context

Motivación y exclusiones en [proposal.md](proposal.md); delta del cambio en [core-operations](specs/core-operations/spec.md). Baseline inspeccionado antes de implementar, al 13/10/2026: **38 operaciones, SQLite v7, sin preview/apply ni lotes**. Las decisiones de este diseño ya están implementadas y verificadas: **40 operaciones y 22/22 tareas**, sin migración; evidencia en `docs/verification-phase-4.md` y ADR 0010. Las referencias de la tabla siguiente corresponden al baseline, no a líneas actuales. Home permanece independiente, implementado con 9/17 tareas verificadas y ocho aceptaciones nativas pendientes.

Fronteras comprobadas:

| Evidencia actual | Consecuencia para este cambio |
| --- | --- |
| `crates/noteka-core/src/lib.rs:26–82,105–199`: catálogo, límites, deserialización y despacho; apertura de Workspace por solicitud | Añadir dos operaciones sin duplicar validación en CLI/Tauri. Separar su apertura del dispatcher que migra. |
| `contracts.rs:357–459` y `crates/noteka-cli/src/main.rs:182–269`: enum público y mapeo de todos los subcomandos; stdin/archivo limitado a 2 MiB | La superficie individual ya existe; «CLI completo» significa verificar paridad y agregar preview/apply, no inventar herramientas de fases futuras. |
| `workspace.rs:44–156,188–261`: configuración WAL, preflight privado y migración automática; incluso abrir sólo lectura puede coordinar SHM | No usar `Workspace::open` para un preview ni migrar como efecto oculto de apply. No prometer inmutabilidad física del SHM. |
| `workspace.rs:263–335`: Backup/StepResult con deadline de 2.000 ms | Reutilizar la técnica de copia coherente, no el método de backup que publica archivos y adjuntos. |
| `entities.rs:175–262`: `create_in_transaction` ya une entidad/documento/contexto/membresía/historial/FTS | Precedente de helper que recibe conexión y deja commit al llamador; ampliar sin sustituir sus validaciones. |
| `entities.rs:264–359`, `database.rs:557–639,674–713,747–904`, `formulas.rs:359–448`: wrappers propietarios de transacciones individuales | Extraer cuerpos para las nueve mutaciones y la consulta final; nunca invocar wrappers con transacción anidada desde el lote. |
| `database_contracts.rs:68–103,169–235`, `formula_contracts.rs`, `documents.rs:45–69`: IDs en claves de values, AST, configuración y attrs de bases incrustadas | Bindings explícitos y tipados por slot; no reemplazos globales de strings ni un segundo AST. |
| `scripts/subscription-fixture.mjs:13–39`, `tests/formulas.rs:379–456`: reparto exacto existente, construcción secuencial con lecturas de revisión | Convertir la construcción a lote; conservar el motor y añadir la página referenciada que falta en ese script. |
| `src-tauri/src/lib.rs:9–24`: adaptador genérico a `execute_json` | No cambiar el host ni agregar UI de aprobación en esta entrega; compilar contratos y no afirmar nueva aceptación nativa. |

Las operaciones de esquema actuales no crean filas en `changes`: esa tabla referencia entidades. El resultado del lote puede describirlas; este cambio no inventa un historial persistente de esquema ni fuerza IDs de colección en una FK de entidad.

## Goals / Non-Goals

**Goals:** reutilización de reglas y errores existentes, un solo commit real, preview coherente y no mutante a nivel de dominio, referencias sin ambigüedad, confirmación ligada al efecto examinado, IDs de dominio iguales para cálculos y páginas, rollback verificable y un recorrido completo sin modelos/pantalla.

**Non-Goals:** guardar planes o recibos en SQLite; autorizar mediante hashes; promover una copia como commit; aceptar operaciones arbitrarias o extensiones SQL desconocidas en lotes; garantizar atomicidad de archivos ajenos a SQLite. No agregar una política financiera distinta ni nuevas funciones del AST. Las exclusiones funcionales están en proposal.

## Decisions

### 1. Plan v1 cerrado y nueve mutaciones; consultas sólo al final

Contratos nuevos en `batch_contracts.rs`, reexportados por `contracts.rs`; ejecutor/resolver en `batches.rs`. `changes.preview` recibe directamente un `BatchPlan`; `changes.apply` recibe `{plan, expected}`. Mantener `api_version: 1`: adición compatible de operaciones y esquemas, no cambio del significado de entradas previas.

Shapes propuestos:

```text
BatchPlan { version: 1, batch_id: UUID, steps: [BatchStep], checks: [BatchCheck] }
BatchStep { key, operation: BatchOperation, input: JSON, bindings: [Binding] }
Binding { kind: value | property_key, target: JSON Pointer, source: {step, export} }
BatchCheck { key, input: JSON de DatabaseQuery, bindings: [Binding],
             require_complete: true, require_no_calculation_errors: true }
BatchReceipt { version: 1, workspace_id, path_hash, plan_hash, base_hash, effect_hash }
BatchPreview { applied: false, receipt, evaluated_at, steps, checks, diff, history_summary }
BatchApplied { applied: true, batch_id, executed_at, steps, checks, diff, history_summary }
```

Los flags de checks usan true por defecto, no «flag ausente permite resultado parcial». `steps` en las salidas contienen `{key, operation, data, exports}`; `data` reutiliza el resultado individual, y `exports` expone sólo campos enumerados. Las consultas de check se realizan después de **todas** las mutaciones, no intercaladas con resultados incompletos del reparto. Un error intermedio devuelve la envoltura de error, sin un array de éxitos aparentes ni recibo aplicable.

Lista cerrada: `entity.create`, `entity.update`, `collection.create`, `property.define`, `formula.set`, `record.create`, `record.attach`, `record.update`, `view.save`. El tipo `BatchOperation` no contiene `Operation` recursivo. `database.query` sólo aparece en checks. No agrupar operaciones de archivos/workspace, reconstrucción FTS, archivo/restauración, relaciones built-in ni plantillas; siguen en el catálogo individual, con compatibilidad de lote falsa.

Límites nuevos: **1–100 pasos, 0–10 checks, 512 bindings en total, 64 caracteres por key, 512 bytes por pointer, entrada y salida completas hasta 2 MiB**. Keys únicas entre pasos/checks: letra ASCII inicial, luego letras/dígitos/guion/underscore. Mantener profundidad/tamaño de request y documentos vigentes; la envoltura de apply también debe caber en los 2 MiB del transporte. No aumentar ese límite para acomodar un plan demasiado grande. Rechazar límites/forma/operaciones antes de abrir conexión de escritura.

**Alternativas:** una lista de `Operation` aceptaría acciones con efectos no transaccionales y lotes recursivos; llamar el CLI por paso reintroduce commits parciales; habilitar todas las variantes que pueden escribir a la vez diluye el alcance. La whitelist cubre la construcción solicitada y prueba la arquitectura antes de ampliarla.

### 2. Bindings por slot, sin interpolación de texto ni selección arbitraria

Cada paso exporta campos tipados, no JSON Pointers arbitrarios sobre su respuesta:

| Operación | Exports disponibles |
| --- | --- |
| collection.create | collection_id, collection_revision, title_property_id |
| property.define / formula.set | property_id, collection_id, collection_revision |
| entity.create / entity.update | entity_id, entity_revision |
| record.create / record.attach / record.update | entity_id, entity_revision, collection_id |
| view.save | view_id, view_revision, collection_id |

`property.define` consulta la revisión resultante dentro de la misma conexión; `formula.set` obtiene la propiedad creada/actualizada por la identidad validada, no por un índice accidental del bundle. `title_property_id` se obtiene de la única propiedad Title. No exponer `revision` ambiguo para una propiedad que no tiene esa revisión.

Un binding `value` sustituye un placeholder **ya presente** en el input. Registry por operación admite únicamente sus slots semánticos de ID/revisión: extremos/contexto/membresía, IDs del AST reconocido, columnas/filtros/sorts/grupos/calendario, IDs de referencias locales en bloques y destinos de relación. También admite IDs de registros dentro de un valor relation, validando el tipo de la propiedad en esa conexión. No aceptar slots title, texto, literal Text del AST, moneda, payload arbitrario o clave desconocida por coincidir su nombre con un sufijo `_id`.

`property_key` renombra una clave placeholder en `/values/<clave>` o `/record/values/<clave>` y exige un export property_id. Primero resolver todos los `value` sobre el input original, después renombrar claves: así un array relation puede referenciar un registro antes de renombrar la clave de su propiedad. Para validar ese array, resolver la identidad de su propiedad mediante el binding de clave ya declarado, sin depender de que el renombre haya ocurrido. Rechazar destinos duplicados/solapados entre bindings value, renombres repetidos, referencias futuras, exports ausentes, tipos incompatibles, punteros no canónicos y colisión con cualquier clave ya existente. No crear rutas ni implementar JSON Patch general. Aplicar escapes RFC 6901 (`~0`, `~1`) sin reinterpretar strings literales.

Ejemplo reducido, los placeholders no son aún un DTO válido:

```json
{
  "key": "cost",
  "operation": "property.define",
  "input": {"collection_id": null, "expected_revision": null,
            "spec": {"name": "Costo", "type": "money", "currency": "BOB"}},
  "bindings": [
    {"kind": "value", "target": "/collection_id",
     "source": {"step": "subscriptions", "export": "collection_id"}},
    {"kind": "value", "target": "/expected_revision",
     "source": {"step": "subscriptions", "export": "collection_revision"}}
  ]
}
```

Una vez resueltos, deserializar por el DTO individual estricto y ejecutar el helper de dominio. El catálogo debe distinguir `input` previo a bindings del esquema del DTO **resuelto**, declarar exports/slots y ofrecer ejemplos ejecutables, no hacer pasar JSON arbitrario por un input de dominio ya validado.

**Alternativas:** strings `$alias` y reemplazo recursivo corrompen texto que sólo parece un ID; transformar nombres dentro de AST/views/documentos sin slots borra límites; duplicar todos esos DTOs con variantes de referencia crea un segundo contrato de dominio. El sidecar modifica únicamente sitios declarados y reutiliza la validación vigente.

### 3. Helpers con conexión y contexto explícitos; commit propiedad del ejecutor

Extraer los cuerpos de las operaciones participantes a helpers que reciben `&Connection`, root del workspace y un `MutationContext` explícito. Los wrappers públicos siguen abriendo Immediate y confirmando una sola operación. El lote posee una Immediate y llama helpers directamente; checks usan `database::query_in_connection` sin abrir una transacción nueva. El wrapper de consulta conserva su snapshot de lectura habitual.

El contexto transporta origen, request_id, fuente de IDs, reloj y observación de efectos. No usar estado global/thread-local ni detectar silenciosamente que existe una transacción para cambiar el comportamiento del método público. `create_in_transaction` continúa siendo la única creación de entidad compartida con registros/captura/plantillas; sus llamadores individuales conservan IDs aleatorios y comportamiento actual. La adaptación de su firma no habilita plantillas como pasos de lote.

`entity.update` mantiene fusión de payload, documentos no editables y reindexación vigente; record.attach/update mantienen membresía y patch tipado; formula.set conserva inferencia/grafo/AST desconocido; view.save mantiene validación de layout/config y revisión. No aprovechar la refactorización para cambiar esas reglas ni el formato de dinero.

Historial de entidad dentro de apply usa su request_id y origen reales compartidos. El resultado incorpora `history_summary` con entidad/operación/revisiones, no una tabla nueva. Preview genera esas filas únicamente en memoria. Los cambios de esquema se describen en diff/resultados; su persistencia histórica sigue fuera de esta entrega.

**Alternativas:** savepoints sobre wrappers que ya hacen commit no crean una transacción global; simular validaciones sin ejecutar los helpers omite FKs/grafo/FTS; mover reglas al script produce dos motores. Helpers con ownership explícito permiten comprobar paridad antes de añadir batching.

### 4. IDs deterministas de dominio; timestamps de ejecución no confundidos con identidad

El cliente elige una vez un batch_id UUID canónico; el script usa generación aleatoria local y conserva el mismo plan para preview/apply. En modo lote, el contexto deriva cada ID nuevo desde SHA-256 de un namespace fijo `noteka.batch.ids.v1`, batch_id en bytes, key del paso, rol interno y ordinal por rol, con campos delimitados por longitud. Tomar 16 bytes y fijar bits de UUID versión 8/variante RFC; publicar la receta y tests de vectores. Roles separados para entity, collection, property, relation, view y change; el ordinal se reinicia por paso/rol. Se reutilizan uuid/sha2 existentes, sin requerir una nueva dependencia.

Los IDs existentes jamás se recalculan. La colisión de un ID reservado con un objeto existente aborta; no actualizarlo usando upsert como si fuera el objetivo original. Debe verificarse explícitamente antes de inserts/upserts de creación, incluidas fórmulas y vistas. Los IDs de bloques enviados por el cliente se conservan: no se generan implícitamente desde texto. Fórmulas allocate, orden de vistas y referencias incrustadas utilizan los mismos IDs en ambos modos.

El reloj del lote fija un instante al comenzar su simulación/ejecución. Preview etiqueta metadatos como simulados; apply usa su propio instante de ejecución, no el reloj enviado por el cliente ni una fecha de preview vieja. No prometer hora exacta del commit físico. El hash de efecto excluye sólo metadatos de ejecución identificados por tipo (created_at/updated_at de Entity, sus propiedades CreatedTime/UpdatedTime, request_id/origen/ID/fecha de historial), **no** fechas civiles, payloads con claves de igual nombre, IDs de dominio, revisiones, cálculos o errores. Cambios de orden/cobertura causados por timestamps en una consulta siguen comparándose; una divergencia aborta y requiere otro preview.

**Alternativas:** UUIDs nuevos al aplicar pueden alterar el desempate monetario y las referencias; asignar IDs manualmente en el script expone reglas internas; guardar reservas exige migración/limpieza. La derivación evita esas tres fronteras, sin afirmar idempotencia general.

### 5. Preview coherente en memoria y apertura que nunca migra

Introducir apertura específica para changes, separada de `Workspace::open`. Prevalidar versión/estructura sobre copia privada como actualmente, **sin usarla como snapshot definitivo**. Si WAL desaparece entre su detección y copia por el checkpoint de la última conexión, descartar cualquier WAL privado parcial y volver a copiar la base checkpointada; otros errores de IO siguen propagándose. No reintentar ni rebasar escrituras. Versiones anteriores devuelven unsupported_workspace_version con current/supported y `migration_required: true`; el cliente puede usar workspace.open por separado y volver a previsualizar. Futuras y esquemas SQL no reconocidos para batching se rechazan sin intentar reparación. Workspace ausente no se crea.

Para preview abrir el original `READ_ONLY`, activar timeout y `query_only`, sin configurar journal_mode ni migrar. Iniciar Deferred y fijar snapshot con la lectura de identidad/versión. Comprobar tamaño dentro de ese snapshot; `page_count * page_size <= 64 MiB`. Antes de poblar el destino en memoria, configurar su page_size igual al de la fuente, porque backup no puede ajustar una base en memoria a un page_size incompatible. Copiar con `Backup::new` a esa conexión distinta mientras la lectura permanece fijada. Pasos de 256 páginas, deadline de contención **2.000 ms**, tratamiento explícito Done/More/Busy/Locked; no `run_to_completion` con reintentos indefinidos. Liberar fuente al completar, validar snapshot v7/estructura, configurar foreign_keys en la copia y ejecutar plan/checks allí. Descartar copia siempre.

La API de backup debe incluir páginas confirmadas en WAL. No usar `immutable=1`, copia manual DB+WAL o `serialize` sin demostrar inclusión del WAL. Documentación oficial consultada: [SQLite backup](https://www.sqlite.org/backup.html), [restricciones de Backup/page_size](https://www.sqlite.org/c3ref/backup_finish.html), [aislamiento](https://www.sqlite.org/isolation.html), [Backup de rusqlite](https://docs.rs/rusqlite/0.38.0/rusqlite/backup/struct.Backup.html); verificar estas llamadas contra la dependencia fijada al implementar.

«Sin writes» significa sin mutaciones de dominio, base/WAL, migraciones, backups publicados ni originales por **esta operación**. SQLite puede crear/modificar archivos de coordinación SHM al leer un WAL; no se garantiza igualdad de bytes/mtime de SHM. El preflight privado evita tocarlo para versiones futuras rechazadas. Un escritor concurrente sí puede cambiar la base; la prueba de coherencia compara estados completos posibles, no exige que ese escritor deje el workspace intacto.

Los helpers de documentos/archivos pueden validar referencias ya existentes usando el root original y sus hashes, sin importar/copiar/modificar originales. La copia en memoria contiene sus metadatos, no duplica archivos. Un original alterado/ausente al aplicar vuelve a fallar validación. La inmutabilidad administrada no es una promesa contra manipulación externa del filesystem.

**Alternativas:** ejecutar y hacer rollback en el original bloquearía escritores y escribiría WAL para un preview; migrar una copia produciría un estado no aplicable al workspace viejo; publicar esa copia pisaría concurrencia e ignoraría adjuntos. La copia sirve exclusivamente para evaluación, nunca para reemplazar el original.

### 6. Huella conservadora del estado y diff observado, no SQL suministrado

`base_hash` SHA-256 incluye workspace/versión y definiciones de `sqlite_schema` ordenadas por tipo/nombre, sin rootpage, más filas crudas de las **13 tablas originales**: workspace_meta, entities, documents, relations, changes, attachments, collections, properties, memberships, property_values, views, calculations, templates. Orden por PK con collation BINARY; claves compuestas explícitas para memberships/property_values. Serialización interna v1 delimitada: nombre/columna/tipo de valor, longitud y bytes; enteros i64 BE, reales por bits IEEE-754, NULL distinto de vacío, TEXT UTF-8 exacto y BLOB exacto. No parsear/normalizar JSON persistente para calcular base_hash ni excluir campos futuros/historial/timestamps.

FTS y sus shadow tables son derivados y se excluyen del contenido base: reconstruir FTS sin cambiar originales no invalida un preview. Las definiciones del esquema sí están vinculadas. Antes de habilitar un lote, verificar columnas/PK esperadas y ausencia de tablas originales extra o triggers/extensiones no reconocidos, exceptuando FTS/shadows SQLite conocidos; no borrar ni convertir esas extensiones. Índices compatibles pueden permanecer y sus definiciones entran en la huella. El modo individual conserva su política actual de apertura; este límite fail-closed es específico del lote. Datos JSON futuros en las tablas conocidas siguen preservados/admitidos para lectura o rechazo de edición, no se confunden con una extensión SQL.

La huella global invalida también una edición ajena al conjunto escrito. Es deliberadamente conservadora para cubrir dependencias de fórmulas, altas de registros y lecturas negativas sin inventar un read-set incompleto. No usar PRAGMA data_version entre conexiones distintas, mtime, bytes del archivo SQLite ni hash de WAL como token de estado.

`plan_hash` usa JSON canónico interno v1 (objetos ordenados, arrays preservados, representaciones de números fijadas), incluido batch_id, bindings, flags y pasos; no el formato whitespace del archivo. `path_hash` deriva de ruta canónica, para admitir `/var` y `/private/var` equivalentes pero rechazar un restore en otro destino con mismo workspace_id.

Los helpers participantes observan claves de dominio afectadas antes de escribir; `ChangeSet` captura filas iniciales y finales para entity/document/collection/property/membership/value/calculation/view/relation y consolida múltiples cambios de una clave. Diff contiene `{kind, key, before?, after?}`, con objetos de dominio y JSON crudo de formato no reconocido; no SQL ejecutable. No incluir adjuntos completos ni contenido no afectado. History_summary es separado; FTS se prueba pero no se presenta como original editado. Si no cambia nada, el diff lo indica.

`effect_hash` vincula diff consolidado, exports, resultados de pasos/checks e history_summary, con normalización **tipada** de los metadatos de ejecución descritos arriba. Datos omitidos de presentación no se omiten de la comparación; superar 2 MiB aborta, nunca truncar y aprobar. Preservar originales en base_hash permite detectar también cambios de datos desconocidos entre preview y apply.

**Alternativas:** sólo expected_revision de targets no detecta una nueva relación/miembro que cambie el reparto; copiar bytes del `.sqlite` ignora WAL y checkpoints; read-set fino exige rastrear todas las dependencias/ausencias. La huella global es más costosa y conflictiva, pero verificable y acotada para esta primera entrega.

### 7. Apply reejecuta bajo lock y compara antes de confirmar

Validar plan/recibo/plan_hash antes de abrir escritura; abrir workspace existente v7 sin migración ni cambio de journal_mode (WAL o rollback journal ya existente), con foreign_keys, synchronous=FULL y busy_timeout vigente. Iniciar Immediate; dentro del lock verificar de nuevo identidad/versión/estructura y scope, calcular base_hash y compararlo con expected. Un conflicto o formato incompatible termina antes de ejecutar helpers.

Con el mismo algoritmo de IDs, ejecutar steps/checks sobre la transacción real, producir diff/effect_hash y comparar con expected. `require_complete` exige cursor de entrada ausente, next_cursor ausente y matching_count igual a filas devueltas, no basta con estar en la última página. `require_no_calculation_errors` verifica las filas de la consulta; no valida cálculos de todo el workspace ni promete cobertura fuera del filtro. Desactivarlo no convierte en éxito errores de filtro/sort que database.query ya rechaza. Con cero checks sólo se validan mutaciones de dominio, no se certifica la ausencia global de errores derivados.

Validar cobertura/cálculos y tamaño/serialización de la respuesta **antes** del commit. Si todo coincide, commit una vez y devolver applied:true. Cualquier error deja rollback; no cambiar a copia/snapshot ni ignorar una divergencia por ser «parecida». No repetir automáticamente operaciones por workspace_busy, storage_error o pérdida de transporte.

Errores mantienen CoreError/exit codes existentes, agregando details `{phase, step_key?, step_index?, operation?, binding_index?, reason?, ...}` sin input completo ni stacktrace. `validation_error`: forma, bindings, límites y recibo incompatible/plan alterado; `revision_conflict`: revisión, base_hash o efecto diferente, con reason que los distingue; `workspace_busy`: lock/deadline; errores de cálculo conservan su código específico en details y el código CoreError vigente; storage/formato conservan sus códigos. No publicar exports/diff de pasos como aplicados en una respuesta fallida.

Un recibo es un mecanismo de comparación, no firma, permiso, secreto o prueba de que un humano aprobó el lote. El cliente posee la autorización local por separado y podría fabricar hashes; el núcleo sigue validando y no evalúa SQL/shell. No guardar recibos, inferir permisos desde notas ni habilitar remoto.

Después de commit, base_hash cambia para una mutación efectiva (incluido historial). Repetir el mismo recibo falla; consultar los IDs planeados permite reconciliar una respuesta perdida. Esto evita repetición accidental del plan aceptado, **no** implementa idempotencia persistente ni prueba qué cliente cambió el estado. Los no-ops pueden repetirse sin efecto; nunca se prometen filas de historial para un no-op que hoy no las genera.

### 8. Recorrido CLI de suscripción, sin reglas financieras nuevas

Mantener argumento de workspace nuevo del script. `workspace.create` es un paso individual previo, fuera del lote; si construcción falla puede quedar el workspace vacío, pero ninguna colección/página parcial. Todos los datos son sintéticos y se guardan fuera del checkout.

La secuencia usa **16 pasos**: 2 colecciones + 5 propiedades + 1 fórmula + 6 registros (plan y cinco miembros) + 1 vista + 1 página = 16. Propiedades: costo/capacidad en Subscriptions; asientos/pago/relación en Members. Propiedad Título automática obtenida por export, no definida otra vez. Formula usa allocate existente con relation/cost/weight IDs y scale 2. Vista Pendientes usa columns/filtro/sort conocidos; la página note con documento v3 contiene un collection_view que referencia esa colección/vista originales. Conservar ID del bloque explícito en el plan.

Checks finales: consulta completa de todos los miembros y de Pendientes, sin errores calculados. El script examina preview con BigInt centavos: cinco filas, pesos suman seis, 48.57 + 4×24.28 = 14569, capacity 6 en resultado del registro de plan, pagos falsos y referencias de página válidas. Después aplica **el mismo plan** y receipt; no refrescar revisiones automáticamente para conseguir aceptación. Verificar resultados confirmados con consultas públicas, cerrar/reabrir CLI y backup/restauración en otro destino. Variante inválida y cambio concurrente prueban ausencia de estructura parcial.

Para el recálculo, editar costo y pago con operaciones públicas individuales y revisiones actuales; consultar Pendientes y todas las cuotas para comprobar total vigente. No inferir cobros por capacidad vacante, sobreasignación, vencimientos o períodos. Se mantiene la política sintética previamente verificada, no un sistema financiero real autorizado.

**Alternativas:** un endpoint subscription.create codifica el caso en el núcleo; un fixture que escriba SQL prueba un camino privilegiado; una maqueta UI no demuestra construcción por herramientas. El script usa sólo catálogo/CLI compartidos y fórmulas genéricas.

## Risks / Trade-offs

- [Refactor transaccional altera operaciones previas] → tests de paridad helper/wrapper y regresión existente; conservar la creación única usada por templates/captura.
- [Lectura de sólo lectura modifica coordinación SQLite] → distinguir dominio/base/WAL de SHM; no afirmar igualdad física de éste ni usar immutable para ignorar WAL.
- [Huella global causa conflictos ajenos] → comportamiento conservador documentado; nuevo preview explícito, sin rebase ni retries silenciosos.
- [Hash/copia consumen memoria y sostienen lock demasiado] → snapshot máximo 64 MiB, presupuesto cooperativo de ejecución de 5 s (checks entre fases/pasos/filas y progress handler SQL), error workspace_busy por agotamiento, no commit después de detectar deadline. Activar únicamente el feature `hooks` de la dependencia rusqlite existente para interrumpir SQL; instalar/remover handler con guardia de lifetime y mapear SQLITE_INTERRUPT por deadline sin disfrazar otros errores. [API oficial](https://docs.rs/rusqlite/0.38.0/rusqlite/struct.Connection.html#method.progress_handler). El presupuesto no promete tiempo real estricto frente a IO del sistema operativo ni suma preflight/lock como si fuera una sola duración. Medir release p50/p95, tiempo con lock y RSS en M1 Max con 10.000 originales, sin subir límites para hacer pasar tests.
- [Normalización oculta un cambio real] → recorrer resultados según tipos/propiedades, conservar payload/fechas/cálculos/errores; test con claves textuales created_at y empate monetario.
- [Diff/hash ignoran objetos secundarios] → observación de todas las claves mutadas por cada helper, tests sobre membresía, valores, contexto, historial y FTS, no sólo entidad principal.
- [Original referenciado cambia externamente] → revalidar hash/existencia en apply; sin commit/atomicidad de archivos ni importación en batch.
- [Recibo usado como control de seguridad o idempotencia] → documentar limitaciones; se valida/ejecuta sobre estado real y se compara antes del commit.
- [Plan demasiado grande requiere partirlo] → error de límite y lotes más pequeños con pérdida de atomicidad global explícita; no encadenarlos como si fueran una sola transacción.
- [Pruebas headless se anuncian como host nativo] → core/CLI son aceptación de este cambio; cualquier regresión UI se ejecuta headless y no cierra tareas nativas de Home.

## Migration Plan

**Sin migración:** SQLite v7, documentos/AST/propiedades/vistas/templates y manifiestos permanecen intactos. No agregar tablas batch/idempotency/audit ni borrar extensiones desconocidas. Los workspaces anteriores se migran sólo mediante operación individual explícita antes de pedir preview. Cliente anterior ignora los nuevos comandos y conserva lectura de v7.

Orden: helpers y paridad individual → contratos/resolver/IDs → apertura/snapshot/huellas/diff → preview → apply/checks/rollback → catálogo/CLI/contratos generados → script y recuperación → documentación/regresión/revisión. Registrar ADR **0010** al implementar, con evidencia de IDs/huellas/ownership, no como decisión ya ejecutada por crear este diseño. Guía nueva `docs/batches.md` y verificación `docs/verification-phase-4.md` sólo describirán comandos realmente disponibles al completar sus tareas.

Rollback de software: retirar nuevas operaciones vuelve al CLI individual sin invalidar los datos originales confirmados; no restaurar una copia vieja sobre trabajo ajeno. No hay undo automático del lote; recuperación mediante backup/restauración existente en destino nuevo. Los recibos efímeros no necesitan migración ni eliminación de datos personales.

Antes de sincronizar/archivar: escenarios trazados a tests nuevos sobre SQLite y CLI reales, límites/fronteras, simultaneidad con conexiones independientes, fallos antes de commit, preservación de formatos/originales, roundtrip backup, catálogo completo, fmt/Clippy/contratos/check/build y release sin lanzar el host. Si se tocan consumidores frontend, regresión UI exclusivamente headless; Home sigue sin archivarse.

## Open Questions

Ninguna bloqueante para esta entrega. Ampliar whitelist, read-set fino, aprobación visual, auditoría persistente de esquema, undo e idempotencia requieren cambios posteriores con contrato propio. No se difieren investigación de ownership, formato de referencias ni política de commit a una tarea genérica.
