# Revisión del verificador de evidencia

Revisión final aprobada de `optimize-local-index-verification`,6/6 verificado.

| Invariante | Comprobación |
| --- | --- |
| Todas las fuentes canónicas seleccionadas | La misma carga/configuración/proyección/hash anterior precede a copias y postings; no depende de filtro, página o hits. |
| Copias exactas | Secuencia completa por rowid contra referencias canónicas, cinco campos con ValueRef estricto, cardinalidad y duplicados; sin coerción o normalización. |
| Postings íntegros | Verificadores auxiliar/stream existentes; todos los términos/columnas/posiciones/cardinalidades y bitsets por documento, no un hash o muestra. |
| Estadísticas de ranking | Docsize y totales comparados con conteos de tokenización canónicos; tipos y varints acotados del SQLite incorporado. Rechaza la brecha previa aun sin candidatos. |
| Consulta y score | Los hits conservan comparación independiente con FTS esperado completo. Un cero sólo evita esa materialización si tampoco coincide ninguna clase canónica global. Los filtros no certifican ausencia global. |
| Snapshot y presupuesto | Las clases provienen de esta llamada/transacción, se reconstruyen y se destruyen con ella; Budget/cooperación5s y espera2s sin ampliación. |
| Autoridad y recuperación | No hay escritura a main/originales, reparación, extracción o selección implícitas; el error conserva revisión y reconstrucción explícita anterior. |
| Compatibilidad |57 contratos completos idénticos, datos/ranking/cursores/cobertura A/B exactos y backup/restauración con cliente57 anterior, seis formatos/desconocidos conservados. |
| Superficie | Sin DDL/dependencias/operaciones/permisos nuevos. NOTEKA_CLI sólo selecciona el binario del puente de pruebas; no es input de notas o IPC de producto. Home/App y helpers Apple conservados. |

El cuerpo previo de verify está congelado en tests y contrastado con su archivo
de baseline. El cambio de estadísticas fortalece un requisito ya existente:
no se afirma equivalencia con la aceptación defectuosa anterior de esos estados
corruptos. La equivalencia de datos válidos se comprobó contra binarios previos
independientes, además del diferencial interno.

La comparación A/B incluye proceso CLI/sandbox y no sustituye el gate interno.
El piloto aprobó100,671–177,237ms p95 de consulta y14,543ms de lock. Después de
toda regresión aprobaron los cuatro gates finales: query máximo182,232ms y
lock máximo18,445ms; dos repeticiones consecutivas de cada gate original.
Los fallos anteriores permanecen, sin causa puntual demostrada ni excepción.

Regresión completa aprobada:384 Rust por perfil,28 SDK por perfil,101 frontend,
5146 headless y host sin ventana. Pins protegidos y cuerpo del oráculo contrastados.
Aceptación final300/150 aprobada, sin excepciones. No sincronizar el lector de tablas incompleto al
archivar este refactor con skip_specs. No adelantar5b/5c ni fase6.
