# Design

## Context

Fases 0–2 archivadas; FTS v3 agrega un índice derivado sin cambiar el formato semántico. Entidades built-in tienen IDs/revisiones y notas tienen bloques. Se suma esquema personalizado sin sustituir esos originales ni crear otro almacén de páginas.

## Goals / Non-Goals

**Goals:** colecciones genéricas, tipos exactos, identidad compartida, vistas reutilizables, CLI/UI offline.

**Non-Goals:** fórmulas/rollups (3b), vistas avanzadas/bases incrustadas/templates (3c), lotes/harness (4/5c), importación Notion, eliminación física o coerción/cambio de tipo automático.

## Decisions

SQLite v4 agrega collections (ID/nombre/revisión), properties (definición JSON estricta, ID estable), memberships (collection_id/entity_id) y property_values (entity_id/property_id/JSON). Un registro es la membresía de la misma entidad, no otra página: nuevas páginas usan note con inbox false; built-ins existentes se incorporan por ID y no se reescriben. Se puede pertenecer a varias colecciones. Campos de título y timestamps son proyecciones read-only de entidad, excepto título editable por entity.update. Propiedades no se borran ni cambian tipo en esta entrega.

property.define valida nombre único, tipos y configuración con revisión esperada de colección. Siempre existe propiedad title por colección. Tipos text/number/decimal/money/checkbox/date/date_range/select/multi_select/url/email/files/relation/created_at/updated_at. Decimal usa string canónica sin exponente (hasta 28 dígitos y 9 decimales); money usa esa string y moneda fija ASCII de tres mayúsculas en la definición. No hay floats para dinero ni redondeo implícito. Fecha civil canónica; rango {start,end} ordenado. Null borra un valor; vacío string/array es distinto y se conserva. Select/multi_select usan opciones únicas declaradas inmutables. URL sólo http/https; email validación sintáctica conservadora, sin prometer entregabilidad. Files arrays UUID con hash original verificado; relation arrays de IDs miembros de colección destino sin duplicados, archivados conservados.

Valores son patch por property_id dentro de una transacción inmediata con revisión de entidad; desconocidos/props de otra colección se rechazan y nunca se eliminan campos omitidos. record.create valida pertenencia/valores dentro de la misma transacción de entity.create. record.attach incorpora existente con control de revisión; record.update sólo valores, entity.update conserva reglas de título/payload/documento. record.get devuelve entity + valores proyectados + memberships. Relaciones son valores lógicos, inversa calculada por consulta; no se guarda otra copia del edge. Sin detach/delete automático.

views (ID/collection_id/nombre/revisión/layout/config JSON) guardan columnas, hasta 16 filtros AND (eq/contains/is_empty) y hasta 4 ordenamientos (asc/desc) por propiedad; null al final y desempate por ID. Valores decimales comparados exactamente en Rust con mantisa i128 y escala, no lexicográficamente. Consulta carga candidatos de una colección acotada a 10.000 membresías, evalúa/sortea en Rust y devuelve páginas 1–100/cursor offset, sin snapshot multipágina. UI presenta hasta 50 filas: tabla nativa/lista paginadas, no importa TanStack para reglas de dominio. Optimización SQL/virtualización de volúmenes superiores requiere medición/cambio posterior.

La UI de colección usa controles de esquema y formularios JSON explícitos para valores/configuración, sin interpretar código. Las páginas abren el editor existente y comparten guardia global; el buffer de valores/configuración protege navegación/cierre y copia JSON recuperable. No se altera un esquema desconocido al leerlo. Migración 004 agrega tablas y valida con backup de SQLite/adjuntos v3 previo; rollback y manifiesto íntegro mantienen recuperación anterior.

## Risks / Trade-offs

IDs explícitos y JSON son funcionales pero menos cómodos que pickers; quedan revisables sin ocultar capacidad. Tipos se congelan para no inventar migraciones destructivas. Colección de 10.000 tiene consultas in-memory acotadas; no se promete p95 sin medición. Relación nativa de productividad y relación genérica de propiedad tienen semántica distinta y ambas apuntan a los mismos IDs, sin reemplazar vínculos previos. Campos built-in continúan en payload hasta una proyección declarada futura; no se duplican sus valores como autoridad nueva.
