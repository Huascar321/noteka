# ADR 0017: XLSX tipado persistido por generación

Fecha: 15/10/2026. Estado: adoptada y verificada en
[`add-local-xlsx-extraction`](../../openspec/changes/archive/2026-10-15-add-local-xlsx-extraction/proposal.md),
con [aceptación final3](../verification-xlsx-extraction.md); sincronizado y archivado.

## Contexto

El lector aislado preserva números/fórmulas/cachés y admite texto de hasta 128 KiB.
La cola actual limita fragmentos a 4 KiB y sólo conoce UTF-8/PDF. Convertir cada
celda en un fragmento perdería libros admitidos o impondría offsets ficticios.

## Decisión

SQLite v11 aditiva: guardar una sola representación Workbook JSON canónica v1
<=1 MiB, separada en chunks UTF-8 consecutivos <=4 KiB/512. Manifiesto con
parser/versión, SHA-256, bytes y conteos. FKs compuestas generation/job y cascades.
La generación general conserva hash/tamaño del original; fragment_count cuenta
chunks internos, no celdas. El presupuesto 32 MiB suma bytes UTF-8/PDF/JSON XLSX
sin contar nuevamente sus proyecciones públicas.

Reader público paginado de celdas completas 1–64 con manifiesto derivado, ordinal
global y hoja/A1; reconstruye/verifica todo y fuente antes de responder <=2 MiB.
Core sólo enlaza protocolo/DTOs; helper descartable procesa bytes fuera del lock.
No indexación XLSX en esta entrega: rechazo previo explícito, FTS UTF-8/PDF intacto.

## Recuperación

Backup consistente v10 antes de 011 y transacción con validación antes de commit.
Un fallo deja v10 íntegro; no se modifica ninguna tabla existente salvo versiones.
Backup v11 valida JSON/chunks/conteos/hash y originales; restore a ruta nueva lee
sin helper. No downgrade in-place: el binario v10 restaura el backup previo en
otra ruta; no mezclarlo con escrituras posteriores v11. Versiones futuras se
rechazan sin cambios; configuraciones de extractor futuras se conservan crudas.

## Alternativas y consecuencias

- Filas por celda más JSON: duplicación y dos autoridades; descartado.
- Celda <=4 KiB: rompe subconjunto válido; descartado.
- Números f64/texto PDF: pérdida de lexemas/tipos; descartado.
- Parser in-process: debilita contención; descartado.
- Supervisor genérico refactorizado: posible después, no arriesgar PDF ahora.

Cada página valida hasta 1 MiB; medir coste y memoria, no prometer rendimiento
universal ni RSS/sandbox OS. Modificar representación requiere versionado/migración,
no reserializar campos desconocidos. El formato exacto y resultados de aceptación
se documentan en la guía y verificación; no implican Excel genérico ni fase 5a completa.
