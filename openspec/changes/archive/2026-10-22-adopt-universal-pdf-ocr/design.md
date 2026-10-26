# Design

## Worker externo

`native/universal/paddle_ocr_worker.py` recibe JSON por stdin con PDF base64, hash y presupuesto; escribe JSON con parser/version/páginas. Usa `pypdfium2 5.14.0` si hace falta para render y `paddleocr 3.7.0` PP-StructureV3 con modelos PP-OCRv6 small/estructura indicados por `NOTEKA_PADDLE_MODEL_PATHS`. El proceso se lanza desde `paddle_supervisor` con entorno mínimo (`HOME`, `PATH`, variables Paddle) y sin workspace/SQL.

Rust valida hash, páginas consecutivas, texto ≤1 MiB, salida ≤8 MiB, stderr acotado y deadline. El worker no se invoca desde open/search; sólo desde `extraction.run` explícito. El presupuesto de este extractor es 30 s porque la inicialización/estructura Paddle supera los 5 s del resto; no se eleva el target de búsqueda.

## Persistencia

`paddle_ocr` escribe en `extraction_pdf_generations/pages/fragments` con parser `paddleocr_ppstructurev3`. El lector `extraction.pdf_fragments` acepta `paddle_ocr`; el índice lo trata como página PDF. Sin migración. Las tablas HTML/estructura no se persisten en este recorte; sólo el texto de cada página. Se documenta como brecha explícita, no se presenta como tabla estructurada.

## Configuración

Variables: `NOTEKA_PADDLE_PYTHON`, `NOTEKA_PADDLE_WORKER`, `NOTEKA_PADDLE_MODEL_PATHS`, `NOTEKA_PADDLE_DEVICE` (default cpu). Los modelos deben estar instalados; no se descargan. En macOS se probó el entorno de investigación; Windows/Linux requieren instalar wheels/modelos y verificar el perfil.

## Alternativas

Vision/CoreGraphics sigue como opción macOS ya verificada. PDFium+PP-StructureV3 se elige por resultados de tablas del benchmark, aunque este recorte sólo persiste texto. Un binding Rust nativo de Paddle se difiere por tamaño/packaging.

## Verificación

Worker real con el entorno Paddle, extracción/lectura/index/búsqueda, ausencia de configuración, PDF inválido, límites; UI headless; fmt/Clippy/contratos/OpenSpec. La calidad se limita al corpus de benchmark y no se generaliza.
