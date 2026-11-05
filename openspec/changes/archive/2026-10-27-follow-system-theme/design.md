# Design

## Context

La preferencia visual se guarda localmente como `dark` o `light`, mientras App aplica el tema al documento y calcula la paleta del acento.

## Goals / Non-Goals

**Goals:** mantener la selección separada del tema efectivo y reaccionar a los cambios del sistema.

**Non-Goals:** cambiar datos del workspace, tema nativo del sistema o proveedor externo.

## Decisions

Agregar `system` a la clave existente, conservando `dark` como fallback de valores ausentes, inválidos o inaccesibles y preservando preferencias anteriores. App observa `prefers-color-scheme` mediante un listener con limpieza y usa el tema resuelto tanto para el documento como para la paleta. Las opciones fijas ignoran los cambios del sistema. Frente a una API indisponible se conserva oscuro como fallback.

## Risks / Trade-offs

- [Diferencias entre WebViews] → verificar comportamiento con medios emulados y señalar la aceptación nativa pendiente por plataforma.

## Migration Plan

No hay migración de datos. Los valores anteriores se leen sin cambios; una versión anterior interpretaría `system` como oscuro.
