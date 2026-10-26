# Proposal

## Why

El usuario pide un indicador persistente como la referencia: una marca por título
y una marca activa que siga la posición de scroll. Home y las Notes largas no
ofrecen hoy esa orientación fuera de su scrollbar normal.

## What Changes

- Rail discreto y persistente a la derecha del contenido, siempre visible en la
  pestaña activa de Home y del detalle de una Note normal.
- Home representa Tasks, Notes, Projects, Tags, Goals y People, en ese orden.
- Notes representa el título de página y sus encabezados H1/H2/H3 visibles; una
  Note sin encabezados conserva una marca para su título.
- Resaltado por posición de scroll, actualización al editar/refluir/cargar contenido,
  tooltip/nombre accesible y navegación local al pulsar una marca.
- Conservación de scroll por pestaña, borradores, IDs, foco de escritura y temas.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: orientación y navegación persistentes por secciones de Home y Notes.

## Impact

Componente Svelte compartido, integración en Home/WorkspacePane y CSS. Sin
dependencias, contratos, Rust, migraciones o escrituras de documentos. No agrega
un bloque de tabla de contenido, no indexa notas y no activa fuentes externas.
Otros detalles y listados conservan su composición. Meeting Notes no participa.
