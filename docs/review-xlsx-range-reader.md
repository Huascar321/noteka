# Revisión de lectura XLSX por rango

Revisión de código/diff del21/10/2026, aceptación técnica completa:367 Rust por perfil,28 SDK por perfil,90 frontend,4860 headless y cinco gates aprobados.

- Rango se valida en Rust con la gramática A1 existente; ancho y header explícitos.
  No hay SQL construido con input, conversión numérica de celdas, expansión densa
  ni selección FTS implícita. Número no finito se rechaza en transporte UI antes
  de que JSON lo convierta a null opcional.
- Transacción de lectura captura job/generación/fuente. Loader verifica todos los
  chunks/owners/hash/canónico antes de filtrar, conservando corrupción exterior.
  Cursor compara toda la forma y admite sólo offsets no nulos dentro de coincidencias.
- Headers referencian celdas completas, no títulos inferidos. null de item es
  ausencia física; cached null conserva celda presente. El encabezado también
  sigue en items. Ordinal global se calcula antes de filtrar, sin renumerar.
- Respuesta<=2MiB incluye envelope. Corpus exacto1MiB demuestra rechazo sin
  truncamiento al duplicar headers y recuperación mediante elección explícita.
- Scope clasifica read de forma exhaustiva; MCP deriva schemas de core y mantiene
  siete exclusiones/permisos. Batch enum rechaza antes de preview, sin añadir steps.
- UI sólo amplía panel/controlador existentes; misma cola/cupo físico/lifetime,
  página de20 sin acumulación, errores sin retries y procedencia sin HTML activo.
  Home, tema, navegación, nativos y parsers permanecen idénticos al baseline.
- SQLite/representación/FTS/snapshots no cambian. Baseline16 independiente conserva
 55 contratos anteriores completos y restaura backup nuevo sin helpers.

Trazabilidad: dos requisitos nuevos local-extraction→xlsx_range_tests/corpus CLI;
contrato core→comparación de55 DTOs y catálogos/CLI/scoped; política MCP→28 SDK
por perfil; escritorio→90 cliente y1016 lector headless. Diez requisitos numéricos
existentes se actualizan en los deltas sin perder escenarios; cinco requisitos
nuevos/ diez escenarios. Ver [verificación](verification-xlsx-range-reader.md).

Limitaciones: no tablas PDF, detección automática ni formato visual Excel; elección
por ID/A1, no grid editable; lectura valida libro completo. Headless no acredita
host nativo y benchmarks no prueban latencia universal. Revisión manual de UX
queda diferida en review-queue, sin bloquear desarrollo local autorizado.
