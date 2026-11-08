# Proposal

## Why

El usuario aprobó una identidad propia para Noteka: una «n» azul con una esquina de nota adhesiva despegada y un nombre de peso liviano. La app conserva `n.` y un ícono verde anterior; hay que integrar la identidad aprobada y el material nativo de macOS.

## What Changes

- Vectorizar la marca con el espacio de la esquina levantada realmente vacío, solapa clara y geometría compartida.
- Reemplazar la marca de bienvenida, sidebar y favicon, conservando densidad, temas y acciones existentes.
- Proveer un documento por capas de Icon Composer, íconos de compatibilidad y generación local reproducible.
- Integrar el catálogo nativo Liquid Glass en macOS, también en el paquete temporal de desarrollo; comprobar builds y recursos sin abrir datos personales.
- Documentar la identidad y su verificación. No incluye publicación, firma de distribución, cambios de dominio ni alteraciones de datos.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: identidad visual local coherente y recursos de ícono nativo/compatibilidad.

## Impact

`src/App.svelte`, estilos de marca, un componente visual compartido, recursos públicos y `src-tauri/icons/`, configuración/paquete temporal de macOS y tooling local de generación. Se reutilizan Xcode/Icon Composer instalados y las dependencias existentes. Rust cambia sólo en integración de recursos de escritorio, sin API, SQLite, migraciones o proveedores.
