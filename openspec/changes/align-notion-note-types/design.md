# Design

## Context

Los valores provienen del selector inspeccionado en Notion. Daily y Journal son opciones distintas y la vista Journal incluye ambas. Clientes anteriores con enums cerrados no pueden editar tipos nuevos.

## Goals / Non-Goals

Conservar la distinción de tipos y filtrar antes de paginar; no implementar colecciones genéricas, sincronización ni suponer URL.

## Decisions

El enum Rust suma variantes aditivas sin DDL. Una consulta `notes_diary` devuelve Daily/Journal y la UI la usa; `notes_journal` sigue filtrando sólo Journal. Notes generales excluye Daily. Los nombres y opciones se comparten en la interfaz; el recuperador conserva sus guardias de backup/CAS.

## Risks / Trade-offs

Clientes antiguos requieren contratos nuevos para editar esos registros. La recuperación sobre datos existentes requiere una acción explícita posterior; este cambio no la ejecuta.
