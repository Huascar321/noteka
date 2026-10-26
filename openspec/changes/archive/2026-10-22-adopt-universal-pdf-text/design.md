# Design

## Backend

`noteka-pdfium-worker` es un proceso descartable con el mismo framing binario acotado que el worker PDF existente: header con bytes/tiempo, PDF, y reply JSON con `parser_name=pdfium`, versión del binding/PDFium y páginas. `pdfium-render 0.9.4` carga la librería dinámica indicada por `NOTEKA_PDFIUM_LIB` o un sibling (`libpdfium.dylib`, `.so`, `pdfium.dll`). No se enlaza estáticamente ni se descarga nada.

El supervisor del núcleo reutiliza las garantías de pipes no bloqueantes, deadline, stderr/stdout acotados y hash del original. El parser se mantiene separado del core: el core sólo conoce el protocolo.

## Persistencia

No hay migración. `pdfium_text` escribe en `extraction_pdf_generations/pages/fragments` con el parser PDFium; el lector canónico `extraction.pdf_fragments` carga cualquier parser y muestra `parser_name/version`. `pdf_storage::integrity`, `retained` y la búsqueda se amplían para incluir el extractor. Las generaciones `pdf_text`/`pdfkit_text` no se reescriben.

## Prioridad y fallback

La UI/CLI ofrecen `pdfium_text` como primera opción para PDF nuevo; el catálogo publica el extractor. Si la librería no está disponible, el trabajo falla explícitamente con `pdfium_worker_unavailable`; no se cambia automáticamente a PDFKit/lopdf, porque eso alteraría el perfil elegido. El usuario puede reintentar con `pdf_text`/`pdfkit_text`. La disponibilidad se comprueba al ejecutar, no al abrir.

## Alternativas

- PDFKit: se conserva como opción nativa macOS ya verificada; no es prioridad universal.
- lopdf: se conserva como parser acotado de texto; PDFium es la base portable elegida.
- Enlazar PDFium estáticamente o descargarlo: descartado por tamaño, licencias de binarios y reproducibilidad; la ruta se configura y se registra.
- PaddleOCR: fuera de este recorte de texto; se propone en un cambio separado para no mezclar OCR/estructura con el lector PDFium.

## Límites

PDFium puede aceptar más PDFs que el subconjunto lopdf, pero se mantienen 100 páginas, 1 MiB de texto, 1 MiB de fuente, 512 fragmentos y deadline de 5 s. La versión del binario PDFium forma parte de la procedencia; no se promete compatibilidad Windows sin ejecutar ese perfil.

## Verificación

Worker con librería real del entorno de investigación, PDF sintético con página vacía/texto, ausencia de librería, PDF inválido y límites; extracción/lectura/index/búsqueda; UI headless; migración no aplica. `cargo fmt`, Clippy, contratos y OpenSpec estricto.
