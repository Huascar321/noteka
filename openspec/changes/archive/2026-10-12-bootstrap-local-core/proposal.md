# Proposal

## Why

Noteka necesita una base local que conserve conocimiento estructurado y permita operarlo desde la aplicación y desde agentes. Crear primero un núcleo Rust verificable evita que las reglas de productividad y la persistencia dependan del editor o de un proveedor de IA.

## What Changes

- Crear el scaffold Tauri 2 + Svelte 5 + TypeScript y un workspace Rust con biblioteca de dominio y CLI.
- Crear y abrir un workspace SQLite versionado, con migraciones, backup consistente y recuperación verificable.
- Persistir notas con documento JSON semántico mínimo, tareas, proyectos, tags tipados y personas; relacionarlos y archivarlos sin borrar sus datos.
- Exponer operaciones compartidas con validación, control de revisión y errores estructurados; añadir un CLI JSON que funcione con la ventana cerrada.
- Añadir una interfaz inicial para capturar y consultar notas, navegar entidades y mostrar estados de guardado y errores.
- Mantener esta entrega operativa sin cuentas, red ni modelos.

## Capabilities

### New Capabilities

- `local-workspace`: almacenamiento local, versión de esquema, apertura segura, backup y restauración.
- `knowledge-entities`: identidades estables, notas JSON, entidades PARA mínimas, relaciones y archivo reversible.
- `core-operations`: contratos compartidos, validación transaccional, concurrencia y CLI estructurado.
- `desktop-shell`: navegación de escritorio y captura mínima de notas mediante el mismo núcleo.

### Modified Capabilities

Ninguna. El repositorio es nuevo y no contiene especificaciones implementadas.

## Impact

Se crearán el frontend, el adaptador Tauri, crates Rust y pruebas. Los datos del usuario vivirán fuera del checkout, en un workspace explícito o en el directorio de datos de la aplicación. Los contratos de operaciones serán la base posterior del editor, bases de datos, harness y MCP.

## Non-goals

Este cambio no implementa Tiptap, fórmulas, vistas de bases de datos, búsqueda híbrida, enriquecimiento, proveedores de IA, Google Drive, MCP, LoRA ni sincronización. La planificación completa de esas entregas está en [el plan general](../../../docs/plan-noteka.md); no se declaran aquí como comportamiento actual.
