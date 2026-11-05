# Design

## Context

El checkpoint `2d43a66` conserva las tres escenas. `fd4f436` conserva la primera composición mixta y la corrección del lateral. El usuario descartó la portada mixta luego de verla y pide volver a la alternancia. Las tareas/evidencias anteriores documentan ese experimento, no una aceptación vigente de su apariencia.

## Goals / Non-Goals

**Goals:** recuperar la portada anterior de forma exacta, preservar el lateral sin scrollbar horizontal y conservar los demás avances.

**Non-Goals:** nuevos modos, ajustes de geometría, cambios de contenido o reinicios forzados de la sesión.

## Decisions

- Restaurar AsciiCoverRenderer, AsciiCreaturesRenderer y AsciiOceanRenderer desde `2d43a66`. Selector, geometría, AsciiShark y OceanClock ya coinciden con ese checkpoint. Verificar equivalencia de archivos y transiciones, sin rehacer las escenas.
- Retirar mixed-ocean-scene y su prueba específica al quedar fuera de uso. El historial del primer experimento sigue disponible en `fd4f436`; ADR0053 pasa a descartada y vuelve a regir ADR0052.
- Conservar App/style del arreglo del lateral: cuerpo con overflow-y:auto separado del handle interior. No se revierte el commit completo ni los anchos recordados, peeks, editor o indicador.
- El delta final sólo modifica Lateral plegable sin reconstruir pestañas. Los requisitos de portada pertenecen a `add-random-ascii-cover-scenes` y se conservan; se elimina de este delta la sustitución por composición mixta.

## Risks / Trade-offs

- [Deshacer mejoras ajenas al ASCII] → restauración por archivos, comparación de App/style con `fd4f436`.
- [Bitmap vacío o transición perdida] → pruebas existentes de selector/reloj, matriz por tema/ancho y muestreo del fundido.
- [Documentación que aún exige mezcla] → guías actuales de tres escenas; anteriores como evidencia histórica.

## Migration Plan

Sin migración, datos o preferencia nuevos. La instancia nativa con Vite recibe la actualización sin cerrar borradores. Deltas pendientes de sincronización/archivo; no publicar ni modificar las specs principales como parte de esta restitución.
