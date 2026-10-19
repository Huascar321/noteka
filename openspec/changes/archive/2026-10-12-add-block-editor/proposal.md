# Proposal

## Why

La fase 0 conserva texto local, pero no permite organizar bloques enriquecidos ni adjuntos. Esta entrega amplía la captura sin atar el formato persistente a Tiptap ni perder documentos antiguos.

## What Changes

- Documento semántico v2 con párrafos enriquecidos, encabezados, listas, checklists, citas, divisores, código, imágenes y archivos; lectura de v1 sin migración destructiva.
- Editor Tiptap local con IDs estables, comandos de bloque, historial, guardado explícito y protección de conflictos/buffer existente.
- Adjuntos originales administrados, importación por copia con hash, referencias validadas y backups que incluyen originales.
- Exportación fiel JSON/directorio y Markdown con advertencias explícitas; lectura de JSON desconocido conservada.
- Migración SQLite v2 con backup previo. Sin IA, red, sincronización, toggles, columnas ni tablas enriquecidas.

## Capabilities

### New Capabilities

- `block-editor`: edición enriquecida local y adaptación semántica versionada.
- `managed-attachments`: originales, integridad, referencias y exportación portable.

### Modified Capabilities

- `local-workspace`: extender backups/restauración a originales administrados.

## Impact

Rust valida documentos/referencias y publica operaciones de adjuntos/exportación; CLI y Tauri comparten reglas. Tiptap 3 se integra sin servicios comerciales. La v1 sigue legible; sólo una edición explícita crea v2. Las revisiones de UX que requieran al usuario se registran en docs/review-queue.md, sin bloquear desarrollo.
