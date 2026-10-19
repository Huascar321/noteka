# Revisión técnica: búsqueda XLSX

Estado al 16/10: **revalidación alternativa aprobada** con evidencia durable.
[Criterios y limitación histórica autorizada](xlsx-search-evidence-recovery.md).
No se recuperó el baseline v11 ni se finge repetir sus comparaciones. Revisión de
código/requisitos/diseño y de los nuevos harnesses completada. Preservación fresca,
259 enlaces y OpenSpec estricto 15/15 aprobados antes del sync. Cinco deltas
sincronizados y contrastados con main, cambio archivado el 16/10, con las
limitaciones autorizadas explícitas. No hay un defecto crítico de implementación
conocido; el cierre incluye validación estricta **14/14**, delta archivado válido
en copia documental exacta para el CLI y **272 enlaces reubicados/20 documentos**
aprobados antes de marcar la tarea administrativa. La revisión final abarca
**309 enlaces/30 documentos**, diecinueve tareas completas y los cinco deltas
idénticos al merge revisado; no es un baseline histórico de producto.
[Evidencia](verification-xlsx-search.md), [matriz](verification-xlsx-search-scenarios.md)
y [tareas](../openspec/changes/archive/2026-10-16-add-local-xlsx-search/tasks.md).

## Completeness

| Dimensión | Resultado |
| --- | --- |
| Completeness | 19/19; cinco deltas/14 requisitos/49 escenarios sincronizados, contrastados y archivados |
| Correctness | Evidencia fresca de dominio/transportes/oracles; comparaciones históricas H no repetibles explícitas |
| Coherence | Proyección, barrera v12, verificación íntegra y recorrido FTS5 alineados con diseño/ADR |

- Cinco deltas, 14 requisitos y 49 escenarios, incluidos los 32 nombres heredados.
  La matriz nombra cada escenario y la evidencia ejecutable correspondiente.
- Rust debug **y release frescos**: 287 aprobados/diez ignores explícitos por perfil. No son pruebas
  funcionales fallidas deshabilitadas: dos fixtures se invocan desde su padre,
  los gates pertinentes y diagnóstico mixto se invocan separadamente; los otros
  benchmarks explícitos conservan su evidencia histórica, sin afirmar repetición.
- Repetición completa aprobada: 216 XLSX/235 PDF/83 oracle por perfil, SDK 21 por
  perfil, tres raw y veinte rondas concurrentes por perfil, fixtures SQL v10/v11,
  contratos actuales, host no-bundle y frontend 13+3+4 tests/606 checks headless.
- Dos gates mixtos consecutivos frescos **167,02–270,74 ms p95** y dos intactos
  **114,31–152,19 ms**; lock p95 máximo 28,208 ms. Mantienen corpus/caps/validación.
  Sonda nueva/autotest aprobado, diez operaciones CLI, pico observado 22.298.624 B
  y backup/restore mixto sin helpers. No es máximo físico universal o exclusivo APFS.

## Correctness

- `xlsx_index_projection::project`: sólo Workbook validado por `xlsx_storage::load`;
  incluye ocultas, respeta orden y separa los seis campos, caché antes de fórmula.
  No consulta ZIP/XML ni helper. Citas identifican celda/campo/rango, no chunks.
- `attachment_search_storage::canonical`: conserva serialización/hash v1 UTF-8/PDF;
  incorpora manifiesto XLSX completo y proyección localizada. El oracle Python
  compara hashes/textos/celdas/hits contra CLI, sin usar el proyector Rust como oracle.
- La publicación usa los mismos caminos CAS fuera/dentro del escritor, verifica
  fuente/generación/cuotas y limita respuesta antes de commit. Hay barreras para
  put/put/reset/reemplazo/fuente y rollback, no sólo tests secuenciales felices.
- Get/search verifican todas las selecciones, copias/postings/estadísticas, también
  fuera del filtro y con cero hits. Search confronta candidatos/score y originales
  de página/lookahead; get no afirma verificar fuentes. No hay confianza acumulada.
- La saturación conserva cada selección; se explican límites dominados en vez de
  simular un Workbook inválido. Empty/no_text no acreditan contenido visual buscado.
- Backup mixto de tres extractores incluye representación e índice; se rechazan
  adulteraciones de postings con copia de texto restaurada y hash exterior reparado.
  Destino nuevo/no overwrite, formatos futuros y exclusión batch están probados.

## Coherence

- `012.sql`, init/migrate/registry y SCHEMA_VERSION=12: sólo barrera de versión;
  no DDL FTS, stride, perfil o reindexación. Cliente v11 necesita su backup previo,
  no downgrade. Recibos batch v11 vencen; cursor de índice sólo-migración se conserva.
- API v1/52, MCP 24/44/45 y nueve mutaciones quedan iguales en conteo. El checker
  fresco valida adiciones exactas, descriptor/límites y 415 refs por perfil, más
  igualdad al catálogo v12 de reinicio. La comparación v11/50 contratos es evidencia
  **histórica perdida**, no un check fresco; no se fabrica catálogo ni se usa un
  normalizador abierto para ocultar cambios. Reader probado contra su golden.
- No cambia UI/Home, host, lectores/protocolos/supervisores, dependencias, modelos o
  permisos remotos. Bindings/schema frontend son las únicas modificaciones de src/.
- El fallback de postings mantiene el stream completo. C API fija evita
  conversión dinámica, sin omitir tipos/valores/duplicados/cardinalidad/deadlines.
  Statement RAII y buffers prestados sólo tras ROW y antes de step/finalize;
  tipos originales se validan antes de conversiones, text antes de bytes y NULL se
  rechaza. Tests diferenciales incluyen vocabulario grande, materializados sin
  afinidad, tipos inválidos, offsets extremos, orden arbitrario y presupuesto cero.
- El gate exigió otra optimización: 256 sigue como frontera algorítmica, hasta
  ella un término por query, por encima OR de hasta 64 y xPhraseFirst/Next públicos.
  Verifica la misma información de instancias sin combinar/ordenar un array xInst.
  El mapeo frase→término, bitsets y cardinalidad no dependen del orden; estructura
  de frases sólo se comprueba una vez por MATCH ligado, nunca se cachean postings.
  Iteradores viven exclusivamente en su fila/callback y se comprueba cada rc;
  tests diferenciales cubren fronteras, lotes parciales y filas dispersas/corruptas.

## Preservación y límites

La revisión previa usó copias independientes, pero **ya no están disponibles**.
La revisión actual usa 405 archivos v12 capturados y 203 hashes de fuentes/
scripts/fixtures fijados al retomar, no Git sin commit ni un baseline histórico.
Desde esa captura no cambió producto, Home/host, parser, corpus, caps, lockfiles o
gates; sólo documentos y cuatro herramientas de verificación nuevas. Se revisaron
el proyector/dispatch/hash y el recorrido público FTS5, además de los harnesses
SQL/contratos/sonda. Las fixtures y schemas actuales no prueban el cliente viejo.

**Advertencias aceptadas explícitamente:** comparación exacta de archivos/50
contratos v11 y admisión/restore con los binarios viejos no repetibles; fallos
históricos sólo conservados documentalmente, sin sus logs crudos. Son limitaciones
de evidencia, no nuevas garantías ni una excepción al target. Los intentos nuevos
se conservan completos y no se cuentan como aprobación si fallaron. No quedan
defectos funcionales conocidos o checks técnicos actuales fallidos sin resolver.

Los resultados son locales al M1 Max/64 GiB y corpus fijados, no una promesa de
300 ms para cualquier libro/caché. Ningún check headless acredita IPC/foco/cierre
nativos de Home; sus ocho tareas siguen pendientes. No completa 5a/5b ni Excel
genérico/OCR. No se enviaron datos personales, publicaron cambios o instalaron modelos.
