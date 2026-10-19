# OCR explícito de imágenes administradas

Implementación verificada de `add-local-image-ocr`; no es cierre de toda 5a.
El lector aislado de [ADR 0019](decisions/0019-bounded-native-ocr-reader.md) se
integra mediante [ADR 0020](decisions/0020-local-image-ocr.md). Rust controla
admisión, cola, persistencia, lectura e índice. SQLite v13/API v1/53; las nueve
mutaciones batchables permanecen iguales.

## Secuencia local

Importar una imagen no ejecuta OCR. Elegir su ID administrado, encolar
`image_ocr`, consultar la cola y reanudarla explícitamente si corresponde. Un
`run` por ID reclama como máximo un trabajo. Leer la generación es otra acción;
seleccionarla mediante `extraction.index` es necesaria para que Search la use.
Abrir paneles, leer, buscar, migrar o restaurar no arranca el helper.

```sh
noteka --workspace /ruta/workspace --json extraction enqueue <<'JSON'
{"attachment_id":"UUID-administrado","extractor":"image_ocr"}
JSON
noteka --workspace /ruta/workspace --json extraction ocr-regions <<'JSON'
{"id":"UUID-del-trabajo","limit":20}
JSON
```

Los placeholders deben reemplazarse por UUID reales. `extraction.control` y
`extraction.index` exigen la revisión consultada; put exige además la generación
esperada. No reintentar automáticamente tras conflicto o pérdida de transporte.
La UI usa las mismas operaciones y conserva los buffers/guardias existentes.
CLI `ocr-regions`, operación `extraction.ocr_regions`, herramienta MCP
`noteka_extraction_ocr_regions`: lector disponible con permiso de lectura.
Ejecutar sigue requiriendo permisos write+run; seleccionar sólo write.

## Alcance y límites

PNG/JPG/JPEG administrados <=1 MiB y MIME compatible; el formato reconocido debe
coincidir con la extensión. Una imagen, <=4096 píxeles por eje y <=4 Mp, EXIF1–8.
Vision r3/fast, es-ES/en-US fijos, sin corrección ni detección automática. Sólo
macOS con el perfil disponible: el catálogo describe el contrato, no garantiza
que ese motor exista en una instalación. Fallar no instala modelos ni activa
alternativas o proveedores. [Contención y límites nativos](ocr-reader.md).

Hasta 512 regiones, 131072 bytes UTF-8 por región y 1 MiB de representación
completa. Se rechaza el exceso, sin truncar. Watchdog/cooperación <=5 s, lease 15 s,
CPU 4 s y RSS nativo **observado**128 MiB por muestreo 5 ms; no límite duro RSS/GPU.
Retención derivada compartida 32 MiB, snapshots 64 MiB; límites previos intactos.
Admisión por tamaño no promete terminar en toda máquina o carga.

Texto inferido: puede ser incorrecto o faltar; no garantiza orden visual,
reconstrucción de tablas ni descripción de la imagen. Confianza del motor no es
probabilidad de exactitud. `complete=true` y `covered_bytes=source_bytes`
certifican procesamiento de la fuente completa con el perfil, no transcripción
perfecta. Regiones vacías constituyen un resultado válido, con manifiesto.

## Evidencia y formato

La representación v1 conserva imagen/orientación, motor/OS/build, espacio
`oriented_normalized_bottom_left`, cinco limitaciones y todas las regiones.
Confianza y bbox son **cadenas decimales canónicas**: transporte privado v2
`NKOCR002` emite `String(Double)`; Rust parsea con su parser estándar y normaliza
una vez con `to_string()`. Hasta 512 bytes por escalar, finito/rango/geometría
verificados. No se cambia serde_json global ni el protocolo privado v1 aislado.

SQLite 013 agrega manifiesto/chunks con ownership compuesto de generación, hash
SHA-256 y partición UTF-8 <=4096 bytes/512 chunks. `fragment_count` del job cuenta
chunks internos, no regiones. El lector revalida original y representación
completos antes de paginar, 1–64/default 32; UI 20. El cursor liga operación,
workspace, job, generación y límite. Leer luego de retry/fallo puede devolver
la generación anterior, identificada expresamente.

La proyección `ocr_region_text_v1` indexa sólo texto reconocido, sin metadata,
umbral de confianza o ranking especial. Localizador `image_ocr_region_text`:
ordinal de región, bbox/confianza, espacio, bytes totales de región y rango del
texto inferido. No son offsets del PNG/JPEG. Search conserva ámbitos separados y
consulta agrupada con grupos/cursores independientes. Selección sin texto puede
producir cero fragmentos y no prueba ausencia universal.

Índice:1024 selecciones/16384 fragmentos/8 MiB de texto, con validación íntegra
de representación, hash, copias, postings y fuentes de candidatos. No cache de
confianza, reindexación implícita, purga ni reparación para superar errores.

## Recuperación y distribución

Migración v12→13 con backup consistente previo, transacción e integridad antes
de commit. Cliente v12 rechaza v13; recuperar el backup con cliente antiguo en
otra ruta. No downgrade in-place. Restore/lectura/búsqueda de resultados
persistidos funcionan sin helper. Configuraciones futuras se conservan y se
rechazan cuando no son compatibles, sin reinterpretar o borrar.

El preparador general compila OCR sólo para target macOS. El overlay
`src-tauri/tauri.macos.conf.json` agrega el tercer helper a los dos previos sin
alterar bundles de otras plataformas. SDK/Swift existentes, sin descargas.
`prepare-ocr-worker.mjs --tauri` respeta target/perfil y nunca distribuye probes.
Si NOTEKA_TEST_TMP no se define, la compilación usa un directorio temporal nuevo
fuera del checkout. Build no-bundle y pruebas headless no prueban IPC/foco/cierre
nativo; Home continúa aparte 9/17.

[Aceptación técnica](verification-image-ocr.md), [matriz](verification-image-ocr-scenarios.md) y [revisión](review-image-ocr.md).
