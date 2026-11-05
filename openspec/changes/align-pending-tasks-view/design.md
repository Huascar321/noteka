# Design

## Context

Notion aplica `Status is not Done` a All tanto en Home como en Tasks. `tasks_all` ya es pública y agrupa todas las tareas, y otros clientes pueden depender de ese contrato.

## Goals / Non-Goals

**Goals:** una vista All de escritorio coherente con la referencia, sin alterar la consulta anterior, filtros Inbox ni preferencias previas de Today.

**Non-Goals:** modificar estados persistidos, resolver las funcionalidades aún pendientes, o equiparar fórmulas remotas no inspeccionadas.

## Decisions

- Agregar `tasks_pending` a `home.query`, compartiendo orden, grupos y paginación de `tasks_all` pero admitiendo sólo estados todo/doing interpretables.
- Conectar All en Home y Tasks a la proyección nueva; Done permanece separada. `tasks_all` y búsqueda conservan su semántica anterior.
- Actualizar tipos generados desde Rust y probar prefiltrado, cursores, enlaces, errores y transición de estado sin contenido personal.

## Risks / Trade-offs

- [Compatibilidad] → el enum crece de forma aditiva; clientes antiguos mantienen sus consultas y no deben enviar el nuevo valor hasta actualizarse.
- [Campos futuros] → estados incompatibles no se convierten a completados ni se pierden: la proyección cuenta su indisponibilidad y la consulta anterior sigue disponible.
