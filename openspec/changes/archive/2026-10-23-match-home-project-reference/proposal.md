## Why

Projects en Home todavía presenta filas genéricas y avances incompletos porque la importación perdió relaciones. La referencia solicitada usa un tablero compacto de cinco estados con iconos originales, progreso calculado y contexto navegable. Hay que recuperar únicamente metadatos acreditados y conservar documentos, identidades y cambios locales.

## What Changes

- Sección Projects con encabezado y captura propios, pestañas Active/Planned/Board, Board inicial y cinco columnas compactas, accesibles y adaptables a pantallas estrechas.
- Tarjetas con icono original, progreso con precisión de décimas, tareas pendientes/vencidas y referencias reales a tags y páginas Goals importadas.
- Procedencia y orden opcionales validados en proyectos; las marcas de archivo de origen filtran Home sin archivar ni eliminar entidades locales.
- Recuperación explícita de relaciones y arte acreditados mediante backup, revisiones CAS e informe privado; no modificar cuerpos ni estados de tareas para copiar porcentajes.

## Capabilities

### New Capabilities

### Modified Capabilities

- `desktop-shell`: tablero Projects, cálculo de resúmenes y recuperación acotada de datos de referencia.
- `knowledge-entities`: metadatos opcionales de procedencia y orden de proyectos, compatibles con payloads anteriores.

## Impact

Rust concentra filtro, orden y resumen; UI y contratos generados incorporan presentación y precisión. No cambia el esquema SQLite. No se implementa un dominio Goals ni sincronización remota. Se conservan fuentes originales y cambios preexistentes; compilación y revisión visual manual, sin ejecutar suites no solicitadas.
