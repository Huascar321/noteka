# Proposal

## Why

Al proponer este cambio, el núcleo tenía 51 operaciones verificadas por CLI, pero ningún transporte MCP implementado: un cliente compatible no podía descubrirlas ni operar el workspace por `stdio`. La dependencia de fase 4 ya estaba cumplida; este recorte de 6c avanza sin modelos, cuentas, UI nueva ni las pruebas nativas pendientes de Home. Estado de implementación/aceptación: [evidencia](../../../../docs/verification-mcp.md).

## What Changes

- Proponer un binario Rust `noteka-mcp`, limitado a un workspace existente y actual elegido al arrancar, sin puertos, creación, migración ni recuperación implícitas. Compartir despacho, DTOs, CAS, errores y reglas del núcleo; conservar SQLite v10/API v1/51 y CLI/UI anteriores.
- Empezar en modo de lectura: 23 herramientas, incluidas acciones `get` de control/índice y preview aislado. `--allow-write` habilita 43 herramientas; `--allow-run` exige además escritura y agrega únicamente `extraction.run` (44). Comprobar permisos en cada llamada, no confiar en annotations, identidad del cliente o contenido de documentos.
- Excluir explícitamente siete operaciones de esta superficie: las cuatro `workspace.*`, `attachment.import`, `attachment.read` y `document.export`. Administran rutas/publicaciones o transfieren originales binarios; siguen disponibles por CLI y en el catálogo completo. No se declara paridad de transporte para esas siete operaciones.
- Derivar herramientas y esquemas del catálogo compartido, con nombres `noteka_` más operación con puntos reemplazados por `_`, paginación estable, outputs estructurados/envelope v1 y fallback textual equivalente. Publicar un recurso estático `noteka://catalog` que distingue las 51 operaciones del núcleo de las herramientas realmente habilitadas.
- Implementar MCP moderno `2026-07-28` con discovery/metadatos por solicitud y compatibilidad legacy `2025-11-25`, sin confundirlo con un handshake obligatorio para ambas eras. Acotar framing, concurrencia y salidas; no reintentar mutaciones ni prometer que cancelar la espera revierte un commit.
- Verificar el binario con un cliente SDK oficial independiente en ambas eras, procesos CLI concurrentes, fixtures sintéticas, red denegada y ventana cerrada. Documentar límites, configuración de ejemplo y riesgo de que un cliente conectado a un proveedor remoto retransmita resultados; no configurar clientes personales automáticamente.

**Non-goals:** acceso a archivos/rutas arbitrarios, administración/importación/exportación por MCP, HTTP/SSE/túneles, notificaciones de cambios/suscripciones, prompts/sampling/elicitation, agentes/embeddings/modelos, cuentas, proveedores, publicación o empaquetado firmado. No modifica Home ni sus ocho tareas nativas; no completa 5a/5b ni todo el alcance futuro de 6c.

## Capabilities

### New Capabilities

- `local-mcp`: transporte local `stdio`, workspace fijo, permisos efectivos, catálogo filtrado, framing/ciclo de vida acotados e interoperabilidad comprobable sin LLM.

### Modified Capabilities

- `core-operations`: incluir el transporte MCP habilitado en la paridad de contratos y añadir ejecución scoped sobre versión actual con lecturas realmente sin escrituras persistentes; preservar las 51 operaciones, nueve mutaciones de lote y comportamiento CLI/UI anterior.

## Impact

Crate `crates/noteka-mcp`, entrada/opener scoped aditiva en core, pruebas de transporte y cliente, scripts, guía y ADR. SDKs adoptados: `rmcp =3.5.1` con features mínimas de servidor/IO y cliente de prueba `@modelcontextprotocol/client =2.3.1`, ambos Apache-2.0; [investigación](research.md). La planificación original se apoyó en un baseline externo de 302 archivos y no acreditaba ejecución; apply tomó su propio snapshot final de inicio (309) y registra resultados reales en la guía de verificación. Home permanece 9/17, sin pantalla ni activación remota.
