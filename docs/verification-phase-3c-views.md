# Verificación de 3c: vistas avanzadas

12 de octubre de 2026. Cambio `add-advanced-collection-views` verificado y [archivado](../openspec/changes/archive/2026-10-12-add-advanced-collection-views/), **primer tramo** de 3c: SQLite v6, filtros compuestos, tablero/calendario/galería, configuración protegida y proyecciones Rust por ID. No completa templates/bases incrustadas ni el Home de referencia. Los cinco requisitos/ocho escenarios se sincronizaron y contrastaron íntegramente en `collection-views` antes del archivo; `--skip-specs` sólo evitó repetir esa sincronización ya verificada.

## Checks ejecutados

- `cargo test --offline -p noteka-core -p noteka-cli`: **59 tests**, 51 core (5 unitarios y 46 de integración) y 8 CLI. Seis tests dedicados de vistas.
- Contratos regenerados y `contracts:check`; cinco tests de adaptador aprobados. svelte-check sin errores/advertencias y Vite aprobado.
- Pasada nativa ampliada: **117 checks** (104 principal, 2 reapertura y 11 restauración), en `/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-native-bqf9qs/results.json`. Host WKWebView, IPC y procesos CLI reales con red denegada. Incluye galería con division_by_zero visible, reinicio de layout al crear colección y recorrido de 51 IDs en galería/tablero/calendario (50 + 1 y vuelta al inicio), además de buffers/conflictos/fallos/backup. Sustituye la pasada inicial de 106 checks.
- fmt/Clippy workspace/all-targets (incluido feature native-smoke), contratos/frontend, release Apple Silicon `--no-bundle` y OpenSpec estricto: aprobados con el código final. La validación documental es un check separado de las pruebas de la aplicación.
- Diff revisado contra `noteka-before-advanced-views.tar.gz`; módulos nuevos revisados directamente. Evidencia externa `noteka-advanced-views-review.diff`. El checkout no tiene commit inicial; Git sólo enumera untracked y no sirve como base del diff. No se hizo commit/push ni publicación.

## Trazabilidad: 5 requisitos, 8 escenarios

| Escenario | Evidencia |
| --- | --- |
| Consulta compuesta sobre originales | `compound_filters_exact_null_types_limits_and_visible_calculation_errors`, `filter_boundaries_and_successful_calculated_conditions_are_deterministic`; AND legacy + árbol, null/false/decimales, cálculo válido, orden; `advanced_view_projections_filters_and_conflicts_match_desktop` compara CLI/núcleo desktop |
| Árbol inválido o cálculo fallido | Ramas inválidas aun cortocircuitadas, límite de profundidad 8/9, nodos 64/65 y condiciones combinadas 32/33; dinero no float; cálculo division_by_zero referenciado rechaza query antes de filtrar |
| Grupos y cambio de valor | `board_declared_empty_groups_null_checkbox_and_updates_share_ids`; opciones en orden declarado, false/true/null distintos, grupos vacíos y patch mueve el mismo ID |
| Mes, rangos y vacíos | `calendar_civil_leap_ranges_extreme_months_and_invalid_configs`; Honolulu no convierte a UTC; rango cruza límites, fechas únicas, sin fecha; febrero 1900/2000/2024, enero 0001 y diciembre 9999 |
| Configuración de calendario inválida | Guardado/consulta rechazan mes inválido o no ASCII, propiedad ajena/tipo incompatible y campos ajenos al layout |
| Recorrer presentaciones | `every_layout_paginates_unique_records_and_groups_reference_only_page`; cinco layouts guardados, grupos sólo refieren IDs de la página, matching_count no suma apariciones. Nativo: 51 IDs completos en gallery/board/calendar, vuelta al inicio, error calculado en tarjeta y apertura del documento original |
| Reapertura, conflicto y restauración offline | `advanced_views_revision_conflicts_unknown_format_and_backup_preserve_raw`; revisiones propias de vista, backup conserva bytes crudos. Unitario `view_format_migration_failure_preserves_v5_and_json` prueba rollback, backup v5 restaurado/abierto y vista antigua intacta |
| Formato futuro y fallo de consulta | Config/layout/operador de árbol desconocido rechazado sin reemplazo y backup/restore preserva JSON crudo; recorrido nativo incluye retirar filas tras fallo al seleccionar vista y recuperación al reintentar |

## Coherencia y límites

| Dimensión | Resultado |
| --- | --- |
| Completitud | **7/7 tareas**, cinco requisitos implementados, ocho escenarios trazados y recorridos ampliados aprobados |
| Corrección | Filtros y proyección: `database_filters.rs`, `collection_views.rs`, `database.rs:437–493,720–882`; clientes: `DatabasePanel.svelte:38–119,145–160`, tests Rust/CLI y `tests/native/smoke.js:275–356` |
| Coherencia | Diseño y ADR 0007 respetados: reglas Rust, migración sin reescritura, cobertura por página y buffers separados del snapshot |

No quedan problemas críticos ni divergencias de specs/diseño identificadas para este cambio. Los límites enumerados abajo son alcance explícito, no capacidades completadas de Home o de toda 3c.

Reglas, selección y grupos viven en Rust; DTOs compartidos no tienen SQL cliente. Migración 006 versiona formato sin reescribir JSON. UI separa buffer layout de snapshot confirmado; columnas/layout/config/datos se publican después de query exitosa y errores no relabelan filas anteriores. [Contrato](views.md), [ADR 0007](decisions/0007-core-view-projections.md).

Durante la ampliación se corrigieron fixtures (número JSON calculado 2.0, ID del cálculo obtenido del esquema) y precondiciones del runner: foco nativo de ventana **y** webview solicitado fuera del callback de carga, espera de foco real y recuperación antes del recorrido del localizador. Se espera la carga de relaciones después del commit y el foco diferido del editor antes de sus asserts; una consulta CLI de la fixture fallida confirmó que la relación ya existía. No se cambió CSS, se emuló `document.hasFocus()` ni se quitaron asserts. Intentos previos de activación externa/bundle temporal fallaron y se retiraron. Los cambios de activación sólo existen con el feature de pruebas y se contrastaron con las APIs y fuentes de Tauri/Tao instaladas. El release normal no los contiene.

No hay benchmark de vistas, prueba con lectores de pantalla ni snapshot entre páginas bajo escrituras externas. Galería textual, formularios JSON/IDs y ausencia de drag-and-drop son límites explícitos; no son paridad completa del Home/Ultimate Brain. Manifiesto de backup sigue v2. Firma/notarización/distribución pendientes.
