# Proposal

## Why

El usuario eligió vidrio transparente para búsqueda global y menús flotantes, priorizando recursos.

## What Changes

- Aplicar una superficie translúcida legible a búsqueda y menús contextuales.
- Limitar el desenfoque a superficies abiertas y retirar el desenfoque de pantalla completa existente de búsqueda.
- Suspender la animación de Home cuando una ventana modal bloquea la interacción.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: presentación de búsqueda y superficies flotantes; pausa de portada ante bloqueo modal.

## Impact

Presentación y ciclo de animación; sin datos, migraciones, proveedores ni dependencias.
