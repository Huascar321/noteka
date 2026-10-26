# ADR0028: materialización diferida durante verificación exacta de evidencia

Estado: aceptado6/6. Regresión y cuatro gates finales300/150 aprobados; ver
[verificación](../verification-index-verification.md).

Los gates mixtos detectaron dos picos p95 mayores a300ms entre tres intentos;
la causa puntual no quedó demostrada y el A/B anterior/actual no mostró una
regresión sostenida. El diagnóstico sí identificó tokenización duplicada: el
verificador construía un FTS esperado completo antes de volver a construir un
FTS de clases byte-idénticas para comprobar todos los postings reales.

Se conserva la admisión, no se agregan caches o muestras. Dentro del mismo
snapshot/presupuesto, canonical valida todas las generaciones/proyecciones y
sus selecciones. Sus fragmentos generan referencias prestadas ordenadas por
rowid. Una lectura completa del contenido FTS compara cardinalidad y los cinco
campos con tipos SQLite exactos, sin coerción ni normalización de texto. IDs
duplicados o cualquier fila faltante/extra fallan.

El verificador existente por clases idénticas sigue contrastando cada posting,
término/columna/offset, duplicados, cardinalidad y estadísticas con presupuesto
cooperativo. Sólo cuando no confirma igualdad se materializa el FTS esperado
completo y se usa el fallback íntegro anterior. Search también lo construye para
contrastar candidatos y puntajes, salvo ausencia global comprobada de forma
independiente contra todas las clases canónicas del mismo snapshot. Un match
global exige contraste completo aunque un filtro quite sus resultados. Las
clases se destruyen con el snapshot, sin reutilización entre llamadas.

Se verifican docsize y totales FTS5 contra conteos canónicos aun sin hits, con
tipos/varints acotados conforme al SQLite incorporado. Los tests encontraron
que el verificador anterior podía delegar algunos fallos de estadísticas a
la comparación de puntajes; esa brecha se rechaza antes de responder también
en get o consultas vacías. No se cambia un formato persistente.

Ningún derivado persistido se
repara, indexa o modifica al consultar. Los límites300/150 y restantes cuotas
permanecen idénticos.

Se conserva el cuerpo anterior bajo cfg(test) como oráculo independiente del
flujo optimizado. Los diferenciales deben comprobar corpus válidos y corrupción
de copias/tipos/canónicos/postings/estadísticas, también en ausencia de hits.
Mediciones de rendimiento y compatibilidad son una aceptación separada; no
atribuir automáticamente los picos al lector de tablas, al SO o al OCR.

No requiere migración, perfil, contrato o dependencia nuevos. El cambio
`optimize-local-index-verification` omite deltas por ser un refactor de los
requisitos vigentes. No publica comportamiento del lector de tablas todavía
abierto ni avanza5b/5c o modifica Home.
