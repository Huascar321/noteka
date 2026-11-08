# Proposal

## Why

El calendario Due ya tiene vidrio y un pop desde una esquina, pero no expresa la continuidad entre el campo y el panel que caracteriza las presentaciones de Liquid Glass de Apple. El usuario pide investigar ese movimiento e intentar replicarlo en el calendario de la referencia.

## What Changes

- Transformar la superficie desde la geometría real de Due hasta el calendario, con expansión elástica y retorno al campo al cerrar.
- Separar material y contenido para evitar aplastar texto durante el morph.
- Conservar selección, guardado, teclado, foco, anclaje, temas y alternativas accesibles.
- Documentar fuentes primarias, aproximación web y evidencia reproducible.
- Alcance limitado al picker Due de Task; no extender a otros menús ni introducir render nativo, refracción física, dependencias o cambios de datos.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: transición contextual accesible del calendario Due.

## Impact

`DueDatePicker.svelte`, estilos del calendario, recorrido headless y documentación visual. Rust, SQLite, contratos y operaciones permanecen iguales. La implementación web es una aproximación al movimiento nativo, sin depender de APIs de SwiftUI.
