# Design

## Context

Las operaciones de relaciones son transacciones independientes con revisión esperada del origen. El borrador Svelte conserva título, payload y documento sin persistir.

## Goals / Non-Goals

**Goals:** confirmar vínculos sin guardar contenido y conservar la capacidad de guardar luego sin conflictos falsos ni sobrescribir concurrencia.

**Non-Goals:** autoguardar contenido, crear automáticamente páginas nuevas, cambiar el modelo de datos o permitir edición de páginas bloqueadas.

## Decisions

Se permite llamar `relation.add/remove` con borrador pendiente en una entidad ya persistida. Cuando el origen es la página abierta, se adopta sólo el resultado confirmado como base de revisión, sin sustituir ni serializar el borrador. Las relaciones entrantes usan la revisión de su origen real y no cambian la base del borrador. Ante conflicto de revisión del origen abierto se marca el conflicto y se conserva el contenido. La actualización asíncrona ignora lecturas obsoletas que llegaron durante o después de una escritura. Se siguen bloqueando escrituras simultáneas y se informa si la operación confirmó pero falló la actualización visual.

## Risks / Trade-offs

- [Una escritura externa entre operaciones] → CAS existente y pruebas de conflicto real, sin adoptar revisiones ajenas.
- [Lectura anterior a una mutación] → comprobación de estado y revisión antes de actualizar la UI.

## Migration Plan

No hay migración; almacenamiento y contratos son los existentes.
