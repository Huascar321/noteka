# Proposal

## Why

Abrir un detalle reemplaza hoy la vista de origen. El usuario pide conservar ese contexto y elegir una presentación similar a las referencias aportadas; confirmó Side peek a la derecha.

## What Changes

- Apertura de entidades y borradores en Side peek derecho inicial, Center peek, Full page o New tab.
- Selector Switch peek mode en el encabezado de todo detalle y preferencia local de apertura predeterminada, global para la aplicación y explícitamente identificada como tal.
- Conservación de la misma instancia de editor al cambiar presentación o promover a pestaña; guardias al cerrar/reemplazar y al abandonar el workspace/host.
- Contexto de origen conservado y scroll independiente; adaptación estrecha, teclado, foco y temas. Integración con el indicador persistente del cambio independiente.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: listados y detalles con presentación configurable, preferencia local y cierre agregado de previews/pestañas.

## Impact

App, navegación, WorkspacePane y un menú compartido de presentación; estilos e iconos locales. No cambia Rust, APIs, SQLite, documentos, guardado explícito ni proveedores. No se implementan colaboración, permisos, bloqueo, sincronización de pestañas ni preferencias por vista. Los peeks son estado de sesión; solamente se persiste el modo predeterminado.
