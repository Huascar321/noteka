# ADR 0006: cálculos derivados tipados y dinero exacto

- Estado: adoptada en `add-formulas-and-rollups`.
- Fecha: 12 de octubre de 2026.

## Contexto

3a ya conserva entidades compartidas y propiedades originales. Hace falta construir sistemas calculados offline, incluyendo importes, sin ejecutar código generado ni contaminar originales con resultados obsoletos.

## Decisión

Rust valida un AST JSON estricto, infiere tipo/moneda y comprueba un grafo conservador antes de confirmar `formula.set` con revisión de colección. SQLite v5 conserva sólo definiciones y metadata. Cálculos/rollups se obtienen bajo demanda en una transacción de lectura, con memoización y límites; fallos se devuelven por propiedad. Filtrar/ordenar un cálculo fallido rechaza la consulta.

Decimal usa strings y checked i128 a escala nueve, sin float financiero ni redondeo silencioso. Reparto ponderado genérico opera en unidades mínimas, por pisos/restos mayores/desempate estable de ID. Políticas financieras reales (vacantes, períodos, exceso de asientos) no se infieren de esta operación. El [contrato](../formulas.md) es la referencia de tipos, null, errores y límites.

Migración 005 añade tabla sin reemplazar valores/documentos; backup previo y restauración contienen los AST. Definición desconocida no se convierte para poder editarla. Clientes comparten núcleo y muestran errores; el formulario AST conserva buffers y revisión.

## Alternativas descartadas

- JavaScript/SQL dinámico: innecesario y amplía superficie de ejecución no autorizada.
- Parser compatible con Notion: agregaría un lenguaje/paridad no necesarios para esta entrega.
- Float monetario: no conserva centavos reproduciblemente.
- Caché persistente/invalidation incremental: requiere coordinación adicional entre app/CLI; snapshot bajo demanda garantiza vigencia con menos estado.
- Redondeo automático a nueve decimales: oculta pérdida de precisión; se opta por error explícito.

## Consecuencias

El núcleo funciona sin modelos y no altera originales al consultar. Grafo conservador rechaza ciclos de ramas no ejecutadas. Escala/rango restringidos pueden producir errores en operaciones matemáticamente válidas; UI debe mostrarlos. Evaluar muchos registros puede ser caro: no hay claim de rendimiento sin benchmark. JSON/IDs es una UX inicial, no un editor de fórmulas amigable. Lotes/preview, bases incrustadas y harness permanecen en sus fases independientes.
