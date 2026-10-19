# Design

## Context

Motivación y recorte: [proposal.md](proposal.md). El baseline al iniciar era SQLite v10/API v1/51, sin servidor MCP. [research.md](research.md) registra inspección y fuentes del 15/10/2026; [local-mcp](specs/local-mcp/spec.md) y [core-operations](specs/core-operations/spec.md) definen el comportamiento, no resultados de pruebas. La implementación adoptada y su evidencia viven en [ADR 0014](../../../../docs/decisions/0014-scoped-local-mcp.md) y [verificación](../../../../docs/verification-mcp.md); la descripción del baseline no es el estado actual.

`execute_json` valida y comparte dominio, pero su ruta genérica usa `Workspace::open`, que puede migrar/configurar WAL incluso para lecturas. Sólo las rutas especializadas de lotes/índice tienen admisión actual dedicada. El catálogo no contiene una política read/write: sus flags actuales son exclusivamente de compatibilidad con lotes. No hay cancelación general del core síncrono.

El repositorio no tiene commit inicial. La implementación futura debe comparar contra el baseline externo y un nuevo snapshot de inicio; un diff Git vacío no demuestra preservación. Home conserva 9/17 y aceptación nativa pendiente; no se modifica su código, diseño ni composición.

## Goals / Non-Goals

**Goals:** separar autorización/admisión del transporte y del dominio; reutilizar operaciones sin heredar migraciones automáticas; hacer observable el subset habilitado, probar ambas eras con otro SDK y acotar trabajo concurrente/entrega sin atribuir a cancelación una atomicidad inexistente.

**Non-Goals:** una sandbox contra un proceso hostil del mismo usuario, autorización multiusuario, listas de acceso por entidad, un API de archivos, paridad MCP de las siete operaciones excluidas o un servicio residente. El permiso de lectura concede acceso al contenido local que esas consultas ya devuelven, incluidos fragmentos derivados y referencias; no es sólo acceso a metadatos. No introduce persistencia, configuración de proveedores, UI o un framework de agentes.

## Decisions

### 1. Adapter independiente y entrada scoped del núcleo

Proponer `crates/noteka-mcp` con binario `noteka-mcp`. El adaptador sólo posee framing/MCP, metadatos de herramientas, paginación y presupuestos de transporte. Core conserva DTOs, autorización scoped por operación/acción, admisión, reglas, CAS, transacciones, errores, originales y ejecución explícita. No invocar el CLI como traductor JSON-RPC ni duplicar consultas SQL en el adaptador.

Añadir en core un scope no serializable, con campos privados para ruta canónica, workspace ID y política inmutable. Su constructor valida en una copia privada sin abrir el original y no utiliza el workspace por defecto. La API scoped recibe el JSON de `Operation` ya existente (`operation`/`input`), valida límites y DTO completo y genera el Response/request_id. El adaptador fija origen `mcp`; no deserializa un scope, origen o Request aportado por el cliente. El Request completo construido internamente sigue sujeto al límite existente de 2 MiB/profundidad 32, incluida la ruta confiable.

Refactorizar únicamente el punto necesario del dispatcher para compartir el match de operaciones, con dos modos de admisión: histórico sin cambios y scoped actual. No implementar el modo nuevo como «validar y después llamar execute_json», porque volvería a abrir con migración implícita y dejaría una ventana entre checks. Mantener la entrada pública anterior y los 51 contratos byte/shape-equivalentes; la metadata de política es una API interna/in-process separada, no un cambio de `describe()`.

**Alternativas:** usar sólo `execute_json` deja una lectura potencialmente migradora; implementar un segundo núcleo en MCP duplica validación; modificar globalmente `Workspace::open` cambia CLI/UI y recuperación existentes. Se rechazan las tres.

### 2. Admisión actual por conexión real, no por confianza de sesión

Resolver ruta e identidad sin efectos al arrancar; cada operación vuelve a comprobar versión e identidad en la conexión/snapshot que usa. Un check de ruta al comienzo no valida una conexión abierta después. Integrar la expectativa opcional de scope en los openers especializados de lotes/índice y su validación bajo snapshot/lock, sin sustituir sus verificadores, fences, límites o reset acotado. En genéricas reutilizar los métodos existentes con un opener actual sin migración. Fallar antes de apertura RW si preflight ya detecta una versión incompatible y volver a comprobar la conexión efectiva antes de efectos.

- Lecturas: `main` READ_ONLY; no `immutable=1`, no journal_mode ni recuperación. TEMP privado sigue permitido cuando se requiere. `query_only` no puede aplicarse indiscriminadamente porque bloquea TEMP de búsqueda de adjuntos.
- Escrituras autorizadas: READ_WRITE sin CREATE/migrate; busy timeout 2 s, foreign_keys y synchronous FULL de conexión. Conservar los locks/CAS/transacciones actuales. No extender NO_MUTEX a otras conexiones: sigue limitado al índice/búsqueda ya verificados.
- Preview: fuente read-only y evaluación en su snapshot privado, mismo scope/recibo/presupuesto. Apply nunca publica la copia sobre el original.
- Versión anterior/futura/identidad distinta: error recuperable; migración sólo por el flujo CLI ya existente fuera de esta sesión.

Sólo el arranque/discovery garantizan ausencia de cambios de archivos del workspace, usando preflight privado. Una consulta SQLite READ_ONLY puede utilizar/crear SHM para WAL; garantizar ausencia de mutaciones **persistentes** y de permiso de escritura a main, no bytes inmutables de todo sidecar durante consultas. Pruebas de filas/revisiones/originales y flags de apertura se separan de observaciones de sidecars. Un filesystem/usuario hostil que sustituye archivos abiertos o conserva fraudulentamente el mismo UUID no queda convertido en una sandbox por esta comprobación de identidad.

### 3. Política exhaustiva y superficie exacta

La siguiente partición clasifica las 51 operaciones existentes. Es metadata del core con comprobación exhaustiva y deny-by-default, no heurística sobre sufijos ni batch_capabilities. Las dos acciones mixtas se clasifican por su enum después de deserializar DTOs estrictos.

| Grupo | Operaciones | Herramientas |
| --- | --- | --- |
| Lectura, incluidos get mixtos | changes.list, changes.preview, collection.get/list, database.query, entity.get/list, extraction.control(get), extraction.fragments/get, extraction.index(get), extraction.list/pdf_fragments/search, home.query, operations.describe, record.get/inverse, relation.list, search.query, template.get/list, workflow.query | 23 |
| Escritura añadida con allow-write | changes.apply, collection.create, entity.archive/create/restore/update, extraction.cancel/enqueue/retry, formula.set, property.define, record.attach/create/update, relation.add/remove, search.rebuild, template.instantiate/save, view.save | 20 |
| Acciones añadidas sin crear herramientas | extraction.control(set), extraction.index(put/reset) | 0 |
| Ejecución, requiere ambos flags | extraction.run | 1 |
| Excluidas en cualquier modo | workspace.create/open/backup/restore, attachment.import/read, document.export | 7 |

Modos válidos: lectura 23, escritura 43, escritura+run 44; `--allow-run` solo es error de arranque. Cambiar modo requiere reiniciar. Una operación nueva exige clasificación explícita y tests de cobertura; nunca se expone automáticamente por entrar al catálogo. Permisos se comprueban otra vez en cada llamada, aunque no haya sido listada. Una operación conocida pero deshabilitada devuelve Response v1 `validation_error`, `details.reason=permission_denied`; nombre desconocido es error MCP de parámetros.

`changes.preview` puede simular las nueve mutaciones existentes sin habilitar apply ni ejecución: sus efectos son temporales aislados. Enqueue/control set no arrancan workers. Con un cupo core por servidor, extraction.cancel no interrumpe simultáneamente un run del mismo proceso: se puede usar otro cliente CLI, que conserva el fence del núcleo. No se promete una consola de control concurrente dentro de esa sesión.

Annotations derivadas, no permisos: lecturas `readOnlyHint=true`, `destructiveHint=false`, `idempotentHint=true`; mutaciones/run y herramientas mixtas en modo escritura, hints conservadores false/true/false respectivamente. `openWorldHint=false` para todas: no hay red ni acceso a un dominio externo. No usar títulos, instrucciones de notas, clientInfo o recibos como autoridad.

**Alternativas:** exponer todo con allow-write añade rutas arbitrarias, backup/restauración y originales base64 de hasta 64 MiB; fragmentar binarios o diseñar roots/sandbox sería otro cambio. Se conserva la exclusión explícita, sin retirar esas operaciones del CLI.

### 4. Herramientas, esquemas y recurso estático

Nombres `noteka_` + puntos a `_`; comprobar ASCII, longitud <=128 y ausencia de colisiones en el catálogo completo. Tabla de mapeo estable con validación de cobertura; nunca derivar el nombre inverso por sustitución ambigua de underscores.

InputSchema se copia del catálogo. Para control/index, añadir raíz `type: object` a su oneOf; en lectura conservar sólo rama get, en escritura todas. No añadir ni quitar campos de DTO por conveniencia del SDK. Los `$defs` y `$ref` siguen siendo locales y resolubles, incluso recursivos. OutputSchema deriva de Response v1 y del schema de dato de la operación; componer defs sin colisiones ni referencias al root de un documento distinto. Diferenciar rama de éxito con data y rama de error con CoreError, sin exigir data a un error. Probar todos los esquemas con validador independiente, incluidos datos reales de operaciones y errores, no sólo comparaciones de strings.

`tools/list`: orden ASCII por nombre, páginas de 8, cursor versionado opaco <=2 KiB con posición y huella de catálogo/política. Rechazar versión/huella/posición/formato ajenos. La huella no es un secreto ni permiso. No persistir cursores ni cachear confianza de integridad entre llamadas. Sin `listChanged`: catálogo y flags permanecen fijos durante la sesión.

`resources/list` devuelve una entrada `noteka://catalog`, MIME application/json; sin templates, subscriptions o listChanged. `resources/read` acepta sólo esa URI exacta. Su JSON incluye `core_catalog` completo, lista/mapeo de herramientas habilitadas, acciones/hints, exclusiones, versiones y caps efectivos. No incluye ruta de workspace, IDs de usuario, notas, tokens ni runtime. Es estático por binario/flags y no abre la base. `operations.describe` devuelve el catálogo core intacto; ambas superficies se explican sin fingir que las 51 son invocables.

### 5. Dos eras con SDK oficial y features mínimas

Proponer `rmcp =3.5.1`, default-features false, server/transport-io y sólo extensiones necesarias para codec/canales acotados. Usar sus modelos, lifecycle y serialización por era; no implementar a mano una versión alternativa del protocolo. SDK moderno no equivale a defaults modernos: configurar/validar discover, las dos versiones admitidas y el routing apropiado.

- Modern `2026-07-28`: server/discover y metadata obligatoria por request; no exigir initialize. Identidad/capabilities y `resultType` se emiten donde esa era exige.
- Legacy `2025-11-25`: initialize/initialized y control de sesión antes de tools/resources; no aceptar trabajo de dominio antes de inicialización. No anunciar otras versiones por copiar defaults del SDK.
- Capabilities efectivas: tools/resources estáticos, sin sampling/prompts/elicitation/tasks/streams de cambios o callbacks al cliente. Ping y errores necesarios no abren superficies de dominio adicionales.

Cliente de prueba independiente: devDependency exacta `@modelcontextprotocol/client =2.3.1`, fuera del bundle frontend. Probar modern con pin explícito, legacy explícito verificando versión, y auto/discover para el sondeo sin efectos. Inspeccionar wire además de valores públicos del SDK, que abstraen campos por era. No usar un cliente implementado con el mismo SDK Rust como única evidencia, ni la conexión por defecto TS como supuesta prueba moderna.

**Alternativas:** SDK antiguo/handshake sólo legacy no cumple ambas eras; implementar JSON-RPC manual agrega parsers y estados de seguridad propios. Si la configuración del SDK contradice una garantía de la spec, resolver/revisar el plan con evidencia antes de rebajarla, no anunciar compatibilidad a partir de metadata sola.

### 6. Entrega de resultados y límites sin falso rollback

Para resultados dentro del cap: `structuredContent` es el Response v1 completo y content contiene un bloque text con su serialización JSON compacta equivalente. `isError = !ok`. No imprimir JSON del core directamente al stdout de MCP. El request_id lo genera core; el ID JSON-RPC sigue siendo correlación del transporte, sin garantía de deduplicación/idempotencia.

Errores JSON-RPC/MCP para JSON inválido, método desconocido, tool desconocida o parámetros de transporte mal formados; Response/isError para errores de DTO, permisos y dominio de una operación conocida. Mantener códigos/motivos de core sin reinterpretar un intento de extracción fallido como operación sin efectos previos. Logs sólo método/código/duración/ID, no argumentos, contenido, errores raw del cliente o rutas personales por defecto.

| Presupuesto del transporte | Propuesta |
| --- | --- |
| Mensaje entrante, antes de parsear | 4 MiB sin contar delimitador; cerrar al exceder aun sin newline |
| JSON del Request core interno | 2 MiB y profundidad 32 existentes |
| Envelope Response entregable | 2 MiB de JSON compacto UTF-8 |
| Mensaje saliente completo | 16 MiB; medir después de escapes y wrapper de era |
| tools/list | 8 herramientas/página, cursor <=2 KiB |
| Solicitudes de protocolo admitidas | 4 activas, sin cola de handlers ilimitada |
| Ejecuciones core | 1 activa, 0 pendientes; exceso server_busy |
| Canales de transporte | entrada/salida acotados a 4 mensajes cada uno, sin bloquear cancelación/EOF por esperar un permiso |
| Escritura/flush stdout | timeout de 5 s; salida atascada inicia cierre sin reiniciar el plazo global |
| Cierre tras EOF/error de salida | hasta 30 s; después terminación acotada y limpieza de hijos propios |

El framing saliente mayor contempla duplicar JSON como texto, escapes y un ID entrante grande; no aumenta los caps de dominio. Configurar codec limitado de rmcp y bounded channels; comprobar la admisión antes de crear tareas/payloads pendientes, no después de acumularlos. Notificaciones de control/cancelación y EOF no esperan el cupo de ejecución. `server_busy`: error JSON-RPC de servidor `-32000`, reason server_busy, antes del despacho; no fingir workspace_busy de SQLite. Si no se puede encolar una respuesta de rechazo porque el cliente no consume la salida, cerrar la sesión con diagnóstico acotado en vez de bloquear el lector de cancelación o acumular rechazos. Escrituras reales conservan el timeout core de 2 s.

No todos los métodos genéricos comprueban respuesta antes de COMMIT: por ejemplo entity_update confirma y después devuelve Entity. Por tanto **no** introducir una falsa garantía de rechazo atómico para un resultado demasiado grande. Si core terminó pero el envelope/framing excede el cap, devolver error de entrega JSON-RPC `-32603` con reason response_too_large, request_id, core_completed=true y effects_may_be_committed acorde a la clase de operación. No truncar, enviar data parcial, devolver validation_error como si no se hubiera ejecutado, ni repetir automáticamente. El cliente consulta revisiones/IDs/historial o usa CLI para reconciliar; si la respuesta completa se perdió no se garantiza conocer un ID de alta. Los verificadores pre-COMMIT ya existentes de lotes/extracción/índice se mantienen sin cambios.

El cap de wire no es un límite de memoria total del core ni un mecanismo de paginación universal: una lectura que produciría demasiado contenido debe reducir limit/filtros o usar CLI. No cambiar las respuestas de dominio para ocultar ese límite.

### 7. Cancelación, trabajo síncrono y cierre

Ejecutar core fuera del hilo async del protocolo. Mover el permiso de ejecución **dentro** del trabajo síncrono: abandonar el Future no libera la exclusión mientras la operación sigue corriendo. Registrar inicio/completado/cancelado de la solicitud y comprobar cancelación antes de despacho y antes de publicar; no enviar respuestas posteriores a una cancelación ya procesada. Una respuesta que salió antes de que llegue la cancelación no se retira.

No abortar un thread Rust a la fuerza ni intentar convertir drop de spawn_blocking en rollback. Cancelar la espera no llama extraction.cancel, no modifica control, no recupera leases y no permite otra operación core hasta terminar la anterior. Una mutación puede haber confirmado; un run puede haber confirmado claim aunque no publique generación. Tests distinguen ambos de un rechazo anterior a despacho.

En EOF/broken pipe/salida atascada: detener admisión, descartar salidas pendientes y esperar trabajo activo con plazo global de 30 s sin depender del shutdown ilimitado por defecto del runtime. Ante bloqueo de stdout, un timeout de write/flush de 5 s detecta al cliente que no lee; los 30 s cuentan desde esa primera escritura bloqueada y no se suman otros 30 después del timeout. Para EOF o broken pipe cuentan desde el evento. Helper PDF conserva Process/Drop kill+wait y budgets/fences existentes; añadir a la entrada scoped sólo el control de apagado necesario para que su supervisión pueda finalizar y limpiar un hijo al cerrar, sin tratar una notificación de cancelación ordinaria como esa señal. Verificar cleanup con helper retenido/fallido, no asumir que process::exit ejecuta destructores de otro thread. Si se agota la gracia, cerrar hijos propios antes de terminar; una transacción no confirmada queda a cargo de la recuperación SQLite y un claim confirmado sigue siendo recuperable. No afirmar supervivencia ante SIGKILL ni ante bloqueo del kernel.

### 8. Verificación y evidencia separadas

Todas las fixtures nuevas son sintéticas y temporales fuera de Git; ningún perfil personal o original del usuario se usa como corpus. Tests Rust/CLI/SDK del servidor con red denegada y sin Tauri. El SDK cliente local constituye cliente MCP real sin LLM, no acceso de una cuenta o app gráfica particular. Descargas de tooling se separan de ejecución offline.

| Área | Evidencia requerida en apply |
| --- | --- |
| Scope/política | Cada operación clasificada, cada acción mixta y modo; rechazo antes de RW; versiones/identidad cambiadas incluso en rutas especializadas; intento directo sin tools/list; origen confiable |
| Lecturas | Todas las clases de lectura con main READ_ONLY, WAL confirmado, jobs vencidos, futuros/daño; tablas/revisiones/originales sin cambios; TEMP permitido donde corresponde y startup/discovery con manifiesto físico intacto |
| Contratos | Snapshot completo de 51 inputs/outputs/defs/errores y root properties; schemas MCP resueltos/validados, counts/paginación y catálogo real del CLI sin modificaciones |
| Cliente SDK/wire | Modern pin, legacy 2025-11-25 y auto; discovery/initialize correctos, resource, schemas, llamadas, fallos, permisos y versiones inválidas |
| Paridad y concurrencia | Dominio individual, suscripción exacta BOB 145,69 por preview/apply, DTOs inválidos, CAS con CLI, recibo obsoleto, escritor ocupado, trabajos UTF-8/PDF/índice y cancel CLI |
| Recursos/fallos | Línea partida/excesiva/sin newline, backpressure y cliente que no lee, saturación, cancelación antes/después, envelope/framing excesivos tras commit, EOF y limpieza de helper; sin liberar cupo prematuramente |
| Regresión | fmt/check/Clippy, debug/release, tests core/CLI, corpus y contención PDF, concurrencia/benchmarks existentes de adjuntos; contratos y frontend/606 checks headless sin modificar asserts o Home |
| Medición | Versiones/tree/licencias, RSS y tiempo de arranque/discovery, latencia release MCP y overhead frente al mismo CLI/core; corpus y condiciones explícitos, sin adjudicar a MCP los resultados previos del core |

El target existente de búsqueda y sus caps siguen siendo criterios del core y no se relajan por este transporte. Repetir su benchmark si se modifica el camino de admisión/ejecución y conservar ambos resultados, incluidos fallos; medir end-to-end MCP por separado sin inventar un SLA ya validado. Headless sólo usa loopback del bridge existente cuando se prueba UI; eso no certifica IPC nativo ni autoriza pantalla. Documentar matriz requisito/escenario → prueba/log antes de marcar tareas, sincronizar o archivar.

### 9. Margen de rendimiento sin reducir integridad

Durante 6.2, cuatro de seis mediciones válidas de adjuntos superaron el target. El diagnóstico identificó coste del recorrido por fila de 1.199.240 postings; no una regresión causal demostrada de MCP. Se adopta [ADR 0015](../../../../docs/decisions/0015-exact-fts5-instance-traversal.md): recorrido exacto con la API auxiliar pública FTS5 ya incluida, enumerando cada término real y cada documento/instancia, verificando término/columna/rowid/offset/duplicado y cardinalidad con el mismo bitset independiente. Más de 256 términos, API no disponible o vocabulario independiente conservan el stream completo anterior; no es un cap de admisión. Sin reutilización de tokenización sigue el comparador completo anterior. No cambia hash/tokenizer/caps/estadísticas/copias/originales ni persistencia; sin caché entre snapshots y con deadline/RAII de punteros/temporales. Tests diferenciales contra el verificador completo anterior, benchmark heredado intacto y regresión fresca son condición de aceptación, no el uso de una API aparentemente más rápida.

## Risks / Trade-offs

- [Scope no es sandbox del mismo usuario] → flags y directorio elegidos por el host, sin rutas en tools; identidad revalidada por conexión y límites de amenaza explícitos.
- [Cliente remoto retransmite notas obtenidas localmente] → no configuración personal automática; advertir y exigir selección de uso remoto fuera de este cambio. El servidor no puede controlar el destino de bytes que su cliente local autorizado ya recibió.
- [Core síncrono continúa tras cancelación] → cupo retenido, sin retry, reconciliación y cleanup de cierre probado; no promesa de rollback.
- [Respuesta enorme después del commit] → error de entrega con estado explícito, sin truncar ni confundirlo con validación de dominio; CLI sigue disponible.
- [Features/versiones SDK no equivalen a interoperabilidad] → pin exacto, árbol del binario por separado, wire y cliente independiente en ambas eras; no habilitar features de red/auth por conveniencia.
- [Refactor de admisión altera clientes históricos o rendimiento] → APIs aditivas, snapshots de contratos, tests antiguos y benchmarks existentes con caps/targets intactos.
- [Allow-write incluye control/cancel que puede afectar otro run] → permiso explícito conservador, CAS/ownership del núcleo; no reducir su impacto a un hint cosmético.

## Migration Plan

No migración de datos: conservar 001–010, SQLite v10, API v1/51 y documentos v1/v2/v3/futuros. Añadir crate/lock y devDependency sólo en apply; revisar licencias/árbol resuelto y separar cambios de dependencias necesarios de actualizaciones ajenas. Mantener default-members actuales para no convertir comandos históricos en arranque de MCP.

Implementar y verificar primero scope/política, luego transporte y cliente; no conectar un workspace personal para «probar». Registrar decisión duradera en un ADR de docs/decisions al adoptar el código y actualizar guías/roadmap con evidencia real. Rollback del transporte: dejar de lanzar el binario y retirar su configuración de ejemplo; los datos confirmados siguen siendo datos ordinarios válidos de Noteka, no se restauran snapshots sobre cambios ajenos. Ningún instalador, login, publicación o activación remota forma parte del rollout.

La fase de planificación termina con artefactos completos, tareas sin marcar y validación documental. Implementación, sincronización a main y archivo requieren sus etapas posteriores; Home permanece independiente.
