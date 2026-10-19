# Proposal

## Why

Las vistas genéricas ya están verificadas, pero una nota todavía no puede referenciar una colección y cada página de registro se construye desde cero. Completar este tramo de 3c permite reutilizar estructuras y consultar los mismos originales dentro de páginas, antes de reproducir el Home confirmado.

## What Changes

- Plantillas de páginas de registro por colección: título inicial, documento semántico y valores editables por defecto. Guardado con revisión, lectura/listado paginado e instanciación atómica mediante operaciones Rust compartidas.
- Cada instancia obtiene entidad y bloques nuevos; referencias explícitas a adjuntos, relaciones y vistas siguen apuntando a los originales. Un override parcial conserva defaults omitidos; null los limpia. Nunca se copian resultados calculados como valores editables.
- Documento v3 con bloque `collection_view`: referencia a colección/vista guardada, sin snapshot de registros ni formato propio del editor. Lectura v1/v2 conservada; formatos desconocidos no se convierten.
- Editor: inserción/eliminación del bloque, undo/redo, presentación paginada de los cinco layouts, actualización explícita y apertura protegida del registro original. Las vistas incrustadas son de lectura en este tramo; los valores se editan al abrir el original.
- **BREAKING** para clientes antiguos de workspace: SQLite v7/migración 007 con backup previo y tabla de plantillas; no reescribir documentos, cálculos, valores ni vistas existentes. Los clientes anteriores (hasta v6) deben rechazar el nuevo workspace; recuperación mediante backup previo en otra ruta.
- Exportación/búsqueda distinguen texto fuente de una referencia dinámica: export JSON conserva referencias, Markdown advierte límites y backup de workspace conserva sus destinos.

## Capabilities

### New Capabilities

Ninguna: se amplían las capacidades existentes del motor, documentos y presentación.

### Modified Capabilities

- `database-engine`: plantillas versionadas y creación atómica de instancias independientes con defaults tipados.
- `block-editor`: documento v3 con referencia a colección/vista, buffers conservados y evolución/exportación sin pérdida de originales.
- `collection-views`: presentación incrustada acotada, errores explícitos y apertura del mismo ID sin expansión recursiva.

## Impact

Contratos/catálogo/CLI, `entities.rs`, `documents.rs`, motor de colecciones, migraciones/backups y módulo de plantillas; adaptador/Tiptap, `BlockEditor.svelte`, presentación reutilizable extraída de `DatabasePanel.svelte` y guardias de `App.svelte`; tests core/CLI/adaptador/nativos y documentación/ADR.

Alcance adoptado: templates y bases incrustadas según el plan. Elecciones propuestas: páginas note del motor actual, defaults estáticos, referencias a vistas guardadas y presentación incrustada de lectura. **Non-goals:** Home/Goals/fotos/clima, plantillas de esquemas completos, plantillas built-in de tareas/proyectos, automatizaciones/fechas relativas, sincronización/Notion, IA, extracción y recursos remotos. No hay dependencias de cuentas, credenciales o modelos.
