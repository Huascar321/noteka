# Design

## Context

La referencia incluye Voice aunque tenga URL y excluye algunas otras notas con URL. El origen local puede estar en bloques importados y Home no expande documentos; no se debe inferir ausencia de URL ni escanear documentos sin límites.

## Goals / Non-Goals

**Goals:** incluir explícitamente Voice en Notes y preservar los contratos anteriores.

**Non-Goals:** implementar grabación, clasificación por URL sin proyección canónica, migración o paridad no comprobada.

## Decisions

- Agregar `notes_general` con la regla local anterior más Voice, filtrando biblioteca y archivo como antes; la vista Notes usa la proyección nueva.
- Conservar `notes_library`, Voice, All y búsqueda intactas. Documentar que URL y Daily no están plenamente modelados.

## Risks / Trade-offs

- [Filtro de URL] → la vista sigue siendo una aproximación parcial, no presentar equivalencia completa ni modificar documentos importados.
- [Compatibilidad] → variante aditiva; contratos anteriores y datos permanecen intactos.
