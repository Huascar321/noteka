# Proposal

## Why

La búsqueda modal detenía la portada ASCII de Home y no se cerraba al hacer clic fuera del panel, lo que obligaba a usar Escape o el botón Close.

## What Changes

- Mantener la animación ASCII activa mientras la búsqueda está abierta, sin habilitar los controles bloqueados del fondo.
- Cerrar el diálogo al hacer clic en el backdrop; los clics dentro del panel no lo cierran.
- Conservar Escape y el botón Close.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: interacción del popup de búsqueda.

## Impact

Svelte y documentación; sin cambios de datos, contratos, dependencias ni CSS.
