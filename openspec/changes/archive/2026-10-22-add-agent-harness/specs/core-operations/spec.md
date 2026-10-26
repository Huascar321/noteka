# Spec Delta

## ADDED Requirements

### Requirement: Operaciones versionadas del harness de agentes

El núcleo SHALL exponer `agent.ask`, `agent.cancel` y `agent.trace` en API v1 con esquemas y errores. `agent.ask` MUST requerir ejecución, `agent.cancel` escritura y `agent.trace` lectura; CLI, interfaz y MCP MUST aplicar la misma política. MUST NOT aceptar SQL, shell, credenciales, endpoints remotos ni herramientas fuera de la lista.

#### Scenario: Catálogo y scopes coherentes
- **WHEN** se consulta el catálogo o se ejecutan las operaciones por CLI/MCP
- **THEN** los nombres, esquemas, acciones y permisos coinciden; `agent.trace` funciona en lectura y `agent.ask` requiere ejecución

#### Scenario: Operación prohibida
- **WHEN** se solicita una herramienta, endpoint o acción no incluida
- **THEN** el núcleo rechaza la solicitud sin iniciar proveedor, escribir traza ni modificar datos
