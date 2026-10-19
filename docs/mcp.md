# MCP local por stdio

`noteka-mcp` conecta un cliente local con operaciones Rust compartidas. **No es acceso desde una cuenta de ChatGPT**, un daemon ni una sandbox contra otros procesos del mismo usuario. El núcleo no necesita LLM ni red; SQLite y los adjuntos originales siguen siendo la autoridad. El transporte cerró **22/22** en v10/51; [evidencia histórica](verification-mcp.md). La [integración XLSX verificada](xlsx-extraction.md) añadió lectura en v11/52 y la [búsqueda XLSX](xlsx-search.md) lleva el producto vigente a v12/52, sin nuevas tools: 24/44/45. [Aceptación fresca](verification-xlsx-search.md): 21 SDK por perfil, tres rondas raw por perfil y lectura/index/search sin helpers, sin nueva UI ni proveedores.

## Preparación y arranque

```sh
cargo build --offline --locked -p noteka-cli -p noteka-mcp -p noteka-pdf-worker -p noteka-xlsx-worker
# Usá una ruta EXISTENTE y actual. Para crear/migrar, usá explícitamente el CLI.
target/debug/noteka-mcp --workspace /ruta/ficticia/workspace
# Escritura explícita; no ejecuta la cola automáticamente:
target/debug/noteka-mcp --workspace /ruta/ficticia/workspace --allow-write
# Ejecución explícita, un trabajo por llamada extraction.run:
target/debug/noteka-mcp --workspace /ruta/ficticia/workspace --allow-write --allow-run
```

Hace falta preparar las dependencias del lock antes del build offline. Los helpers PDF/XLSX deben estar al lado del ejecutable, con el mismo layout usado por el CLI. Leer resultados persistidos no necesita helpers; run falla cerrado si falta el correspondiente. No se instalan modelos ni se activa un scheduler. `--allow-run` solo es inválido. Flags desconocidos/repetidos, ruta ausente o formato incompatible terminan con código no cero y diagnóstico mínimo en stderr; stdout se reserva para MCP.

Ejemplo **ficticio**, no instalado automáticamente en ningún cliente:

```json
{"mcpServers":{"noteka":{"command":"/ruta/ficticia/noteka-mcp","args":["--workspace","/ruta/ficticia/workspace"]}}}
```

Un cliente autorizado puede retransmitir las notas/fragmentos recibidos a su proveedor externo. Stdio no concede consentimiento remoto. Revisá ese cliente y seleccioná explícitamente el uso remoto antes de conectarlo a datos personales; este cambio no configura cuentas, OAuth, HTTP, túneles ni perfiles personales.

## Permisos y superficie

La ruta canónica, identidad y política se fijan al iniciar. No son DTOs ni se aceptan overrides de workspace/origen/rutas en herramientas, notas, metadata o recibos. Cada llamada deserializa el DTO estricto y comprueba su operación/acción antes de efectos. Una lectura abre main READ_ONLY; preview simula en su copia privada. TEMP privado es admisible y SQLite puede usar SHM: no equivale a inmutabilidad física de sidecars durante consultas. Arranque/discovery sí usan un sondeo privado que no modifica archivos del workspace.

| Modo | Herramientas | Acciones |
| --- | --- | --- |
| Predeterminado | 24 | Lectura, preview, control/index sólo get |
| allow-write | 44 | Añade 20 herramientas de mutación y control set/index put/reset |
| allow-write + allow-run | 45 | Añade únicamente extraction.run |

Lectura: `changes.list/preview`, `collection.get/list`, `database.query`, `entity.get/list`, `extraction.control/fragments/get/index/list/pdf_fragments/search/xlsx_cells`, `home.query`, `operations.describe`, `record.get/inverse`, `relation.list`, `search.query`, `template.get/list`, `workflow.query`. Celdas XLSX completas, números léxicos, fórmulas/cachés sin recálculo y metadatos; index/search trabajan sobre campos persistidos seleccionados, no offsets del ZIP. Put/reset requieren allow-write pero no allow-run; extraer sí necesita ambos permisos.

Escritura añadida: `changes.apply`, `collection.create`, `entity.archive/create/restore/update`, `extraction.cancel/enqueue/retry`, `formula.set`, `property.define`, `record.attach/create/update`, `relation.add/remove`, `search.rebuild`, `template.instantiate/save`, `view.save`. `changes.preview` no concede apply ni amplía las nueve mutaciones y database.query de la whitelist de lotes.

Siempre excluidas: `workspace.create/open/backup/restore`, `attachment.import/read`, `document.export`. Hay **52 operaciones del CLI/core**, con los 51 contratos previos conservados. Adjuntos sintéticos se preparan/importan por CLI; MCP no ofrece rutas arbitrarias, binarios base64 ni exportación. Denegación conocida: envelope v1 `validation_error`, `details.reason=permission_denied`. Nombre desconocido: error MCP. Annotations son hints, no permisos.

## Discovery y contratos

- MCP moderno **2026-07-28**: `server/discover`, metadata estándar por solicitud, sin initialize. Legacy **2025-11-25**: initialize/initialized. No se anuncian otras revisiones ni prompts/sampling/elicitation/tasks/suscripciones.
- Nombre estable: `noteka_` más operación con puntos reemplazados por `_`. El mapeo se verifica sobre todo el catálogo; no se decodifica el nombre por heurística inversa.
- tools/list ordena por nombre y pagina de **8** en 8. Cursor opaco versionado, <=2 KiB, ligado a catálogo/política/posición; no es un permiso ni un secreto.
- InputSchema deriva del DTO; control/index filtran sus ramas según política. OutputSchema deriva de Response v1 y dato específico, con referencias locales namespaced y ramas de éxito/error. No sustituye validación del core, no aplica coerciones ni defaults al input.
- Sólo recurso `noteka://catalog`: contrato completo, subset efectivo, acciones, hints, exclusiones/versiones/caps; no incluye ruta, runtime ni contenido del workspace. No admite URIs genéricas ni suscripciones.
- tools/call entrega el **Response v1 íntegro** en structuredContent y un único bloque text con JSON equivalente. isError = !ok. Errores de protocolo se distinguen de DTO/dominio. El core genera request_id, independiente del ID JSON-RPC; el historial de mutaciones usa origen confiable `mcp`.

## Límites y reconciliación

| Recurso | Cap |
| --- | --- |
| Línea entrante (sin newline) | 4 MiB; +1 cierra incluso sin terminador |
| Request core | 2 MiB / profundidad 32, incluida ruta confiable |
| Response core entregable | 2 MiB JSON compacto |
| Frame saliente | 16 MiB, después de escapes/wrapper |
| Solicitudes de protocolo | 4 admitidas; entrada/salida con 4 mensajes por canal |
| Core | 1 activo, **0 pendientes**; exceso `-32000`, reason server_busy |
| Escritor SQLite | 2 s, sin retry automático |
| stdout | write/flush 5 s; saturación terminal cierra |
| Cierre | máximo 30 s desde EOF/error o primera escritura bloqueada |

El exceso de respuesta puede detectarse **después del commit**. Error de entrega `-32603`, `reason=response_too_large`, request_id, `core_completed=true`, `effects_may_be_committed` según operación. No implica rechazo pre-COMMIT, truncamiento ni rollback. Reducí limit/filtros para lecturas; consultá IDs/revisiones/historial o usá CLI para reconciliar mutaciones. Un alta cuya respuesta se perdió puede no haber comunicado su ID. **No repitas automáticamente la mutación.**

Cancelar antes del despacho evita ejecutar. Cancelar una espera iniciada no aborta un thread Rust, no llama extraction.cancel y no libera el cupo hasta que el trabajo termine. Se suprime una respuesta pendiente; una respuesta cuya escritura ya comenzó no se retira. Con un único cupo, cancelar operativamente un run activo requiere otro cliente CLI con revisión vigente. En EOF se detiene admisión y se señala apagado a los supervisores PDF/XLSX; claims ya confirmados conservan su recuperación explícita. La evidencia histórica de shutdown/helper, EOF, broken pipe y stdout sin lector es PDF en debug/release; XLSX se comprueba separadamente en `test-xlsx-processes.py`. No se garantiza limpieza ante SIGKILL/bloqueo del kernel ni se sustituyen esos límites por promesas del runtime.

## Pruebas sin pantalla ni LLM

```sh
sandbox-exec -p '(version 1)(allow default)(deny network*)' npm run test:mcp
sandbox-exec -p '(version 1)(allow default)(deny network*)' python3 -B scripts/test-mcp-transport.py
cargo test --offline --locked -p noteka-core -p noteka-cli -p noteka-mcp
```

El cliente oficial independiente TS 2.3.1 se fija en modern mediante `versionNegotiation: { mode: { pin: '2026-07-28' } }`; su default es legacy. El harness inspecciona mensajes en la frontera stdio antes de normalización, y el probe Python inspecciona bytes reales por separado. El SDK agrega páginas automáticamente: el aggregate no prueba el tamaño de página wire. Fixtures/transcripts sintéticos quedan en un temporal externo, nunca en Git. [Decisión](decisions/0014-scoped-local-mcp.md).
