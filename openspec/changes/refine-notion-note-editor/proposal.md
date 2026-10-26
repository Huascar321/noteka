# Proposal

## Why

La edición de Notes ya usa Tiptap y bloques semánticos, pero expone formato e
inserciones mediante desplegables separados del contenido. El usuario autorizó
adaptar su apariencia y uso a una página de Notion: escritura directa, controles
contextuales y comandos junto al cursor.

## What Changes

- Composición de Note con columna legible, tipografía y espaciado de página.
- Barra flotante al seleccionar texto, con las marcas y enlaces ya admitidos.
- Menú `/` filtrable con teclado, anclado al cursor y limitado a bloques existentes.
- Controles laterales por bloque para insertar, convertir, duplicar, mover y quitar;
  arrastre de bloques completos con alternativa por teclado/botones.
- Adjuntos locales y vistas incrustadas siguen siendo elecciones explícitas.
- Conservación del guardado explícito, undo/redo, IDs, prefijos importados y guardias.
- Por indicación posterior del usuario, Exportar/Recursos externos/Adjuntos pasan
  del pie a la barra superior de Note, con paneles junto a sus disparadores.
- Seguimientos solicitados: retirar la ayuda inferior y Add paragraph de Note;
  ordenar el diálogo compartido Cancel/Discard/Save and continue, con separación
  sobre las acciones e iconos representativos a la izquierda.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `block-editor`: edición contextual de Notes con selección, comandos y acciones de bloque.
- `desktop-shell`: presentación de la Note como página de escritura adaptable a ambos temas.

## Impact

Interfaz Svelte/CSS y utilidades de Tiptap. Sin migraciones, dependencias nuevas,
operaciones Rust ni cambios del documento v1/v2/v3. Task, Goal y Milestone
conservan su presentación actual; Meeting Notes sigue propuesto. No incluye
autoguardado, colaboración, marcas ni tipos nuevos, descargas o proveedores.
