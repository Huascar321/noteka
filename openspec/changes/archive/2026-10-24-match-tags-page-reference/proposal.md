# Proposal

## Why

La entrada lateral Tags todavía abre el listado genérico, mientras la referencia
solicitada presenta una página con navegación, captura y vistas de relaciones.
Los resúmenes y cuatro consultas de Tags ya existen en el núcleo local.

## What Changes

- Página Tags con icono sobre título, Nav a Tasks/Notes/Projects, New Tag y
  Recent inicial, reutilizando breadcrumb e iconos propios.
- Cinco pestañas: Recent, Fav., A-Z, Types y Table. Table presenta las mismas
  proyecciones locales en columnas, ordenadas con `tags_alphabetical` existente.
- Listas compactas con última nota y tipo; acciones New page/Load more de ancho
  completo y captura protegida. Actualización con ⌘R/Ctrl+R/F5.
- Alcance: composición de la página lateral. Se mantienen las variantes de Home
  y de Tags incrustado en Notes, sus consultas y los originales locales.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: composición de Tags lateral y presentación tabular local.

## Impact

Nuevo TagsPage, variante standalone de HomeTagSection, tabla local y conexión
en WorkspacePane, estilos y documentación. Sin API, dependencia, migración,
cambio Rust ni modificación de entidades o documentos. La referencia decide
la composición; Table como presentación de A-Z es una elección de implementación
documentada. No requiere acceso externo ni sincroniza actividad con Notion.
