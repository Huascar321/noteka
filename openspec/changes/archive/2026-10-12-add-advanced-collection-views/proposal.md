# Proposal

## Why

3a/3b ya conservan registros compartidos y calculan propiedades, pero sólo presentan tabla/lista y filtros AND simples. El primer tramo de 3c amplía consultas y presentaciones verificables sin mezclar todavía la evolución del editor de bloques.

## What Changes

- Agregar filtros tipados anidados AND/OR/NOT y comparadores, con límites, semántica explícita de vacíos y dinero exacto. Mantener `filters` AND existentes.
- Tablero agrupado por select/checkbox, calendario mensual de fechas/rangos civiles y galería de tarjetas textuales con apertura del original. Agrupamientos/localizadores se calculan en Rust, sin duplicar registros ni cambiar valores al presentar.
- Configuración persistente y consultas compartidas CLI/Tauri; formularios JSON/IDs protegidos, grupos vacíos visibles y paginación explícita por página de registros.
- Migración SQLite v6 con backup previo y sin reescribir vistas antiguas; proteger definiciones futuras y revisiones.
- **Non-goals:** drag-and-drop, calendarios externos, previews automáticos de adjuntos, templates, bases incrustadas, lotes de agente, IA y virtualización nueva. Templates/bases incrustadas siguen pendientes en otro cambio de 3c, no se dan por completados con estas vistas.

## Capabilities

### New Capabilities

Ninguna: se amplía la capability existente de consultas/presentaciones de colecciones.

### Modified Capabilities

- `collection-views`: árboles de filtros, tablero/calendario/galería y conservación de configuración e identidad compartida.

## Impact

`database_contracts.rs`, consultas/validación de `database.rs` y un módulo de proyección; migración 006/recuperación, contratos generados, `DatabasePanel.svelte`, tests Rust/CLI/nativos y documentación. Se conservan catálogo y operaciones existentes `view.save`/`database.query`; sus DTOs se amplían. No se necesitan proveedores, cuentas, modelos ni librerías de calendario. Alcance del usuario: vistas avanzadas; elecciones propuestas: agrupación select/checkbox, calendario civil explícito y galería textual inicial. Ninguna dependencia externa bloquea este tramo.
