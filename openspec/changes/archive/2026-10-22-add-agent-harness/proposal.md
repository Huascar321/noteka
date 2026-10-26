# Proposal

## Why

La fase 4 ofrece operaciones estructuradas y la 5b recuperación con fuentes, pero falta un harness que conecte un proveedor local con esas capacidades sin dar acceso genérico a SQL o shell. Se necesita responder preguntas con citas verificables, ejecutar un ciclo de herramientas acotado y dejar trazas locales para auditar qué se hizo.

## What Changes

- Proveedor de chat local configurable por endpoint loopback y modelo, independiente del dominio. Cambiar modelo no exige cambiar operaciones ni datos.
- Protocolo JSON acotado de un paso: el modelo elige una herramienta pública o entrega una respuesta final con citas. Se rechazan campos desconocidos, herramientas no permitidas y JSON inválido.
- Preguntas con evidencia: la respuesta final sólo se acepta si cada cita coincide con un hit de `retrieval.search` de la misma ejecución y su `quote` aparece en el texto fuente.
- Herramientas compartidas de sólo lectura (`retrieval.search`, `entity.get`, `database.query`, `changes.preview`) y `changes.apply` únicamente con un recibo producido por un preview de la misma ejecución y con `allow_apply` explícito. Sin SQL, shell, workspace, archivos ni extracción.
- Límites de pasos, bytes de contexto/herramienta, tiempo, tokens y concurrencia; cancelación cooperativa entre procesos; errores estructurados y traza local persistida con retención acotada.
- Operaciones `agent.ask`, `agent.cancel` y `agent.trace`; migración aditiva 018 para ejecuciones/trazas.

## Capabilities

### New Capabilities

- `agent-harness`: proveedor local, preguntas con citas verificables, herramientas públicas acotadas, preview/apply, límites, cancelación, errores y trazas.

### Modified Capabilities

- `core-operations`: catálogo, CLI, scopes y contratos de `agent.ask`, `agent.cancel` y `agent.trace`.
- `local-workspace`: migración aditiva 018 y backup/restauración de trazas operativas sin convertirías en fuente de verdad.

## Impact

- Rust: `noteka-core` (contratos, almacenamiento de trazas, cliente de chat loopback y bucle del harness), `noteka-cli`, `noteka-mcp`, migración 018.
- SQLite pasa de 17 a 18; API v1 de 59 a 62 operaciones; MCP se recalcula por política. Las nueve mutaciones batchables no cambian.
- No se activan modelos ni proveedores remotos. Las pruebas usan un servidor loopback simulado; no acreditan calidad de inferencia real.
- Home 9/17 permanece intacto. Fase 6 no se inicia.
