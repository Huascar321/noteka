# Design

## Context

La referencia usa Due, escala Week/Month y acceso No date; no se observó filtro de estado. Las consultas históricas sólo devuelven pendientes del período y sus contratos se preservan.

## Goals / Non-Goals

**Goals:** navegación civil en la zona del workspace, incluir completadas y sin fecha, conservar cursores y no guardar antes de confirmar.

**Non-Goals:** recurrencias, arrastre, automatizaciones remotas o cambios de consultas existentes.

## Decisions

- Agregar `tasks_calendar_week` y `tasks_calendar_month` con `as_of`: filtran el período antes de paginar y añaden tareas sin fecha o con fecha incompatible, sin filtrar estado. Fechas válidas primero; sin fecha al final; informar cobertura parcial.
- `onmonth` solicita al núcleo la fecha seleccionada, y Today vuelve al día de la zona del workspace. ISO week cruza correctamente años y se limita a fechas civiles representables.
- Un día prepara un borrador con Due, sin crear antes de Save. Abrir usa el original y sus guardias; conservar las consultas públicas anteriores y búsqueda.

## Risks / Trade-offs

- [Sin fecha puede superar página] → no presentar el conteo como total completo mientras exista cursor; Load more acumulativo.
- [Fechas incompatibles] → mostrar separadas sin cambiar el payload, incluso con estado desconocido.
