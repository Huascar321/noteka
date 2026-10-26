# Design

## Context

Ver propuesta y cinco deltas. Baseline15/API54 y los cinco formatos previos están
verificados, con cliente/checkpoint y fixture independiente de41 tablas/cinco
selecciones antes de016. El lector PDFKit independiente y el OCR de imágenes
ya están cerrados; no se recrean. Home permanece9/17 y no participa.

## Goals / Non-Goals

**Goals:** pipeline PDF→raster real→OCR existente, provenance verificable y una
publicación cercada; la inferencia no sustituye originales ni buffers. Lectura y
búsqueda offline de resultados, sin ejecutar motores al consultar.

**Non-Goals:** ver propuesta. No se promete OCR perfecto ni tablas/orden visual.
El límite actual exige detenerse antes de16:43:39UTC y conservar todo progreso.

## Decisions

### Render descartable separado y Vision existente

Un nuevo helper Swift/CoreGraphics recibe sólo bytes PDF y ordinal de página,
con header privado28B (magic NKPDFR01, versión1, size/ms/page/reservados). Rechaza
framing/cifrado/cero o más de20 páginas. Preflight de todas las MediaBox/rotaciones
antes de dibujar, sin usar el lector textual PDFKit ni su protocolo.

getDrawingTransform(.mediaBox,rect:...,rotate:0,preserveAspectRatio:true) aplica la
rotación del PDF. Raster gris8-bit/fondo blanco, escala objetivo2 reducida para
<=4096 por eje/4MP; total<=16MP. PNG real<=1MiB, con ancho/alto, caja/rotación,
OS/build y SHA256 fuente. JSON/stdout<=8MiB, stderr<=8KiB. CPU4s/watchdog<=5s,
RSS observado128MiB cada5ms; no es límite duro GPU. Sin rutas, flags o secretos.
Rust lanza mediante sandbox-exec deny-network, env vacío/CWD temporal, pipes no
bloqueantes y recolección sólo del hijo. Los probes viven en binario de prueba.

Rust valida header/respuesta/hash/tamaños/base64/geometry y llama al OCR de imagen
ya verificado sobre cada PNG real. Su encode_v2 mantiene cap1MiB y perfil Vision
r3/fast/es/en inalterados. Todos los renders y OCR comparten un Budget5s y cerca;
se ejecutan secuencialmente y liberan el raster al terminar cada página. No hay
nuevo motor/modelo ni bucle Vision duplicado. Si cualquier página falla, no se
publica un prefijo. Alternativa descartada: duplicar Vision y sus validaciones en
un helper PDF monolítico; acopla más código y acumula memoria de ambos frameworks.

### Documento y evidencia

PdfOcrDocument v1 incluye perfil compuesto apple_cgpdf_vision_r3_fast_es_en_v1,
raster_profile cgpdf_media_gray_scale2_png_v1, páginas físicas completas, caja y
rotación, hash/tamaño del PNG, OcrRecognition v2 ya normalizado, motor/OS. El PNG
es derivado efímero, no reemplaza al original ni se representa como fuente PNG
administrada. Se conserva su hash para identificar el espacio de coordenadas.
Cada página exige orientation1, PNG real y dimensiones coincidentes; engines
Vision/renderer deben coincidir en OS/build. Decimales se normalizan en Rust con
el codec OCR existente; texto literal no se modifica.

JSON canónico se serializa una vez para persistir, particionado UTF-8<=4096B.
Dos tablas con owner/FK/version/hash/count; SQL acota tamaño/cantidad/cruces antes
de materializar. Load compara toda representación/partición/metadatos/hash.
Retención se extiende al final de la cadena de seis formatos; snapshots/base.v2
incluyen tablas registradas y excluyen derivados sólo de la huella de originales.

extraction.pdf_ocr_regions pagina regiones globales (1–64/default32), no páginas
PDF; manifiesto contiene descriptores de todas las páginas incluso vacías. Cada
item conserva página1-based y ordinal de región local/global. Cursor usa alcance
existente. Source y todo canónico se verifican sin helper y sin writes.

### Índice y superficies compartidas

Nueva proyección de región a chunks UTF-8<=4096B con límite512/1MiB por selección.
Locator pdf_ocr_region_text incluye perfil, página, región, hash/dimensiones/rotación
del raster, motor, bbox/confianza decimales, marco rendered_page_normalized_bottom_left
y rango del texto inferido. Manifiesto participa sólo en nueva rama de content.v1;
los cinco formatos anteriores no cambian. Validación íntegra, CAS, invalidación,
postings/fuente de candidatos, cuotas8MiB/2MiB y gates300/150 se conservan.

API55 añade sólo lectura, root contract al final para conservar nombres anteriores;
index/search amplían enum/locator. MCP sólo añade lector27/47/48, excluye siete
operaciones previas, conserva batch9. Se reconcilian descriptores viejos de versión
/tools con significado explícito de cliente vigente, sin reinterpretar historia.
Paneles existentes añaden el caso pdf_ocr y veinte regiones; manifiesto completo,
inferencia y coordenadas del raster visibles. App/Home/navegación/temas sin cambios.

## Migration Plan

1. Fixture/cliente/checkpoint15 antes deDDL (ya conservados).
2.016 añade tablas y barrera16 en transacción con backup15 consistente previo.
3. Probar filas/SQL/hashes/cursores/unknowns exactos y tablas nuevas vacías; backup15
restaurable con cliente15, rollback antes/después deversión y admisión15/17 sin writes.
4. Probar seis formatos/32MiB, snapshots64MiB, fuente/derivados/postings corruptos y
originales intactos. No downgrade in-place; restauración en ruta nueva.
5. Sólo tras aceptación: revisión, sync de cinco deltas contrastados y archivo.
Si vence la hora, tasks reales/checkpoint/reporte y cambio abierto, sin sync prematuro.

## Risks / Trade-offs

- Escaneos pueden ser inexactos → corpus propio con ground truth previo, CER/WER y
fallos por separado; confianza no es certeza y no se inventa calidad general.
- PNG gris/raster acotado pierde color o puede no caber → limitación explícita,
original conservado y rechazo entero sin compresión/fallback ocultos.
- Muchos procesos/páginas consumen presupuesto → cinco segundos compartidos,
secuencial/cercado, sin elevar el cap para lograr éxito artificial.
- Memoria nativa depende de OS → watchdog observado, fixtures/probes reales y
provenance OS/build; no se atribuye medición a allocator Rust ni GPU.

## Research

Fuentes primarias consultadas20/10: [transformación Quartz](https://developer.apple.com/library/archive/documentation/GraphicsImaging/Conceptual/drawingwithquartz2d/dq_pdf/dq_pdf.html),
[API actual](https://developer.apple.com/documentation/coregraphics/cgpdfpage/getdrawingtransform(_:rect:rotate:preserveaspectratio:)) y
[Vision CGImage](https://developer.apple.com/documentation/vision/vnimagerequesthandler/init(cgimage:orientation:options:)-63ojm).
La guía archivada explica composición de MediaBox/Rotate; la disponibilidad/firma
se contrastará con el SDK local y los casos reales de rotación. No se presupone
que documentación o fixtures de imagen prueben la nueva ruta PDF.
