# Design

## Context

Ver [proposal.md](proposal.md) para motivación y alcance. Estado inspeccionado al
16/10/2026, antes de implementar: SQLite v12/API v1/52 operaciones; `extraction.search`
ya publicaba `AttachmentSearchPage` y tres variantes de localizador. La UI se
implementó según estas decisiones; ver [estado y evidencia](../../../../docs/verification-evidence-search-ui.md).
Home sigue activo 9/17 e independiente.

Fronteras verificadas en el baseline previo a este cambio:

- `src/App.svelte:109–157,368–403`: Search consulta `search.query`, abre entidades
  con `openSearch` y reconstruye sólo su índice. Comparte editor y guardias;
  `refresh:251–269` puede releer la entidad sin reemplazar un buffer sucio.
- `src/lib/api.ts` usa el contrato generado y `OperationError`; no hace falta otro
  transporte. `crates/noteka-core/src/attachment_search_contracts.rs` define
  consulta, cobertura, fuente y rangos. El hit no contiene nombre/MIME ni dueño.
  `attachment.read` también descarga base64: no es un lookup ligero de metadatos.
- `src/lib/embedded-reads.ts` ya separa vida lógica y cupo físico: descartar IPC
  no cancela Rust. Puede reutilizarse sin modificar lectores de Home/embeds.
- `src/style.css:68–100` usa sidebar/lista/editor; Home tiene rama y shell propios.
  `tests/native/smoke.js:246–259` prueba búsqueda de notas y guardias. El runner
  headless conserva seis sesiones y 14 checks de teclado/intercalados, 606 en total.
- `scripts/headless-bridge.mjs` confina rutas/origen y llama CLI real con red
  denegada. No tiene fixture de evidencia. `tests/headless-run.cjs:79–94` puede
  retener transporte; habrá que probar también respuestas **ya obtenidas** antes
  de publicarlas para cubrir respuestas tardías, no sólo demorar el inicio.

Se guardó un baseline durable de 409 archivos versionables, hashes de cuatro
binarios CLI/MCP y catálogo v12 antes de crear el cambio. La ruta se descubre con
`~/.local/share/noteka-verification/active-evidence-search-ui-path.txt`. Esto es un
baseline pre-UI v12, **no recupera** los baselines históricos v11 perdidos. No hay
commit inicial contra el cual basar `git diff`; se comparará con esta copia.

## Goals / Non-Goals

**Goals:** adaptar la lectura existente, conservar evidencia byte a byte y aislar
estado asíncrono sin trasladar validación de negocio a TypeScript. Ver el delta
de [local-text-search](specs/local-text-search/spec.md), un requisito modificado y
seis añadidos, diecinueve escenarios incluido el heredado de notas.

**Non-Goals:** no ampliar lectores ni abrir archivos externos, no consultar
base64/nombres, no crear selección o extracción desde Search y no reemplazar el
editor por un visor de adjuntos. Sin cambios de Home, Rust, schema, catálogo,
MCP, parsers, corpus/goldens, gates, dependencias o lockfiles.

## Decisions

### 1. Ámbitos separados con integración mínima en App

Añadir selector nombrado «Notas y entidades» / «Adjuntos» dentro de Search,
inicialmente notas. Mantener búsqueda de notas y `openSearch` con filtros, límite
30, guardias y botón de reconstrucción existentes. En adjuntos se monta un nuevo
`AttachmentSearchPanel.svelte` en la columna de lista, con clave de workspace y
vida limitada al ámbito/sección; no se cambia el editor ni la grilla de Home.

`loadList` no debe lanzar `search.query` si Search está en adjuntos, ni publicar
una lectura de notas que terminó después del cambio de ámbito. El selector no
borra selección/buffer: no reemplaza el editor. Navegar a otra sección/workspace
y abrir notas siguen usando las guardias centrales. Header «Actualizar» delega
la consulta de adjuntos al panel, una sola vez, y conserva la relectura de entidad
que ya protege el buffer; no equivale a reconstruir o indexar. Cambiar de ámbito
descarta páginas/cursores de evidencia, no transporta filtros de un contrato al otro.

Alternativas descartadas: ranking combinado falsearía comparabilidad de scores;
reescribir toda Search o mover el editor añade riesgo innecesario a notas/Home;
un lector completo requiere más operaciones y otra definición de frescura.

### 2. Una operación de lectura, sin descubrimiento que descargue originales

El panel sólo invoca `extraction.search` por `operate<Contract['attachment_search']>`.
Entrada: texto tal como se presenta, IDs opcionales con blancos externos retirados
y vacíos omitidos, extractor de la unión generada, `limit:20`, cursor opaco cuando
corresponde. No cortar texto para admitirlo, reinterpretar FTS ni duplicar las
reglas de términos/bytes/UUID en TypeScript: Rust devuelve el error autoritativo.
La UI impide envío vacío/duplicado y explica 1–16 términos/256 bytes; un maxlength
de caracteres no puede presentarse como validación UTF-8.

Sin consultas al montar, debounce, polling, lecturas N+1 por hit ni contenido en
localStorage/logs. Se puede inspeccionar el hit completo recibido sin otra llamada.
Buscar o refrescar nunca llama index/get/put/reset, enqueue/run/retry/control,
readers ni attachment.read. El ámbito de notas conserva su reconstrucción explícita.
Los IDs manuales son una limitación de UX deliberada; nombres/metadata-only lookup
se proponen después si su necesidad se confirma, no mediante base64 oculto.

### 3. Página reemplazable, criterios editables y vida lógica separada del IPC

Un controlador TypeScript nuevo (`attachment-search.ts`) conserva estado inicial,
cargando, página, error y criterios enviados. Sólo guarda la página actual de
hasta 20 hits, su cursor siguiente y contador de página; «Siguiente» la reemplaza,
«Buscar de nuevo»/«Volver al inicio» envía la primera. Sin acumulación, historial
de páginas, auto-scroll/infinite scroll ni conteo de hits a partir de cobertura.

Editar cualquier criterio retira página/detalles/cursor/error, invalida la lectura
y espera un envío explícito. No reutiliza cursores con otra entrada. Una falla
retira la página y cobertura anteriores y ofrece buscar de nuevo desde el inicio,
nunca fallback automático ni reparación. Se muestra código/razón del error del
núcleo con mensaje seguro: validation_error, revision_conflict, workspace_busy,
storage_error, unsupported_workspace_version o fallo de transporte no son cero hits.

Reutilizar `ReadLifetime` por instancia y una `ReadQueue(1)` exclusiva de adjuntos,
compartida por sus instancias durante la sesión, sin modificar esos helpers.
Al invalidar/desmontar se aborta la espera lógica; un IPC iniciado conserva el
cupo físico hasta resolverse. No puede haber más de un IPC de adjuntos activo;
cada instancia admite una sola petición lógica vigente, y al invalidarla elimina
su espera anterior de la cola. No se bloquea globalmente Home/editor mientras
termina una lectura descartada. Éxito, rechazo y finalización sólo publican estado
si siguen vivos la instancia y sus criterios. Desmontar y volver inicia vacío.

Alternativas descartadas: suponer que AbortController detiene `invoke`, iniciar
una lectura por tecla, o instanciar un pool nuevo al alternar permitirían acumular
trabajo real; conservar páginas previas como vigentes ante error mezcla snapshots.

### 4. Evidencia exacta en texto, sin nuevas promesas de origen

Tarjetas con extractor, referencia tipada, texto canónico y detalle desplegable
de fuente. Usar expresiones Svelte, nunca `{@html}`, linkificación, imágenes remotas
ni evaluadores. Preservar saltos/espacios con `white-space:pre-wrap` y hashes/IDs
con `overflow-wrap:anywhere`; no usar ellipsis/slicing sobre la evidencia completa.
El detalle puede plegarse sin descargar ni verificar otra cosa.

- UTF-8: bytes `[byte_start,byte_end)`, líneas originales y ordinal de fragmento.
- PDF: página 1-based, bytes derivados, scope y todas las limitaciones devueltas;
  aclarar que no son geometría, bytes del PDF ni OCR.
- XLSX: proyección `xlsx_cell_fields_v1`, sheet_id/A1, ordinal global de celda
  0-based, campo de los seis posibles, bytes totales y rango `[start,end)`; indicar
  porción si no cubre el campo. Caché no es recálculo; fórmulas no se ejecutan ni
  reciben `=` inventado. No inferir nombre o visibilidad de hoja desde sheet_id.

Todo hit conserva fuente/versión/hashes/generación y flag de intento anterior.
El detalle muestra evidencia verificada **al consultar**, no certificado permanente
de frescura; inspeccionarlo no reabre el original. No crear botones «Abrir original»
que en realidad sólo muestren texto, ni adjudicar el archivo a una nota.

### 5. Cobertura y accesibilidad sin modificar reglas ni Home

Mostrar valores del núcleo: generaciones seleccionadas/compatibles sin indexar/
incompatibles/trabajos sin generación, fragmentos/bytes, perfil, versión y revisión.
Explicar `selected_generations`, `canonical_text_and_postings` y
`returned_candidates`: integridad del índice completo frente a originales de
página/lookahead, no todos los archivos. Ante error no conservar cobertura vieja.
Una selección sin texto no implica OCR o libro vacío; compatibilidad no garantiza
que quepa en cuotas. Indicar que la preparación se hace explícitamente por CLI/MCP.

Labels y controles nativos (select/input/button/details), regiones de carga/estado
y error, Enter/Tab/Space y foco visible. Estilos del componente o selectores nuevos
exclusivos; no tocar `.home*`, estilos globales de grilla ni orden/nav/clima/secciones.
Comprobar ambos temas y el ancho actual de lista con texto largo; no prometer
rediseño responsive general. No capturar escritorio ni iniciar Tauri con ventana.

### 6. Pruebas reales separadas de dobles asíncronos

- Tests Node de promesas controladas para el controlador, pool físico y descarte
  de éxito/error/finally; incluyen cambio de criterios, cola cancelada, dispose y
  nueva instancia. Son pruebas de lógica del cliente, no del núcleo.
- Fixture headless propia, dentro del corpus confinado y en workspace nuevo,
  preparada por operaciones CLI reales: UTF-8, PDF acotado y XLSX con caché/fórmula,
  Unicode, múltiples páginas, porciones largas y fuentes anteriores. Reutilizar
  generadores sintéticos existentes mediante un helper de prueba de invocación
  fija; no cambiar corpus/goldens de aceptación ni insertar hits inventados en SQLite.
- Preparar por CLI casos seleccionados/sin seleccionar/vacíos y trabajos sin
  generación. Para configuración futura incompatible, usar inyección SQL
  exclusivamente sintética sobre una generación creada por CLI, antes del
  snapshot de lectura, como en los tests de recuperación existentes; no simular
  hits ni relajar validación. Comparar UI con lecturas directas y goldens del corpus.
  Cambios de revisión o fuente para pruebas son acciones explícitas del fixture,
  separados del snapshot antes/después de **cada lectura de producto**.
- Extender sólo transporte headless con controles de fixture acotados, registro
  de operaciones y snapshots lógicos/archivos fuente/generaciones/índice. Verificar
  que buscar/inspeccionar no muta y no emite operaciones de preparación. Preservar
  confinamiento de rutas/symlinks, loopback/origen, sandbox sin red y perfil efímero.
- Añadir recorrido headless de evidencia separado de las seis sesiones existentes;
  conservar sus 606 asserts, no editar controles/fixtures nativos para omitir checks.
  Retener respuestas reales, no sólo inicio de llamadas, al cambiar consulta,
  ámbito, sección y workspace; probar resolución y rechazo tardíos. Forzar cursor
  vencido por CLI y fuente alterada sólo dentro del corpus. Medir teclado, texto
  escapado sin requests remotos, guardias y Home sin atribuir cobertura nativa.

## Risks / Trade-offs

- [Lista estrecha y hashes largos] → detalle plegable, wrap sin truncado y pruebas
  de legibilidad/teclado en ambos temas; no ensanchar Home ni vaciar el editor.
- [Archivo cambia después de responder] → indicar momento/alcance de validación;
  nueva búsqueda explícita, no afirmar que desplegar detalle revalida la fuente.
- [Workspace cambia mientras `invoke` sigue activo] → clave, lifetime y cupo
  físico compartido, con respuestas reales retenidas y tests de rechazo tardío.
- [Fixture invade comportamiento de producto] → controles sólo de prueba, corpus
  confinado, snapshots y registro de llamadas; ninguna preparación dentro del panel.
- [Regresión de validación/latencia camuflada por cambio visual] → congelar Rust,
  contratos/corpus/gates; correr ambos gates release intactos dos veces consecutivas
  cada uno, query p95 <=300 ms y lock p95 <=150 ms, sin ampliar caps ni excepciones.
  La latencia de render/IPC headless se informa aparte, no como benchmark Rust.
- [Cierre exagerado de fase o host] → 5a sigue parcial, Home 9/17 sin archivo;
  ni no-bundle ni puente CLI acreditan foco/IPC/cierre WKWebView.

## Migration Plan

No se migra ni se cambia formato persistente, contrato o dependencia. Workspace
ya abierto debe usar v12; la apertura/migración/backup existentes siguen siendo
la única vía explícita para versiones previas. Search no migra por su cuenta.

Antes de implementar, conservar también copias de binarios y helpers existentes,
además de hashes/catálogo/copias fuente pre-UI, fuera de Git. Verificar que el
catálogo y contratos completos siguen idénticos a ese baseline v12, no sólo que
mantienen 52 operaciones. Rollback del frontend restaura la UI anterior sin
downgrade SQLite; nunca restaurar un workspace viejo sobre cambios del usuario.

Al cerrar: matriz de los diecinueve escenarios, regresión headless/contratos/
core/host no-bundle y gates frescos; revisión de diff contra el baseline, fuentes
congeladas y aislamiento de Home; sincronizar **sólo este delta** implementado y
verificado de local-text-search y archivar sólo este cambio. La propuesta y los
checks documentales no satisfacen esos criterios por sí solos.
