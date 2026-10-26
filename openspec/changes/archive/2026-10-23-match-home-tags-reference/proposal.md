## Why

Tags todavía usa una lista genérica con relaciones incompletas. La referencia solicitada agrupa trece tags activos en Area/Resource/Entity, con cuatro vistas, iconos originales, última nota y contador. Hay que recuperar vínculos acreditados y calcular los resúmenes desde notas locales, conservando documentos y cambios previos.

## What Changes

- Encabezado y toolbar de Tags, Recent/Fav./A-Z/Types, Types inicial y grupos compactos con color, conteo y creación contextual.
- Filas con arte original local, última nota navegable y contador real de notas relacionadas.
- Clasificación Entity adicional sin cambiar Topic, favorito y procedencia opcionales, actividad de origen validada; archivo de origen filtra Home sin archivar la entidad local.
- Recuperación acotada de metadatos, relaciones y arte del export/referencia con backup, CAS e informe externo. No copiar contadores ni crear notas parciales.

## Capabilities

### New Capabilities

### Modified Capabilities

- `desktop-shell`: composición de Tags, vistas y resúmenes desde relaciones reales, recuperación explícita.
- `knowledge-entities`: Entity y metadatos opcionales de tags compatibles con payloads previos.

## Impact

Rust concentra validación, filtros, orden y resúmenes. UI/contratos generados incorporan la nueva presentación. No hay migración SQL, nueva entidad ni sincronización remota. Los originales y datos personales permanecen fuera de Git; revisión de compilación y presentación, sin suites no solicitadas.
