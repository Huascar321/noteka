# Design

## Context

Motivación/alcance en [proposal](proposal.md); fuentes y limitaciones comprobadas en [research](research.md). Baseline observado el 13/10/2026: SQLite v8/API v1/48 operaciones y cola UTF-8 archivada. La implementación incorpora v9/49, helper/reader y corpus de 235 casos; [verificación integrada](../../../../docs/verification-pdf-extraction.md). Cierre 21/21, sincronizado y archivado el 14/10/2026, con evidencia de cada gate. Home continúa 9/17 con aceptación nativa pendiente y sin modificaciones funcionales por este cambio.

Fronteras de partida (v8), conservadas o ampliadas por este cambio:

| Superficie | Supuesto vigente / efecto del cambio |
| --- | --- |
| `text_extractor.rs:8–45`, `extraction.rs:175–245` | Literal UTF-8, offsets/líneas del original y comparación exacta contra sus bytes. Preservar módulo/DTO/reader, no aplicarlos a PDF. |
| `extraction_contracts.rs`, `extraction_storage.rs:73–77,292–368` | Sólo utf8_text/config default; fuente valida extensión/MIME, archivo regular, hash y cap aun si crece. Separar elegibilidad por extractor sin debilitar path/hash/IO. |
| `extraction_worker.rs:150–174,207–309` | Claim corto, extracción fuera del lock y publicación fenced; presupuesto derivado calculado sobre source_bytes porque hoy equivale a texto. Despachar por extractor y contar texto real. |
| `migrations/008.sql` | Generación compartida exige covered_bytes=source_bytes; fragmentos actuales sólo literales. Añadir estructuras PDF sin reescribir 008 ni sus tablas. |
| `workspace.rs:260–309`, `batch_storage.rs:19–35,54–100` | Backup previo/migraciones numeradas e inventario SQL exacto; base v2 incluye control original pero excluye jobs/derivados. Incorporar 009 a listas/integridad/snapshot, sin cambiar autoridad. |
| `src-tauri/src/lib.rs:9–24`, `tauri.conf.json`, CLI/scripts headless | Host delgado usa core en proceso; bundle inactivo y ningún helper. Añadir resolución/preparación de runtime local, no UI ni shell del frontend. |

## Goals / Non-Goals

**Goals:** admitir un primer subconjunto útil y verificable de PDF sin falsas garantías; mantener contratos literales y fuente de verdad; supervivencia de CLI/host ante aborts del parser; presupuesto mixto exacto, migración recuperable y paridad core/CLI sin pantalla.

**Non-Goals:** una sandbox de privilegios o una transcripción visual completa. Un proceso separado reduce daño de crash/memoria, pero no constituye aislamiento de permisos frente a una explotación. El listado completo de funciones excluidas vive en proposal; ampliar fuentes/fuentes tipográficas requiere otro recorte con corpus y versión de extractor propios.

## Decisions

### 1. Parser puro Rust en proceso descartable

Añadir `crates/noteka-pdf-worker/` con binario `noteka-pdf-worker`, `lopdf =0.45.0`, sin features default/async/rayon/imágenes. El núcleo no enlaza ni ejecuta lopdf dentro del proceso cliente; el helper no depende de noteka-core/SQLite y sólo transforma bytes acotados a resultado de páginas. DTOs privados de IPC pequeños pueden vivir en una crate compartida sin parser/DB si evita duplicación; no entran en el catálogo público.

Core conserva selección, fuente, ownership, límites y publicación. CLI y Tauri no implementan reglas PDF; únicamente hacen disponible el ejecutable propio. El helper recibe **bytes**, no una ruta, conexión SQL, token de ownership, workspace, credenciales ni órdenes del contenido. No abre enlaces, evalúa scripts/macros/acciones ni inicia otros programas; entorno limpiado, sin logs de objetos del parser.

**Alternativas:** parser en el core con `catch_unwind` no contiene stack overflow/OOM/abort ni garantiza checks cooperativos; Pdfium añade motor C++/artefactos nativos innecesarios para este subconjunto; un daemon añade lifecycle/concurrencia antes de necesitarlo. No hay fallback en proceso, herramienta del sistema o red si falta el helper.

### 2. Subconjunto conservador y rechazo de degradaciones

Elegibilidad de metadatos: extensión `.pdf` case-insensitive, MIME `application/pdf` o `application/octet-stream`, adjunto administrado <=1 MiB. Enqueue conserva el patrón vigente: valida metadatos/fuente, no parsea. Run valida cabecera al inicio del archivo `%PDF-1.0`–`%PDF-1.7`; otras versiones/preambulos fallan explícitamente. Un PDF de cero bytes o cero páginas no es el caso vacío de UTF-8.

Cargar con `strict=true`, `password=None` y cap de descompresión de 1 MiB por stream de carga. Rechazar `was_encrypted()`/cifrado original o error de decryption, incluso contraseña vacía; no confiar sólo en `is_encrypted()`. Resolver árbol de páginas físico sin ciclos/duplicados ni discrepancias de Count; visitar exactamente 1–100 páginas. Recursos heredados se resuelven dentro del helper y se validan antes de extraer.

| Contenido usado | Regla inicial |
| --- | --- |
| Texto directo `Tj`, `TJ`, `'`, `"`, delimitado por BT/ET y fuente Tf válida | Admitir operandos bien tipados y secuencia textual válida; el orden es de operaciones, no de posición visual. |
| Fuentes latinas Type1 estándar | Sólo Courier/Helvetica/Times y sus doce variantes estándar no embebidas; Encoding ausente/StandardEncoding/WinAnsiEncoding. Rechazar ToUnicode, Differences, FontFile y otros subtipos/nombres usados. Symbol/ZapfDingbats no incluidos. |
| Decodificación | Cada byte textual usado debe tener mapping conocido y válido; rechazar NUL, glyph sin mapping, replacement/fallback o pérdida. No `from_utf8_lossy`. |
| Imágenes XObject | Identificar e ignorar su contenido visual sin render/descompresión de imagen ni OCR; manifiesto explicita la limitación. |
| Form XObject usado, inline image no admitida, texto de annotations/AcroForm/appearance u operador textual desconocido | Fallar todo el intento con `pdf_unsupported_feature`; no certificar páginas con texto omitido. Links/acciones no textuales pueden ignorarse como datos, nunca ejecutarse. |
| Geometría/colores/dibujo y marked content sin sustitución textual | No interpretan lectura visual; validar estructura/operandos relevantes. ActualText u otra sustitución textual fuera del subconjunto se rechaza. |
| Recuperación/diagnóstico del parser | Interceptar Warn/Error con bandera de degradación sin retener/formatear mensaje; cualquier fallback/recuperación detectado impide éxito. Auditar también caminos silenciosos, no usar logs como única validación. |

Obtener contenido/streams/encodings mediante APIs limitadas. Prevalidar integridad de streams, referencias vivas, fuentes, mappings y operadores antes de construir texto por página con el adapter. No se usa `extract_text_with_limit`: sus fallbacks crudos o `Ok(String)` no prueban fidelidad. Decodificar streams sin pérdida y contrastar xref/trailers originales y AST de miembros comprimidos, no sólo el AST final del loader. Si el helper no puede distinguir un camino con pérdida dentro del subconjunto, se rechaza conservadoramente o se corrige un adapter acotado; un rechazo provisional no cierra el gate ni reduce el soporte final decidido.

El texto canónico es la salida UTF-8 del algoritmo versionado para cada página admitida, sin trim, normalización Unicode ni reconstrucción de columnas. La semántica de espacios/saltos producidos por las operaciones queda fijada por fixtures del extractor v1. Una página sin operaciones textuales tiene texto vacío, no el salto sintético de un objeto BT/ET vacío. Todo resultado lleva limitaciones constantes `no_ocr`, `no_visual_order`, `no_geometry`, `no_structured_tables` y scope `page_text_objects`.

El auditor recorre cada revisión original (incluidas reemplazadas/free), framing y
longitudes resueltas en el índice de esa revisión. Audita sufijos, offsets,
solapamientos/comments, precedencia más-reciente y suplemento hybrid. La lectura
completa admite el framing de linearización sin usar sus hints para range IO.
Después añade una **vista de carga** con xref efectivo sintético sin Prev y offsets
originales, con entrada propia; no reconstruye objetos ni repara offsets. Elimina
esa entrada sintética tras cargar y contrasta otra vez índice/AST originales. Esto
evita que lopdf pierda tombstones/free o seleccione la tabla por sobre su suplemento.
Prev en suplemento hybrid ambiguo se rechaza explícitamente; un offset Prev
inválido es invalid_pdf, no una elección de recuperación silenciosa.

**Alternativa:** aceptar cualquier salida de una biblioteca y llamarla transcripción completa es incompatible con los fallbacks observados. El subconjunto restringido rechazará muchos PDFs de productores modernos: es una limitación deliberada y documentada, no paridad PDF genérica.

### 3. Operación de lectura aditiva y medidas inequívocas

Reutilizar `extraction.enqueue` con `{attachment_id, extractor:"pdf_text", config?:{version:1,chunk_bytes:4096}}`; default idéntico de configuración, identidad de extractor distinta. El mapping a implementación/versión se fija en Rust; no se acepta contraseña, helper path, comando o tuning arbitrario del cliente. Dedup conserva attachment/hash/extractor/version/config_hash. `supported` describe soporte de configuración, no instalación de binario; consultar no ejecuta un probe.

Añadir sólo `extraction.pdf_fragments` y CLI `extraction pdf-fragments`. Entrada `{id, limit?, cursor?}` estricta, 1–64/default 32. Salida dedicada `PdfFragmentPage`:

- `job_id`, `generation` con el **shape previo** de ExtractionGeneration, `result_from_previous_attempt`, `items`, `covers_generation`, `next_cursor`.
- `manifest`: version 1, parser name/version, scope, limitaciones, page_count, extracted_bytes y descriptores de **todas** las páginas (page_number, state text/no_text, text_bytes, text_sha256, fragment_count). Como máximo 100 descriptores, sin otro cuerpo de texto duplicado.
- `items`: ordinal global, text y `locator:{kind:"pdf_page_text",page_number,text_byte_start,text_byte_end}`. Rangos 0-based half-open del texto canónico de esa página, continuos y en fronteras UTF-8; no líneas/geometría ni offsets binarios PDF.

`source_bytes` y `covered_bytes` permanecen iguales: certifican fuente completa verificada por hash, **no cantidad de texto recuperado**. El manifest distingue extracted_bytes; `generation.complete=true` certifica que se recorrieron todas las páginas del algoritmo admitido, no que todas las regiones visuales fueron transcritas. `covers_generation` sólo es true si la llamada devuelve todos los fragmentos; retornar todos los descriptores de página no altera esa regla. Documento sólo-imagen válido puede tener succeeded/0 fragmentos/manifest no_text: sin OCR, no equivale a visualmente vacío.

El lector `extraction.fragments` permanece **exclusivo de utf8_text**. Al ampliar `store::supported`, no usar ese booleano genérico para aceptar PDF en el lector literal: cada lector comprueba su extractor/version específicos y devuelve `unsupported_generation_config` para el otro. Así no se renombra/reinterpreta `byte_start/end/line_start/end`, no aparecen nulls y se conserva API v1. Los nuevos roots de generación/schema se agregan **al final** de Contract para evitar renombrar Page<T> anteriores. Comparar los 48 esquemas input/output/error y definitions existentes; sólo Operation/catálogo amplían su enum y metadata de soporte.

**Alternativas:** unión/tag nuevo en fragments o campos previos nullable romperían clientes; bytes PDF ficticios engañarían la procedencia. La lectura aditiva cuesta una operación, pero deja el contrato anterior intacto.

### 4. Persistencia aditiva y presupuesto de texto real

`009.sql` agrega tres tablas sin ALTER/rebuild de tablas previas:

| Tabla incorporada | Datos / restricciones |
| --- | --- |
| `extraction_pdf_generations` | Una fila por generation/job; manifest version, parser/version, page_count 1–100, extracted_bytes 0–1 MiB. FK compuesta a generación compartida, cascada y unicidad. Scope/limitaciones de versión conocida son constantes verificadas, no strings libres del helper. |
| `extraction_pdf_pages` | FK a manifest, page_number 1–100, state text/no_text, text_bytes/text_sha256 y fragment_count. PK generation/page. Página vacía: cero bytes/fragments y SHA-256 de texto vacío. |
| `extraction_pdf_fragments` | FK generación/job y generation/page; ordinal global 0–511, texto 1–4.096 bytes y rangos text_byte_start/end con longitud exacta. PK generation/ordinal y orden/localización sin mezcla de páginas. |

Mantener filas compartidas de jobs/runtime/generations, incluido `fragment_count` global <=512 y `covered_bytes=source_bytes`. Generación PDF usa sólo tablas PDF; utf8_text sólo extraction_fragments. Integridad de tipos conocidos comprueba exclusividad, páginas consecutivas, contadores/sumas, rangos continuos, reconstrucción UTF-8/hashes de página y pertenencia job/generation/source/config/attempt. Configuraciones futuras conservables no se reserializan/descartan ni se ejecutan para validarlas; leer fragmentos de versión no soportada falla explícitamente.

El budget de 32 MiB usa suma de `length(CAST(text AS BLOB))` de los fragmentos confirmados de **ambos** extractores, no source_bytes/covered_bytes de PDF ni tamaños JSON/SQLite. Calcularlo dentro del lock de publicación con enteros comprobados, descontando sólo el resultado anterior del mismo job que se reemplaza; validar también totales de manifests. Sin purge/VACUUM/reparación automáticos. Los metadatos/páginas sí cuentan en el snapshot SQLite completo <=64 MiB, no se eliminan para acomodarlo.

Publicar manifest, páginas, fragmentos, generación compartida y succeeded en **una** transacción corta, revalidando fuente y fence antes de commit. Resultado previo sólo se elimina dentro de esa transacción válida; cascadas y rollback no pueden dejarlo parcialmente borrado. Jobs/config originales/entidades/FTS/historial no reciben escrituras ficticias. Parser y preparación de fragmentos quedan fuera del lock.

Reconstrucción/hash de todas las generaciones PDF se verifica en snapshot read-only
antes de claim; dentro del lock se vuelve a comprobar cola/ownership/configuración,
pertenencia/conteos/bytes, y publicación verifica fence/fuente/budget sobre el estado
actual. Repetir hash/reconstrucción de 32 MiB bajo el escritor excedió 150 ms p95
en el primer benchmark; se separó esa validación sin suprimirla. No existe caché
persistente de integridad ni permiso para reparar corrupción. Backup/restore y el
lector conservan validación completa. El contador allocator del padre es exclusivo
del binario de unit tests para medir Layouts; no añade allocator/cap al core real.

### 5. Supervisión, IPC y fallos sin ventana

Lanzar helper con `std::process::Command` sobre ruta fija del deployment, sin shell/PATH/CWD search, entorno limpiado y pipes privados. CLI/host resuelven sibling de `current_exe`; ejemplos/tests Cargo admiten el padre del directorio conocido `examples`/`deps`. Una inyección programática de runtime para pruebas queda fuera de JSON/DTOs/SQLite; producción no toma path desde el adjunto o workspace. Comprobar binario regular/no symlink y handshake protocol/extractor/parser version, sin afirmar autenticación contra modificación privilegiada del ejecutable.

Protocolo v1: encabezado acotado/fijo con magic/version, longitud de fuente y tiempo **restante**; después bytes crudos del original (sin base64/archivo temporal). Helper comprueba caps y devuelve un único resultado JSON privado con identidad/version/hash de fuente y textos por página, o código de error de una lista cerrada. Padre trata toda respuesta como entrada no confiable: JSON estricto/version/source hash/page_count/suma/texto/rangos/coherencia antes de construir fragments en Rust. Nada de SQL/ownership en IPC. Stdout privado <=8 MiB permite el escape JSON del texto <=1 MiB; no aumenta el transporte público de 2 MiB. Stderr <=8 KiB, drenado sin guardar contenido bruto; sólo categorías sanitizadas.

Pipes no bloqueantes o IO cooperativo dedicado **con finalización verificada**, para evitar deadlock al escribir fuente mientras el hijo escribe salida. Supervisión <=25 ms entre comprobaciones de presupuesto/ownership; no mantener transacción de escritura al esperar. Al perder fence, deadline, IO o cap: cerrar input, matar helper, recoger exit/reap y finalizar drains; cleanup SQLite conserva su fencing condicional vigente. No esperar indefinidamente a un hilo que está en un read/write bloqueante. Orden de salida: estado de cola más reciente prevalece sobre el fallo tardío del parser.

| Presupuesto vigente | Control |
| --- | --- |
| Fuente 1 MiB; 100 páginas; 1 MiB texto total; 512 fragments de 4.096 | Validación en helper y nuevamente en core antes de publicar; exceso falla sin truncar. |
| Stream de carga 1 MiB; contenido descomprimido de una página 4 MiB | LoadOptions y APIs limitadas; no decoder sin cap. Son límites de buffers, no tamaño total del proceso. |
| Heap Rust vivo del helper 128 MiB | Global allocator acotado con cuentas atómicas alloc/alloc_zeroed/realloc/dealloc; al superar, terminación con código fijo sin asignar/formatear contenido. Auditar/revisar unsafe, overflow y rollback de reserva. No llama a esto un cap de RSS ni de memoria nativa/stack. |
| CPU 4 s; core dump 0 | Límites macOS en helper antes de leer/parsing; error al aplicarlos impide procesamiento. No usar RLIMIT_RSS como techo de memoria. |
| Presupuesto core 5 s desde claim; lease 15 s; contención/cleanup 2 s | Conservar Budget/fence actuales; el helper recibe restante, no cinco segundos adicionales. Timer propio monotónico desde inicio limita lectura/parsing/salida aunque el padre muera; watchdog no depende de actividad de CPU. |
| Salida privada 8 MiB / stderr 8 KiB / pública 2 MiB / cursor 2 KiB | Lecturas cap+1 y rechazo; error nunca incluye objetos PDF. Kill/reap y cleanup son finalización separada acotada, no extensión para publicar después del deadline. |

El helper no crea hilos de parsing paralelos ni descendientes; un watchdog con stack pequeño puede ser el único hilo adicional. Al morir el padre, vence por su timer propio aun si parsear nunca retorna; el hijo no puede publicar porque carece de DB. Control de core dump se aplica antes de manipular contenido. OOM/abort quedan en el hijo; las pruebas verifican que el proceso cliente sigue atendiendo otro trabajo/edición.

Motivos previstos, mapeados a códigos públicos existentes: validation_error para `unsupported_source_format`, `invalid_pdf`, `pdf_encrypted`, `pdf_unsupported_feature`, `pdf_decode_error`, `pdf_page_limit`, `pdf_text_limit`, `pdf_stream_limit` y caps compartidos; storage_error para `pdf_worker_unavailable`, `pdf_worker_incompatible`, `pdf_worker_protocol`, `pdf_worker_failed`, `pdf_memory_limit` e integridad; workspace_busy para `execution_deadline`, contención/ownership. Mensajes cerrados, sin raw stderr/panic. El error confirma ID/revisión/intento vía mecanismo actual; si cleanup falla se conserva cleanup_pending/recovery explícita. Pausa/cancelación puede devolver interrupted con el estado confirmado por otro cliente, no transformarlo en failed arbitrario.

### 6. Lectura pura, backups y lotes

`pdf_fragments` fija snapshot de lectura, comprueba source metadata/hash/archivo con el IO seguro común (sin parser), valida manifest y todos sus fragmentos acotados (<=1 MiB/512) para certificar hashes/rangos, y luego pagina por ordinal global. No reparsea PDF en cada página ni certifica igualdad literal con el binario. Cursor v1 específico de operación y scope vigente; cambios de generación/ruta/límite o cursor del reader UTF-8 fallan. Config desconocida devuelve incompatibilidad, no purga/reparación. Las lecturas no recuperan ownership expirado ni requieren el helper instalado.

Backup/restauración incluyen las tres tablas y la generación compartida confirmada; helper, buffers/IPC/PID/timers no son datos persistentes. Validación de backup verifica la integridad PDF sin lanzar parser; restore conserva token/estados como antes pero el scope de otra ruta impide reconocer ejecución activa/cursor ajeno. Leer/abrir no reintenta ni recupera mutando.

Añadir 009 al registry SQL de batch, sin modificar ORIGINAL_TABLES, namespace `noteka.batch.base.v2`, receipt/plan v1 ni las nueve mutaciones y database.query admitidas. Las nueve extraction.* quedan excluidas de steps/checks. Snapshot contiene **todo** SQLite PDF <=64 MiB; apply reejecuta dominio en original, sin promover/borrar la copia. Procesamiento PDF entre preview/apply no invalida por sí solo un recibo; original/control/schema sí. Un recibo de v8 falla tras migrar porque cambiaron workspace_meta y schema vinculados, aunque el namespace siga en v2.

### 7. Preparación local del helper y aceptación sin host gráfico

Agregar preparación reproducible del binario para CLI, cargo examples/tests y Tauri debug/release; no guardar artefactos binarios en Git. Configurar staging/nombre target-triple compatible con externalBin para evolución futura, conservando `bundle.active=false`. No añadir tauri-plugin-shell/permisos de shell en frontend ni abrir un proceso para describir catálogo/importar/leer.

Pruebas de filesystem/layout simulan `Contents/MacOS` y Cargo target dir e invocan el helper real sin iniciar Tauri. `tauri build --no-bundle` verifica compilación/preparación, **no IPC/ejecución nativa ni distribución**. El dispatcher JSON compartido y tests core/CLI prueban dominio; no se atribuye a esto aceptación nativa de Home. Sólo headless para regresiones UI.

Corpus generado sintético con generator/semilla/hash/licencia del propio repositorio: digital Standard/WinAnsi, multibyte, texto largo/multipágina, vacía/imagen/mixta, BT/ET vacío, recursos heredados, Tj/TJ/'/", malformed/truncado/encrypted-empty/encrypted-password, tipos/fuentes/maps excluidos, Form/annotations/ActualText, ciclos y deep nesting, bombs de contenido/xref/object streams, fronteras/exceso y contenido activo. Golden compara bytes canónicos y manifest; no incorporar PDFs privados ni archivos descargados sin permiso/licencia.

Medir release M1 Max con cinco warmups/veinte repeticiones por corpus y tamaños registrados, p50/p95 run/claim/publicación/locks, salida/DB/heap y peak RSS padre/hijo por separado. Objetivo orientativo de lock p95 <=150 ms; 5 s es presupuesto de admisión, no promesa de parsear cualquier PDF. Si el subconjunto permitido no pasa o se exceden límites, diagnosticar y ajustar **artefactos antes de ampliar alcance**; no publicar benchmark de proveedor como evidencia local.

## Risks / Trade-offs

- [Muchos PDFs usan fuentes fuera del subconjunto] → error explícito, lista admitida exacta y corpus; no «soporte PDF completo». Ampliar fuentes/ToUnicode es otro cambio.
- [Funciones limitadas aún hacen fallback] → preflight estricto, señales sanitizadas y auditoría de caminos silenciosos; no aceptar resultados por ser Ok.
- [Múltiples streams/objetos amplifican memoria] → caps de descompresión y heap total del helper, CPU/timer; RSS observado se reporta aparte, sin garantía de sandbox.
- [Hijo muere o padre cae durante pipes] → IO cancelable, timer propio, kill/reap probado con proceso real; no retry/desbloqueo por PID ni thread colgado.
- [Cap de heap deja código unsafe nuevo] → módulo mínimo revisado, pruebas concurrentes/realloc/overflow y fallo cerrado; no allocator custom dentro de la app/core.
- [Igualdad cubierta/fuente parece cantidad de texto] → documentación/manifest extracted_bytes/localizador tipado y tests donde tamaños difieren.
- [Tablas faltantes o generación mixta corrupta] → integridad por tipo/count/hash/FKs, backup rechazado sin reparar ni parsear; registry exacto y caps de snapshot.
- [Preparación local se confunde con distribución] → bundle inactivo, sólo no-bundle/layout tests, firma/notarización/publicación diferidas.

## Migration Plan

1. Capturar baseline acotado de código/contratos/main specs y Home; el repo sigue sin commit inicial, no depender de `git diff` vacío como evidencia.
2. Implementar contratos/helper/gates del parser primero; registrar versiones/features/licencias verificadas antes de habilitar pdf_text. No instalar modelos.
3. Incorporar 009 a bootstrap/migración/registry/integridad/backups y fixtures de versiones (incluidas las que hoy eliminan tablas para bajar a v7). Migración es backup previo + transacción; no borrar/resetear cola ni recuperar running. Rechazar formatos >v9 antes de modificar archivos.
4. Integrar worker/reader/CLI y preparación runtime; comprobar snapshots/recibos y paridad sin UI.
5. Ejecutar integración/procesos/RSS/regresión, matriz de requisitos/escenarios y docs. ADR 0012 adoptada con implementación y gates verificados, sin ampliar el alcance ni certificar PDF genérico.
6. Rollback de software: cliente v8 rechaza v9; restaurar backup v8 en ruta nueva y sólo con operación explícita. No downgrade destructivo ni sobrescritura que pierda escrituras posteriores; formatos desconocidos permanecen intactos.
7. Sincronizar sólo estos tres deltas **implementados y verificados**, luego archivar. No publicar specs propuestas en main ni sincronizar/archivar Home. 5a/5b y non-goals permanecen pendientes.

## Open Questions

Ninguna decisión de alcance pendiente. Rendimiento, árbol transitivo y contención son gates de verificación específicos, no permiso para cambiar parser/formatos/contratos en silencio. Se difieren a cambios posteriores fonts/ToUnicode, OCR/render, UI de cola y firma/distribución, sin que bloqueen la planificación del subconjunto actual.
