# Design

## Context

HomeItem y ProjectContents comparten task-status-mark; HomeTaskSection reutiliza
HomeItem en Home y Tasks. Su color actual procede de accent-text, que puede ser negro.

## Goals / Non-Goals

**Goals:** guion pendiente blanco mediante una regla común.
**Non-Goals:** cambiar paleta, tarea completada, estados, datos o interacción.

## Decisions

Sobrescribir sólo task-status-mark:not(.complete) con #fff. La regla base y el
fondo conservan sus variables actuales; no se alteran componentes ni handlers.

## Risks / Trade-offs

El color del guion deja de depender de la tinta automática del acento por
pedido explícito del usuario. Reversión: retirar la regla, sin migraciones.
