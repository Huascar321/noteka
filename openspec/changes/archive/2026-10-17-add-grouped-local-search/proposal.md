# Proposal

## Why

La búsqueda local exige repetir la misma consulta en dos ámbitos. El siguiente recorte de 5a permite consultar notas/entidades activas y evidencia seleccionada con una acción, conservando los contratos, cobertura y errores separados de cada fuente.

## What Changes

- Añadir «Notas y adjuntos (grupos)» como opción de Search, conservando los dos ámbitos individuales y sus filtros.
- Una consulta literal se envía a `search.query` y `extraction.search`; se muestran dos grupos con una página de veinte cada uno, sin fusionar scores/cursores ni inventar un total global.
- Siguiente, inicio, carga y error por grupo; una falla no oculta el resultado válido del otro. Editar texto retira ambos resultados y no consulta automáticamente.
- Procedencia de notas y evidencia/cobertura de adjuntos completas, texto escapado y apertura de originales por las guardias existentes.
- Desarrollo inicialmente sin importar en App, conservando la aceptación de preparación. Integración con revalidación conjunta de los recortes de UI, conservando Rust/CLI/corpus en ejecución.

## Non-goals

No ranking híbrido/semántico, filtros cruzados de entidad sobre adjuntos, notas archivadas en el grupo conjunto, preparación o reconstrucción de índices, modelos, proveedores ni Home. Los ámbitos individuales conservan sus opciones. «Conjunta» significa una entrada y dos grupos, no un snapshot atómico de todo el workspace ni cobertura universal.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `local-text-search`: consulta local conjunta por grupos explícitos, conservando búsqueda individual y evidencia tipada.

## Impact

Nuevo controlador/panel de presentación y pruebas; integración acotada a Search. Sin operaciones, migraciones, dependencias, cambios Rust/corpus/gates ni alteración de Home/editor. Se reutiliza AttachmentReader y operaciones compartidas, con SQLite v12/API v1/52 y nueve mutaciones intactos. No es recuperación semántica ni habilita 5b.
