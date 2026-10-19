# Design

Implementado, revalidado y archivado el 16/10/2026: **19/19 tareas**, cinco deltas
sincronizados/contrastados. [Evidencia de cierre](../../../../docs/verification-xlsx-search.md)
y [limitación histórica autorizada](../../../../docs/xlsx-search-evidence-recovery.md).
Las estrategias y alternativas de abajo documentan el diseño; no reemplazan esos resultados.

## Context

Motivación/alcance: [proposal](proposal.md). Baseline archivado v11/API v1/52,
MCP 24/44/45, Workbook XLSX canónico <=1 MiB y lector de celdas separado.
La prohibición de indexar XLSX es deliberada en main, no un bug de dispatch.
[Descubrimiento](discovery.md) registra las fronteras verificadas y el baseline
independiente (369 archivos/ocho binarios/catálogo) fuera del repositorio.

El índice almacena selecciones y copias FTS genéricas, no filas de celdas o
locators: los reconstruye desde generaciones confirmadas y compara hashes,
copias/postings y candidatos. El esquema admite hasta 512 fragmentos por slot;
ampliarlo no es un detalle menor porque el rowid usa ese stride.

## Goals / Non-Goals

**Goals:** una proyección verificable de los campos ya leídos, búsqueda mixta con
procedencia precisa y selección explícita, sin tocar el parser ni crear segunda
autoridad. Conservar bytes/hash/candidatos UTF-8/PDF en un índice sin XLSX.

**Non-Goals:** búsqueda estructurada por rangos/estilos, evaluación de fórmulas,
uniones entre celdas/campos, indexación automática, UI o descarga de modelos.
Los fragmentos de búsqueda no reemplazan la lectura de una celda completa.

## Decisions

1. **Proyección pura desde persistencia validada.** Un módulo acotado nuevo,
   propuesto `xlsx_index_projection.rs`, recibe el Prepared/Workbook que devuelve
   `xlsx_storage::load`; no recibe ZIP ni inicia helpers. Reutilizar la validación
   integral de owner/chunks/hash/canonicalidad/semántica antes de proyectar,
   sin reconstruir desde una página del reader ni desde JSON sin validar.
   Checks de presupuesto en recorridos y cortes; SQL sigue sujeto al progress
   handler existente. No refactorizar parser/supervisor PDF/XLSX.
   Alternativas descartadas: indexar los chunks JSON (metadatos/escapes y falsos
   offsets), usar sólo la página pedida o volver a parsear el original.

2. **Campo separado, texto completo y determinista.** Recorrer hojas y celdas en
   su orden canónico, incluidas hidden/very_hidden. Por celda: caché primero,
   fórmula después. Proyectar `Text`, `Number`, `Error`, `DateIso` con el string
   exacto del DTO; `Boolean` como `true`/`false` ASCII. Campos ausentes o vacíos
   producen cero documentos. No agregar `=`, etiquetas, nombres de hoja, estilos,
   merges ni metadatos al texto. Fórmulas shared/array conservan sólo su texto
   presente, sin expandir seguidores ni tratar caché como recálculo.
   Alternativas: sólo textos (pierde números/fórmulas útiles) o concatenar caché
   y fórmula (mezcla evidencias y permite AND cruzados sin sentido).

3. **Locator nuevo y dos ordinales.** Ampliar AttachmentTextLocator con
   `kind=xlsx_cell_text`, `projection=xlsx_cell_fields_v1`, `cell_ordinal` global
   0-based del reader, `sheet_id`, `reference`, `field`, `field_bytes` y
   `text_byte_start/end` 0-based/end exclusivo sobre el campo decodificado.
   `field`: cached_text/number/boolean/error/date_iso o formula_text, como enum
   cerrado. `ordinal` del hit es global de fragmento, consecutivo desde cero;
   no es cell_ordinal, ordinal de chunk ni offset al ZIP. El locator debe permitir
   identificar si el texto es sólo parte del campo a partir de rango/field_bytes.
   Para booleanos el rango refiere a su proyección canónica, no al XML `0`/`1`.
   Valores/opciones ajenos no se aceptan por coerción ni como campos abiertos.

4. **Cortes UTF-8 y límites heredados.** Cada campo no vacío se divide en el
   mayor prefijo <=4096 bytes que termine en frontera UTF-8, sin overlap ni trim;
   cada fragmento conserva el rango. No unir celdas para forzar <=512 ni aumentar
   el stride. Rechazar la selección completa al requerir 513 fragmentos o exceder
   1 MiB de texto, sin cambiar el reader que admite 10.000 celdas. Contar texto
   proyectado real, no JSON/ZIP; copias FTS usan el presupuesto existente de 8 MiB
   separado de los 32 MiB de derivados de extracción. El snapshot incluye ambos.
   Se explicita la limitación lexical en cortes: sin matching entre campos/filas
   FTS o porciones, y un token cortado puede dejar de coincidir como palabra entera.

5. **Perfil, hash y barrera v12, sin DDL destructivo.** Mantener control de índice
   versión 1/perfil `literal_unicode61_v1`: define tokenizer, consulta literal,
   ranking y formato FTS, no una lista eterna de extractores. La semántica XLSX
   tiene nombre propio `xlsx_cell_fields_v1` en el locator y el contenido hasheado.
   Generalizar sólo el argumento interno de manifiesto del hash: null UTF-8 y
   objeto PDF mantienen exactamente su serialización y namespace v1; XLSX incluye
   el manifiesto completo derivado (hash de representación, metadatos/limitaciones)
   además de generación/fuente/fragmentos/localizadores. Así un cambio de metadatos
   fuera del texto también invalida una selección; no usar sólo texto proyectado.
   No cambiar fingerprints viejos para acomodar un hash distinto.

   Aunque 010 ya permite las filas, **012 incrementa únicamente las dos versiones
   del workspace a 12** en una transacción validada con backup previo. Evita que
   un cliente v11, cuyo reset reconoce perfil v1 pero no XLSX, opere sobre ese
   contenido. No editar 001–011, schema FTS, control/revisión ni selecciones previas;
   registrar 012 en init/migrate/registry. Una evolución futura de la proyección
   exige migración/versionado explícitos, no reinterpretar entradas ya guardadas.
   Alternativas: no versionar (clientes viejos no protegidos); reset/rebuild al
   migrar (pérdida de selección, revisión y hashes); segundo índice (duplica cuotas,
   políticas y ranking); perfil global v2 (rehash innecesario de todo UTF-8/PDF).

6. **Mismo put/get/search y CAS.** Ampliar IndexedExtractor con XlsxCells y las
   ramas de canonical/coverage. put valida/prepara fuente e índice completo fuera
   del escritor; dentro revalida revisión, generación, fuente, hash, cuotas y
   respuesta bajo PageLimit hasta commit. No-op cumple las mismas fronteras.
   Mismo rowid `(slot-1)*512+fragment_ordinal+1`; invalidación por FK/trigger retira
   sólo el job reemplazado. Snapshot/registry y namespace base.v2 de lotes no
   convierten índice en dato original ni amplían las nueve mutaciones.

7. **Validación completa, no confianza acumulada.** verify reconstruye toda
   selección XLSX, manifiesto/proyección/hash, todas las copias y cada posting,
   incluso si el filtro no la incluye o no hay hits. Mantener API auxiliar FTS5,
   bitsets y fallback exacto, estadísticas/candidatos/score confrontados con TEMP.
   Get no lee originales (`not_checked`); search verifica los originales de todas
   las generaciones de página/lookahead (`returned_candidates`), sin refill tras
   error ni promesa sobre fuentes ajenas. No usar caches de confianza entre calls.
   Reset preserva el rechazo de formatos/perfiles futuros y atomicidad existente.

   El diagnóstico del gate mixto fallido ubicó ~161–167 ms en el fallback íntegro
   de más de 256 términos. El primer cambio a C API tipada conservó ese stream
   completo, pero no alcanzó el target. La solución mantiene 256 como frontera
   algorítmica: hasta ella consulta un término por vez; por encima usa OR en lotes
   internos de hasta 64 términos, con `xPhraseFirst/Next` de la API pública FTS5
   para no construir/ordenar el array combinado de `xInst`. Cada término real,
   documento, columna y posición sigue contrastado con bitset/cardinalidad/deadline.
   No es un cap de vocabulario ni permite omitir instancias. El stream C tipado
   completo sigue siendo el fallback y verifica entradas materializadas sin
   afinidad/orden asumidos. Sin páginas FTS privadas, cachés o cambios persistentes.
   Se conservan las etapas fallidas y la comparación diferencial; los gates
   mixtos 11/12 e intactos 3/4 pasaron. Ver ADR 0018 y evidencia de regresión.

8. **Compatibilidad acotada, no igualdad falsa.** API v1 y conteos 52/24-44-45
   quedan iguales; dos operaciones amplían enums/ref defs. `operations.describe`
   anuncia `attachment_index.extractors` y `xlsx_projection` con campos/límites/
   limitaciones. Contrastar automáticamente con baseline: 50 inputs/outputs,
   errores, reader XLSX y whitelist idénticos; sólo adiciones nombradas del índice,
   sus definiciones transitivas y descriptor cambian. Regenerar schema/TS y probar
   schemas MCP por ambas eras. Clientes con enums exhaustivos necesitan actualizar
   bindings; filtros utf8_text/pdf_text conservan tipos conocidos. Agregar XLSX al
   índice puede cambiar BM25 aun con esos filtros: usa estadísticas globales, no
   se promete conservar ranking al cambiar el corpus seleccionado.

9. **Cursores y cobertura sin trabajo implícito.** Reusar cursor v1 ligado a
   scope/ruta/query/filtros/límite/perfil/revisión. Migración de versiones sola
   conserva validez del cursor del índice porque no cambia selección/revisión;
   el siguiente put/reset/reemplazo lo invalida. Tools/list cambia su hash de
   catálogo automáticamente. Recibos batch v11 sí son incompatibles en v12.
   Coverage agrega XLSX compatible por extractor/configuración, no por cuota libre
   o coste de proyectar todas las generaciones no seleccionadas. Un libro vacío
   seleccionado vale cero documentos, como PDF no_text; cero hits no significa
   que se buscaron nombres, formatos visuales o todos los adjuntos.

## Risks / Trade-offs

- Muchos campos chicos agotan 512 antes de 1 MiB → rechazo completo explícito,
  casos 512/513, mantener reader completo; no vender soporte universal de búsqueda.
- Nuevo locator rompe consumidores de enum cerrado → cambios BREAKING explícitos,
  descriptor, bindings regenerados y tests de allowlist, sin fingir igualdad de 52.
- Proyectar/revalidar bajo el escritor agrega coste → preparar fuera, revalidar
  completamente dentro y medir locks. Optimizar con evidencia sin quitar checks.
- Cortes cambian tokens/límites de frase → documentar y probar con golden; no
  prometer coincidencia de palabra/campo entero para cadenas arbitrariamente largas.
- Caché obsoleta/hoja oculta puede confundir → campo/tipo explícitos y guía del
  reader para metadatos/limitaciones. Seleccionar el libro incluye sus hojas ocultas.
- La ampliación puede ralentizar FTS mixto → gates separados fijos (incluido
  vocabulario de más de 256 términos y ambos recorridos de postings), perf antes
  del archivo; no se acepta excepción a 300 ms ni caps más altos.

## Migration Plan

1. Verificar hashes del baseline v11 independiente y registrar ADR 0018 al aplicar.
   Implementar 012 de compatibilidad y rollback/admisión antes de nuevas selecciones.
2. Migrar con CLI open explícito, backup consistente v11 y validación transaccional;
   nuevos workspaces recorren 012. MCP/index/search/preview no migran al admitir.
3. Comparar todas las filas/tablas/DDL/shadows y originales del v11 poblado (índice
   UTF-8/PDF, XLSX extraído, running/futuro) salvo ambas versiones. No helpers.
4. Seleccionar XLSX sólo después de migrar y por put. Verificar backup/restauración
   mixta, corrupción interna con hash externo reparado y publicación concurrente.
5. Rollback de software: binario v11 restaura su backup previo en otra ruta; no
   reducir PRAGMA ni sobrescribir escrituras v12. V13 y perfiles futuros fallan
   cerrados, sin reset ni reparación. Sync/archivo sólo después de aceptación.

## Validation Strategy

**Adenda del 16/10 autorizada por el usuario:** la auditoría temporal original se
perdió y no hay copia. Se repite regresión/gates/recursos con evidencia durable
y una captura del estado actual, no un baseline v11. Las fixtures SQL v10/v11
ejercitan migraciones pero no sustituyen clientes anteriores independientes.
La comparación exacta histórica de archivos/50 contratos y el cliente viejo ya
no pueden repetirse; se registra esa limitación, sin alterar el comportamiento
requerido ni caps/targets. El plan original de abajo queda como antecedente;
el criterio operativo es [la recuperación autorizada](../../../../docs/xlsx-search-evidence-recovery.md).

- Golden independiente para proyección desde el expected del corpus Python del
  lector; no usar el proyector Rust como oracle ni modificar sus 216 golden/pins.
  Casos específicos adicionales en `tests/fixtures/xlsx-search/`, sintéticos y
  fijados antes de medir: campos, vacíos/hidden/shared/array, tokens y Unicode.
- Unit/SQLite y procesos CLI/MCP: campos exactos, rangos/hash, 512/513, cuotas
  mixtas/exceso por un byte, corrupción fuera de filtro/página, CAS/no-op/fences,
  cursores, reemplazos/resultado previo y main READ_ONLY. Helpers ausentes.
- Baseline v11 real debug/release para migración, backup/restore y clientes viejos;
  conservar el harness v10 existente y fingerprints anteriores.
- Gates release, sin procesos concurrentes: dos corridas consecutivas del gate
  UTF-8/PDF **sin modificarlo** y dos del mixto nuevo. Mixto fijado en 8 MiB de
  texto copiado (2 MiB UTF-8/2 MiB PDF/4 MiB XLSX), dentro de retención/snapshot,
  con 5 warmups/20 muestras por caso, query p95 <=300 ms y lock p95 <=150 ms.
  Registrar hashes, consultas, conteos, corpus, entorno M1 Max/64 GiB, p50/p95/max,
  RSS SELF/CHILDREN diferenciados y temporales/scratch real del nuevo flujo.
- fmt/check/Clippy y Rust debug/release, CLI/parser/helper XLSX/PDF, SDK/raw,
  contratos/frontend y 606 checks headless. Hashes protegen Home/host/parsers/
  corpus/gates; sin aceptación nativa ni afirmar que 5a/5b están completas.

## Open Questions

Ninguna decisión material se deja para adivinar durante la implementación.
Los tiempos, RSS y scratch reales se medirán: no son resultados de este diseño.
