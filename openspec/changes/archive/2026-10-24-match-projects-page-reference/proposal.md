# Proposal

## Why

Projects del menú lateral todavía usa el listado genérico. El usuario pidió la composición de su referencia Notion, con Active inicial, seis vistas y accesos By Tag/By People.

## What Changes

- Página Projects amplia con icono propio sobre título, Nav, By Tag/By People, New Project y breadcrumb Home → Projects.
- Active inicial agrupado en Doing/Ongoing, filas con relaciones, pendientes/vencidas y progreso local; Planned, Board, Deadlines, Completed y On Hold reales.
- On Hold y agrupaciones por tag/persona se calculan en Rust sobre relaciones existentes, con identidades navegables y grupos sin relación explícitos.
- Captura protegida, filas textuales de ancho completo, paginación/actualización, temas y ancho estrecho con componentes compartidos.
- Home y Projects integrado en Tasks conservan sus composiciones. No se importan datos ni se cambian documentos, formatos persistentes, fórmulas remotas o permisos de colaboración.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: página Projects y consultas locales por estado/relación, compartidas por escritorio, CLI y MCP.

## Impact

ProjectsPage, variantes de HomeProjectSection y TaskPageProjectRow, WorkspacePane y estilos acotados. El catálogo existente agrega el SVG estático Pause del paquete fijado. home.query agrega tres vistas y una referencia opcional de grupo, con contratos generados; no aumenta operaciones, cuotas ni agrega migraciones/dependencias. Deadlines conserva la lista local por fecha existente. Compilación Rust/frontend e inspección visual manual sobre copia local, sin agregar ni ejecutar suites.
