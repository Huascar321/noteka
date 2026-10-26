# Proposal

## Why

El detalle comparte un único «Más opciones» que esconde grupos de funciones. Projects conserva una presentación genérica pese a contar con estado, fechas, progreso y registros relacionados locales; el usuario pidió reproducir la composición de su detalle real de Notion.

## What Changes

- Inventariar el componente y sus contextos; sustituirlo por iconos discretos por grupo. Notes conserva Exportar/Recursos externos/Adjuntos; los otros contextos muestran grupos aplicables y estados disponibles reales.
- Cada icono abre un panel, sin ejecutar acciones. Conservar controles, condiciones y estado de los paneles al cerrarlos; tooltip, aria-controls/expanded, foco, Tab/Enter/Espacio/Escape.
- Projects: portada decorativa local, icono/título, Status/Target Deadline/Progress, relaciones compactas, Nav y secciones Tasks/Notes basadas en datos locales existentes.
- Usar captura Playwright del Project vinculado a la Note de referencia, sin editar Notion. Identidad de origen y cálculo Rust acreditados, sin copiar porcentajes ni datos personales a código.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: grupos de herramientas accesibles y detalle Project con cabecera/relaciones/contenidos locales.

## Impact

Svelte/CSS y documentación; sin cambios Rust, contratos, SQLite, documentos, dependencias ni datos. Reutilizar exportación, recursos externos, preparación/lector de adjuntos, workflow.query/context, home.query/projects_board, CRUD y guardias existentes. Compilación y revisión UI sobre copia restaurada con escrituras bloqueadas; conservar trabajo previo.

## Non-goals

Sin edición de Notion, recuperación de datos, nuevos modelos, rich document de Project, autoguardado, nuevas vistas Board/Calendar de tareas o colaboración simulada. Sin tocar Eco Armario, voicebot ni otros repositorios. Meeting Notes permanece propuesto 0/19; no usar pantalla nativa, publicar, commit o push.
