# Proposal

## Why

Las notas enriquecidas ya necesitan un recorrido de trabajo offline, no entidades de productividad editables sólo por CLI. La fase 2 conecta captura, planificación civil y contextos compartidos sin IA ni calendarios externos.

## What Changes

- Crear/editar tareas, proyectos, tags y personas desde escritorio; asignar/quitar relaciones explícitas.
- Tareas con prioridad, vencimiento y fecha planificada civil; completar/reabrir conserva la misma identidad.
- Inbox explícito de notas/tareas y My Day determinista; pendientes y vencidas excluyen archivo y completadas.
- Dashboards de proyecto/tag/persona con entidades relacionadas, paginación y creación en contexto transaccional.
- Mantener payloads antiguos: valores nuevos tienen defaults no destructivos. Sin subtareas, recurrencias, calendario externo, colecciones genéricas ni modelos.

## Capabilities

### New Capabilities

- `productivity-workflows`: planificación local, consultas de trabajo, captura contextual y dashboards.

### Modified Capabilities

Ninguna: las operaciones básicas y estados anteriores se conservan; esta capability agrega campos y recorridos.

## Impact

Rust valida metadatos y filtros; operaciones `workflow.query` y `entity.create.context` compartidas por CLI/Tauri. Se amplía JSON payload sin cambiar SQLite ni convertir documentos. UI conserva buffers/conflictos para todos los tipos. Reglas de Inbox/My Day son decisiones propuestas explícitas y revisables en docs/review-queue.md, no clasificación automática ni promesa de copiar todo Ultimate Brain.
