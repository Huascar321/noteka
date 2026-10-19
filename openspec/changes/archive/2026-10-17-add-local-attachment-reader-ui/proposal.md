# Proposal

## Why

El núcleo permite leer generaciones completas por páginas, pero el escritorio sólo muestra hits seleccionados de Search y metadatos de preparación. Este recorte de 5a expone los tres lectores existentes sin exigir indexación ni ejecutar procesamiento.

## What Changes

- Lector explícito de generaciones por ID de trabajo y tipo elegido, independiente de Search y del estado seleccionado del índice.
- Páginas reemplazables de veinte fragmentos UTF-8/PDF o celdas XLSX; generación/procedencia/cobertura y error visibles.
- PDF muestra manifiesto de páginas y localizadores derivados; XLSX conserva nombres/visibilidad/rangos/merges/estilos, tipos léxicos y fórmula/caché separados. No es render visual ni grilla expandida.
- Controlador con lifetime, cupo físico y descarte de respuestas tardías; sólo consulta cuando el usuario lo pide. Lecturas revalidan fuente/derivados mediante Rust.
- Pruebas de cliente y aceptación headless sobre CLI real/golden independientes. Integración de App/puente después de la aceptación UI aislada, seguida por revalidación conjunta de ambas UIs; Rust/CLI/corpus en regresión permanecen inmutables.

## Non-goals

No parsing, importación, encolado/indexación/reparación implícitos, edición de celdas, recálculo, OCR, PDF/Excel genéricos, vista visual del original, proveedores/modelos ni Home. No completa por sí solo toda 5a. No añade operaciones o migraciones.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `local-extraction`: lectura de generaciones desde escritorio, con procedencia y cobertura de los lectores compartidos.

## Impact

Componentes/controlador Svelte/TypeScript y pruebas headless, integración acotada al editor. SQLite v12/API v1/52, MCP, helpers, contratos y nueve mutaciones se conservan. Depende de los lectores archivados; el panel puede recibir ID manual sin necesitar preparación ni búsqueda. La integración modifica App/puente y revalida conjuntamente las UIs; Rust/CLI/corpus de preparación permanecen inmutables durante su aceptación.
