# Proposal

## Why

Obligar a guardar el contenido antes de editar una relación agrega pasos y guarda cambios que podrían no estar listos.

## What Changes

- Permitir vincular o desvincular en páginas guardadas con borrador pendiente, sin guardar ni descartar ese borrador.
- Conservar revisión, conflictos y protección de páginas bloqueadas; páginas todavía no creadas requieren su primer guardado.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: edición independiente de relaciones con contenido pendiente.

## Impact

Estado y presentación Svelte; operaciones Rust y almacenamiento existentes, sin migración.
