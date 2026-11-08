# Proposal

## Why

Las vistas de Tasks conservaban textos y controles que el usuario retiró en esta
iteración: Inbox repetía una nota explicativa, Week y Month listaban un bloque
`No date` y ofrecían Load more sobre un calendario paginado, y las lecturas de
Home avisaban con un texto técnico cuando una actualización reiniciaba páginas.
Además, Scheduled, No Due y All cargaban diez tareas por página en lugar de las
catorce solicitadas.

## What Changes

- Quitar la nota `Pending tasks without a local Project relation...` de Inbox,
  tanto en la página Tasks como en Home.
- Week y Month: eliminar el bloque `No date` y el mensaje de cobertura parcial;
  no ofrecer Load more y limitar el período a cien tareas cargadas.
- Scheduled, No Due y All: cargar catorce tareas por página; Load more acumula
  las páginas siguientes.
- Omitir el aviso `Additional pages were reset. Load more if available.` de las
  lecturas Home; una actualización exitosa sigue volviendo a la primera página.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: navegación, paginación y avisos de las vistas de Tasks.

## Impact

Sólo markup y estado de UI, textos y pruebas headless. Sin cambios de datos,
operaciones, contratos, SQLite, migraciones ni dependencias.
