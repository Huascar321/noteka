# Proposal

## Why

La portada ASCII de Home se detenía al abrir Settings y al pasar un detalle a
Center peek, aunque la portada siguiera visible detrás del popup. Search tenía
una excepción aislada; Settings heredaba `blocked` y Center peek el cambio de
foco. El usuario pide que la animación continúe con cualquier popup delante.

## What Changes

- La animación ASCII se decide por la visibilidad de la portada de Home, no por
  el foco de edición ni por `blocked`.
- Settings, Search y Center peek mantienen la animación mientras Home siga
  visible; los controles del fondo siguen bloqueados o inertes.
- Se conservan las pausas por pestaña inactiva, portada fuera de vista, documento
  oculto, movimiento reducido y Full page que cubre Home.
- Se retiran `searchOpen` y `animateAlongsideSidePeek` del camino de la portada
  al quedar cubiertos por la visibilidad real.
- Auditoría del resto de popups: la guardia de cambios pendientes, el diálogo
  de mover Project, menús, pickers, paneles plegables y herramientas no
  suspenden el renderer.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: la portada ASCII continúa animándose bajo popups y Center
  peek, y sólo se pausa cuando deja de ser visible.

## Impact

Svelte (`App`, `WorkspacePane`, `HomePanel`), una prueba headless nueva y
documentación. Sin cambios de datos, Rust, SQLite, contratos, dependencias,
preferencias, red ni migraciones.
