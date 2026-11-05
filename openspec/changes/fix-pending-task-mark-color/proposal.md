# Proposal

## Why

El marcador azul de tareas pendientes toma el color de texto del acento, que
puede ser negro. El usuario pide que el guion sea blanco.

## What Changes

- Color blanco para el glifo del marcador de tarea pendiente compartido por
  Home, Tasks y contenidos de Project, conservando fondo y comportamiento.
- El marcador de tarea completada conserva su presentación actual.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: color del guion de tareas pendientes.

## Impact

Una regla CSS del marcador compartido. Sin cambios Rust, datos, operaciones,
dependencias o eventos. Verificación por build y revisión visual headless.
