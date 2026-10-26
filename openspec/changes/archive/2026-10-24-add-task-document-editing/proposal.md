# Proposal

## Why

El detalle de Task sólo permite título y propiedades: el núcleo rechaza documentos en tareas y la interfaz no monta su editor. El usuario pidió poder editar el contenido como en su Task de Notion y autorizó inspeccionar esa referencia con Playwright; la página observada está vacía y presenta inicio de contenido/plantillas.

## What Changes

- Documento enriquecido opcional en Task, con las operaciones y límites existentes, sin convertirla en Note ni cambiar su identidad.
- Inicio visible de contenido, escritura directa, bloques `/`, undo/redo, Enter desde el título y estructuras locales Página vacía/Lista de pasos/Notas de tarea.
- Detalle con Status, Project y Due principales; Inbox, prioridad y planificación en View details. Relaciones originales y guardias compartidas.
- Preservación de Tasks anteriores sin documento, formatos desconocidos y propiedades ajenas; no escritura por abrir o cambiar metadatos si el cuerpo sigue ausente.
- Subtareas/parent tasks, Content, comentarios, recurrencia y time tracking de la referencia requieren cambios propios; no se copian sus controles como funciones ficticias.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `knowledge-entities`: documento opcional de Task con CRUD/CAS y recuperación conservadora.
- `block-editor`: inicio visible de cuerpo de Task y estructuras de contenido sobre el editor existente.
- `desktop-shell`: composición de propiedades y contenido en el detalle Task con navegación/edición protegidas.

## Impact

Rust de entidades, Svelte/CSS del editor y detalle, documentación. SQLite sigue en 20 y documentos en v1/v2/v3; misma tabla documents, sin migración o operación nueva. Notes y los otros tipos conservan su contrato. No suites nuevas ni ejecución de suites; compilación, revisión de código y previsualización sobre copia, sin guardar datos personales. Meeting Notes sigue propuesto 0/19.
