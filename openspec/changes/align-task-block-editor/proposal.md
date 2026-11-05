# Proposal

## Why

El usuario pidió que la escritura en bloques de Tasks se comporte como la de Notes.

## What Changes

- Activar en Task la edición contextual compartida de Notes, incluidos formato, menú `/`, controles por bloque y continuación desde el espacio inferior.
- Mantener las estructuras iniciales de Task, el guardado explícito y sus documentos opcionales.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `block-editor`: ofrecer en Tasks la misma interacción contextual que en Notes.

## Impact

Editor Svelte compartido, estilos y revisión con workspace sintético. Sin operaciones Rust, migración, formato nuevo ni dependencias.
