# Proposal

## Why

El movimiento de Projects Board previsto por la spec no llega al contenido de la ventana nativa cuando Tauri intercepta el arrastre.

## What Changes

- Permitir que el WebView entregue el drag/drop HTML al tablero y al editor.
- Evitar navegación por archivos y enlaces soltados desde afuera sin importar contenido.
- Comprobar movimientos sobre workspace sintético, manteniendo CAS, recibo y payloads.
- Ante el reporte del 29/10, manejar el gesto de tarjeta por puntero y mostrar el destino durante el movimiento.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

Ninguna: se corrige la implementación del requisito vigente de `desktop-shell`.

## Impact

Configuración Tauri, protección de arrastres externos y pruebas; sin migraciones ni cambios de operaciones o datos personales.
