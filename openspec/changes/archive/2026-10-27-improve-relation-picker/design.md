# Design

## Context

Las operaciones existentes validan tipos, revisión y direccionalidad. `entity.list` pagina entidades locales sin requerir índice FTS; las relaciones entrantes pueden tener un origen distinto de la página abierta.

## Goals / Non-Goals

**Goals:** selector compacto, búsqueda por nombre, paginación, identificación de existentes y protección de borradores.

**Non-Goals:** nuevos tipos, migraciones, creación de entidades o indexación.

## Decisions

El selector usa páginas de `entity.list` y filtra títulos localmente, recorriendo páginas hasta obtener resultados o agotar cobertura; indica si quedan más. No depende de FTS potencialmente incompleto. Las relaciones actuales aparecen aunque el destino esté archivado. El núcleo continúa validando vínculos, CAS y restricciones de Goal. Sólo se ofrecen tipos compatibles con el origen; entradas se muestran y quitan con la revisión de su propio origen. En edición pendiente se ofrece guardar explícitamente, sin descartar ni guardar de forma implícita. El panel se ancla al control y conserva foco, cierre y teclado.

## Risks / Trade-offs

- [Recorrer muchos registros sin filtro de título en API] → lectura progresiva con cancelación, resultados parciales, paginación y estados explícitos.
- [Cambios simultáneos] → conservar los controles de revisión y errores del núcleo.

## Migration Plan

No se cambia almacenamiento ni contratos. Volver a una versión anterior preserva los vínculos.
