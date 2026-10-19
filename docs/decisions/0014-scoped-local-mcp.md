# ADR 0014 — MCP stdio con admisión scoped compartida

- Fecha: 15/10/2026
- Estado: adoptada y verificada; cambio 22/22 sincronizado y archivado
- Cambio: [add-local-mcp](../../openspec/changes/archive/2026-10-15-add-local-mcp/design.md)

## Contexto

execute_json histórico abre genéricas mediante Workspace::open y conserva migración/configuración WAL para CLI/UI. Un transporte read-only no puede hacer un precheck y después invocar esa ruta: sería autorización cosmética. El catálogo core v1/51 y SQLite v10 no cambian. Home y modelos quedan fuera del recorte.

## Decisión

Entrada aditiva LocalScope no serializable, con ruta canónica/identidad/política privadas. Clasificación exhaustiva por enum/acción, deny-by-default; metadata separada del catálogo público. Dispatcher genérico compartido con opener READ_ONLY o RW sin CREATE/migrate; lotes/índice verifican la expectativa también en conexión/snapshot/lock reales. Startup privado no altera workspace, mientras consultas permiten TEMP/SHM sin mutaciones persistentes. No se convierte el scope en sandbox del mismo usuario.

Binario separado noteka-mcp, rmcp exacto 3.5.1 sin features HTTP/auth/default. Modelos/lifecycle/serialización por era son del SDK; Transport propio añade codec SDK limitado, canales/admisión anteriores a tareas SDK y control de cancelación/EOF sin esperar un permiso core. Un permiso viaja dentro de spawn_blocking y no se libera por drop del waiter. Cliente oficial TS exacto 2.3.1, dev-only; Ajv 8.20.0 valida schemas 2020-12 independientemente, sin coerciones/defaults (formatos numéricos Rust tratados como annotations).

Sólo subset 23/43/44, siete operaciones de filesystem/administración excluidas aun con ambos flags. Recurso estático de contrato, no API genérica de archivos. Modern 2026-07-28 inline y legacy 2025-11-25 con handshake; no se adopta la variante inline-legacy que el SDK también permite. No hay red, scheduler, modelos ni activación remota.

Responses grandes detectadas después de ejecutar son errores de **entrega**, con request_id y posible commit; nunca falsa validación precommit. Cancelación no equivale a rollback ni cancelación de extracción; cierre señala al supervisor propio y tiene plazo global.

## Alternativas descartadas

- Precheck + dispatcher histórico: reabre con migración y deja una brecha de identidad.
- Cambiar globalmente Workspace::open: altera compatibilidad/recuperación CLI/UI.
- Traducir por subprocess CLI o duplicar SQL: separa contratos/reglas/errores y complica fences.
- JSON-RPC/lifecycle propio o SDK legacy-only: multiplica estados o incumple la era moderna.
- Exponer las 51 por MCP: incorpora rutas/destinos/binarios fuera de alcance.

## Consecuencias y verificación

SDK más adaptador bounded requiere pruebas de wire, no sólo datos normalizados; se documentaron dos errores del harness (aggregate y reemplazo de onmessage durante discovery). Retirar una sesión nunca restaura snapshots sobre commits válidos. Stdio local no controla dónde retransmite el cliente: consentimiento remoto sigue siendo una decisión explícita separada.

[Guía](../mcp.md) y [evidencia real/brechas](../verification-mcp.md). La aceptación/archivo exige todas las tareas y escenarios; builds o validación OpenSpec por sí solos no los acreditan.
