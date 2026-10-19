# Propiedades calculadas locales (3b)

`formula.set` define un AST JSON; no ejecuta JavaScript, SQL, shell ni código de un modelo. El cálculo ocurre en Rust, offline, sin LLM. No admite la sintaxis textual de Notion.

## Definir y editar

```json
{
  "collection_id": "id-coleccion",
  "expected_revision": 3,
  "name": "Doble",
  "expression": {
    "op": "binary", "operator": "mul",
    "left": {"op": "property", "property_id": "id-importe"},
    "right": {"op": "decimal", "value": "2"}
  }
}
```

CLI: `noteka --json --workspace /ruta/al/workspace formula set`, entrada por stdin/`--input`. Devuelve un bundle de colección con `calculations`, indexado por ID de propiedad. El catálogo `operations describe --json` contiene el contrato completo. Para actualizar, agregá `property_id` y mantené el mismo nombre, tipo y moneda de resultado; una revisión vieja devuelve conflicto, nunca merge silencioso. `property.define` no crea fórmulas.

Escritorio: Collections → Fórmula. El formulario conserva el AST pendiente al cancelar navegación/cierre, ante conflicto o fallo; recuperar el buffer devuelve el texto original. Las calculadas no están en el patch editable de registros. Tabla/lista muestran resultado o código de error; los cálculos no incrementan revisión ni reemplazan originales.

## Subconjunto AST

| `op` | Campos y reglas |
| --- | --- |
| `number` | `value`: número JSON finito, no monetario |
| `decimal` | `value`: string decimal exacto; `currency` opcional, tres mayúsculas ASCII |
| `boolean` / `text` | `value`: booleano / string de hasta 16 KiB |
| `property` | `property_id`: propiedad escalar de esta colección |
| `binary` | `operator`: add/sub/mul/div/eq/lt/and/or; `left`, `right`: AST |
| `not` | `value`: AST booleano |
| `if` | `condition`: booleano; `then`, `otherwise`: mismo tipo/moneda |
| `rollup` | `relation_id`, `aggregate`: count/sum/avg/min/max; `property_id` excepto count; `inverse` default false |
| `allocate` | `relation_id`, `cost_id`, `weight_id`, `scale`: 0–9 |

Campos desconocidos rechazados. Resultado escalar: number/decimal/boolean/text; listas, archivos y rangos no son escalares. Fechas/instantes/selecciones se leen como texto, sin aritmética de fechas. `eq`/`lt` exigen tipos/monedas iguales; texto usa orden binario Unicode y booleanos false < true. Number y decimal no se convierten entre sí.

Null se propaga en aritmética/comparaciones; `if` con condición null devuelve null sin evaluar ramas. `not`/and/or conservan null; and false y or true cortocircuitan. El grafo valida ambas ramas aunque una no vaya a ejecutarse.

## Dinero y precisión

Decimal/money usa strings sin exponente: máximo 28 dígitos y nueve decimales, sin ceros iniciales ni cero negativo, como [propiedades originales](collections.md). Aritmética checked i128 con escala nueve. Producto/división no representables producen `precision_loss`; no redondean. `1 / 3` decimal es error; `1 / 2` es `"0.5"`. Se cancelan factores antes de multiplicar para evitar overflow intermedio innecesario; resultados fuera de rango dan `overflow`.

- Suma/resta/comparación requieren la misma moneda (o ambas sin moneda).
- Multiplicar/dividir dinero por decimal sin moneda conserva moneda; multiplicar decimal sin moneda por dinero también.
- Dividir dos importes de la misma moneda produce decimal sin moneda.
- No se multiplican dos monedas ni se mezclan monedas. Number usa float finito sólo para valores no financieros.

Rollup count cuenta relaciones, no celdas pobladas. Sum ignora null y devuelve cero tipado si no hay valores; avg/min/max ignoran null y devuelven null si no hay valores. Un error fuente se propaga: no se elimina como si fuera cero. Relaciones directas/inversas conservan destinos archivados.

## Reparto genérico y fixture

`allocate` exige exactamente un destino por participante, costo decimal/money no negativo en destino y pesos number enteros positivos ≤1.000.000. El costo debe ser representable en la unidad `scale` (2 = centavos). Se consideran participantes vinculados, incluidos archivados; pisos, restos mayores y desempate por ID estable conservan el total exacto. Se memoiza el reparto completo dentro del snapshot.

Fixture exclusivamente sintética, sobre una ruta nueva fuera de Git:

```sh
cargo build -p noteka-cli
node scripts/subscription-fixture.mjs /ruta/nueva/al/workspace
```

Crea Subscriptions/Members, costo BOB 145,69, seis asientos declarados, cinco participantes con pesos 2/1/1/1/1, pago false y vista Pendientes. Comprueba 48,57 + cuatro 24,28 = 145,69 usando enteros, no floats. El script usa operaciones públicas secuenciales: **todavía no es un lote atómico ni un agente**. Los asientos declarados son informativos; no hay reglas implícitas de vacantes, exceso de asientos ni períodos. Antes de usar datos reales, revisá la [cola de revisión](review-queue.md).

## Errores, snapshot y recuperación

`record.get`/`database.query` devuelven éxitos en `values` y fallos en `calculation_errors` por ID. Un fallo no publica un valor válido. Filtro/orden sobre una calculada con error rechaza la consulta, aunque otra condición excluiría ese registro; sin filtro/orden sobre ella, puede mostrarse la celda con error. Filtros comparan decimales numéricamente y `contains` acepta texto calculado.

Códigos de cálculo: `invalid_syntax`, `type_error`, `missing_reference`, `cycle`, `complexity_limit`, `invalid_value`, `division_by_zero`, `precision_loss`, `overflow`. Definiciones inválidas dan `validation_error` con `details.calculation_error`; conflictos mantienen `revision_conflict`. Un AST almacenado desconocido impide recuperar el bundle como definición reconocida; no se convierte ni se sobrescribe, permanece en backup y los registros señalan el error.

Cada consulta usa una transacción de lectura SQLite y memoización (entidad,propiedad), sin caché persistente. La siguiente consulta refleja cambios de valores/relaciones; no hay push ni garantía de snapshot entre páginas. Límites: 256 nodos/profundidad AST 32, grafo/evaluación de dependencias acotados a 128 niveles, millón de pasos por evaluador, relación directa 100 IDs, inversa 10.000 fuentes. El límite general de profundidad JSON 32 del contrato puede rechazar antes un AST anidado dentro de la solicitud. No hay benchmark de fórmulas ni promesa de latencia.

SQLite v5 agrega `calculations(property_id,expression)` con migración 005 y backup previo de v4, sin reescribir originales. El manifiesto de backup continúa en formato v2; restaura AST, metadata y originales, no resultados calculados. Versiones futuras/AST desconocidos se preservan. Decisión: [ADR 0006](decisions/0006-safe-derived-calculations.md).
