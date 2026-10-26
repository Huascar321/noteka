# Proposal

## Why

People lateral conserva el listado genérico aunque Home ya ofrece la galería y
las siete vistas reales. El usuario pide la composición de la página Notion.

## What Changes

- Página People con icono sobre título, Nav, New Person y Contacts inicial.
- Galería, fotos originales y siete vistas compartidas con Home; Load more
  acumulativo y creación/apertura con las guardias existentes.
- Nav enlaza Contacts, Pipeline y Cal al calendario local de cumpleaños.
  Actualización activa, temas, breadcrumb y ancho estrecho reutilizados.
- Meeting Notes se diseña en un cambio separado por su dominio de audio,
  transcripción, hablantes y recuperación; no se simulan esas funciones aquí.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: composición de People lateral con los datos/vistas existentes.

## Impact

Componentes Svelte, ruta People y estilos acotados; guías y revisión. Sin cambios
Rust, contratos, operación pública, migración ni recuperación de datos personales.
La posición y composición vienen del usuario; reutilizar HomeReader y el calendario
de cumpleaños son decisiones de implementación documentadas, sin calendario externo.
