# Proposal

## Why

La ventana dedica una fila nativa sólo a Noteka y otra a las pestañas. El usuario pidió aprovechar ese espacio con el control del lateral y los tabs junto a los botones de Mac, una solución equivalente en Windows y una variante web.

## What Changes

- Una sola barra superior para el control de navegación, pestañas de sesión y nueva pestaña.
- macOS conserva los botones del sistema mediante título superpuesto y reserva de espacio; Windows usa barra propia con minimizar, maximizar/restaurar y cierre protegido.
- El lateral puede plegarse en escritorio; en anchos estrechos conserva un panel superpuesto con cierre accesible.
- Web presenta la misma navegación sin botones de ventana ni comandos nativos.
- Conservación de buffers, paneles, búsqueda, teclado y guardias existentes. Validación Windows nativa diferida explícitamente por el usuario.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: barra integrada por plataforma y control de lateral independiente del estado de las pestañas.

## Impact

Presentación Svelte/CSS, configuración Tauri por plataforma y permisos de ventana principal. Sin cambios de SQLite, entidades, documentos, API del núcleo ni dependencias. Meeting Notes conserva su diseño pendiente 0/19. No se publica ni modifica contenido personal; la lectura visual usa una copia aislada.
