# ADR 0026: proyección independiente de tablas PDF OCR

Estado: aceptada y verificada. Fecha:21/10/2026.

## Contexto

Las regiones OCR persistidas conservan texto/geometría, pero no prueban estructura
de tabla. PDFKit independiente no se modifica para añadir interpretación semántica.

## Decisión

Biblioteca Rust separada: columnas y rectángulo explícitos, filas con intersección
vertical común, fuente completa una vez y referencias por ordinal. Regiones exteriores
se conservan y cruces/ambigüedad fallan sin dividir texto ni inventar spans. Se usa
f64 sólo para comparar geometría, conservando strings fuente exactos. Hash declarado
no acredita archivo o workspace. Ver [contrato](../pdf-table-projector.md).

## Alternativas y consecuencias

Asignar por centro ocultaría regiones atravesadas; dividir texto inferiría palabras
sin geometría. Detección genérica/modelos no están autorizados ni evaluados. Esta
proyección cerrada es reproducible pero puede rechazar tablas comunes reconocidas
como líneas enteras. La integración de producto debe verificar fuente/canónico,
proveniencia/cursores/cuotas y UI, en otro cambio antes de cerrar5a. No persistencia
nueva ni operación API57 en esta entrega; SQLite16/API56 y Home9/17 se conservan.
