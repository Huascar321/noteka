# Proposal

## Why

El usuario pidió retirar la ayuda inferior de atajos que aparece en algunos editores.

## What Changes

- Eliminar el texto «/ for blocks · ⌘Z to undo» de todas las instancias del editor.
- Conservar los comandos de bloques, deshacer y la edición existente.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `block-editor`: retirar la ayuda inferior de atajos de la interfaz compartida.

## Impact

Componente Svelte y estilos del editor; sin cambios de persistencia, contratos ni dependencias.
