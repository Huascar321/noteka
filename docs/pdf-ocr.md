# OCR local de PDF

Estado: `add-local-pdf-ocr` verificado12/12, cinco deltas sincronizados/contrastados
y archivado como `2026-10-20-add-local-pdf-ocr`. [Evidencia](verification-pdf-ocr.md).

La preparación explícita `pdf_ocr` acepta un PDF administrado de hasta1MiB. Rust
manda sólo bytes a un rasterizador CoreGraphics descartable y reconoce cada PNG
real con el helper Vision de imágenes existente. No instala OCR, modelos ni
servicios. SQLite y el PDF original siguen siendo la fuente de verdad; los PNG
son derivados efímeros. No hay OCR al importar, leer, buscar o abrir paneles.

El perfil compuesto es `apple_cgpdf_vision_r3_fast_es_en_v1`; el raster usa
`cgpdf_media_gray_scale2_png_v1`, MediaBox y la rotación declarada por el PDF,
gris8-bit/fondo blanco, escala objetivo2, máximo4096/eje,4MP/página y16MP/documento.
Se admiten1–20 páginas completas, incluso sin regiones. Cada PNG debe caber en
1MiB. Un fallo rechaza el documento entero, sin truncar ni publicar prefijos.
Render y Vision comparten el presupuesto5s del claim y su cerca de dominio.

Ambos helpers se ejecutan con red denegada, entorno vacío y CWD temporal. El
rasterizador tiene CPU4s, watchdog<=5s y máximo RSS observado128MiB cada5ms; no
es un límite duro de asignación o de GPU. Pipes acotados8MiB/8KiB y recolección
del hijo propio. Probes viven únicamente en un ejecutable de prueba separado.
El lector PDFKit textual independiente y su protocolo no cambian.

`extraction.pdf_ocr_regions` / CLI `extraction pdf-ocr-regions` pagina regiones
con ordinal global0-based, página física1-based y ordinal local0-based; default32,
máximo64. UI usa veinte. El manifiesto completo describe todas las páginas,
incluidas vacías: caja/rotación/dimensiones, hash/tamaño del raster, motor/OS/build
y limitaciones. Las coordenadas normalizadas tienen origen inferior izquierdo
en el raster renderizado, no son coordenadas ni offsets del PDF original.
Decimales se conservan como cadenas canónicas; texto reconocido queda literal.

El canónico v1 completo ocupa<=1MiB y se parte por límites UTF-8 de4096B/512
chunks. Migración016 añade dos tablas y la barrera16, con backup15 anterior;
no hay downgrade en el lugar. La retención32MiB y snapshots64MiB incluyen los
seis formatos. Lectores verifican propietarios, fuente, todos los chunks,
metadatos, canónico y hashes aun fuera de la página solicitada. Backup/restore
funcionan sin helpers y conservan formatos/bloques/configuraciones desconocidos.

La selección FTS sigue siendo explícita e independiente de extraer. Proyección
`pdf_ocr_region_text_v1` incluye sólo texto de regiones; no cruza regiones o
páginas ni indexa JSON/metadata. Locator conserva perfil, página, región local y
global, raster completo, motor, bbox/confianza y rango UTF-8 del texto inferido.
Hay rechazo entero por encima de512 fragmentos/1MiB por selección. Consulta y
CAS/invalidation usan la validación íntegra existente, sin reparaciones implícitas.
API v1 tiene55 operaciones; MCP27/47/48 según permisos; batch mantiene nueve
mutaciones y siete exclusiones de administración/archivos.

El texto puede faltar o ser incorrecto. Vision fast/revisión3/es-ES/en-US no
reconstruye tablas ni garantiza orden visual. Confianza no es probabilidad de
exactitud. No se infiere automáticamente la orientación del texto: aplicar
`/Rotate` puede dejar texto de costado si así aparece en el original. El corpus
limpio propio de esta verificación dio CER1,99% y WER10,45% sobre352 caracteres/
67 palabras, sin normalizar; siete errores fueron una tilde perdida por caso.
Esto no estima calidad general de escaneos reales. Los casos de texto de costado
se registran por separado y no se cuentan como éxito de calidad.

Home sigue activo9/17. Este recorte no termina5a: faltan tablas/ampliaciones PDF
y hojas/encabezados/rangos XLSX. 5b y5c requieren cerrar antes5a; fase6 queda fuera.
