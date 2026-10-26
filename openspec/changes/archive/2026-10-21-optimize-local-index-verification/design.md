# Design

## Context

Ver propuesta y evidencia externa `gate-failure-review.json`. Canonical carga y
valida íntegramente cada generación/proyección; verify compara su metadata con
selecciones persistidas, materializa un FTS esperado completo, compara copias y
valida postings. El camino exacto de textos repetidos vuelve a tokenizar sólo
clases byte-idénticas; no necesita el FTS esperado completo salvo en su fallback.

## Goals / Non-Goals

**Goals:** eliminar la primera tokenización redundante sin reducir el conjunto
de bytes/campos/tipos/postings/estadísticas comprobados en cada snapshot.
**Non-Goals:** ver propuesta; no aceptar datos por hash, cache o consulta sin hits.

## Decisions

1. Construir primero todos los Selected desde canonical y metadata validada,
   dentro de la misma transacción y Budget existentes. Derivar referencias
   prestadas `(rowid,job,generation,ordinal,text)`, ordenar por rowid y rechazar
   duplicados. Sin clonar los8MiB ni normalizar Unicode.
2. Comparar filas reales ordenadas contra esas referencias, con ValueRef estricto
   Integer/Text en los cinco campos, sin coerción; comprobar cardinalidad total,
   filas extra/ausentes/duplicadas y budget. No es sólo comparar textos/conteos.
3. Crear vocab real y llamar al verificador exacto por clases ya cerrado. Some(true)
   finaliza; cualquier otro resultado materializa el FTS esperado completo y usa
   same_postings completo, conservando exactamente la admisión anterior también
   ante un resultado negativo del camino auxiliar.
   Search conserva el FTS completo independiente para contrastar candidatos y
   puntajes. Sólo si no hay candidatos y todas las clases canónicas del snapshot
   tampoco coinciden, acredita ausencia sin construirlo. Las clases no aplican
   filtros: un match global siempre obliga al fallback completo. Se destruyen al
   salir del snapshot y se reconstruyen en cada llamada, sin cache entre consultas.
   El control de estadísticas compara cada docsize y total global contra conteos
   de tokenización canónicos. El formato varint se contrasta con el SQLite
   incorporado; blobs excesivos/tipos ajenos/malformados fallan antes de copiarlos.
   Esto cierra una brecha previa detectada por los tests: algunas estadísticas
   alteradas se admitían en verify y sólo fallaban al producir candidatos.
   Tanto auxiliar FTS5 como fallback stream conservan su implementación.
4. Conservar un oráculo del algoritmo anterior bajo cfg(test), independiente del
   camino optimizado. Diferenciales con entradas válidas/repetidas/únicas/vacías,
   tipos/identidad/texto/filas, postings ocultos/estadísticas y datos canónicos
   corruptos fuera de filtro; comparar rechazo sin repair y estado persistido.
5. Medir old57/new57 sobre fixture fijada y same-source/shapes, más dos pares de
   gates originales300/150 sin cambios. A/B no reemplaza los gates. No atribuir
   automáticamente los picos al OCR, SO o nuevo lector; conservar fallos.

## Risks / Trade-offs

- Omitir un tipo/rowid al evitar temp FTS → comparación estricta de los cinco
  campos, cardinalidad y pruebas diferenciales del oráculo anterior.
- Cambiar fallback de vocab → mantener ruta íntegra para textos no repetidos y
  probar ambos verificadores de postings, incluidos duplicados y corrupción oculta.
- Más referencias prestadas/ordenamiento → máximo16384 fragmentos, sin copia
  textual, budget cooperativo y sin caches entre llamadas o snapshots.

## Migration Plan

Sin DDL, formatos o contratos nuevos. Snapshot/binaries anteriores ya conservados;
rollback de código no requiere transformar datos. Refactor bajo skip_specs no
sincroniza comportamiento del lector de tablas todavía incompleto. Sólo archivar
este cambio tras sus pruebas/gates; después retomar el cierre del lector y5a.
