# Proposal

## Why

Search ya muestra evidencia seleccionada, pero preparar un adjunto exige CLI/MCP. Este recorte de 5a permite operar la cola y seleccionar una generación desde una nota sin convertir una lectura o búsqueda en procesamiento automático.

## What Changes

- Panel de preparación separado de Search, dentro del editor: referencias de archivo/imagen de la nota actual e ingreso explícito de ID administrado. Los rótulos del bloque no se presentan como metadatos del original.
- Consultas explícitas paginadas de trabajos, selección de trabajo y consulta de control/índice, sin descargar originales ni lanzar helpers.
- Acciones independientes para encolar con extractor elegido, pausar/reanudar globalmente, ejecutar sólo el ID elegido, cancelar/reencolar con revisión y seleccionar la generación observada con CAS del índice.
- Estados, resultados anteriores, soporte, errores, límites y confirmaciones visibles. Una falla no repite escrituras ni actualiza revisiones automáticamente.
- Verificación headless en ambos temas y regresión/gates intactos, con baseline/evidencia durable fuera de Git.

## Non-goals

No listado global de adjuntos, importador nuevo, lector completo, OCR, ampliación de parsers, búsqueda unificada, purga/reset del índice, scheduler, modelos, proveedores, pantalla nativa ni cambios de Home. No completa toda 5a. La importación existente y las operaciones Rust se reutilizan sin ampliar contratos.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `local-extraction`: preparación explícita desde escritorio, independiente de lectura/búsqueda y con revisiones/resultados del núcleo.

## Impact

Svelte/TypeScript: panel/controlador, conexión acotada a `App.svelte` y preparación local de ambos helpers en el hook de Tauri; pruebas de cliente y puente headless. Sin cambios Rust/SQLite v12/API v1/52/MCP 24/44/45, dependencias o nueve mutaciones batchables. Sin alterar Home, editor semántico, guardias ni corpus/gates existentes. La instrucción del usuario autoriza completar propuesta, implementación, verificación, sync y archivo en esta ejecución.
