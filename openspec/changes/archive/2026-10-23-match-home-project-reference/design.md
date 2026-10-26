# Design

## Context

El núcleo ya tiene cinco estados y resúmenes, pero Home usa composición genérica. La fuente local conserva páginas y tareas; varias relaciones no fueron importadas. El usuario pidió reproducir su tablero de referencia. El checkout contiene trabajo preexistente que debe preservarse.

## Goals / Non-Goals

**Goals:** tablero compacto, imágenes acreditadas, relaciones verificables, porcentajes calculados en Rust y protección de datos originales.

**Non-Goals:** dominio Goals, arrastrar tarjetas, automatizaciones de Notion, sincronización remota, sustituir documentos o modificar estados para copiar un porcentaje.

## Decisions

1. HomeProjectSection y HomeProjectCard comparten el encabezado y controles de Tasks/Notes. Board inicial; columnas de 196 px con separación de 12 px, fondo y chip por estado, tarjetas con título que envuelve y referencias visibles. Active/Planned conservan filtros reales. Acciones existentes únicamente; crear prepara captura con estado.
2. ProjectFields añade source con el contrato NoteSource existente y board_order acotado. No hay migración SQL. Rust valida y filtra archivo de origen antes de paginar. Entidades locales y vistas generales conservan acceso; las consultas Home de tareas por proyecto también respetan archivo de origen.
3. HomeSummary::Project añade progress_tenths; se calcula con enteros redondeando completas*1000/total. progress_percent anterior se conserva. No se guardan porcentajes derivados ni contadores importados. Un proyecto vacío muestra Sin tareas.
4. El contexto admite además notas con procedencia Goals validada mediante la relación Note→Project existente. Estas notas se abren como documentos originales; no se crea una entidad Goal ni se infieren flujos de objetivos.
5. Script de recuperación separado y genérico, dry-run predeterminado, backup nuevo obligatorio al aplicar, operaciones core con CAS e informe incremental privado. Se vinculan IDs originales por manifiesto y bloque inicial de propiedades del export, verificando toda la secuencia CSV para resolver las entidades tipadas. Referencias originales de Tasks/Tag/Goal acreditan vínculos; arte sólo desde URL observada y adjunto PNG local. Se comparan IDs y digest de documentos antes/después.

## Risks / Trade-offs

- [Metadata de fuente como snapshot] → documentar fecha y alcance; no prometer sincronización ni porcentajes idénticos si cambian estados locales.
- [Cliente anterior con payload cerrado] → actualizar cliente antes de editar proyectos enriquecidos; backup e informe de revisiones.
- [Secuencia del importador o identidad ambiguas] → exigir coincidencia completa y propiedades inequívocas; omitir y reportar en vez de adivinar.
- [Escrituras parciales] → informe incremental, operaciones idempotentes y CAS por entidad; no prometer atomicidad global.

## Migration Plan

Generar contratos, compilar Rust/UI, revisar preview de recuperación, respaldar y aplicar. Revisar manualmente composición, consultas, claro/oscuro/estrecho sobre copia del workspace. Revertir por informe con revisión vigente; no restaurar un backup sobre cambios posteriores automáticamente. No ejecutar suites no solicitadas.
