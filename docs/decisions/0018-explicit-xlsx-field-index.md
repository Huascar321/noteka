# ADR 0018: proyección explícita de campos XLSX y barrera v12

Fecha: 15/10/2026, revisada 16/10. Estado: adoptada y revalidada.

## Contexto

La extracción XLSX v11 conserva un Workbook canónico; el índice genérico sólo
admite UTF-8/PDF. Sus filas pueden almacenar XLSX sin DDL nuevo, pero un cliente
v11 reconoce el perfil y podría resetear selecciones que no entiende.

## Decisión

- Proyectar caché tipada y texto de fórmula por separado desde el Workbook
  completamente validado, sin ZIP/XML, helper ni recálculo. Incluir hojas ocultas.
- Mantener strings exactos y booleanos `true`/`false`; omitir campos ausentes/vacíos.
  Cortar por UTF-8 hasta 4096 bytes, sin overlap, trim, etiquetas o normalización.
- Localizador cerrado `xlsx_cell_text`/`xlsx_cell_fields_v1`: hoja/A1, ordinal de
  celda, campo, longitud total y rango del campo. El ordinal del fragmento no es
  ordinal de celda/chunk ni offset al contenedor.
- Selección completa explícita; rechazo si excede 512 fragmentos/1 MiB, sin purga
  ni truncamiento. Mantener todas las cuotas, validaciones y target de 300 ms.
- Migración 012 sólo actualiza ambas versiones a v12, con backup/transacción;
  conserva esquema FTS, perfil `literal_unicode61_v1`, filas, revisión y hashes
  UTF-8/PDF. El hash XLSX incorpora manifiesto completo y proyección localizada.
- API v1/52 y MCP 24/44/45 no cambian de conteo; index/search amplían sus enums
  cerrados. Los otros 50 contratos y el reader XLSX se preservan.

## Alternativas descartadas

Indexar JSON confunde chunks/metadatos con evidencia. Reparsear el original añade
ejecución innecesaria. Concatenar campos mezcla fórmulas/caché. Aumentar el stride
rompe rowids/cuotas. Reset/migración a perfil global v2 pierde selección o exige
rehash de datos intactos. No subir versión deja clientes anteriores sin barrera.

## Consecuencias

Libros legibles pueden no caber en el índice. AND/tokens no cruzan fragmentos.
La caché no acredita cálculo actualizado; nombres/estilos no se buscan. BM25 puede
cambiar al agregar libros por sus estadísticas globales. Consumidores de enums
cerrados deben actualizar bindings. Volver al software v11 requiere restaurar su
backup en otra ruta, nunca bajar versiones in-place sobre escrituras v12.

## Recorrido exacto de postings

El gate mixto fijado encontró query p95 366,10 ms; se conserva el fallo. El
diagnóstico aisló ~161–167 ms en el stream completo de postings (verify ~248–258
ms). Para reducir conversión dinámica por celda se implementó el mismo SELECT fijo de
cuatro columnas con C API/statement RAII en `attachment_postings.rs`. Conserva
todos los tipos/valores y la prueba exacta con bitsets/cardinalidad, sin quitar
el fallback a fts5vocab(instance). El stream
materializado independiente conserva igual trato. No toca schema, perfil, hash,
copias, fuentes ni presupuestos; requiere regresión diferencial y gates nuevos.

La mejora a ~142–145 ms no bastó (gate 7, página p95 336 ms). OR en lotes de
64 con `xInst` bajó postings a ~91–94 ms, pero el segundo gate falló a 343,14 ms.
La decisión final conserva 256 como frontera **algorítmica**, no como límite de
entrada: hasta 256 rige el recorrido previo de un término; por encima, los lotes
OR recorren cada frase mediante `xPhraseFirst/Next`, sin materializar/ordenar el
array combinado de instancias. El término de cada frase se liga al vocabulario
real; se verifican tamaño de frase, documento, columna, offset, duplicados y
cardinalidad completos. Presupuesto por término/1024 instancias y progress handler
siguen vigentes. No se deduplica evidencia, se cachea confianza o cambia el ranking.

Esto sustituye la elección inicial de usar siempre el stream por encima de 256;
no sustituye su fallback íntegro ni su oracle independiente. Streams materializados
no se redirigen a un índice diferente. Tests diferenciales cubren 255/256/257,
lotes parciales, vocabulario grande, Unicode/repeticiones/filas dispersas, vacíos,
corrupción y tipos/offsets hostiles. El diagnóstico final tomó ~54–55 ms en postings;
dos gates mixtos consecutivos pasaron query p95 164,92–197,02 ms y dos intactos
112,62–127,04 ms, sin cambiar corpus/caps/validación. Tras perder la auditoría
temporal, el usuario autorizó repetir aceptación con evidencia durable y declarar
la comparación v11 exacta no repetible. Regresión fresca 287 Rust por perfil y
606 checks headless aprobada; dos gates mixtos nuevos **167,02–270,74 ms** y dos
intactos **114,31–152,19 ms** p95, mismos targets y corpus. Ver el [incidente y
criterio autorizado](../xlsx-search-evidence-recovery.md): no se recuperaron los
clientes/catálogo viejos ni se presenta una fixture SQL como baseline independiente.

API consultada el 15/10/2026: [FTS5 auxiliary API](https://www.sqlite.org/fts5.html#custom_auxiliary_functions).
`xPhraseFirst/Next` devuelve la misma información de instancias que `xInst`,
separada por frase; iteradores sólo viven durante la fila/llamada actual.

Los punteros de texto sólo se toman tras SQLITE_ROW y verificar tipo original;
text precede bytes, NULL/OOM se rechaza, y ningún buffer cruza step/reset/finalize.
Ver [contrato C de valores](https://www.sqlite.org/c3ref/column_blob.html) y
[sqlite3_step](https://www.sqlite.org/c3ref/step.html), consultados el 15/10/2026.

Ver [diseño](../../openspec/changes/archive/2026-10-16-add-local-xlsx-search/design.md) y
[verificación](../verification-xlsx-search.md). No modifica Home ni habilita modelos.
