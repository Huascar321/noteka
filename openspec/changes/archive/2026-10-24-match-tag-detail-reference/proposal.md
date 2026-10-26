# Proposal

## Why

El usuario pidió adaptar el detalle de Tags a la misma referencia real de Notion usada para Notes/Projects. El Tag se identificó por sus vínculos y su ID de origen, se capturó con Playwright y conserva en Noteka una presentación genérica de propiedades y registros mezclados.

## What Changes

- Detalle Tag con portada decorativa local, icono/título, Type/Favorite/Note Count compactos y metadatos de origen en View details.
- Contador desde el resumen Rust existente; relaciones de contexto sin duplicar todos los contenidos.
- Nav a Notes/Tags y secciones Notes/Clips a partir de los originales del contexto; Tasks conserva su captura/acceso en un grupo adicional.
- Conservar PageTools, condiciones, guardias, paginación, errores y teclado. No editar Notion ni recuperar datos como parte del ajuste visual.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: cabecera de Tag y composición de sus contenidos relacionados.

## Impact

Svelte/CSS y documentación. Reutilizar CRUD/CAS, workflow.query/context, home.query/tags_alphabetical y PageTools. Sin cambios Rust/contratos/SQLite/documentos/dependencias ni datos personales. Compilar y revisar UI sobre copia restaurada con escrituras bloqueadas; conservar trabajo previo.

## Non-goals

Sin jerarquía Parent/Sub-Tags, relaciones People/Goals inventadas, nuevas consultas/vistas de dominio, captura web/audio, documento enriquecido de Tag, cambios de biblioteca o sincronización Notion. Meeting Notes sigue propuesto 0/19. Eco Armario y voicebot quedan fuera de alcance; sin pantalla nativa, publicación ni Git commit/push.
