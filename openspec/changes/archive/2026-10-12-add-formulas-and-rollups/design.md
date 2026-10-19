# Design

## Context

3a archivada: propiedades estrictas/valores patch/IDs compartidos y vistas consultadas en Rust. No hay evaluador ni estado derivado calculado. La referencia de Notion confirma la experiencia de propiedades calculadas, no se importa su sintaxis ni motor.

## Goals / Non-Goals

**Goals:** evaluación sin modelos; AST seguro, monedas, grafo y errores; reparto genérico y fixture exacta; clientes compartidos.

**Non-Goals:** parser textual, compatibilidad Notion, funciones avanzadas de fecha/listas, automatización, persistir caché de resultados, lote de agente o política financiera real.

## Decisions

SQLite v5 agrega calculations(property_id,expression JSON). `formula.set` crea propiedad formula/rollup e infiere tipo escalar (number/decimal/boolean/text y moneda opcional); actualizar conserva nombre/tipo/moneda y exige revisión esperada de colección. Expresiones estrictas, hasta 256 nodos/profundidad 32; AST literal tipado, property, binary (add/sub/mul/div/eq/lt/and/or), not, if, rollup y allocate. No se evalúa SQL/shell/JS.

Tipos number y decimal distintos sin coerción automática. Decimal usa escala nueve i128, operaciones checked y rechaza precision_loss en división/producto no representables. Moneda se conserva en suma/resta; multiplicar/dividir por decimal sin moneda mantiene moneda, cociente de la misma moneda queda sin moneda. No multiplicar dos monedas ni mezclarlas. Null se propaga en aritmética/comparaciones; if/null y boolean/null devuelven null. Condicional y booleanos cortocircuitan. Number usa float finito no financiero y controla overflow.

Rollup valida relación/destino/propiedad, directo o inverso: count cuenta relacionados; sum ignora null (vacío total cero del tipo); avg/min/max ignoran null y devuelven null si todos vacíos. Error de una fuente se propaga, no se omite como cero. Referencias archivadas siguen incluidas, acorde a 3a. Grafo de propiedades con edges de AST/rollup/allocate se valida antes de commit; ciclos rechazados. Evaluación memoizada por (entity,property) dentro de una transacción de lectura, con límite de pasos y detección de ciclo defensiva.

Resultados no se persisten: cada query hidrata todas las propiedades calculadas devueltas y sus dependencias alcanzables, usando snapshot actual (no sólo las columnas visibles). record.get/database.query agregan valores calculados exitosos y mapa `calculation_errors`; originales permanecen en property_values. Filtrar/ordenar un campo calculado con error rechaza la consulta, no lo interpreta como vacío o monto válido. UI muestra errores en celda, excluye calculadas del patch editable y conserva guardia de buffer al definir/editar AST.

allocate es una operación genérica de costo ponderado: cada fuente tiene exactamente un destino por relación, costo decimal/money en destino y peso number positivo entero ≤1.000.000 en fuentes. Calcula participantes vinculados (incluye archivados), suma pesos y divide costo no negativo en unidades mínimas según scale 0–9. Piso/restos mayores/desempate ID estable garantizan suma exacta; costo no representable en esa unidad da precision_loss. No impone reglas de asientos vacantes, sobreasignación o períodos; sólo distribuye entre pesos existentes explícitos. Memoiza el reparto completo por destino/configuración dentro del snapshot.

## Risks / Trade-offs

Escala fija limita división → error explícito en vez de redondear implícitamente. Grafo conservador incluye ambas ramas → rechaza ciclos incluso si una rama no se ejecuta. Recalcular bajo demanda puede ser costoso → memoización, límites y sin claims de rendimiento no medidos. Formularios AST JSON menos cómodos → documentar como UX inicial y mantener tipos/errores observables.

## Migration Plan

Migración 005 transaccional con backup previo de v4; añade tabla sin reescribir datos/esquemas existentes. Nueva metadata de tipo es opcional y se conserva al leer. Probar rollback, backup/restauración de AST y formatos futuros; no convertir una definición desconocida al editarla.
