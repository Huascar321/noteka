# Proposal

## Why

Recuperar foco o activar una página puede iniciar lecturas duplicadas y Home retira sus resultados al comenzar. Las mediciones sintéticas muestran ocho consultas Home para una ráfaga de veinte eventos, sin demostrar saturación ni daño de datos.

## What Changes

- Coordinar eventos y lecturas por workspace, limitando concurrencia y una pasada posterior por consulta sin cancelar físicamente un IPC ya iniciado.
- Conservar datos visibles durante el refresco y ante fallo identificándolos como anteriores; reiniciar páginas adicionales al confirmar el resultado actualizado.
- Conservar fecha, filtros y borradores; detectar revisiones externas y evitar mover tarjetas usando un tablero no verificado.
- Medir comparación sintética y verificar el foco nativo autorizado, sin polling ni detector persistente.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: coordinación de refrescos, estados de vigencia y preservación de contenido durante actualizaciones.

## Impact

Coordinación de lecturas TypeScript/Svelte y pruebas; no se modifican contratos Rust, formato SQLite, datos personales o capacidades de los lectores especializados.
