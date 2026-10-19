# Design

## Context

El núcleo v1 comparte operaciones con Tauri/CLI. La UI tiene buffers y conflictos probados. Ya se archivó el bootstrap; se conserva el contrato anterior al extender el editor.

## Goals / Non-Goals

**Goals:** edición rica local, referencias seguras, originales en backup, exportación recuperable.

**Non-Goals:** IA, Internet, importación Notion, toggles/columnas/tablas, eliminación física de originales y procesamiento OCR.

## Decisions

Documento v2: árbol de bloques con ID globalmente único, type, content inline, children y attrs opcionales. V1 mantiene su serialización exacta. Marcas bold/italic/strike/code/link y saltos de línea; tipos paragraph/heading/bullet_list/ordered_list/list_item/task_list/task_item/blockquote/horizontal_rule/code_block/image/file. Rust valida límites de 1 MiB, 10.000 bloques, profundidad 32 y forma/contexto de cada nodo. Sólo enlaces http/https/mailto son admitidos y no se abren al editar.

Tiptap 3 + StarterKit/UniqueID/TaskList/TaskItem; adaptador explícito entre JSON de presentación y documento semántico. UniqueID conserva IDs al editar/split/paste; Rust sigue rechazando duplicados. El buffer se reemplaza sólo por navegación/recarga autorizada, no por cada transacción del editor.

SQLite v2 añade attachments, sin reescribir documentos. Migración con backup previo y transacción; formatos futuros se rechazan antes de escribir. Archivos inmutables en attachments/<uuid>, sin nombres/rutas del usuario como componentes. Importación desde ruta explícitamente elegida, copia limitada a 64 MiB, hash streaming, publicación del archivo antes del commit; fallo elimina sólo la copia propia. No hay garbage collection automática.

Backup usa snapshot SQLite y enumera adjuntos desde ese snapshot. Como originales son inmutables y no se borran, puede copiar el mismo estado confirmado. Manifiesto v2 valida lista exacta de archivos administrados contra el snapshot y sus hashes; se mantiene lectura de backups v1 sin adjuntos. Restauración valida staging antes de publicar. Exportación JSON de nota incluye entity/document/relaciones como referencias y todos sus adjuntos; Markdown explicita pérdidas. No se declara importación Markdown hasta implementarla en otro cambio acotado.

## Risks / Trade-offs

- Copias sin referencias tras crash son huérfanos recuperables; no se borran automáticamente.
- Renderizar imágenes sólo usa data URLs locales; pegar una imagen remota no habilita descarga.
- El paquete por nota conserva relaciones como IDs, no exporta por accidente todo el workspace.
- Elecciones de UX y límites se documentan para revisión posterior; no se promete paridad con Notion.
