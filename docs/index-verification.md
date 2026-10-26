# Verificación de integridad del índice de adjuntos

Refactor `optimize-local-index-verification` aceptado6/6. No hay formato o
operación nuevos: SQLite16/API57 bajo verificación del lector de tablas.

Cada lectura conserva la validación completa del snapshot: selecciones,
configuración, representaciones canónicas, proyecciones, copias y postings.
Buscar cero resultados o filtrar un extractor no evita comprobar corrupción
fuera de página/filtro. Los originales de candidatos/página/lookahead conservan
su validación separada; get no promete comprobar todos los archivos originales.

El contenido FTS se compara con todos los fragmentos canónicos por rowid,
identidad de job/generación, ordinal y texto, incluidos tipos SQLite exactos y
cardinalidad. No acepta sólo hashes, conteos o textos normalizados. Reutiliza el
mapa completo de tokenización de textos byte-idénticos y contrasta todos los
postings; cuando no confirma igualdad, materializa el FTS esperado completo
y aplica el fallback anterior. Search conserva el contraste independiente de
candidatos/puntajes: sólo evita materializar el índice completo cuando prueba
que ninguna clase canónica del snapshot coincide, sin confiar en el cero del
índice real. Totales y tamaños de tokenización por fila también se verifican
contra canónico, incluidos resultados vacíos. No persisten caches entre consultas.

El [ADR0028](decisions/0028-lazy-exact-evidence-verification.md) describe la
prueba de equivalencia y alternativas. La copia del algoritmo anterior existe
sólo en tests. Fuentes/binarios/diagnósticos/fallos previos se preservan fuera
de Git bajo `active-phase5-path.txt`. La regresión completa y dos pares de gates
originales300/150 aprobaron; [evidencia](verification-index-verification.md).
