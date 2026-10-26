# Proposal

## Why

Notes mezcla 1.172 páginas importadas, incluidas páginas de sistema, y no refleja la base de notas de la referencia. El usuario pide el mismo ajuste aplicado a Tasks, incluidos arte original y metadatos reales.

## What Changes

- Encabezado, New Note, pestañas Notes/Inbox/Fav./Clips/Voice/Journal/Meetings/All, filas compactas con contexto, tipo y antigüedad.
- Clasificación y favoritos opcionales de nota; procedencia importada explícita para separar Notes de otras páginas sin eliminarlas. Proyecciones y paginación en Rust.
- Recuperación acotada desde export/manifiesto y referencias autorizadas, con backup, revisión esperada e informes personales fuera de Git. Iconos de imagen usan los adjuntos ya soportados.
- Conservar documentos/IDs, historial local y acceso a todas las páginas mediante All y búsqueda. Sin grabación de voz, scraping de contenido de notas, nuevos servicios remotos ni suites no solicitadas.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `knowledge-entities`: tipo, favorito y procedencia opcionales de notas validados por el núcleo.
- `desktop-shell`: composición y vistas locales de Notes diferenciadas, con recuperación explícita de metadatos e iconos.

## Impact

Contratos Rust/TypeScript, validación de payload, home.query, Home, edición de propiedades y un importador de enriquecimiento. Campos aditivos en JSON existente, sin migración SQLite ni reescritura del documento. Los clientes anteriores conservan lectura estructurada, pero necesitan actualizarse para editar los payloads enriquecidos. Compilación e inspección visual acotadas; no se acredita una nueva aceptación Tauri.
