# Proposal

## Why

Inbox y Unclassified describen hoy dos cosas distintas: Inbox es una bandera
manual (`payload.inbox`, por defecto `true`) y Unclassified una vista derivada de
relaciones. En Tasks eso obliga a mantener dos conceptos para el mismo caso de
uso y deja tareas "sin clasificar" dentro de Inbox, o fuera de Inbox con una
marca que ya no aporta información.

El usuario quiere que Inbox sea el depósito automático de lo que todavía no
pertenece a un proyecto: al vincular la tarea o nota a un proyecto, abandona
Inbox; al quitar el vínculo, vuelve. Sin marca manual que mantener.

## What Changes

- Inbox de Tasks pasa a ser **tareas pendientes (`todo`/`doing`) sin relación
  local a un Project** (`todo`/`doing`, igual que la vista Unclassified de
  tareas).
- Inbox de Notes pasa a ser **notas activas sin relación local a un Project**;
  los vínculos a Tag o People no las excluyen. Las notas importadas fuera de la
  biblioteca también cuentan.
- La vista lateral/workflow Inbox usa la misma regla para notas y tareas
  pendientes.
- Se retira el control "Inbox" de las propiedades de tarea/nota y la tab
  Unclassified de Tasks en la interfaz (el núcleo conserva la operación
  `tasks_unclassified` como alias compatible).
- Notes conserva su vista Unclassified (biblioteca sin Tag/Project/People), que
  sigue siendo más estricta que Inbox.
- `payload.inbox` permanece almacenado y expuesto en metadatos, pero deja de
  decidir pertenencia: no hay migración ni descarte de datos.

## Capabilities

### Modified Capabilities

- `productivity-workflows`: Inbox derivado de relaciones Project.
- `desktop-shell`: Inbox en Tasks/Notes, retiro del control manual y de la tab
  Unclassified de Tasks.

## Impact

Rust (`home.rs`, `productivity.rs`), pruebas Rust y headless, Svelte
(`EntityFields`, `HomeTaskSection`, `HomeNoteSection`), docs y specs. Sin
cambios de esquema SQLite, migraciones ni dependencias. Cambia el resultado de
`home.query` (`tasks_inbox`, `notes_inbox`) y `workflow.query` (`inbox`), no sus
contratos.
