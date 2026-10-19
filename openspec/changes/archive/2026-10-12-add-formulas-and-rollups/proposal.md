# Proposal

## Why

El motor 3a ya organiza registros pero no calcula propiedades. El siguiente cambio permite cálculos deterministas y la fixture de suscripción sin modelos ni código ejecutable arbitrario.

## What Changes

- AST JSON validado: literales, propiedades, aritmética, comparaciones, booleanos y condicionales, con tipos y moneda explícitos.
- Fórmulas y rollups de conteo/suma/promedio/mínimo/máximo; grafo de dependencias, límites, ciclos y errores identificables.
- Decimal exacto y reparto ponderado de unidades mínimas por restos mayores/desempate ID para la fixture Bs 145,69 y seis asientos.
- Cálculo derivado bajo demanda en snapshot consistente, sin reemplazar valores originales; filtros/vistas muestran o rechazan errores, no importes ficticios.
- `formula.set` compartido Rust/CLI/Tauri y controles JSON de escritorio protegidos; migración SQLite v5 con backup y pruebas.
- Sin lenguaje de texto compatible con Notion, funciones de fechas/listas avanzadas, automatizaciones, bases incrustadas ni agentes. La referencia oficial se consultó el 12/10/2026; no se promete paridad de sintaxis.

## Capabilities

### New Capabilities
- `calculated-properties`: AST tipado, dependencias, rollups, dinero y resultados derivados verificables.

### Modified Capabilities
Ninguna: se agregan propiedades calculadas y sus errores sin retirar operaciones/valores de 3a.

## Impact

Rust, contratos/CLI, SQLite y panel de colecciones. La evaluación es local/offline; no necesita runtime de modelos ni proveedor externo. Política financiera de la fixture sigue pendiente de validación para datos reales en review queue.
