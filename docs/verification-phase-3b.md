# Verificación de fase 3b: fórmulas y rollups

12 de octubre de 2026. Cambio `add-formulas-and-rollups`. SQLite v5, AST tipado, dependencias, rollups, reparto exacto, `formula.set`, valores/errores de sólo lectura y formulario protegido. No completa 3c, lotes, harness ni IA.

## Checks ejecutados

- `cargo test --offline -p noteka-core -p noteka-cli`: **51 tests**, 44 core (4 unitarios y 40 de integración) y 7 CLI. Ocho tests de fórmulas; migración v4→v5 fallida mantiene versión/backup.
- fmt/Clippy workspace/all-targets sin warnings; contratos generados frescos. Cinco tests del adaptador aprobados.
- svelte-check sin errores/advertencias, Vite y release Apple Silicon `--no-bundle` aprobados.
- **92 checks nativos offline**: 81 principal, 2 reapertura, 9 restauración. Tres sesiones WKWebView/IPC/CLI bajo sandbox deny network. Evidencia temporal: `noteka-native-5FT8pR/results.json`.
- Fixture CLI `scripts/subscription-fixture.mjs` ejecutada sobre workspace nuevo sintético: cinco importes, sin errores, total entero **14.569 centavos BOB**. Evidencia: `noteka-subscription-fixture.json` fuera de Git.
- OpenSpec estricto aprobado antes del cierre; no demuestra comportamiento de aplicación por sí solo.
- Revisados archivos nuevos y diff contra `noteka-before-formulas.tar.gz`; sin commit/push, firma/notarización ni publicación.

## Trazabilidad: 4 requisitos, 6 escenarios

| Escenario | Evidencia |
| --- | --- |
| Expresión válida y tipos | `ast_types_short_circuit_null_exact_math_and_visible_errors`; decimal exacto, number no monetario, if/not, originales/revisiones intactos; `boolean_short_circuit_and_missing_values_have_explicit_semantics` |
| Definición inválida o cíclica | `invalid_types_currency_cycles_limits_and_old_writers_leave_schema_unchanged`; sintaxis/monedas/tipos/IDs ajenos/profundidad/nodos/ciclos y revisión vieja; `dependency_height_is_bounded_independently_of_graph_traversal_order` acepta 128 edges y rechaza 129 sin commit |
| Cambiar una fuente | `rollups_direct_inverse_empty_error_and_related_recalculation`; count/sum/avg/min/max, cambio de valor/relación, archivo incluido y revisión ajena intacta |
| Vacío y fallo | División/precisión/overflow, null y errores visibles; ordenar/filtrar cálculo fallido rechaza query; filtro decimal exacto/texto calculado; `persisted_metadata_mismatch_and_malformed_inverse_are_not_valid_results` comprueba metadata alterada e inversa duplicada |
| Suscripción y recálculo | `subscription_fixture_exact_remainder_allocation_and_cost_changes`, CLI fixture: 145,69 = 48,57 + 4×24,28; cambiar a 145,70 asigna centavo al menor ID de un asiento; peso cero es error |
| Uso offline/restauración | `formula_backups_migration_unknown_ast_and_failed_set_are_recoverable`, CLI parity y recorrido nativo: AST/moneda/restauración, celda division_by_zero, patch no editable, buffers/conflictos; backup de AST desconocido conserva JSON crudo y no lo reemplaza |

## Revisión de coherencia

4/4 tareas y 4/4 requisitos cubiertos; sin bloqueantes identificados tras los checks. El diseño ahora explicita que query hidrata todas las calculadas devueltas, no sólo columnas visibles. La revisión reforzó la validación de metadata AST y fuentes inversas; el grafo memoiza alturas para que el límite no dependa del orden de recorrido. Las relaciones también consumen presupuesto de evaluación.

No se midió rendimiento de fórmulas, ni se probó carga máxima de la inversa con 10.001 registros o lectores de pantalla. Los límites tienen comprobaciones en código; eso no equivale a benchmark. AST desconocido impide el bundle reconocido, pero conserva original/backup y expone error de registro; no hay editor de reparación de formatos futuros. [Contrato y límites](formulas.md), [ADR 0006](decisions/0006-safe-derived-calculations.md), [revisiones del usuario](review-queue.md).
