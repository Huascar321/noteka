# Proposal

## Why

El usuario pide quitar Navegación y Actualizar Home del encabezado y actualizar mediante ⌘R, Ctrl+R o F5. Los encabezados de sección y el lateral ya ofrecen navegación.

## What Changes

- Retirar ambos controles de Home y conservar los accesos desde las secciones.
- Actualizar las lecturas del Home activo con ⌘R/Ctrl+R o F5, interceptando la recarga del WebView.
- Conservar borradores y pestañas; impedir el atajo durante diálogos o estados bloqueados y retirar el listener al desmontar.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: encabezado Home simplificado y actualización por teclado.

## Impact

HomePanel y paso de blocked desde WorkspacePane; adaptación de cuatro llamadas existentes del recorrido Home a F5, documentación y spec desktop-shell. Sin cambios Rust, API, SQLite, dependencias ni datos. Compilación y revisión de código; sin suites no solicitadas.
