# Tasks

## 1. Núcleo

- [x] 1.1 Derivar `tasks_inbox` y `notes_inbox` de la ausencia de relación Project local, reutilizando el bloque de vínculos de Unclassified (Tasks: sólo Project; Notes Inbox: sólo Project; Notes Unclassified conserva Project/Tag/People).
- [x] 1.2 Derivar `workflow.query` Inbox (notas activas y tareas pendientes sin Project) y dejar `payload.inbox` fuera de ambos filtros.
- [x] 1.3 Actualizar las pruebas Rust de Home y workflow a la regla derivada, incluyendo Tag/People sin efecto en Inbox, proyecto archivado, quitar vínculo y bandera inerte.

## 2. Interfaz

- [x] 2.1 Retirar el control "Inbox" de tarea/nota, la tab Unclassified de Tasks y el seed de Inbox al crear notas; conservar Unclassified en Notes.
- [x] 2.2 Explicar la regla en las opciones de vista y en una línea de estado de Inbox (Tasks y Notes).
- [x] 2.3 Reescribir el recorrido headless de Inbox derivado (Tasks/Notes, bandera inerte, Tag/People, proyecto, tabs) y actualizar el smoke nativo que usaba el control retirado.

## 3. Documentación y verificación

- [x] 3.1 Guías (`home`, `productivity`, `tasks-reference`, `notes-reference`, `notes-page`, `note-detail`, `task-page-content`, `collections`, `templates`, `development`), ADR0059, plan, review queue y nota de supersesión.
- [x] 3.2 Ejecutar `cargo test` focal, `npm run check`, `npm run build` y `npm run test:headless-inbox`; registrar evidencia en `docs/verification-derived-inbox.md`.

## Verification

Ver `docs/verification-derived-inbox.md`.
