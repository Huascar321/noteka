# Spec Delta

## ADDED Requirements

### Requirement: Operaciones versionadas de recuperación semántica

El núcleo SHALL exponer `embedding.index` (status/build/reset) y `retrieval.search` (híbrido/semántico/léxico) dentro de API v1, con esquemas de entrada, salida y errores. MUST aplicar las mismas validaciones, límites y permisos en interfaz, CLI y MCP; build/reset requieren escritura, status y search son lectura. MUST NOT aceptar SQL, shell, rutas de modelos, credenciales ni endpoints no loopback.

#### Scenario: Catálogo, CLI y MCP coherentes
- **WHEN** se consulta el catálogo o se ejecutan las nuevas operaciones por CLI y MCP
- **THEN** los nombres, esquemas, acciones y metadatos de lote coinciden, `embedding.index status` y `retrieval.search` funcionan en scope de lectura, y build/reset requieren escritura

#### Scenario: Entrada inválida o prohibida
- **WHEN** se envía una acción desconocida, un campo adicional, un límite fuera de rango, un endpoint remoto o una operación no incluida en el scope
- **THEN** el núcleo rechaza la solicitud con validación o permiso denegado, sin crear generaciones ni modificar datos
