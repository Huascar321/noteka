# Proposal

## Why

Goals todavía es un aviso pendiente y sus páginas importadas no tienen estado,
fechas o progreso tipados. El usuario pide Goals entre Tags y People y una
página con las vistas y Milestones de su referencia Notion.

## What Changes

- Séptima entrada lateral Goals, icono propio, búsqueda/navegación/breadcrumb
  canónicos y página con Nav, By Activity/By Deadline/By Tag/Achieved, grupos
  Active/Dream/Achieved, contexto, progreso y fecha.
- Goal y Milestone como subtipos tipados de las páginas `note` existentes,
  conservando IDs, documentos y operaciones compartidas. Estado Dream por
  defecto, fechas civiles, hito con un Goal opcional y progreso derivado en Rust.
- Lecturas Goals/Milestones acotadas en home.query, incluyendo List de hitos
  completados y lectura de todos los hitos de un Goal en su detalle editable.
- Recuperación explícita de metadata/relaciones por IDs de origen, backup/CAS e
  informe privado. Actividad importada con snapshot y revisiones para distinguir
  recuperación de futuras ediciones locales, sin copiar porcentajes.
- Se reconcilian las reglas vigentes de menú, Home y contexto de Projects.
- Goals real sustituye el aviso pendiente de Home. Milestones no crea una
  entrada lateral adicional. Se conservan guardias, temas y acciones de fila.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `knowledge-entities`: subtipos Goal/Milestone, validación e integridad sin
  reemplazar el tipo persistido ni documentos.
- `productivity-workflows`: consultas y progreso de objetivos compartidos,
  actividad, paginación, relaciones y cobertura.
- `desktop-shell`: Goals lateral, página, detalle y sección local en Home.

## Impact

Rust/contracts/HomeQuery, consultas y validación de entidades/relaciones;
Svelte/navigation/iconos/estilos; helper de recuperación, ADR y guías. La tabla
SQLite y EntityKind permanecen compatibles; payloads añaden campos opcionales,
sin migración SQL ni nueva operación pública. Clientes de payload cerrado deben
actualizarse para editar los nuevos campos. No se implementan sincronización,
colaboración, Action Center de Notion, fórmulas universales, proveedores ni
publicación. El usuario decide posición/composición; el subtipo sobre identidad
existente y las reglas acotadas de actividad son decisiones de implementación.
