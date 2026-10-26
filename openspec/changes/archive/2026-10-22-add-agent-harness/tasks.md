# Tasks

## 1. Contratos y persistencia

- [x] 1.1 Definir DTOs de `agent.ask`, `agent.cancel`, `agent.trace`, configuración de proveedor, límites, estados, citas y trazas.
- [x] 1.2 Escribir la migración aditiva 018 y integrarla en apertura, migración, `schema_check`, integridad, backup/restore y registro de esquema de lotes.
- [x] 1.3 Probar migración desde 17, fallo, versión futura, backup/restore y preservación de dominio.

## 2. Proveedor y protocolo

- [x] 2.1 Implementar chat loopback OpenAI-compatible con validación de endpoint, timeout, tope de respuesta y errores estructurados.
- [x] 2.2 Implementar parser estricto del protocolo (`tool_call`/`final`), rechazo de campos/acciones desconocidas y límites de contexto.
- [x] 2.3 Probar éxito, JSON inválido, herramienta prohibida, error HTTP, timeout y respuesta grande con servidor simulado.

## 3. Herramientas y citas

- [x] 3.1 Implementar despacho de herramientas públicas con DTOs originales y sin SQL/shell/archivos.
- [x] 3.2 Implementar preview/apply con recibo de la misma ejecución y `allow_apply`; probar recibo alterado y apply sin preview.
- [x] 3.3 Implementar verificación de citas, insuficiencia explícita y errores `citation_required`/`citation_unverified`.
- [x] 3.4 Probar respuestas con citas válidas, citas inventadas, texto ausente y `retrieval.search` con fuentes.

## 4. Límites, cancelación y trazas

- [x] 4.1 Implementar límites de pasos/contexto/herramienta/tiempo/tokens y reserva de concurrencia.
- [x] 4.2 Implementar `agent.cancel` cooperativo y estados `cancelled`/`limited`.
- [x] 4.3 Implementar persistencia/consulta de trazas con paginación y tope de 1 MiB.
- [x] 4.4 Probar cancelación entre procesos, segunda ejecución concurrente, deadline, límite de pasos y traza consultable.

## 5. Superficies y documentación

- [x] 5.1 Registrar operaciones en catálogo, CLI, scopes y MCP; actualizar conteos y pruebas.
- [x] 5.2 Documentar `docs/agent-harness.md`, ADR 0030, plan, configuración y desarrollo.
- [x] 5.3 Verificar que no se activan modelos/proveedores reales y que las pruebas no atribuyen calidad de inferencia.

## 6. Verificación y cierre

- [x] 6.1 Ejecutar pruebas Rust por perfil, CLI/MCP, contratos, fmt, Clippy, `git diff --check` y OpenSpec estricto.
- [x] 6.2 Revisar diff/diseño/escenarios y conservar fallos previos.
- [x] 6.3 Sincronizar deltas terminados y archivar `add-agent-harness`; detenerse antes de fase 6.
