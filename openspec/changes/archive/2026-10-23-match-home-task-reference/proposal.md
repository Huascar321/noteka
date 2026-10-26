# Proposal

## Why

La sección Tasks conserva una tabla visual y una vista inicial distinta de la referencia del usuario. El usuario solicita copiar composición, iconos e imágenes de Notion, permitiendo Lucide para iconos nativos.

## What Changes

- Encabezado grande, acción New Task, pestañas con iconos, toolbar útil y filas compactas sin separadores/subrayados repetidos.
- Vistas reales Week, Month, Scheduled, No Due y tareas de proyectos activos/en progreso, con filtros y paginación en Rust. Today/Inbox/All/Done conservan su semántica; Recurring/Active Content siguen diferidas.
- Proyectos como grupos plegables con icono, nombre, conteo de esta página y creación contextual; relación y fecha alineadas a la derecha.
- Iconos de imagen guardados como adjuntos locales mediante una clave de icono compatible con el campo existente; render compartido con fallback Lucide. Recuperación explícita de recursos autorizados, sin URLs remotas en runtime ni datos personales en Git.
- Recuperación de relaciones omitidas por el export previo, sólo con fuente explícita, sin adivinar duplicados.
- Cobertura y reglas accesibles en un desplegable; errores/carga permanecen explícitos.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: fidelidad de Tasks, vistas acotadas y arte local por entidad.

## Impact

HomePanel/HomeItem, componente de Tasks, iconos compartidos y controlador de imágenes; home.query en Rust y tipos generados. Sin migración SQLite: imágenes como adjuntos existentes y referencia en payload.icon; se conserva backup previo al enriquecimiento de iconos. Se recuperan sólo vínculos tarea/proyecto explícitos de la referencia con coincidencia inequívoca y backup; no se modifican contenidos, fechas ni estados para imitar la captura. No se activan automatizaciones, recurrencias, IA ni proveedores. Los recursos personales se conservan en el workspace, fuera de Git.
