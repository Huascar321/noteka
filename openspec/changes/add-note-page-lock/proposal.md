# Proposal

## Why

El usuario quiere evitar ediciones accidentales con `Lock page` en el menú `…`.
Necesita distinguir el bloqueo guardado de una apertura temporal para editar,
con indicadores `Locked` y `Re-lock` que sólo existan después de activar el bloqueo.

## What Changes

- Preferencia opcional `page_locked` validada por Rust y guardada en el payload
  original de Note, con revisión esperada; ausencia significa no bloqueada.
- Opción con icono y switch en el menú de Note normal; indicador de cabecera
  `Locked` para desbloquear temporalmente y `Re-lock` para volver a bloquear.
- Bloqueo de edición de título, cuerpo, icono, propiedades, relaciones y
  asociaciones externas en la interfaz; lectura, navegación y exportación siguen
  disponibles. Archive/Restore requiere desbloquear primero.
- Guardias existentes para resolver cambios pendientes antes de bloquear o
  volver a bloquear. El desbloqueo temporal no escribe la preferencia y se pierde
  al cerrar/navegar fuera de esa instancia de página.
- Regeneración del contrato compartido, pruebas de persistencia/recuperación y UI.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `knowledge-entities`: preferencia opcional de bloqueo en metadatos de Notes.
- `block-editor`: protección de edición y ciclo bloqueado/temporal/permanente.

## Impact

Contratos Rust, componentes Svelte y CSS; se apoya en el menú implementado por
`refine-notion-note-editor`, todavía activo. No cambia operaciones, tablas,
documentos v1/v2/v3, dependencias ni versión SQLite. ADR0054 documentará formato,
compatibilidad y reversión antes de modificar persistencia. Es una protección
de interfaz contra accidentes; no añade permisos ni impide operaciones explícitas
de CLI/MCP. No activa proveedores, descarga modelos ni usa datos personales.
