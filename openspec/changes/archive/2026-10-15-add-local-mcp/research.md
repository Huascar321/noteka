# Research

## Apply dependency evidence — 15/10/2026

SDKs instalados como tooling, separados de ejecución offline: rmcp exacto 3.5.1, default-features false, features efectivas schemars/server/transport-async-rw/transport-io/uuid; Apache-2.0, rust-version 1.88. Tokio resuelto 1.53.2 (MIT, 1.71), tokio-util 0.7.20 (MIT, 1.85); proyecto probado con Rust 1.99.0. Cliente independiente dev-only @modelcontextprotocol/client y core TS 2.3.1 (Apache-2.0, Node >=20), Node real 22.22.3. Ajv exacto 8.20.0 (MIT) añadido sólo para validar schemas 2020-12, sin coerción/defaults; formatos numéricos Rust se tratan como annotations. Build frontend/contracts/check pasan sin cliente en src/bundle; no se actualizan dependencias previas por conveniencia.

Árbol/metadata/logs externos: `$MCP_AUDIT/tree-features-final.txt`, `metadata-final.json`, `mcp-dependency-review.json`, `frontend-final7.log`; raíz reproducible en [verificación](../../../../docs/verification-mcp.md). Build/check offline y red denegada; no hay features HTTP/auth del SDK servidor.

Cierre normal/build del binario: 94 paquetes; los **91 externos** declaran licencia (expresiones MIT/Apache-2.0/Unicode-3.0/LLVM-exception/Unlicense/Zlib/MPL-2.0), sin GPL/AGPL identificadas. Tres crates locales: pdf-protocol declara MIT, core/MCP no declaran licencia de distribución; no se agrega una licencia ni se publica el proyecto implícitamente. `mcp-normal-build-licenses.json` conserva metadata paquete por paquete, no constituye asesoramiento legal.

Inspección del SDK confirmó canales internos y spawn de handlers que por sí solos no garantizan los caps del diseño. El Transport bounded admite antes de entregar requests al SDK y consume cancelación/control sin cola core; lifecycle/modelos y codec limitado siguen siendo del SDK. Se restringe su variante inline-legacy: 2025 requiere initialize/initialized. El cliente TS agrega páginas automáticamente y cambia onmessage al descubrir: el harness observa la frontera de transporte durante todas esas asignaciones, además de probes raw independientes. No inferir tamaño de página wire a partir del aggregate SDK.

Investigación **previa al apply**, **15/10/2026**: documentación y navegación read-only, entonces sin build, instalación, test de protocolo ni implementación MCP. Las secciones siguientes conservan ese antecedente de planificación; la evidencia de implementación está arriba y en la guía de verificación, no en las afirmaciones históricas de ausencia de código.

## Current Repository

Baseline independiente de **302 archivos**, idéntico al manifiesto del cierre de búsqueda de adjuntos; copia y `baseline.json` en el temporal aprobado `noteka-mcp-plan-tJVSU9/`. No existe commit inicial. `catalog-v10-51.json` procede del CLI release existente, `operations describe`, sin workspace y con red denegada; confirma API v1/51, no comportamiento MCP.

| Evidencia inspeccionada | Consecuencia para el plan |
| --- | --- |
| `crates/noteka-core/src/lib.rs:54–135,158–217` | Catálogo/DTOs y despacho reutilizables. `operation_capabilities` sólo describe lotes: no constituye política de permisos. `execute_json` no acepta modo read-only ni cancelación. |
| `crates/noteka-core/src/workspace.rs:171–220,255–267` | Preflight en copia privada; el opener genérico abre RW, puede migrar y configura WAL. Llamar una operación de lectura por el dispatcher actual no alcanza para garantizar sólo lectura. |
| `crates/noteka-core/src/lib.rs:194–215` | Índice/búsqueda de adjuntos y lotes ya tienen admisión dedicada; no reemplazar sus fences, validación ni manejo TEMP. |
| `crates/noteka-core/src/attachment_search_tests.rs:127–158` | Prueba real de main READ_ONLY con TEMP escribible y rechazo de DELETE; patrón verificable, no claim de filesystem absolutamente inmutable. |
| `crates/noteka-core/src/attachments.rs:105–188` | Import recibe ruta arbitraria; read devuelve original completo base64, hasta el cap existente de 64 MiB. Ambas quedan fuera del primer transporte; no introducir transferencia binaria gigante ni fingir que read sólo devuelve metadata. |
| `crates/noteka-core/src/documents.rs:376–449` | Export publica directorio, JSON/Markdown, manifiesto y originales en un destino; no es una lectura pura. Se excluye, junto con administración/backup/restauración. |
| `crates/noteka-cli/src/main.rs:282–308,355–407` | CLI comparte execute_json, input 2 MiB y tests de igualdad de sus 51 operaciones/schemas. Conservarlo como baseline y cliente concurrente; no serializar el protocolo MCP mediante stdout del CLI. |

Main specs `core-operations` y `local-workspace` se leyeron completas mediante `openspec show --type spec`: ya conservan contratos, CAS, errores y migración explícita de operaciones especializadas. Sólo se amplía core para el nuevo cliente/entrada scoped; no se cambia el formato persistente ni la semántica histórica de `Workspace::open`.

## Protocol Sources

1. [MCP 2026-07-28, versionado](https://modelcontextprotocol.io/specification/2026-07-28/basic/lifecycle): **no handshake moderno**. Cada request lleva versión/capabilities en `_meta`; `server/discover` es obligatorio para servidores. Legacy `2025-11-25` conserva initialize. Discovery/identidad no autentican al cliente.
2. [Discovery](https://modelcontextprotocol.io/specification/2026-07-28/server/discover): devolver versiones/capabilities/identidad correctas; no anunciar recursos, notificaciones o extensiones no implementados. Identidad moderna se expresa en `_meta` del resultado, no asumir el shape legacy.
3. [stdio](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio): mensajes JSON-RPC por línea, stdout sólo protocolo, stderr para diagnóstico; EOF es cierre normal. La página exige no seguir emitiendo mensajes de una solicitud cancelada.
4. [Cancelación](https://modelcontextprotocol.io/specification/2026-07-28/basic/patterns/cancellation): hay carreras y trabajo que puede no ser cancelable. El core síncrono actual no permite prometer rollback por abandono de un Future; no reintentar automáticamente ni confundir esta notificación con `extraction.cancel`.
5. [Tools](https://modelcontextprotocol.io/specification/2026-07-28/server/tools) y [paginación](https://modelcontextprotocol.io/specification/2026-07-28/server/utilities/pagination): inputSchema con raíz object, outputSchema/structuredContent, errores de herramienta diferenciados y annotations sólo informativas. Nombres ASCII 1–128; cursores opacos, estables e inválidos rechazados. No inventar que el orden de las herramientas del core ya constituye una política.

Context7 se consultó para protocolo, SDK Rust y cliente TS; las páginas oficiales anteriores resuelven detalles de era que los ejemplos genéricos de inicialización pueden omitir. No se toma una referencia de SDK como evidencia de interoperabilidad del producto.

## SDK Candidates

### Rust server

[rmcp 3.5.1](https://docs.rs/rmcp/3.5.1/rmcp/), publicado el **05/10/2026**, SDK Rust oficial, licencia **Apache-2.0**, MSRV **1.88** según [metadata de crates.io](https://crates.io/api/v1/crates/rmcp/3.5.1). El toolchain del baseline es Rust 1.99; satisfacer MSRV no demuestra que compile este proyecto.

Proponer `default-features=false`, `server` y `transport-io`; añadir sólo lo necesario para codec acotado. [Features exactas](https://docs.rs/crate/rmcp/3.5.1/features): server trae async-rw/schemars/uuid/pastey; no habilitar HTTP, auth, elicitation, request-state ni cliente HTTP. Las dependencias de servidor no prueban ausencia de sockets en todo el árbol de Tauri: revisar el árbol del binario MCP por separado.

`ServerHandler` ofrece discover, supported_protocol_versions, list_tools, call_tool y contexto de cancelación; `JsonRpcMessageCodec::new_with_max_length` permite acotar framing. Se propone usar SDK, no reimplementar JSON-RPC o codecs de eras con código propio. Ajustar el servicio/transport para servir ambas eras debe demostrarse con wire y cliente real: no alcanza con devolver ambas versiones en metadata.

### Independent client

[`@modelcontextprotocol/client` 2.3.1](https://registry.npmjs.org/@modelcontextprotocol/client/2.3.1): Apache-2.0 y Node >=20; el baseline usa Node 22.22.3. Proponer devDependency exacta, no importarla en frontend/producto ni usar npx con descargas durante tests.

La [guía versionada de migración](https://github.com/modelcontextprotocol/typescript-sdk/blob/v2.3.1/docs/migration/support-2026-07-28.md) aclara que `Client.connect()` por defecto sigue legacy. Para demostrar modern se requiere `versionNegotiation: { mode: { pin: '2026-07-28' } }` y comprobar la era; usar modo legacy explícito para `2025-11-25`. Probar además auto/discover y error de versión por wire, sin inferir negociación moderna de ver un `_meta` en un probe.

`auto` con StdioClientTransport puede lanzar un proceso hermano descartable para sondear; por eso incluso el arranque/discovery deben ser puros. El cliente puede consumir `resultType`/campos wire internamente: validar ambos niveles, modelo público y transcript raw. El cliente local SDK sin LLM satisface una prueba de transporte real; no prueba acceso de ChatGPT, un cliente gráfico particular ni permiso para transmitir a un proveedor.

## Verification Boundaries

- Ningún peso, credencial, perfil personal o configuración de un cliente existente se leyó o instaló.
- Las futuras descargas de dependencias son tooling; pruebas Rust/SDK/CLI se ejecutan offline/con red denegada, sin pantalla. La UI headless, si se repite por regresión, sólo necesita loopback del puente existente.
- Cualquier resultado de latencia será posterior a build/tests, con corpus/versiones/condiciones documentados. Los 214,20–231,07 ms previos pertenecen al core de búsqueda de adjuntos, no a MCP.

## Planning Review

Al 15/10/2026: cuatro artefactos requeridos preparados y **0/22 tareas implementadas/verificadas**. Los deltas contienen catorce requisitos añadidos y uno modificado: cuarenta escenarios nuevos y cinco heredados conservados íntegros. Se contrastó la partición propuesta 23 lecturas/20 escrituras/1 run/7 exclusiones con los 51 nombres del catálogo real, sin colisiones al mapear herramientas.

Validación documental estricta: cambio aprobado, **12/12 specs vigentes y 14/14 items** sin issues. Auditoría contra baseline: 302 → 309 archivos, siete artefactos nuevos, sólo plan/workflow/contexto modificados y 299 archivos previos idénticos, incluidos código, tests, locks, migraciones, Home, main specs y archivos históricos. Los logs, manifiestos, diff y auditoría reproducible quedan en el temporal aprobado indicado arriba. No se ejecutaron pruebas de aplicación ni se instaló o compiló MCP durante esta planificación; interoperabilidad, contención y rendimiento continúan siendo tareas de apply.
