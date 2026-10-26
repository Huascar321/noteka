# ADR 0025: lectura explícita de rangos XLSX

Estado: aceptada y verificada. Fecha:21/10/2026.

## Contexto

La representación XLSX ya conserva hojas/celdas dispersas y fórmulas/cachés exactas.
La fase5a necesita consultar regiones y encabezados sin convertir una inferencia
visual en datos originales ni recalcular Excel.

## Decisión

Operación aditiva de lectura `extraction.xlsx_range` sobre el loader íntegro existente,
con hoja/rango/fila elegidos, ordinal global y cursor ligado a todos los criterios.
Encabezados son proyecciones de celdas originales; valores ausentes/duplicados no
se rellenan. Celdas del encabezado también siguen en items. No se persiste elección,
no se filtra FTS implícitamente ni se inicia parser. Se conserva SQLite16 y los55
contratos previos; UI/CLI/MCP comparten Rust. Ver [contrato](../xlsx-range-reader.md).

## Alternativas y consecuencias

Ampliar xlsx_cells con otro shape complicaría la conservación contractual; un
endpoint separado distingue cobertura de rango/libro. Inferir encabezados o
expandir merges oculta ambigüedad; guardar tablas espejo crea otra autoridad.
El resultado disperso exige leer coordenadas, pero preserva procedencia.
Repetir headers largos puede superar2MiB; error explícito y elección de rango/límite
menor conservan el presupuesto, sin truncamiento o fallback. No migración: clientes16
previos y backups conservan formato. Fase5a sigue abierta por tablas PDF.
