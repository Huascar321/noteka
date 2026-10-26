# Proposal

## Why

La entrada Notes del menú lateral todavía abre el listado genérico de entidades. El usuario pidió que esa página siga la composición de sus capturas de Notion, como ya ocurre con Tasks.

## What Changes

- Página Notes amplia con icono propio sobre título, Nav e Inbox/Fav./All/Tags, New Note y las ocho vistas locales existentes.
- Filas con arte, relaciones, tipo y antigüedad reales; páginas acumulativas de diez y acciones textuales de ancho completo.
- Tags debajo, inicialmente en Recent, con sus cuatro vistas, última nota y clasificación, reutilizando consultas locales.
- Breadcrumb Home → Notes, apertura/captura protegidas, actualización por atajo, ambos temas y adaptación a ancho estrecho.
- Se reutilizan datos y reglas existentes. No se importan datos, cambian documentos, agregan proveedores ni simulan controles remotos de Notion. El editor de una nota individual conserva su composición.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: composición propia de la página Notes y sección Tags integrada.

## Impact

NotesPage y variantes de HomeNoteSection/HomeTagSection, integración en WorkspacePane y estilos acotados. Se reutilizan HomeReader, home.query, AppIcon y guardias existentes. Sin migración SQLite, cambio de contratos/API, dependencias o escrituras de datos. Compilación e inspección visual manual sobre copia local; no se agregan ni ejecutan suites.
