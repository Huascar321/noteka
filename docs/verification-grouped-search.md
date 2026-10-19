# Verificación de búsqueda agrupada

`add-grouped-local-search`: UI integrada y regresión/gates compartidos
aprobados. Baseline durable de 454 archivos fuera de Git, verificado por SHA-256;
ubicación en `~/.local/share/noteka-verification/active-unified-search-path.txt`.
No recupera evidencia histórica v11.

**9 tests de cliente** y **238 checks headless de producto** (119 por tema)
aprobados, con CLI/SQLite reales. `logs/headless-final.log` y
`runtime/noteka-headless-gOuLXR/` conservan reportes, capturas sintéticas, snapshots
y operaciones. Fixture previa de evidencia intacta más 23 notas por operaciones
públicas. Cero requests externos; ninguna escritura de la UI ni preparación o
indexación implícita. La prueba no acredita IPC/cierre/foco nativos.

El delta modifica un requisito conservando sus tres escenarios previos y añade
tres requisitos: diez escenarios en total, siete nuevos y tres heredados.

| Escenario | Código/contrato | Evidencia ejecutada |
| --- | --- | --- |
| Buscar y abrir sin red | App.openSearch/guardia y SearchHit | Headless consulta real y apertura de original con bloque/revisión; regresión previa intacta. |
| Cambiar de ámbito sin cambiar el editor | Selector Search conserva ámbitos/filtros originales | Regresión de 1.248 checks del ámbito adjuntos y 606 anteriores; título pendiente durante consulta conjunta. |
| Consultar adjuntos no es procesarlos | Sólo search.query/extraction.search | Registros y snapshots idénticos antes/después, sin llamadas import/enqueue/run/put/reset. |
| Consulta local conjunta explícita | GroupedTextSearch, una entrada literal y dos contratos | Cliente payloads exactos; headless resultados contrastados con ambas consultas CLI, sin total/ranking/snapshot global. |
| Paginación por grupo | Lectores/cursor independientes, reemplazo de página | Cliente y headless tres páginas de entidades y dos de adjuntos; el grupo no paginado permanece igual. |
| Consulta excesiva y alcance | Entrada literal sin corte y archived:false | Cliente Unicode >256 bytes y sin filtros cruzados; headless error real por longitud y generaciones sin seleccionar excluidas. |
| Fallo de una fuente | Error/reintento por grupo | Cliente falla cada motor; headless error parcial, otro snapshot visible y reintento sin repetir la otra consulta. |
| Índice cambiado entre páginas | Cursor Rust y descarte de adjuntos | Cliente/headless CAS externo vence cursor, retira sólo adjuntos y conserva notas sin rotular snapshot conjunto vigente. |
| Criterios nuevos y llamadas tardías | Lifetime/cupos físicos por lector | Cliente respuesta tardía/remount/espera cancelada; headless cambiar texto/ámbito no publica respuestas viejas ni realiza writes. |
| Abrir nota con buffer y usar teclado | Callback App, sin persistencia del panel | Headless guardar/descartar/cancelar y fallo de guardar, teclado/foco, ambos temas y viewport estrecho. |

Durante revisión se corrigió una exigencia inicial demasiado amplia: abrir otra
nota no invalida una búsqueda global; sí la invalida cambiar sección, ámbito o
workspace. Diseño, delta y tests reflejan ese comportamiento comprobado.
No se modificaron motores Rust, validación, corpus, cuotas o gates.

## Aceptación técnica final conjunta

La [aceptación conjunta](verification-local-preparation-closure.md) registra las
36 etapas frescas, cuatro gates nuevos, 222 pins y **3.116 checks headless** sobre
la integración final. Sustituye los conteos parciales de los hitos anteriores,
que se conservan como historia. En el estado final hay 206/209 fuentes originales
iguales; App, puente y ayuda Search son las tres diferencias identificadas y
revalidadas. No se cuentan la regresión/gates compartidos como ejecuciones
independientes para cada cambio. Los tres deltas se sincronizaron/contrastaron y los cambios se archivaron el
17/10; no se declara toda 5a ni aceptación nativa.
